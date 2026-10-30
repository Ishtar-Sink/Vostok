//! Comandos `context`: carregamento de contexto para o agente.
//! Reproduzem `load-context.sh`, `load-extra-context.sh` e `lib/context.sh`. Com `--persona`, o
//! contexto carregado é seguido das personas pedidas.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Subcommand;

use crate::harness::Harness;
use crate::support::{config_get, config_list, exit_failure, fail, is_kebab_case};

const PERSONAS_DIR: &str = "personas";
const PERSONA_USAGE: &str = "Uso: vostok context load --persona <nome>";
const PERSONA_WITH_PATHS: &str = "Uso: vostok context load [--skill <nome>] [--persona <nome>]...; --persona não se combina com caminhos";

#[derive(Debug, Subcommand)]
pub enum ContextCommand {
    /// Carrega o contexto obrigatório; com caminhos ou `--skill`, o contexto adicional; com
    /// `--persona`, também a persona.
    Load {
        /// Skill cuja entrada do mapa `skills` do `config/context.yml` é carregada.
        #[arg(long)]
        skill: Option<String>,
        /// Persona de `personas/` carregada depois do contexto; repetível, na ordem dada.
        #[arg(long = "persona", value_name = "NOME")]
        personas: Vec<String>,
        /// Arquivos ou pastas, relativos à raiz do harness.
        paths: Vec<String>,
    },
}

pub fn run(command: ContextCommand) -> Result<()> {
    let harness = Harness::from_current_dir()?;
    let ContextCommand::Load {
        skill,
        personas,
        paths,
    } = command;
    if !personas.is_empty() && !paths.is_empty() {
        fail(PERSONA_WITH_PATHS);
    }
    let persona_files = persona_files(&harness, &personas);
    let printer = Printer::new(&harness);
    let complete = match skill {
        Some(skill) => load_skill(&harness, &printer, &skill)?,
        None if paths.is_empty() => load_required(&harness, &printer)?,
        None => load_paths(&printer, &paths)?,
    };
    if complete && !persona_files.is_empty() {
        printer.header("Persona");
        for file in &persona_files {
            printer.path(file);
        }
    }
    if !complete {
        exit_failure();
    }
    Ok(())
}

/// Caminhos absolutos das personas pedidas, na ordem dada; recusa nome vazio ou inexistente.
fn persona_files(harness: &Harness, names: &[String]) -> Vec<String> {
    let root = harness.root.to_string_lossy().into_owned();
    names
        .iter()
        .map(|name| {
            if name.is_empty() {
                fail(PERSONA_USAGE);
            }
            let path = format!("{root}/{PERSONAS_DIR}/{name}.md");
            if !is_kebab_case(name) || !Path::new(&path).is_file() {
                let mut available: Vec<String> =
                    fs::read_dir(harness.root.join(PERSONAS_DIR))
                        .into_iter()
                        .flatten()
                        .flatten()
                        .filter_map(|entry| {
                            let file = entry.file_name().to_string_lossy().into_owned();
                            file.strip_suffix(".md").map(str::to_string)
                        })
                        .collect();
                available.sort_unstable();
                if available.is_empty() {
                    fail(format!(
                        "Persona '{name}' não existe; nenhuma persona instalada em {PERSONAS_DIR}/; execute `vostok update`"
                    ));
                }
                fail(format!(
                    "Persona '{name}' não existe; as disponíveis são: {}",
                    available.join(", ")
                ));
            }
            path
        })
        .collect()
}

/// Contexto obrigatório: cada item de `required`. Um item ausente só gera aviso.
fn load_required(harness: &Harness, printer: &Printer) -> Result<bool> {
    let sources = config_list(&harness.config_file("context.yml"), "required")?;
    printer.header("Contexto obrigatório do harness");
    for source in sources.iter().filter(|source| !source.is_empty()) {
        if !printer.path(&format!("{}/{source}", printer.root)) {
            eprintln!("Aviso: '{source}' não encontrado em {}", printer.root);
        }
    }
    Ok(true)
}

/// Contexto adicional: os caminhos pedidos, mesmo depois de um ausente.
fn load_paths(printer: &Printer, paths: &[String]) -> Result<bool> {
    printer.header("Contexto adicional");
    let mut complete = true;
    for source in paths {
        if !printer.path(&printer.resolve(source)?) {
            eprintln!("Aviso: '{source}' não encontrado");
            complete = false;
        }
    }
    Ok(complete)
}

/// Contexto adicional de uma skill: os itens da entrada dela no mapa `skills`.
fn load_skill(harness: &Harness, printer: &Printer, skill: &str) -> Result<bool> {
    if skill.is_empty() {
        fail("Uso: vostok context load --skill <nome>");
    }
    let skills_config = harness.config_file("skills.yml");
    let exists = ["dirs.default", "dirs.custom"]
        .iter()
        .try_fold(false, |found, key| {
            let dir = config_get(&skills_config, key)?;
            Ok::<_, anyhow::Error>(
                found || harness.resolve(&dir).join(skill).join("SKILL.md").is_file(),
            )
        })?;
    if !exists {
        fail(format!("Skill '{skill}' não existe"));
    }
    let map = fs::read_to_string(harness.config_file("context.yml"))?;
    printer.header("Contexto adicional");
    let mut complete = true;
    for source in skill_entries(&map, skill) {
        if !printer.path(&printer.resolve(&source)?) {
            eprintln!("skills.{skill}: '{source}' não existe");
            complete = false;
        }
    }
    Ok(complete)
}

/// Itens da entrada `skills.<skill>` do mapa, na ordem do arquivo.
fn skill_entries(map: &str, skill: &str) -> Vec<String> {
    let mut in_skills = false;
    let mut in_entry = false;
    let mut entries = Vec::new();
    for line in map.lines() {
        if is_blank_or_comment(line) {
            continue;
        }
        if !line.starts_with([' ', '\t']) {
            in_skills = is_skills_header(line);
            in_entry = false;
        } else if !in_skills {
            continue;
        } else if is_map_item(line) {
            if in_entry {
                entries.push(map_item(line));
            }
        } else {
            in_entry = map_key(line.trim_start_matches([' ', '\t'])) == skill;
        }
    }
    entries
}

fn is_skills_header(line: &str) -> bool {
    line.strip_prefix("skills:").is_some_and(|rest| {
        let rest = rest.trim_start_matches([' ', '\t']);
        rest.is_empty() || rest.starts_with('#')
    })
}

/// Linha vazia ou só de comentário: `^[ \t]*(#|$)`.
pub(crate) fn is_blank_or_comment(line: &str) -> bool {
    let trimmed = line.trim_start_matches([' ', '\t']);
    trimmed.is_empty() || trimmed.starts_with('#')
}

/// Item de lista indentado: `^[ \t]+-[ \t]`.
pub(crate) fn is_map_item(line: &str) -> bool {
    let trimmed = line.trim_start_matches([' ', '\t']);
    trimmed.len() < line.len() && is_dash_item(trimmed)
}

/// Item de lista com ou sem indentação: `^[ \t]*-[ \t]`.
pub(crate) fn is_any_item(line: &str) -> bool {
    is_dash_item(line.trim_start_matches([' ', '\t']))
}

fn is_dash_item(trimmed: &str) -> bool {
    trimmed.starts_with("- ") || trimmed.starts_with("-\t")
}

/// Chave de uma linha `chave: ...`: o texto antes do primeiro `:`, sem espaços no fim.
pub(crate) fn map_key(line: &str) -> &str {
    line.split(':')
        .next()
        .unwrap_or("")
        .trim_end_matches([' ', '\t'])
}

/// Valor de um item de lista do mapa, sem o traço, o comentário e as aspas.
pub(crate) fn map_item(line: &str) -> String {
    let trimmed = line.trim_start_matches([' ', '\t']);
    let value = trimmed
        .strip_prefix('-')
        .unwrap_or(trimmed)
        .trim_start_matches([' ', '\t']);
    let value = strip_trailing_comment(value).trim_end_matches([' ', '\t']);
    let quoted = value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')));
    if quoted {
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    }
}

/// Remove `[ \t]+#.*$`: o primeiro `#` precedido de espaço, com os espaços antes dele.
pub(crate) fn strip_trailing_comment(value: &str) -> &str {
    let bytes = value.as_bytes();
    let Some(hash) = (1..bytes.len())
        .find(|&index| bytes[index] == b'#' && matches!(bytes[index - 1], b' ' | b'\t'))
    else {
        return value;
    };
    value[..hash].trim_end_matches([' ', '\t'])
}

/// Impressão dos arquivos de contexto, com os caminhos relativos à raiz do harness.
struct Printer {
    root: String,
}

impl Printer {
    fn new(harness: &Harness) -> Self {
        Self {
            root: harness.root.to_string_lossy().into_owned(),
        }
    }

    fn header(&self, title: &str) {
        println!("# {title}");
        println!();
        println!("Raiz do harness: `{}`", self.root);
        println!("Caminhos relativos citados em cada arquivo partem da pasta daquele arquivo.");
    }

    /// Caminho absoluto, relativo à raiz do harness ou, se não existir lá, ao diretório atual.
    fn resolve(&self, source: &str) -> Result<String> {
        let source = source.strip_prefix("./").unwrap_or(source);
        if Path::new(source).is_absolute() {
            return Ok(source.to_string());
        }
        let in_harness = format!("{}/{source}", self.root);
        if Path::new(&in_harness).exists() {
            return Ok(in_harness);
        }
        Ok(format!(
            "{}/{source}",
            crate::support::current_dir()?.display()
        ))
    }

    /// Imprime um arquivo ou todos os arquivos de uma pasta. Falso se o caminho não existir.
    fn path(&self, path: &str) -> bool {
        let target = Path::new(path);
        if target.is_file() {
            self.file(path);
        } else if target.is_dir() {
            let mut files = Vec::new();
            collect_files(target, &mut files);
            let mut files: Vec<String> = files
                .iter()
                .map(|file| file.to_string_lossy().into_owned())
                .collect();
            files.sort_unstable();
            for file in files {
                self.file(&file);
            }
        } else {
            return false;
        }
        true
    }

    fn file(&self, path: &str) {
        let shown = path
            .strip_prefix(&format!("{}/", self.root))
            .unwrap_or(path);
        println!("\n--- arquivo: {shown} ---\n");
        let content = fs::read(path).unwrap_or_default();
        println!("{}", String::from_utf8_lossy(&content));
    }
}

/// Arquivos regulares de uma pasta e das subpastas, sem os ocultos (como o `find ! -name '.*'`).
fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if kind.is_dir() {
            collect_files(&entry.path(), files);
        } else if kind.is_file() && !entry.file_name().to_string_lossy().starts_with('.') {
            files.push(entry.path());
        }
    }
}
