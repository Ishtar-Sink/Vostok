//! Comando `update`: regrava no projeto os arquivos que vêm do binário e preserva os do projeto
//! (RF-09 a RF-12, RN-01, RN-02).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::embedded;
use crate::harness::Harness;
use crate::manifest::{self, Manifest};
use crate::routing;
use crate::support::{config_list, sha256_hex};

const CONFIG_DIR: &str = "config/";

/// Itens que o contexto obrigatório precisa ter desde a v0.2.0 (RF-25).
const REQUIRED_CONTEXT: &[&str] = &["personas/coordenador.md"];

/// O que a atualização mudou, na ordem em que aparece na saída.
#[derive(Debug, Default)]
struct Changes {
    updated: Vec<PathBuf>,
    recreated: Vec<PathBuf>,
    kept_locals: Vec<PathBuf>,
    added_keys: Vec<String>,
    routing: Vec<PathBuf>,
    conflicts: Vec<String>,
    manifest: bool,
}

impl Changes {
    fn is_empty(&self) -> bool {
        self.updated.is_empty()
            && self.recreated.is_empty()
            && self.kept_locals.is_empty()
            && self.added_keys.is_empty()
            && self.routing.is_empty()
            && !self.manifest
    }
}

pub fn run() -> Result<()> {
    let harness = Harness::from_current_dir()?;
    let previous = Manifest::read(&harness)?;
    let current = Manifest::current(&harness)?;
    let mut changes = Changes::default();

    update_binary_files(&harness, &previous, &current, &mut changes)?;
    merge_configs(&harness, &mut changes)?;
    ensure_required_context(&harness, &mut changes)?;
    let routing = routing::generate(&harness)?;
    changes.routing = routing.changed;
    changes.kept_locals.extend(routing.kept_locals);
    changes.conflicts = routing.conflicts;
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
    let path = harness.root.join("config/context.yml");
    let Ok(content) = fs::read_to_string(&path) else {
        return Ok(());
    };
    let current = config_list(&path, "required").unwrap_or_default();
    let mut lines: Vec<String> = content.lines().map(str::to_string).collect();
    let mut added = Vec::new();
    for item in REQUIRED_CONTEXT {
        if current.iter().any(|existing| existing == item) {
            continue;
        }
        let Some(header) = lines
            .iter()
            .position(|line| line.trim_end() == "required:" || line.trim_end() == "required: []")
        else {
            continue;
        };
        if lines[header].trim_end() == "required: []" {
            lines[header] = "required:".to_string();
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
        added.push(format!("config/context.yml: required.{item}"));
    }
    if added.is_empty() {
        return Ok(());
    }
    let mut merged = lines.join("\n");
    merged.push('\n');
    write(&path, merged.as_bytes())?;
    changes.added_keys.extend(added);
    Ok(())
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
    for path in &changes.kept_locals {
        println!("Versão local guardada: {}", shown(path));
    }
    for key in &changes.added_keys {
        println!("Configuração acrescentada: {key}");
    }
    for path in &changes.routing {
        println!("Roteamento regenerado: {}", shown(path));
    }
    println!("Harness atualizado para a versão {}.", manifest::VERSION);
}

fn write(path: &Path, content: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("não foi possível criar {}", parent.display()))?;
    }
    fs::write(path, content).with_context(|| format!("não foi possível gravar {}", path.display()))
}
