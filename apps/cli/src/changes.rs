//! Diff de uma tarefa desde o commit base, sem o que já estava alterado no início (SPEC-063).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};

use crate::harness::Harness;
use crate::support::{config_get, sha256_hex};

/// Pasta de trabalho do harness, relativa à raiz do projeto; nunca entra na diff da tarefa.
const HARNESS_WORK_DIR: &str = "vostok/work/";
/// Pastas que a instalação do harness grava fora da raiz dele; ficam fora da diff, menos o que está no "Mudar".
const INSTALL_REDIRECTS: [&str; 4] = [
    ".claude/skills/",
    ".claude/agents/",
    ".agents/skills/",
    ".codex/agents/",
];
/// Frases do agente por caminho e intervalo do trecho, na pasta da tarefa (RF-14).
const NOTES_FILE: &str = "frases.json";
/// Hash gravado para um arquivo que não pôde ser lido (apagado, pasta, submódulo).
const ABSENT_HASH: &str = "-";

/// O que aconteceu com um arquivo desde o commit base.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeAction {
    Created,
    Modified,
    Removed,
    Binary,
}

impl ChangeAction {
    pub fn label(&self) -> &'static str {
        match self {
            ChangeAction::Created => "criado",
            ChangeAction::Modified => "alterado",
            ChangeAction::Removed => "removido",
            ChangeAction::Binary => "binário",
        }
    }
}

/// Um arquivo da diff da tarefa, com o caminho relativo à raiz do projeto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChange {
    pub path: String,
    pub action: ChangeAction,
    pub added: usize,
    pub removed: usize,
    pub hunks: Vec<String>,
    /// O arquivo já estava alterado no início da tarefa e mudou de novo depois.
    pub before: bool,
}

/// Diff da tarefa no formato da rota `mudancas` do painel, com o motivo quando não há diff.
pub fn report(harness: &Harness, spec: &str, task: &str) -> Result<Value> {
    let tasks = harness.path("tasks")?;
    let live = tasks.join(spec).join(task);
    let folder = if live.join("estado.yml").is_file() {
        live
    } else {
        let archived = harness.path("archive")?.join(spec).join("tasks").join(task);
        if !archived.join("estado.yml").is_file() {
            bail!("tarefa {task} não encontrada em {spec}");
        }
        archived
    };
    let state = fs::read_to_string(folder.join("estado.yml"))
        .with_context(|| format!("não foi possível ler {}", folder.display()))?;
    let field = |key: &str| state_field(&state, key);
    let status_name = |key: &str| {
        config_get(&harness.config_file("tasks.yml"), &format!("status.{key}"))
            .unwrap_or_else(|_| key.to_string())
    };
    let status = field("status").unwrap_or_default();
    let not_started = ["pending", "approved", "queued"];
    if not_started.iter().any(|key| status == status_name(key)) {
        return Ok(empty_report(Value::Null, false, "nao_iniciada"));
    }
    if status == status_name("done") {
        let frozen = folder.join("mudancas.json");
        if !frozen.is_file() {
            return Ok(empty_report(Value::Null, true, "nao_gravada"));
        }
        let text = fs::read_to_string(&frozen)
            .with_context(|| format!("não foi possível ler {}", frozen.display()))?;
        return serde_json::from_str(&text)
            .with_context(|| format!("{} não é um JSON válido", frozen.display()));
    }
    let root = harness.path("root")?;
    let commit =
        field("baseline_commit").filter(|value| !["", "null", "-"].contains(&value.as_str()));
    let Some(commit) = commit.filter(|commit| commit_exists(&root, commit)) else {
        return Ok(empty_report(Value::Null, false, "sem_base"));
    };
    let dirty = field("baseline_dirty")
        .map(|value| stored_dirty(&value))
        .unwrap_or_default();
    let exclude = other_tasks_paths(&tasks.join(spec), &folder)?;
    let own = task_paths(&folder);
    let prefixes = harness_prefixes(harness, &root);
    let files: Vec<FileChange> = task_changes(&root, &commit, &dirty, &exclude)?
        .into_iter()
        .filter(|file| {
            own.contains(&file.path)
                || !prefixes
                    .iter()
                    .any(|prefix| file.path.starts_with(prefix.as_str()))
        })
        .collect();
    if files.is_empty() {
        return Ok(empty_report(json!(commit), false, "sem_mudancas"));
    }
    let notes = task_notes(&folder);
    let added: usize = files.iter().map(|file| file.added).sum();
    let removed: usize = files.iter().map(|file| file.removed).sum();
    let files: Vec<Value> = files
        .iter()
        .map(|file| {
            json!({
                "path": file.path,
                "action": file.action.label(),
                "added": file.added,
                "removed": file.removed,
                "hunks": file.hunks,
                "notes": hunk_notes(&notes, &file.path, &file.hunks),
                "before": file.before,
            })
        })
        .collect();
    Ok(json!({
        "base": commit,
        "frozen": false,
        "reason": null,
        "added": added,
        "removed": removed,
        "files": files,
    }))
}

/// A linha `@@` do trecho até o segundo `@@`, sem o contexto de função que o git escreve depois.
fn hunk_range(hunk: &str) -> Option<&str> {
    let first = hunk.lines().next()?;
    let rest = first.strip_prefix("@@ ")?;
    let end = rest.find(" @@")?;
    Some(&first[..3 + end + 3])
}

/// Frases de `frases.json`; arquivo ausente, inválido ou sem objeto na raiz vale como sem frases (CB-12).
fn task_notes(dir: &Path) -> serde_json::Map<String, Value> {
    fs::read_to_string(dir.join(NOTES_FILE))
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .and_then(|value| match value {
            Value::Object(map) => Some(map),
            _ => None,
        })
        .unwrap_or_default()
}

/// Uma frase por trecho, na ordem dos trechos; `null` quando o intervalo não tem frase (CB-11).
fn hunk_notes(notes: &serde_json::Map<String, Value>, path: &str, hunks: &[String]) -> Vec<Value> {
    hunks
        .iter()
        .map(|hunk| {
            hunk_range(hunk)
                .and_then(|range| notes.get(path)?.get(range)?.as_str())
                .map_or(Value::Null, |note| Value::String(note.to_string()))
        })
        .collect()
}

/// A raiz do harness relativa à raiz do projeto (`vostok/` neste) e os redirecionamentos da instalação.
fn harness_prefixes(harness: &Harness, root: &Path) -> Vec<String> {
    let mut prefixes = Vec::new();
    let relative = fs::canonicalize(&harness.root)
        .ok()
        .zip(fs::canonicalize(root).ok())
        .and_then(|(harness_root, project_root)| {
            harness_root
                .strip_prefix(&project_root)
                .ok()
                .map(Path::to_path_buf)
        });
    if let Some(relative) = relative {
        let parts: Vec<String> = relative
            .components()
            .map(|part| part.as_os_str().to_string_lossy().into_owned())
            .collect();
        if !parts.is_empty() {
            prefixes.push(format!("{}/", parts.join("/")));
        }
    }
    prefixes.extend(INSTALL_REDIRECTS.iter().map(|prefix| prefix.to_string()));
    prefixes
}

fn empty_report(base: Value, frozen: bool, reason: &str) -> Value {
    json!({
        "base": base,
        "frozen": frozen,
        "reason": reason,
        "added": 0,
        "removed": 0,
        "files": [],
    })
}

/// Valor de uma linha `chave: valor` do `estado.yml`, sem aspas.
fn state_field(state: &str, key: &str) -> Option<String> {
    state.lines().find_map(|line| {
        let value = line.strip_prefix(key)?.strip_prefix(':')?;
        Some(value.trim().trim_matches('"').to_string())
    })
}

fn commit_exists(root: &Path, commit: &str) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["cat-file", "-e", &format!("{commit}^{{commit}}")])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// Lista `[caminho=hash, ...]` do `baseline_dirty`, dividida no último `=` de cada item.
fn stored_dirty(value: &str) -> Vec<(String, String)> {
    let inner = value.trim().trim_start_matches('[').trim_end_matches(']');
    inner
        .split(", ")
        .filter_map(|item| item.rsplit_once('='))
        .map(|(path, hash)| (path.trim().to_string(), hash.trim().to_string()))
        .filter(|(path, _)| !path.is_empty())
        .collect()
}

/// Caminhos do "Mudar" das outras tarefas da spec, menos os desta.
fn other_tasks_paths(spec_dir: &Path, folder: &Path) -> Result<Vec<String>> {
    let own = task_paths(folder);
    let mut paths = Vec::new();
    let Ok(entries) = fs::read_dir(spec_dir) else {
        return Ok(paths);
    };
    for entry in entries.flatten() {
        let dir = entry.path();
        if dir == folder || !dir.join("estado.yml").is_file() {
            continue;
        }
        paths.extend(
            task_paths(&dir)
                .into_iter()
                .filter(|path| !own.contains(path)),
        );
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn task_paths(folder: &Path) -> Vec<String> {
    fs::read_to_string(folder.join("execucao.md"))
        .map(|text| change_paths(&text))
        .unwrap_or_default()
}

/// Caminhos dos itens da subseção `### Mudar`.
fn change_paths(text: &str) -> Vec<String> {
    text.lines()
        .skip_while(|line| line.trim_end() != "### Mudar")
        .skip(1)
        .take_while(|line| !line.starts_with("### "))
        .filter_map(change_item)
        .map(|(_, path)| path)
        .collect()
}

/// Item "- `caminho` (ação)" da subseção "Mudar", como ação e caminho.
pub fn change_item(line: &str) -> Option<(String, String)> {
    let (path, rest) = backticked_item(line)?;
    let action = rest.strip_prefix(" (")?;
    let end = action.find(')')?;
    let action = &action[..end];
    let valid = !action.is_empty() && action.bytes().all(|b| b.is_ascii_lowercase());
    valid.then(|| (action.to_string(), path.to_string()))
}

fn backticked_item(line: &str) -> Option<(&str, &str)> {
    let rest = line.strip_prefix("- `")?;
    let end = rest.find('`')?;
    (end > 0).then(|| (&rest[..end], &rest[end + 1..]))
}

/// Arquivos alterados ou fora do git na raiz do projeto, com o hash do conteúdo atual.
pub fn dirty_files(root: &Path) -> Result<Vec<(String, String)>> {
    let repo = repo(root)?;
    let (code, output) = git(
        &repo.top,
        &[
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
            "--no-renames",
        ],
    )?;
    if code != 0 {
        bail!("git status falhou: {}", String::from_utf8_lossy(&output));
    }
    let mut files: Vec<(String, String)> = nul_records(&output)
        .filter_map(|record| record.get(3..).map(str::to_string))
        .filter_map(|path| repo.to_root(&path))
        .filter(|path| !harness_work(path))
        .map(|path| {
            let hash = content_hash(root, &path);
            (path, hash)
        })
        .collect();
    files.sort();
    files.dedup();
    Ok(files)
}

/// Diff da tarefa desde `base`, sem a pasta de trabalho do harness, sem os caminhos de `exclude`
/// e sem os arquivos de `dirty` que não mudaram desde o início.
pub fn task_changes(
    root: &Path,
    base: &str,
    dirty: &[(String, String)],
    exclude: &[String],
) -> Result<Vec<FileChange>> {
    let repo = repo(root)?;
    let (code, _) = git(
        &repo.top,
        &["cat-file", "-e", &format!("{base}^{{commit}}")],
    )?;
    if code != 0 {
        bail!("commit base não encontrado: {base}");
    }
    let mut changes = Vec::new();
    for (path, action, tracked) in candidates(&repo, base)? {
        if harness_work(&path) || exclude.contains(&path) {
            continue;
        }
        let before = match dirty.iter().find(|(dirty_path, _)| *dirty_path == path) {
            Some((_, hash)) if *hash == content_hash(root, &path) => continue,
            Some(_) => true,
            None => false,
        };
        let patch = file_patch(&repo, base, &path, tracked)?;
        changes.push(file_change(path, action, &patch, before));
    }
    changes.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(changes)
}

/// Caminhos mudados desde a base, relativos à raiz: rastreados e fora do git.
fn candidates(repo: &Repo, base: &str) -> Result<Vec<(String, ChangeAction, bool)>> {
    let (code, output) = git(
        &repo.top,
        &["diff", "--name-status", "-z", "--no-renames", base, "--"],
    )?;
    if code != 0 {
        bail!(
            "git diff --name-status falhou: {}",
            String::from_utf8_lossy(&output)
        );
    }
    let mut paths = Vec::new();
    let mut records = nul_records(&output);
    while let (Some(status), Some(path)) = (records.next(), records.next()) {
        let action = match status.chars().next() {
            Some('A') => ChangeAction::Created,
            Some('D') => ChangeAction::Removed,
            _ => ChangeAction::Modified,
        };
        if let Some(path) = repo.to_root(&path) {
            paths.push((path, action, true));
        }
    }
    let (code, output) = git(
        &repo.top,
        &["ls-files", "--others", "--exclude-standard", "-z"],
    )?;
    if code != 0 {
        bail!("git ls-files falhou: {}", String::from_utf8_lossy(&output));
    }
    paths.extend(
        nul_records(&output)
            .filter_map(|path| repo.to_root(&path))
            .map(|path| (path, ChangeAction::Created, false)),
    );
    Ok(paths)
}

/// Patch de um arquivo desde a base; o de fora do git é comparado com `/dev/null`.
fn file_patch(repo: &Repo, base: &str, path: &str, tracked: bool) -> Result<String> {
    let target = repo.to_top(path);
    let mut args = vec!["diff", "--no-color", "--no-ext-diff", "--no-textconv"];
    if tracked {
        args.extend(["--no-renames", base, "--", &target]);
    } else {
        args.extend(["--no-index", "--", "/dev/null", &target]);
    }
    let (code, output) = git(&repo.top, &args)?;
    let accepted = code == 0 || (!tracked && code == 1);
    if !accepted {
        bail!("git {} falhou com o código {code}", args.join(" "));
    }
    Ok(String::from_utf8_lossy(&output).into_owned())
}

fn file_change(path: String, action: ChangeAction, patch: &str, before: bool) -> FileChange {
    if patch.lines().any(|line| line.starts_with("Binary files ")) {
        let action = match action {
            ChangeAction::Removed => ChangeAction::Removed,
            _ => ChangeAction::Binary,
        };
        return FileChange {
            path,
            action,
            added: 0,
            removed: 0,
            hunks: Vec::new(),
            before,
        };
    }
    let hunks = split_hunks(patch);
    let (added, removed) = count_lines(&hunks);
    FileChange {
        path,
        action,
        added,
        removed,
        hunks,
        before,
    }
}

/// Cada hunk vai da linha `@@` até antes da próxima `@@` ou do fim do patch.
fn split_hunks(patch: &str) -> Vec<String> {
    let mut hunks: Vec<Vec<&str>> = Vec::new();
    for line in patch.lines() {
        if line.starts_with("@@") {
            hunks.push(vec![line]);
        } else if let Some(hunk) = hunks.last_mut() {
            hunk.push(line);
        }
    }
    hunks.into_iter().map(|lines| lines.join("\n")).collect()
}

fn count_lines(hunks: &[String]) -> (usize, usize) {
    let lines = || hunks.iter().flat_map(|hunk| hunk.lines());
    let added = lines().filter(|line| line.starts_with('+')).count();
    let removed = lines().filter(|line| line.starts_with('-')).count();
    (added, removed)
}

/// Registros da saída `-z` do git, sem o vazio do fim.
fn nul_records(output: &[u8]) -> impl Iterator<Item = String> + '_ {
    output
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
        .map(|record| String::from_utf8_lossy(record).into_owned())
}

/// Repositório git que contém a raiz do projeto, com o caminho da raiz a partir do topo.
struct Repo {
    top: PathBuf,
    /// Caminho da raiz do projeto a partir do topo, terminado em `/`; vazio quando é o topo.
    prefix: String,
}

impl Repo {
    /// Caminho do git (relativo ao topo) como relativo à raiz; `None` fora do projeto.
    fn to_root(&self, path: &str) -> Option<String> {
        path.strip_prefix(&self.prefix).map(str::to_string)
    }

    fn to_top(&self, path: &str) -> String {
        format!("{}{path}", self.prefix)
    }
}

fn repo(root: &Path) -> Result<Repo> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--show-toplevel"])
        .stdin(Stdio::null())
        .output()
        .context("não foi possível executar o git")?;
    if !output.status.success() {
        bail!(
            "git rev-parse --show-toplevel falhou: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let top = PathBuf::from(stdout.strip_suffix('\n').unwrap_or(&stdout));
    let canonical_root = fs::canonicalize(root)
        .with_context(|| format!("não foi possível resolver {}", root.display()))?;
    let canonical_top = fs::canonicalize(&top)
        .with_context(|| format!("não foi possível resolver {}", top.display()))?;
    let relative = canonical_root
        .strip_prefix(&canonical_top)
        .with_context(|| format!("{} está fora do repositório", root.display()))?;
    let parts: Vec<String> = relative
        .components()
        .map(|part| part.as_os_str().to_string_lossy().into_owned())
        .collect();
    let prefix = if parts.is_empty() {
        String::new()
    } else {
        format!("{}/", parts.join("/"))
    };
    Ok(Repo { top, prefix })
}

/// Roda o git no topo do repositório e devolve o código de saída e o `stdout`.
fn git(top: &Path, args: &[&str]) -> Result<(i32, Vec<u8>)> {
    let output = Command::new("git")
        .arg("-C")
        .arg(top)
        .args(["-c", "core.quotePath=false"])
        .args(args)
        .stdin(Stdio::null())
        .output()
        .with_context(|| format!("não foi possível executar git {}", args.join(" ")))?;
    Ok((output.status.code().unwrap_or(-1), output.stdout))
}

fn content_hash(root: &Path, path: &str) -> String {
    fs::read(root.join(path))
        .map(|content| sha256_hex(&content))
        .unwrap_or_else(|_| ABSENT_HASH.to_string())
}

fn harness_work(path: &str) -> bool {
    path.starts_with(HARNESS_WORK_DIR)
}
