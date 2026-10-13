//! Reprodutor das fixtures de equivalência (`tests/fixtures/golden`, formato em `tests/fixtures/README.md`).
//!
//! Para cada caso de um grupo, reconstrói o estado exato antes da chamada, roda o subcomando do
//! `vostok` numa pasta temporária e compara saídas, código de saída e arquivos com o que os scripts
//! bash produziram, normalizando o que muda com o relógio e com a pasta.
//!
//! Variáveis de ambiente úteis na depuração:
//! - `GOLDEN_CASE=0131` reproduz só esse caso (o estado anterior continua sendo reconstruído).

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use sha2::{Digest, Sha256};
use tempfile::TempDir;

const MAX_REPORTED: usize = 15;

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/golden")
}

/// Raiz fixa usada pelo gerador; aparece no conteúdo de alguns arquivos.
const GOLDEN_ROOT: &str = "/tmp/vostok-golden-root";

/// Nome do autor no `git config` da máquina que gerou as fixtures; os comandos que gravam o autor
/// (`spec new`, `skill create`) precisam dele em qualquer máquina.
const GOLDEN_AUTHOR: &str = "Victor Batalha";

#[derive(Debug, Clone)]
pub enum Change {
    Write { path: String, sha: String },
    Delete { path: String },
}

#[derive(Debug, Clone, Default)]
pub struct Case {
    pub number: u32,
    pub group: String,
    pub script: String,
    pub usage: bool,
    pub args: Vec<String>,
    pub argfiles: Vec<(usize, String)>,
    pub vostok: Vec<String>,
    pub cwd: String,
    pub stdin: Vec<u8>,
    pub stdout: String,
    pub stderr: String,
    pub exit: i32,
    pub pre: Vec<Change>,
    pub post: Vec<Change>,
}

/// Diferença encontrada ao reproduzir um caso.
#[derive(Debug)]
pub struct Failure {
    pub case: u32,
    pub script: String,
    pub args: Vec<String>,
    pub reason: String,
}

fn parse_sections(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut sections = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        let line_end = index
            + bytes[index..]
                .iter()
                .position(|b| *b == b'\n')
                .expect("cabeçalho de seção");
        let header = std::str::from_utf8(&bytes[index + 1..line_end]).expect("cabeçalho UTF-8");
        let (name, len) = header.rsplit_once(' ').expect("tamanho da seção");
        let len: usize = len.parse().expect("tamanho numérico");
        let start = line_end + 1;
        sections.push((name.to_string(), bytes[start..start + len].to_vec()));
        index = start + len + 1;
    }
    sections
}

fn parse_changes(text: &str) -> Vec<Change> {
    text.lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            if let Some(path) = line.strip_prefix("D ") {
                Change::Delete {
                    path: path.to_string(),
                }
            } else {
                let rest = line.strip_prefix("W ").expect("linha W ou D");
                let (sha, path) = rest.split_once(' ').expect("sha e caminho");
                Change::Write {
                    path: path.to_string(),
                    sha: sha.to_string(),
                }
            }
        })
        .collect()
}

fn text(bytes: Vec<u8>) -> String {
    String::from_utf8_lossy(&bytes).into_owned()
}

fn parse_case(number: u32, bytes: &[u8]) -> Case {
    let mut case = Case {
        number,
        ..Case::default()
    };
    for (name, value) in parse_sections(bytes) {
        match name.as_str() {
            "group" => case.group = text(value),
            "script" => case.script = text(value),
            "usage" => case.usage = value == b"1",
            "arg" => case.args.push(text(value)),
            "vostok" => case.vostok.push(text(value)),
            "cwd" => case.cwd = text(value),
            "stdin" => case.stdin = value,
            "stdout" => case.stdout = text(value),
            "stderr" => case.stderr = text(value),
            "exit" => case.exit = text(value).parse().expect("código de saída"),
            "pre" => case.pre = parse_changes(&text(value)),
            "post" => case.post = parse_changes(&text(value)),
            other => {
                let index = other.strip_prefix("argfile.").expect("seção conhecida");
                case.argfiles
                    .push((index.parse().expect("índice do argumento"), text(value)));
            }
        }
    }
    case
}

/// Todos os casos, de todos os grupos, na ordem global de execução.
pub fn load_cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for group in fs::read_dir(golden_dir()).expect("pasta golden") {
        let group = group.unwrap().path();
        if !group.is_dir() || group.ends_with("blobs") {
            continue;
        }
        for file in fs::read_dir(&group).unwrap() {
            let file = file.unwrap().path();
            let number: u32 = file
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .parse()
                .expect("número do caso");
            cases.push(parse_case(number, &fs::read(&file).unwrap()));
        }
    }
    cases.sort_by_key(|case| case.number);
    cases
}

fn blob(sha: &str) -> Vec<u8> {
    fs::read(golden_dir().join("blobs").join(sha)).unwrap_or_else(|_| panic!("blob {sha} ausente"))
}

type State = BTreeMap<String, String>;

fn initial_state() -> State {
    let content = fs::read_to_string(golden_dir().join("initial.state")).expect("initial.state");
    let mut state = State::new();
    apply(&mut state, &parse_changes(&content));
    state
}

fn apply(state: &mut State, changes: &[Change]) {
    for change in changes {
        match change {
            Change::Write { path, sha } => {
                state.insert(path.clone(), sha.clone());
            }
            Change::Delete { path } => {
                state.remove(path);
            }
        }
    }
}

const READONLY: &[u8] = b"readonly\n";

fn materialize(root: &Path, state: &State) {
    let mut indexes = Vec::new();
    let mut readonly = Vec::new();
    for (path, sha) in state {
        if path == ".git/index.ls" || path.ends_with("/.git/index.ls") {
            indexes.push((path.clone(), sha.clone()));
            continue;
        }
        let target = root.join(path);
        if path.ends_with('/') {
            fs::create_dir_all(&target).unwrap();
            if blob(sha) == READONLY {
                readonly.push(target);
            }
            continue;
        }
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(&target, blob(sha)).unwrap();
        if path.ends_with(".sh") || path.contains("/hooks/") {
            make_executable(&target);
        }
    }
    for (path, sha) in indexes {
        let repo = root.join(path.trim_end_matches(".git/index.ls"));
        let mut child = Command::new("git")
            .args([
                "-C",
                repo.to_str().unwrap(),
                "update-index",
                "-z",
                "--index-info",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()
            .expect("git update-index");
        let entries = blob(&sha);
        child.stdin.take().unwrap().write_all(&entries).unwrap();
        assert!(
            child.wait().unwrap().success(),
            "índice do git de {path} não recriado"
        );
        for entry in entries.split(|b| *b == 0) {
            let entry = String::from_utf8_lossy(entry);
            if let Some(("100755", file)) = entry
                .split_once(' ')
                .map(|(mode, rest)| (mode, rest.split_once('\t').map_or("", |(_, file)| file)))
            {
                let target = repo.join(file);
                if target.is_file() {
                    make_executable(&target);
                }
            }
        }
        hide_scripts(&repo);
    }
    for dir in readonly {
        set_writable(&dir, false);
    }
}

#[cfg(unix)]
fn set_writable(path: &Path, writable: bool) {
    use std::os::unix::fs::PermissionsExt;
    let mode = if writable { 0o755 } else { 0o555 };
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
}

#[cfg(not(unix))]
fn set_writable(_path: &Path, _writable: bool) {}

fn is_readonly(path: &Path) -> bool {
    fs::metadata(path).is_ok_and(|meta| meta.permissions().readonly())
}

/// Devolve a escrita às pastas protegidas, para a pasta temporária poder ser apagada.
fn release(dir: &Path) {
    if is_readonly(dir) {
        set_writable(dir, true);
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            release(&entry.path());
        }
    }
}

/// Os scripts versionados ficam fora dos estados (são os gravadores); o `git status` não deve vê-los.
fn hide_scripts(repo: &Path) {
    let listed = Command::new("git")
        .args([
            "-C",
            repo.to_str().unwrap(),
            "ls-files",
            "-z",
            "--",
            "*.sh",
            "*.sh.real",
        ])
        .output()
        .unwrap();
    if listed.stdout.is_empty() {
        return;
    }
    let mut child = Command::new("git")
        .args([
            "-C",
            repo.to_str().unwrap(),
            "update-index",
            "--skip-worktree",
            "-z",
            "--stdin",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .expect("git update-index --skip-worktree");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&listed.stdout)
        .unwrap();
    assert!(
        child.wait().unwrap().success(),
        "scripts do índice não escondidos"
    );
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) {}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Estado de uma pasta no mesmo formato das fixtures, com os conteúdos em memória.
fn snapshot(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut files = BTreeMap::new();
    walk(root, root, &mut files);
    files
}

fn walk(root: &Path, dir: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let entries: Vec<_> = entries.flatten().collect();
    if dir != root && is_readonly(dir) {
        files.insert(relative(root, dir) + "/", READONLY.to_vec());
    } else if entries.is_empty() && dir != root {
        files.insert(relative(root, dir) + "/", Vec::new());
    }
    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type().unwrap();
        if file_type.is_dir() {
            if entry.file_name() == ".git" {
                let out = Command::new("git")
                    .args(["-C", dir.to_str().unwrap(), "ls-files", "-s", "-z"])
                    .output()
                    .unwrap();
                let rel = relative(root, &path.join("index.ls"));
                files.insert(rel, out.stdout);
            }
            walk(root, &path, files);
        } else if file_type.is_file() {
            let rel = relative(root, &path);
            if rel == ".git/index" || rel.ends_with("/.git/index") {
                continue;
            }
            files.insert(rel, fs::read(&path).unwrap());
        }
    }
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap()
        .to_string_lossy()
        .into_owned()
}

// --- Normalização ---

/// Troca os temporários do `mktemp` (`<pasta temporária>/tmp.XXXX`) por `<TMP>`; a pasta temporária
/// é `/tmp` nas fixtures e a do sistema (`TMPDIR`, ex.: `/var/folders/...` no macOS) na reprodução.
fn replace_tmp(text: &str) -> String {
    let system = std::env::temp_dir();
    let system = format!("{}/tmp.", system.to_string_lossy().trim_end_matches('/'));
    let mut text = text.to_string();
    for prefix in ["/tmp/tmp.", system.as_str()] {
        let mut out = String::new();
        let mut rest = text.as_str();
        while let Some(start) = rest.find(prefix) {
            out.push_str(&rest[..start]);
            let after = &rest[start + prefix.len()..];
            let len = after
                .find(|c: char| !c.is_ascii_alphanumeric())
                .unwrap_or(after.len());
            out.push_str("<TMP>");
            rest = &after[len..];
        }
        out.push_str(rest);
        text = out;
    }
    text
}

fn is_digit_at(bytes: &[u8], index: usize) -> bool {
    bytes.get(index).is_some_and(u8::is_ascii_digit)
}

fn digits(bytes: &[u8], index: usize, count: usize) -> bool {
    (0..count).all(|offset| is_digit_at(bytes, index + offset))
}

/// Tamanho do resto de uma data de modificação do `diff` (` HH:MM:SS[.n] +zzzz`), se houver.
fn diff_mtime_len(bytes: &[u8]) -> Option<usize> {
    let clock = bytes.first() == Some(&b' ')
        && digits(bytes, 1, 2)
        && bytes.get(3) == Some(&b':')
        && digits(bytes, 4, 2)
        && bytes.get(6) == Some(&b':')
        && digits(bytes, 7, 2);
    if !clock {
        return None;
    }
    let mut index = 9;
    if bytes.get(index) == Some(&b'.') {
        index += 1;
        while is_digit_at(bytes, index) {
            index += 1;
        }
    }
    let zone = bytes.get(index) == Some(&b' ')
        && matches!(bytes.get(index + 1), Some(b'+' | b'-'))
        && digits(bytes, index + 2, 4);
    zone.then_some(index + 6)
}

/// Troca datas, horas e sequências hexadecimais (hashes) por marcadores.
fn replace_volatile(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = String::new();
    let mut index = 0;
    while index < bytes.len() {
        let boundary = index == 0 || !bytes[index - 1].is_ascii_alphanumeric();
        if boundary
            && digits(bytes, index, 4)
            && bytes.get(index + 4) == Some(&b'-')
            && digits(bytes, index + 5, 2)
            && bytes.get(index + 7) == Some(&b'-')
            && digits(bytes, index + 8, 2)
        {
            index += 10;
            let time = bytes.get(index) == Some(&b'T')
                && digits(bytes, index + 1, 2)
                && bytes.get(index + 3) == Some(&b':')
                && digits(bytes, index + 4, 2)
                && bytes.get(index + 6) == Some(&b':')
                && digits(bytes, index + 7, 2);
            if time && bytes.get(index + 9) == Some(&b'Z') {
                out.push_str("<DATETIME>");
                index += 10;
            } else if let Some(len) = diff_mtime_len(&bytes[index..]) {
                out.push_str("<MTIME>");
                index += len;
            } else {
                out.push_str("<DATE>");
            }
            continue;
        }
        if boundary
            && digits(bytes, index, 2)
            && bytes.get(index + 2) == Some(&b':')
            && digits(bytes, index + 3, 2)
            && bytes.get(index + 5) == Some(&b':')
            && digits(bytes, index + 6, 2)
        {
            out.push_str("<TIME>");
            index += 8;
            if bytes.get(index) == Some(&b'.') {
                index += 1;
                while is_digit_at(bytes, index) {
                    index += 1;
                }
            }
            continue;
        }
        if boundary {
            let len = bytes[index..]
                .iter()
                .take_while(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(b))
                .count();
            let ends = !bytes
                .get(index + len)
                .is_some_and(u8::is_ascii_alphanumeric);
            let has_digit = bytes[index..index + len].iter().any(u8::is_ascii_digit);
            if (7..=64).contains(&len) && ends && has_digit {
                out.push_str("<HEX>");
                index += len;
                continue;
            }
        }
        let ch = text[index..].chars().next().unwrap();
        out.push(ch);
        index += ch.len_utf8();
    }
    out
}

/// Normalização aplicada às duas pontas da comparação.
pub fn normalize(text: &str, root: &Path) -> String {
    let text = text
        .replace(root.to_str().unwrap(), "<ROOT>")
        .replace(GOLDEN_ROOT, "<ROOT>");
    replace_volatile(&replace_tmp(&text))
}

// --- Reprodução ---

fn is_git_internal(path: &str) -> bool {
    let in_git =
        path.starts_with(".git/") || path.contains("/.git/") || path.starts_with(".remote.git/");
    in_git && !path.ends_with(".git/index.ls")
}

fn run_case(case: &Case, root: &Path, temp: &Path) -> std::process::Output {
    let substitute = |value: &str| value.replace("<ROOT>", root.to_str().unwrap());
    let mut args: Vec<String> = case.vostok.iter().map(|arg| substitute(arg)).collect();
    for (index, sha) in &case.argfiles {
        let original = &case.args[*index];
        let file = temp.join(format!("argfile-{index}"));
        fs::write(&file, blob(sha)).unwrap();
        for arg in &mut args {
            if arg == original {
                *arg = file.to_string_lossy().into_owned();
            }
        }
    }
    let mut command = Command::new(env!("CARGO_BIN_EXE_vostok"));
    command
        .args(&args)
        .current_dir(root)
        .env("PWD", root)
        .env("GIT_AUTHOR_NAME", "T")
        .env("GIT_AUTHOR_EMAIL", "t@t")
        .env("GIT_COMMITTER_NAME", "T")
        .env("GIT_COMMITTER_EMAIL", "t@t")
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "user.name")
        .env("GIT_CONFIG_VALUE_0", GOLDEN_AUTHOR)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().expect("executar o caso");
    child.stdin.take().unwrap().write_all(&case.stdin).ok();
    child.wait_with_output().unwrap()
}

/// Os scripts do harness aparecem nos estados como gravadores do gerador; não fazem parte da comparação.
fn is_recorder(sha: &str) -> bool {
    blob(sha).starts_with(b"#!/usr/bin/env bash\nexec python3 \"$GOLDEN_RECORD\"")
}

fn compare_case(
    case: &Case,
    before: &State,
    root: &Path,
    output: &std::process::Output,
) -> Option<String> {
    let mut problems = Vec::new();
    let actual_exit = output.status.code().unwrap_or(-1);
    if case.usage {
        return (actual_exit == 0).then(|| "erro de uso aceito: deveria falhar".to_string());
    }
    if actual_exit != case.exit {
        problems.push(format!(
            "código de saída {actual_exit}, esperado {}",
            case.exit
        ));
    }
    let norm = |text: &str| normalize(text, root);
    let stdout = norm(&String::from_utf8_lossy(&output.stdout));
    let stderr = norm(&String::from_utf8_lossy(&output.stderr));
    let expected_stdout = norm(&case.stdout);
    let expected_stderr = norm(&case.stderr);
    if stdout != expected_stdout {
        problems.push(format!(
            "stdout diferente\n--- esperado\n{expected_stdout}\n--- obtido\n{stdout}"
        ));
    }
    if stderr != expected_stderr {
        problems.push(format!(
            "stderr diferente\n--- esperado\n{expected_stderr}\n--- obtido\n{stderr}"
        ));
    }
    let mut expected = before.clone();
    apply(&mut expected, &case.post);
    let actual = snapshot(root);
    let mut paths: Vec<&String> = expected.keys().chain(actual.keys()).collect();
    paths.sort();
    paths.dedup();
    for path in paths {
        if is_git_internal(path) || path.starts_with("argfile-") {
            continue;
        }
        if expected.get(path).is_some_and(|sha| is_recorder(sha)) {
            continue;
        }
        match (expected.get(path), actual.get(path)) {
            (Some(_), None) => problems.push(format!("arquivo ausente: {path}")),
            (None, Some(_)) => problems.push(format!("arquivo a mais: {path}")),
            (Some(sha), Some(content)) if *sha != sha256(content) => {
                let want = norm(&String::from_utf8_lossy(&blob(sha)));
                let got = norm(&String::from_utf8_lossy(content));
                if want != got {
                    problems.push(format!(
                        "conteúdo diferente: {path}\n--- esperado\n{want}\n--- obtido\n{got}"
                    ));
                }
            }
            _ => {}
        }
    }
    (!problems.is_empty()).then(|| problems.join("\n"))
}

/// Reproduz os casos de um grupo com o `vostok` e devolve as diferenças.
pub fn replay_group(group: &str) -> Vec<Failure> {
    let only = std::env::var("GOLDEN_CASE")
        .ok()
        .and_then(|value| value.parse::<u32>().ok());
    let mut state = initial_state();
    let mut failures = Vec::new();
    for case in load_cases() {
        apply(&mut state, &case.pre);
        let selected = case.group == group && only.is_none_or(|number| number == case.number);
        if selected {
            let temp = TempDir::new().unwrap();
            let root = temp.path().join("root");
            fs::create_dir_all(&root).unwrap();
            materialize(&root, &state);
            let output = run_case(&case, &root, temp.path());
            let reason = compare_case(&case, &state, &root, &output);
            release(&root);
            if let Some(reason) = reason {
                failures.push(Failure {
                    case: case.number,
                    script: case.script.clone(),
                    args: case.args.clone(),
                    reason,
                });
            }
        }
        apply(&mut state, &case.post);
    }
    failures
}

/// Falha o teste com um resumo das diferenças do grupo, reproduzido com o `vostok`.
pub fn assert_group(group: &str) {
    let failures = replay_group(group);
    if failures.is_empty() {
        return;
    }
    let mut report = format!(
        "{} caso(s) do grupo '{group}' diferem das fixtures:\n",
        failures.len()
    );
    for failure in failures.iter().take(MAX_REPORTED) {
        let reason: String = failure
            .reason
            .lines()
            .take(30)
            .collect::<Vec<_>>()
            .join("\n    ");
        report.push_str(&format!(
            "\n[{:04}] {} {}\n    {reason}\n",
            failure.case,
            failure.script,
            failure.args.join(" ")
        ));
    }
    if failures.len() > MAX_REPORTED {
        report.push_str(&format!(
            "\n... e mais {} caso(s). Use GOLDEN_CASE=<NNNN> para ver um caso.\n",
            failures.len() - MAX_REPORTED
        ));
    }
    panic!("{report}");
}
