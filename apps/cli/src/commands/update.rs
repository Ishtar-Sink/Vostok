//! Comando `update`: regrava no projeto os arquivos que vêm do binário e preserva os do projeto
//! (RF-09 a RF-12, RN-01, RN-02).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::commands::skill::{self, Scope};
use crate::embedded;
use crate::harness::Harness;
use crate::manifest::{self, Manifest};
use crate::routing;
use crate::support::{config_get, config_list, lexical, sha256_hex};

const CONFIG_DIR: &str = "config/";

/// Itens que o contexto obrigatório precisa ter desde a v0.2.0 (RF-25).
const REQUIRED_CONTEXT: &[&str] = &["personas/coordenador.md"];

/// Itens de lista (arquivo, chave, item) que o binário exige nas listas já instaladas (RF-08).
const REQUIRED_LIST_ITEMS: &[(&str, &str, &str)] = &[
    ("config/tasks.yml", "evidence_fields", "Título"),
    ("config/tasks.yml", "evidence_optional_fields", "Espera"),
    ("config/tasks.yml", "evidence_optional_fields", "Como"),
];

const SKILL_FILE: &str = "SKILL.md";

/// Chaves da comunicação pelo painel que saíram na 0.5.0, por arquivo de configuração (RF-23).
const OBSOLETE_KEYS: &[(&str, &str)] = &[
    ("config/main.yml", "paths.requests"),
    ("config/main.yml", "paths.questions"),
    ("config/main.yml", "paths.approvals"),
    ("config/main.yml", "approval_lock"),
    ("config/main.yml", "folders"),
    ("config/main.yml", "request_dispatch"),
    ("config/main.yml", "toast_dismiss_seconds"),
    ("config/main.yml", "request_note_warning_minutes"),
    ("config/tasks.yml", "manual_attachment_max_mb"),
];

/// Pastas da comunicação: a chave de `paths`, o caminho padrão sem a chave e o rótulo do aviso (RF-24).
const COMMUNICATION_DIRS: &[(&str, &str, &str)] = &[
    ("requests", "./work/pedidos", "pedidos"),
    ("questions", "./work/perguntas", "perguntas"),
    ("approvals", "./work/avais", "avais"),
];

/// O que a atualização mudou, na ordem em que aparece na saída.
#[derive(Debug, Default)]
struct Changes {
    updated: Vec<PathBuf>,
    recreated: Vec<PathBuf>,
    removed: Vec<PathBuf>,
    kept_locals: Vec<PathBuf>,
    added_keys: Vec<String>,
    removed_keys: Vec<String>,
    removed_dirs: Vec<PathBuf>,
    routing: Vec<PathBuf>,
    agents: Vec<PathBuf>,
    hooks: Vec<(&'static str, PathBuf)>,
    conflicts: Vec<String>,
    manifest: bool,
}

impl Changes {
    fn is_empty(&self) -> bool {
        self.updated.is_empty()
            && self.recreated.is_empty()
            && self.removed.is_empty()
            && self.kept_locals.is_empty()
            && self.added_keys.is_empty()
            && self.removed_keys.is_empty()
            && self.removed_dirs.is_empty()
            && self.routing.is_empty()
            && self.agents.is_empty()
            && self.hooks.is_empty()
            && !self.manifest
    }
}

pub fn run() -> Result<()> {
    let harness = Harness::from_current_dir()?;
    let previous = Manifest::read(&harness)?;
    let current = Manifest::current(&harness)?;
    let mut changes = Changes::default();

    remove_dropped_skills(&harness, &previous, &current, &mut changes)?;
    remove_dropped_files(&harness, &previous, &current, &mut changes)?;
    remove_communication_dirs(&harness, &mut changes)?;
    remove_obsolete_keys(&harness, &mut changes)?;
    update_binary_files(&harness, &previous, &current, &mut changes)?;
    merge_configs(&harness, &mut changes)?;
    ensure_required_context(&harness, &mut changes)?;
    ensure_required_list_items(&harness, &mut changes)?;
    match write_tool_hooks(&harness)? {
        ToolHooks::Written(label, path) => changes.hooks.push((label, path)),
        ToolHooks::Invalid(path) => changes.conflicts.push(invalid_settings_warning(
            &routing::project_root(&harness)?,
            &path,
        )),
        ToolHooks::Gap(label) => changes.conflicts.push(gap_warning(label)),
        ToolHooks::Skipped | ToolHooks::Unchanged => {}
    }
    let routing = routing::generate(&harness)?;
    for path in routing.changed {
        if !changes.routing.contains(&path) {
            changes.routing.push(path);
        }
    }
    changes.agents = routing.agents;
    changes.kept_locals.extend(routing.kept_locals);
    changes.conflicts.extend(routing.conflicts);
    changes.manifest = current.write(&harness)?;

    print(&routing::project_root(&harness)?, &changes);
    Ok(())
}

/// Regrava os arquivos do binário que diferem dele. Um arquivo alterado no projeto, cujo hash não
/// é o do manifesto anterior, é guardado antes como `.local` (RF-10); um apagado é recriado (CB-03).
fn update_binary_files(
    harness: &Harness,
    previous: &Manifest,
    current: &Manifest,
    changes: &mut Changes,
) -> Result<()> {
    for path in current.files.keys() {
        let target = harness.root.join(path);
        let content = embedded::file(path).unwrap_or_default();
        match fs::read(&target) {
            Ok(installed) if installed == content => continue,
            Ok(installed) => {
                if previous.files.get(path) != Some(&sha256_hex(&installed)) {
                    changes.kept_locals.push(routing::keep_local(&target)?);
                }
                changes.updated.push(target.clone());
            }
            Err(_) => changes.recreated.push(target.clone()),
        }
        write(&target, content)?;
    }
    Ok(())
}

/// Acrescenta às configurações instaladas as chaves novas do binário, sem mudar as existentes (RF-11).
fn merge_configs(harness: &Harness, changes: &mut Changes) -> Result<()> {
    let configs = embedded::files()
        .into_iter()
        .filter(|path| path.starts_with(CONFIG_DIR));
    for path in configs {
        let target = harness.root.join(&path);
        let shipped = String::from_utf8_lossy(embedded::file(&path).unwrap_or_default());
        let Ok(installed) = fs::read_to_string(&target) else {
            write(&target, shipped.as_bytes())?;
            changes.recreated.push(target);
            continue;
        };
        let (merged, added) = merge_keys(&installed, &shipped);
        if added.is_empty() {
            continue;
        }
        write(&target, merged.as_bytes())?;
        changes
            .added_keys
            .extend(added.into_iter().map(|key| format!("{path}: {key}")));
    }
    Ok(())
}

/// Garante os itens de `REQUIRED_CONTEXT` na lista `required` de `config/context.yml`, que o
/// `merge_keys` não completa por já existir a chave (RF-25, CB-06).
fn ensure_required_context(harness: &Harness, changes: &mut Changes) -> Result<()> {
    let added = ensure_list_items(harness, "config/context.yml", "required", REQUIRED_CONTEXT)?;
    changes.added_keys.extend(
        added
            .into_iter()
            .map(|item| format!("config/context.yml: required.{item}")),
    );
    Ok(())
}

/// Acrescenta os itens de `REQUIRED_LIST_ITEMS` às listas que o projeto já tem, sem tocar nas
/// demais listas (RF-08, CB-06).
fn ensure_required_list_items(harness: &Harness, changes: &mut Changes) -> Result<()> {
    for (file, key, item) in REQUIRED_LIST_ITEMS {
        let added = ensure_list_items(harness, file, key, &[item])?;
        changes.added_keys.extend(
            added
                .into_iter()
                .map(|item| format!("{file}: {key}: {item}")),
        );
    }
    Ok(())
}

/// Acrescenta no fim da lista `key` do arquivo os itens que faltam e devolve os acrescentados.
/// Arquivo ou lista ausente não é tratado aqui: o `merge_keys` cria a chave.
fn ensure_list_items(
    harness: &Harness,
    file: &str,
    key: &str,
    items: &[&str],
) -> Result<Vec<String>> {
    let path = harness.root.join(file);
    let Ok(content) = fs::read_to_string(&path) else {
        return Ok(Vec::new());
    };
    let current = config_list(&path, key).unwrap_or_default();
    let mut lines: Vec<String> = content.lines().map(str::to_string).collect();
    let header_line = format!("{key}:");
    let empty_line = format!("{key}: []");
    let mut added = Vec::new();
    for item in items {
        if current.iter().any(|existing| existing == item) {
            continue;
        }
        let Some(header) = lines
            .iter()
            .position(|line| line.trim_end() == header_line || line.trim_end() == empty_line)
        else {
            continue;
        };
        if lines[header].trim_end() == empty_line {
            lines[header] = header_line.clone();
        }
        let mut insert_at = header + 1;
        for (offset, line) in lines.iter().enumerate().skip(header + 1) {
            let trimmed = line.trim_start();
            if trimmed.starts_with("- ") {
                insert_at = offset + 1;
            } else if !line.is_empty() && !line.starts_with(' ') && !trimmed.starts_with('#') {
                break;
            }
        }
        lines.insert(insert_at, format!("  - {item}"));
        added.push(item.to_string());
    }
    if added.is_empty() {
        return Ok(added);
    }
    let mut merged = lines.join("\n");
    merged.push('\n');
    write(&path, merged.as_bytes())?;
    Ok(added)
}

/// Tira os redirecionamentos e a linha do catálogo de cada skill padrão do manifesto anterior que o
/// binário não traz mais. O redirecionamento diferente do gerado do `SKILL.md` instalado é guardado
/// antes como `.local` (RF-22, CB-06). Uma skill customizada com o mesmo nome fica intacta.
fn remove_dropped_skills(
    harness: &Harness,
    previous: &Manifest,
    current: &Manifest,
    changes: &mut Changes,
) -> Result<()> {
    let config = harness.config_file("skills.yml");
    let default_dir = config_get(&config, "dirs.default")?;
    let prefix = format!(
        "{}/",
        default_dir.trim_start_matches("./").trim_end_matches('/')
    );
    let names: Vec<&str> = dropped(previous, current)
        .filter_map(|(path, _)| path.strip_prefix(&prefix)?.strip_suffix("/SKILL.md"))
        .filter(|name| !name.contains('/'))
        .collect();
    if names.is_empty() {
        return Ok(());
    }
    let root = routing::project_root(harness)?;
    let redirect_dirs: Vec<String> = config_list(&config, "redirect_dirs")?
        .into_iter()
        .filter(|dir| !dir.is_empty())
        .collect();
    for name in names {
        if skill::skill_dir(harness, Scope::Custom, name)?
            .join(SKILL_FILE)
            .is_file()
        {
            continue;
        }
        let skill_file = skill::skill_dir(harness, Scope::Default, name)?.join(SKILL_FILE);
        let expected = if skill_file.is_file() {
            Some(skill::render_redirect(harness, &skill_file)?)
        } else {
            None
        };
        for dir in &redirect_dirs {
            let target = root.join(dir).join(name).join(SKILL_FILE);
            let Ok(installed) = fs::read_to_string(&target) else {
                continue;
            };
            if expected.as_deref() != Some(installed.as_str()) {
                changes.kept_locals.push(routing::keep_local(&target)?);
            }
            remove_file(&target)?;
            changes.removed.push(target);
            remove_empty_dir(&root.join(dir).join(name))?;
        }
        if let Some(catalog) = remove_catalog_row(harness, name)? {
            if !changes.routing.contains(&catalog) {
                changes.routing.push(catalog);
            }
        }
    }
    Ok(())
}

/// Caminhos e hashes do manifesto anterior que o binário atual não traz.
fn dropped<'a>(
    previous: &'a Manifest,
    current: &'a Manifest,
) -> impl Iterator<Item = (&'a String, &'a String)> {
    previous
        .files
        .iter()
        .filter(|(path, _)| !current.files.contains_key(*path))
}

/// Tira do catálogo a linha da skill na seção das padrão e devolve o catálogo se ele mudou.
fn remove_catalog_row(harness: &Harness, name: &str) -> Result<Option<PathBuf>> {
    let catalog = harness.root.join(manifest::catalog_path(harness)?);
    let Ok(content) = fs::read_to_string(&catalog) else {
        return Ok(None);
    };
    let section = config_get(
        &harness.config_file("skills.yml"),
        &format!("catalog_sections.{}", Scope::Default.key()),
    )?;
    let heading = format!("## {section}");
    let row_start = format!("| {name} |");
    let mut inside = false;
    let mut removed = false;
    let mut kept: Vec<&str> = Vec::new();
    for line in content.split_inclusive('\n') {
        if line.starts_with("## ") {
            inside = line.trim_end() == heading;
        }
        if inside && line.starts_with(&row_start) {
            removed = true;
            continue;
        }
        kept.push(line);
    }
    if !removed {
        return Ok(None);
    }
    write(&catalog, kept.concat().as_bytes())?;
    Ok(Some(catalog))
}

/// Apaga os arquivos do manifesto anterior que o binário não traz mais. Um alterado no projeto,
/// cujo hash não é o do manifesto anterior, é guardado antes como `.local` (RF-22, CB-06).
fn remove_dropped_files(
    harness: &Harness,
    previous: &Manifest,
    current: &Manifest,
    changes: &mut Changes,
) -> Result<()> {
    for (path, hash) in dropped(previous, current) {
        let target = harness.root.join(path);
        let Ok(installed) = fs::read(&target) else {
            continue;
        };
        if sha256_hex(&installed) != *hash {
            changes.kept_locals.push(routing::keep_local(&target)?);
        }
        remove_file(&target)?;
        let mut dir = target.parent();
        while let Some(parent) =
            dir.filter(|parent| parent.starts_with(&harness.root) && *parent != harness.root)
        {
            if !remove_empty_dir(parent)? {
                break;
            }
            dir = parent.parent();
        }
        changes.removed.push(target);
    }
    Ok(())
}

/// Apaga as pastas de pedidos, perguntas e avais nos caminhos configurados, ou nos padrão sem a
/// chave (RF-24, CB-05). Nunca apaga uma pasta fora do projeto, a do harness nem uma que guarde as
/// specs ou as tarefas: essa vira um aviso.
fn remove_communication_dirs(harness: &Harness, changes: &mut Changes) -> Result<()> {
    let root = routing::project_root(harness)?;
    let main = harness.config_file("main.yml");
    let work = [
        lexical(&harness.path("specs")?),
        lexical(&harness.path("tasks")?),
    ];
    let harness_root = lexical(&harness.root);
    for (key, default, label) in COMMUNICATION_DIRS {
        let configured = config_get(&main, &format!("paths.{key}")).ok();
        let dir = lexical(&harness.resolve(configured.as_deref().unwrap_or(default)));
        if !dir.is_dir() {
            continue;
        }
        let protected = !dir.starts_with(&root)
            || dir == root
            || dir == harness_root
            || work.iter().any(|path| path.starts_with(&dir));
        if protected {
            let shown = dir
                .strip_prefix(&root)
                .unwrap_or(&dir)
                .display()
                .to_string();
            changes
                .conflicts
                .push(format!("Aviso: pasta de {label} não removida: {shown}"));
            continue;
        }
        fs::remove_dir_all(&dir)
            .with_context(|| format!("não foi possível apagar {}", dir.display()))?;
        changes.removed_dirs.push(dir);
    }
    Ok(())
}

/// Tira das configurações instaladas as chaves de `OBSOLETE_KEYS` (RF-23).
fn remove_obsolete_keys(harness: &Harness, changes: &mut Changes) -> Result<()> {
    let mut by_file: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (file, key) in OBSOLETE_KEYS {
        by_file.entry(file).or_default().push(key);
    }
    for (file, keys) in by_file {
        let target = harness.root.join(file);
        let Ok(installed) = fs::read_to_string(&target) else {
            continue;
        };
        let (content, removed) = remove_keys(&installed, &keys);
        if removed.is_empty() {
            continue;
        }
        write(&target, content.as_bytes())?;
        changes
            .removed_keys
            .extend(removed.into_iter().map(|key| format!("{file}: {key}")));
    }
    Ok(())
}

/// Conteúdo sem as chaves pedidas (`pai` ou `pai.filho`) e as que foram encontradas, na ordem pedida.
/// Uma chave de primeiro nível sai com os comentários de antes; uma filha, só com as linhas dela.
fn remove_keys(installed: &str, keys: &[&str]) -> (String, Vec<String>) {
    let lines: Vec<&str> = installed.lines().collect();
    let installed_blocks = blocks(&lines);
    let mut dropped = vec![false; lines.len()];
    let mut removed = Vec::new();
    for key in keys {
        let (parent, child) = match key.split_once('.') {
            Some((parent, child)) => (parent, Some(child)),
            None => (*key, None),
        };
        let Some(block) = installed_blocks.iter().find(|block| block.key == parent) else {
            continue;
        };
        let range = match child {
            None => block.lead..block.end,
            Some(child) => {
                let Some(found) = children(&lines, block)
                    .into_iter()
                    .find(|other| other.key == child)
                else {
                    continue;
                };
                found.start..found.end
            }
        };
        dropped[range].iter_mut().for_each(|line| *line = true);
        removed.push(key.to_string());
    }
    let kept: Vec<&str> = lines
        .iter()
        .zip(&dropped)
        .filter(|(_, dropped)| !**dropped)
        .map(|(line, _)| *line)
        .collect();
    (kept.join("\n") + "\n", removed)
}

/// Chave de primeiro nível de um YAML de configuração, da linha `chave:` até a última linha recuada.
struct Block<'a> {
    key: &'a str,
    /// Primeira linha dos comentários e linhas em branco logo antes da chave.
    lead: usize,
    start: usize,
    end: usize,
}

/// Chave filha de um bloco de mapeamento, com as linhas mais recuadas que ela.
struct Child<'a> {
    key: &'a str,
    start: usize,
    end: usize,
}

/// Conteúdo instalado com as chaves do binário que faltavam e os nomes delas (`pai` ou `pai.filho`).
fn merge_keys(installed: &str, shipped: &str) -> (String, Vec<String>) {
    let installed_lines: Vec<&str> = installed.lines().collect();
    let shipped_lines: Vec<&str> = shipped.lines().collect();
    let installed_blocks = blocks(&installed_lines);
    let mut inserts: BTreeMap<usize, Vec<&str>> = BTreeMap::new();
    let mut appended: Vec<&str> = Vec::new();
    let mut added = Vec::new();

    for block in blocks(&shipped_lines) {
        let Some(existing) = installed_blocks.iter().find(|other| other.key == block.key) else {
            appended.push("");
            appended.extend(trim_blank_start(&shipped_lines[block.lead..block.end]));
            added.push(block.key.to_string());
            continue;
        };
        let present = children(&installed_lines, existing);
        for child in children(&shipped_lines, &block) {
            if present.iter().any(|other| other.key == child.key) {
                continue;
            }
            inserts
                .entry(existing.end)
                .or_default()
                .extend(&shipped_lines[child.start..child.end]);
            added.push(format!("{}.{}", block.key, child.key));
        }
    }

    let mut merged: Vec<&str> = Vec::new();
    for (index, line) in installed_lines.iter().enumerate() {
        if let Some(lines) = inserts.get(&index) {
            merged.extend(lines);
        }
        merged.push(line);
    }
    if let Some(lines) = inserts.get(&installed_lines.len()) {
        merged.extend(lines);
    }
    merged.extend(appended);
    (merged.join("\n") + "\n", added)
}

fn blocks<'a>(lines: &[&'a str]) -> Vec<Block<'a>> {
    let mut blocks: Vec<Block<'a>> = Vec::new();
    let mut lead = 0;
    for (index, line) in lines.iter().enumerate() {
        if let Some(key) = top_level_key(line) {
            blocks.push(Block {
                key,
                lead,
                start: index,
                end: index + 1,
            });
        } else if line.starts_with([' ', '\t']) {
            if let Some(block) = blocks.last_mut() {
                block.end = index + 1;
            }
        } else {
            continue;
        }
        lead = index + 1;
    }
    blocks
}

fn top_level_key(line: &str) -> Option<&str> {
    if line.starts_with([' ', '\t', '#']) {
        return None;
    }
    line.split_once(':')
        .map(|(key, _)| key.trim_end())
        .filter(|key| !key.is_empty())
}

/// Filhas de um bloco de mapeamento; vazio para escalares, listas e blocos literais.
fn children<'a>(lines: &[&'a str], block: &Block) -> Vec<Child<'a>> {
    let header_value = lines[block.start]
        .split_once(':')
        .map_or("", |(_, value)| value);
    if !strip_comment(header_value).trim().is_empty() {
        return Vec::new();
    }
    let body = || (block.start + 1..block.end).filter(|&index| !is_blank_or_comment(lines[index]));
    let Some(first) = body().next() else {
        return Vec::new();
    };
    if lines[first].trim_start().starts_with('-') {
        return Vec::new();
    }
    let indent = indentation(lines[first]);
    let mut children: Vec<Child<'a>> = Vec::new();
    for index in body() {
        let line = lines[index];
        if indentation(line) == indent && !line.trim_start().starts_with('-') {
            if let Some((key, _)) = line.trim_start().split_once(':') {
                children.push(Child {
                    key: key.trim_end(),
                    start: index,
                    end: index + 1,
                });
                continue;
            }
        }
        if let Some(child) = children.last_mut() {
            child.end = index + 1;
        }
    }
    children
}

fn indentation(line: &str) -> usize {
    line.len() - line.trim_start_matches([' ', '\t']).len()
}

fn is_blank_or_comment(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.is_empty() || trimmed.starts_with('#')
}

fn strip_comment(value: &str) -> &str {
    value
        .char_indices()
        .find(|&(index, char)| char == '#' && value[..index].ends_with([' ', '\t']))
        .map_or(value, |(index, _)| &value[..index])
}

fn trim_blank_start<'a>(lines: &[&'a str]) -> Vec<&'a str> {
    lines
        .iter()
        .skip_while(|line| line.trim().is_empty())
        .copied()
        .collect()
}

fn print(root: &Path, changes: &Changes) {
    let shown = |path: &Path| {
        path.strip_prefix(root)
            .unwrap_or(path)
            .display()
            .to_string()
    };
    for conflict in &changes.conflicts {
        if conflict.starts_with("Aviso:") {
            println!("{conflict}");
            continue;
        }
        println!(
            "Aviso: a skill customizada '{conflict}' tem o mesmo nome de uma skill padrão; a customizada foi preservada e os redirecionamentos apontam para ela."
        );
    }
    if changes.is_empty() {
        println!(
            "O harness já está atualizado (versão {}).",
            manifest::VERSION
        );
        return;
    }
    for path in &changes.updated {
        println!("Atualizado: {}", shown(path));
    }
    for path in &changes.recreated {
        println!("Recriado: {}", shown(path));
    }
    for path in &changes.removed {
        println!("Removido: {}", shown(path));
    }
    for path in &changes.kept_locals {
        println!("Versão local guardada: {}", shown(path));
    }
    for key in &changes.added_keys {
        println!("Configuração acrescentada: {key}");
    }
    for key in &changes.removed_keys {
        println!("Configuração removida: {key}");
    }
    for path in &changes.removed_dirs {
        println!("Pasta removida: {}", shown(path));
    }
    for path in &changes.routing {
        println!("Roteamento regenerado: {}", shown(path));
    }
    for path in &changes.agents {
        println!("Subagente regenerado: {}", shown(path));
    }
    for (label, path) in &changes.hooks {
        println!("Hooks de {label} atualizados: {}", shown(path));
    }
    println!("Harness atualizado para a versão {}.", manifest::VERSION);
}

/// Garantia do contexto por ferramenta de `harness.tool`: rótulo e arquivo de hooks. Sem arquivo,
/// a ferramenta espera um formato que os hooks do harness ainda não falam (lacuna avisada).
const TOOL_HOOKS: &[(&str, &str, Option<&str>)] = &[
    ("claude-code", "Claude Code", Some(".claude/settings.json")),
    ("codex", "Codex", Some(".codex/hooks.json")),
    ("cursor", "Cursor", None),
    ("gemini-cli", "Gemini CLI", None),
    ("copilot", "GitHub Copilot", None),
];

/// Hooks que o harness mantém no formato do Claude Code, que o Codex também usa: evento, matcher
/// e comando (SPEC-050).
const CLAUDE_HOOKS: &[(&str, &str, &str)] = &[
    (
        "SessionStart",
        "startup|resume|clear|compact",
        "vostok hook session-start",
    ),
    ("PreToolUse", "Bash", "vostok hook pre-bash"),
];

pub(crate) enum ToolHooks {
    Skipped,
    Unchanged,
    Written(&'static str, PathBuf),
    Invalid(PathBuf),
    Gap(&'static str),
}

/// Mescla os hooks do harness no arquivo da ferramenta de `harness.tool`, sem tocar nas chaves e
/// nos hooks do usuário e sem duplicar os do harness.
pub(crate) fn write_tool_hooks(harness: &Harness) -> Result<ToolHooks> {
    let Some(tool) = crate::models::harness_tool(harness) else {
        return Ok(ToolHooks::Skipped);
    };
    let Some(&(_, label, file)) = TOOL_HOOKS.iter().find(|(name, ..)| *name == tool) else {
        return Ok(ToolHooks::Skipped);
    };
    let Some(file) = file else {
        return Ok(ToolHooks::Gap(label));
    };
    let path = routing::project_root(harness)?.join(file);
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => "{}".to_string(),
        Err(error) => {
            return Err(error).with_context(|| format!("não foi possível ler {}", path.display()))
        }
    };
    let Ok(serde_json::Value::Object(mut settings)) = serde_json::from_str(&content) else {
        return Ok(ToolHooks::Invalid(path));
    };
    if !add_claude_hooks(&mut settings) {
        return Ok(ToolHooks::Unchanged);
    }
    let mut text = serde_json::to_string_pretty(&serde_json::Value::Object(settings))?;
    text.push('\n');
    write(&path, text.as_bytes())?;
    Ok(ToolHooks::Written(label, path))
}

/// Acrescenta os hooks do harness que faltam e diz se acrescentou algum. `hooks` ou uma lista de
/// evento que não seja do tipo esperado é trocada, porque o Claude Code não a leria.
fn add_claude_hooks(settings: &mut serde_json::Map<String, serde_json::Value>) -> bool {
    let hooks = settings
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}));
    if !hooks.is_object() {
        *hooks = serde_json::json!({});
    }
    let hooks = hooks.as_object_mut().expect("hooks é objeto");
    let mut added = false;
    for (event, matcher, command) in CLAUDE_HOOKS {
        let entries = hooks.entry(*event).or_insert_with(|| serde_json::json!([]));
        if !entries.is_array() {
            *entries = serde_json::json!([]);
        }
        let entries = entries.as_array_mut().expect("lista de hooks");
        if !entries.iter().any(|entry| has_command(entry, command)) {
            entries.push(serde_json::json!({
                "matcher": matcher,
                "hooks": [{ "type": "command", "command": command }],
            }));
            added = true;
        }
    }
    added
}

fn has_command(entry: &serde_json::Value, command: &str) -> bool {
    entry["hooks"]
        .as_array()
        .is_some_and(|hooks| hooks.iter().any(|hook| hook["command"] == command))
}

pub(crate) fn gap_warning(label: &str) -> String {
    format!(
        "Aviso: {label} não tem como receber o contexto obrigatório automaticamente; o agente depende do AGENTS.md (docs/estudos/contexto-por-ferramenta.md)."
    )
}

pub(crate) fn invalid_settings_warning(root: &Path, path: &Path) -> String {
    format!(
        "Aviso: {} não é um JSON válido; os hooks do harness não foram gravados.",
        path.strip_prefix(root).unwrap_or(path).display()
    )
}

fn write(path: &Path, content: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("não foi possível criar {}", parent.display()))?;
    }
    fs::write(path, content).with_context(|| format!("não foi possível gravar {}", path.display()))
}

fn remove_file(path: &Path) -> Result<()> {
    fs::remove_file(path).with_context(|| format!("não foi possível apagar {}", path.display()))
}

/// Apaga a pasta se ela estiver vazia e diz se apagou.
fn remove_empty_dir(dir: &Path) -> Result<bool> {
    let mut entries =
        fs::read_dir(dir).with_context(|| format!("não foi possível ler {}", dir.display()))?;
    if entries.next().is_some() {
        return Ok(false);
    }
    fs::remove_dir(dir).with_context(|| format!("não foi possível apagar {}", dir.display()))?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_keys_adds_the_executed_status() {
        let installed = "status:\n  pending: pending\n  in_progress: in_progress\n  done: done\n\nmax_attempts: 5\n";
        let shipped = "status:\n  pending: pending\n  in_progress: in_progress\n  executed: executed\n  done: done\n\nmax_attempts: 2\n";
        let (merged, added) = merge_keys(installed, shipped);
        let status_block: Vec<&str> = merged
            .lines()
            .skip_while(|line| *line != "status:")
            .skip(1)
            .take_while(|line| line.starts_with("  "))
            .collect();
        assert!(status_block.contains(&"  executed: executed"), "{merged}");
        assert!(
            status_block.contains(&"  in_progress: in_progress"),
            "{merged}"
        );
        assert!(merged.contains("\nmax_attempts: 5\n"), "{merged}");
        assert!(!merged.contains("max_attempts: 2"), "{merged}");
        assert_eq!(added, vec!["status.executed".to_string()]);
    }
}
