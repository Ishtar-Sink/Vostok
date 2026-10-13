//! Comandos `memory`: memória do projeto: decisões, issues e conhecimento.
//! Reproduzem `add-memory.sh`, `search-memory.sh`, `update-memory.sh` e `lib/memory.sh`.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::Subcommand;

use crate::harness::Harness;
use crate::support::{
    config_block, config_get, config_list, config_set, fail, frontmatter_get, join_records,
    records, render, script_title, template_leftovers, utc_date, utc_datetime, yaml_escape,
};

const ADD_USAGE: &str = "Uso: vostok memory add <tipo> \"Título\" | vostok memory init";
const SEARCH_USAGE: &str = "Uso: vostok memory search <tipo|all> [termo...] | --id <ID>";

#[derive(Debug, Subcommand)]
pub enum MemoryCommand {
    /// Cria os arquivos da memória que ainda não existem.
    Init,
    /// Acrescenta uma entrada.
    Add { kind: String, title: String },
    /// Busca entradas por tipo e termos, ou por ID.
    Search {
        #[arg(long)]
        id: Option<String>,
        kind: Option<String>,
        #[arg(allow_hyphen_values = true)]
        terms: Vec<String>,
    },
    /// Atualiza um campo de uma entrada.
    Update {
        id: String,
        field: String,
        value: String,
    },
}

pub fn run(command: MemoryCommand) -> Result<()> {
    let memory = Memory::new(Harness::from_current_dir()?)?;
    match command {
        MemoryCommand::Init => memory.init(),
        MemoryCommand::Add { kind, title } => memory.add(&kind, &title),
        MemoryCommand::Search { id: Some(id), .. } => memory.search_id(&id),
        MemoryCommand::Search {
            kind: Some(kind),
            terms,
            ..
        } => memory.search(&kind, &terms),
        MemoryCommand::Search { .. } => bail!(SEARCH_USAGE),
        MemoryCommand::Update { id, field, value } => memory.update(&id, &field, &value),
    }
}

/// A memória configurada em `config/memory.yml`.
struct Memory {
    harness: Harness,
    config: PathBuf,
    dir: PathBuf,
}

impl Memory {
    fn new(harness: Harness) -> Result<Self> {
        let config = harness.config_file("memory.yml");
        let dir = harness.path("agent_memory")?;
        Ok(Self {
            harness,
            config,
            dir,
        })
    }

    fn setting(&self, key: &str) -> Result<String> {
        config_get(&self.config, key)
    }

    fn types(&self) -> Result<Vec<String>> {
        config_list(&self.config, "types")
    }

    /// Encerra com a lista de tipos se o tipo não existir.
    fn require_type(&self, kind: &str) -> Result<()> {
        let types = self.types()?;
        if !types.iter().any(|known| known == kind) {
            let listed: String = types.iter().map(|known| format!("{known} ")).collect();
            fail(format!(
                "Tipo de memória inválido '{kind}'. Tipos: {listed}"
            ));
        }
        Ok(())
    }

    fn file(&self, kind: &str) -> Result<PathBuf> {
        Ok(self.dir.join(self.setting(&format!("{kind}.file"))?))
    }

    /// Tipo cujo prefixo é o do ID (ex.: DEC-001 -> decision).
    fn type_for_id(&self, id: &str) -> Result<String> {
        let prefix = id.split('-').next().unwrap_or(id);
        for kind in self.types()? {
            if self.setting(&format!("{kind}.prefix"))? == prefix {
                return Ok(kind);
            }
        }
        fail(format!("Nenhum tipo de memória usa o prefixo do ID '{id}'"))
    }

    fn init(&self) -> Result<()> {
        let now = utc_datetime();
        for kind in self.types()? {
            if let Some(file) = self.init_file(&kind, &now)? {
                println!("Arquivo de memória criado: {}", file.display());
            }
        }
        Ok(())
    }

    /// Cria o arquivo de memória do tipo, se ainda não existir, e devolve o caminho criado.
    fn init_file(&self, kind: &str, now: &str) -> Result<Option<PathBuf>> {
        let file = self.file(kind)?;
        if file.exists() {
            return Ok(None);
        }
        let title = self.setting(&format!("{kind}.title"))?;
        let description = self.setting(&format!("{kind}.description"))?;
        let front_matter = config_block(&self.config, "file_front_matter")?;
        let front_matter = render(
            front_matter.trim_end_matches('\n'),
            &HashMap::from([
                ("MEMORY_TYPE", kind.to_string()),
                ("MEMORY_TITLE", yaml_escape(&title)),
                ("MEMORY_DESCRIPTION", yaml_escape(&description)),
                ("DATETIME", now.to_string()),
            ]),
        );
        let template = fs::read_to_string(self.templates()?.join("memory/file.md"))?;
        let content = render_lines(
            &template,
            &HashMap::from([
                ("FRONT_MATTER", format!("---\n{front_matter}\n---")),
                ("MEMORY_TITLE", title),
                ("MEMORY_DESCRIPTION", description),
            ]),
        );
        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent)?;
        }
        write(&file, &content)?;
        Ok(Some(file))
    }

    fn templates(&self) -> Result<PathBuf> {
        self.harness.path("templates")
    }

    fn add(&self, kind: &str, title: &str) -> Result<()> {
        self.require_type(kind)?;
        let title = match script_title(title) {
            Ok(title) => title,
            Err(problem) => {
                eprintln!("{problem}");
                bail!("título inválido. {ADD_USAGE}");
            }
        };
        let now = utc_datetime();
        self.init_file(kind, &now)?;
        let file = self.file(kind)?;
        let shown = file.display();
        if frontmatter_get(&file, "updated_at").is_err() {
            bail!("{shown} não tem front-matter com 'updated_at'. Restaure o cabeçalho do arquivo (ver vostok memory init) antes de continuar");
        }
        let content = read(&file)?;
        let leftovers = template_leftovers(&content);
        if !leftovers.is_empty() {
            let listed: Vec<String> = leftovers
                .iter()
                .map(|(line, text)| format!("{line}:{text}"))
                .collect();
            bail!(
                "há uma entrada sem preencher em {shown}. Preencha-a antes de criar outra:\n{}",
                listed.join("\n")
            );
        }

        let sequences = self.harness.config_file("sequences.yml");
        let sequence_key = self.setting(&format!("{kind}.sequence"))?;
        let sequence = config_get(&sequences, &sequence_key)?;
        let number: u64 = match sequence.parse() {
            Ok(number) if sequence.bytes().all(|b| b.is_ascii_digit()) => number,
            _ => bail!(
                "{sequence_key} inválido em {}: '{sequence}'",
                sequences.display()
            ),
        };
        let digits: usize = self
            .setting(&format!("{kind}.digits"))?
            .parse()
            .unwrap_or(0);
        let prefix = self.setting(&format!("{kind}.prefix"))?;
        let entry_id = format!("{prefix}-{number:0digits$}");
        let heading = format!("## {entry_id}: ");
        if content.lines().any(|line| line.starts_with(&heading)) {
            bail!(
                "{entry_id} já existe em {shown}. Ajuste '{sequence_key}' em {} para o próximo número livre",
                sequences.display()
            );
        }

        let line = content.matches('\n').count() + 2;
        let template = fs::read_to_string(
            self.templates()?
                .join(self.setting(&format!("{kind}.template"))?),
        )?;
        let status = config_list(&self.config, &format!("{kind}_statuses"))?
            .into_iter()
            .next()
            .unwrap_or_default();
        let entry = render_lines(
            &template,
            &HashMap::from([
                ("ENTRY_ID", entry_id.clone()),
                ("ENTRY_TITLE", title),
                ("ENTRY_STATUS", status),
                ("DATE", utc_date()),
            ]),
        );
        write(&file, &format!("{content}\n{entry}"))?;
        set_updated_at(&file, &now)?;
        config_set(&sequences, &sequence_key, &(number + 1).to_string())?;
        println!("{entry_id} criada em {shown}:{line}. Preencha as variáveis #{{...}} da entrada.");
        Ok(())
    }

    /// Arquivos existentes dos tipos pedidos (`all` para todos).
    fn files_for(&self, kind: &str) -> Result<Vec<PathBuf>> {
        let kinds = if kind == "all" {
            self.types()?
        } else {
            self.require_type(kind)?;
            vec![kind.to_string()]
        };
        let mut files = Vec::new();
        for kind in kinds {
            let file = self.file(&kind)?;
            if file.is_file() {
                files.push(file);
            }
        }
        Ok(files)
    }

    fn search_id(&self, id: &str) -> Result<()> {
        if id.is_empty() {
            bail!(SEARCH_USAGE);
        }
        let kind = self.type_for_id(id)?;
        let heading = format!("## {id}:");
        let mut blocks = Vec::new();
        for file in self.files_for(&kind)? {
            for entry in entries(&read(&file)?) {
                if entry[0].starts_with(&heading) {
                    blocks.push(entry.join("\n"));
                }
            }
        }
        print_or_empty(&blocks.join("\n"));
        Ok(())
    }

    fn search(&self, kind: &str, terms: &[String]) -> Result<()> {
        let files = self.files_for(kind)?;
        let mut all = Vec::new();
        for file in &files {
            all.extend(entries(&read(file)?));
        }
        let output = if terms.is_empty() {
            index(&all)
        } else {
            let limit: usize = self.setting("search_max_results")?.parse().unwrap_or(0);
            by_terms(&all, terms, limit)
        };
        print_or_empty(&output);
        Ok(())
    }

    fn update(&self, id: &str, field: &str, value: &str) -> Result<()> {
        let kind = self.type_for_id(id)?;
        let file = self.file(&kind)?;
        if !file.is_file() {
            bail!("arquivo de memória não encontrado: {}", file.display());
        }
        match field {
            "Status" => self.require_allowed(&format!("{kind}_statuses"), value)?,
            "Categoria" => self.require_allowed(&format!("{kind}_categories"), value)?,
            _ => {}
        }
        let header = format!("## {id}:");
        let prefix = format!("- **{field}:** ");
        let mut inside = false;
        let mut found_entry = false;
        let mut found_field = false;
        let content = read(&file)?;
        let lines: Vec<String> = records(&content)
            .into_iter()
            .map(|line| {
                if line.starts_with(&header) {
                    inside = true;
                    found_entry = true;
                } else if line.starts_with("## ") {
                    inside = false;
                } else if inside && line.starts_with(&prefix) {
                    found_field = true;
                    return format!("{prefix}{value}");
                }
                line.to_string()
            })
            .collect();
        if !found_entry {
            bail!("entrada {id} não encontrada em {}", file.display());
        }
        if !found_field {
            bail!("a entrada {id} não tem o campo '{field}'");
        }
        write(&file, &join_records(&lines))?;
        set_updated_at(&file, &utc_datetime())?;
        println!("{id}: {field} = {value}");
        Ok(())
    }

    fn require_allowed(&self, list: &str, value: &str) -> Result<()> {
        let allowed = config_list(&self.config, list).unwrap_or_else(|error| fail(error));
        if !allowed.iter().any(|item| item == value) {
            bail!("valor inválido '{value}'. Aceitos: {}", allowed.join(", "));
        }
        Ok(())
    }
}

/// Entradas `## <PREFIXO>-<número>: <título>` de um arquivo, fora do front-matter, linha a linha
/// (sem `\r`), até a próxima entrada ou o fim do arquivo.
fn entries(content: &str) -> Vec<Vec<String>> {
    let mut entries = Vec::new();
    let mut current: Option<Vec<String>> = None;
    let mut in_front_matter = false;
    for (index, raw) in records(content).into_iter().enumerate() {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        if index == 0 && line == "---" {
            in_front_matter = true;
            continue;
        }
        if in_front_matter {
            in_front_matter = line != "---";
            continue;
        }
        if is_entry_heading(line) {
            entries.extend(current.take());
            current = Some(vec![line.to_string()]);
        } else if let Some(entry) = current.as_mut() {
            entry.push(line.to_string());
        }
    }
    entries.extend(current);
    entries
}

/// `^## [A-Z]+-[0-9]+: `
fn is_entry_heading(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("## ") else {
        return false;
    };
    let letters = rest.bytes().take_while(u8::is_ascii_uppercase).count();
    let Some(rest) = rest[letters..].strip_prefix('-') else {
        return false;
    };
    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
    letters > 0 && digits > 0 && rest[digits..].starts_with(": ")
}

fn field<'a>(entry: &'a [String], name: &str) -> &'a str {
    let prefix = format!("- **{name}:** ");
    entry
        .iter()
        .find_map(|line| line.strip_prefix(&prefix))
        .unwrap_or("")
}

/// Uma linha por entrada: ID e título, status e tags.
fn index(entries: &[Vec<String>]) -> String {
    entries
        .iter()
        .map(|entry| {
            let title = entry[0].strip_prefix("## ").unwrap_or(&entry[0]);
            format!(
                "{title} | {} | {}\n",
                field(entry, "Status"),
                field(entry, "Tags")
            )
        })
        .collect()
}

/// Entradas com ao menos um termo, das que têm mais termos para as que têm menos, até `limit`.
fn by_terms(entries: &[Vec<String>], terms: &[String], limit: usize) -> String {
    let terms: Vec<String> = terms
        .iter()
        .flat_map(|term| term.split('\n'))
        .map(normalize)
        .filter(|term| !term.is_empty())
        .collect();
    let mut found: Vec<(usize, String)> = entries
        .iter()
        .map(|entry| entry.join("\n"))
        .filter_map(|block| {
            let text = normalize(&block);
            let score = terms
                .iter()
                .filter(|term| text.contains(term.as_str()))
                .count();
            (score > 0).then_some((score, block))
        })
        .collect();
    found.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
    let shown = found.len().min(limit);
    let mut output = String::new();
    for (position, (score, block)) in found.iter().take(shown).enumerate() {
        if position > 0 {
            output.push('\n');
        }
        output.push_str(&format!(
            "> Termos encontrados: {score} de {}\n\n{block}\n",
            terms.len()
        ));
    }
    if found.len() > shown {
        output.push_str(&format!(
            "\nEntradas omitidas: {}. Use termos mais específicos para refinar a busca.\n",
            found.len() - shown
        ));
    }
    output
}

/// Minúsculas e sem acentos, para a busca não depender da acentuação.
fn normalize(text: &str) -> String {
    text.chars()
        .flat_map(|c| {
            let folded = match c {
                'á' | 'à' | 'â' | 'ã' | 'ä' | 'Á' | 'À' | 'Â' | 'Ã' | 'Ä' => 'a',
                'é' | 'è' | 'ê' | 'ë' | 'É' | 'È' | 'Ê' | 'Ë' => 'e',
                'í' | 'ì' | 'î' | 'ï' | 'Í' | 'Ì' | 'Î' | 'Ï' => 'i',
                'ó' | 'ò' | 'ô' | 'õ' | 'ö' | 'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' => 'o',
                'ú' | 'ù' | 'û' | 'ü' | 'Ú' | 'Ù' | 'Û' | 'Ü' => 'u',
                'ç' | 'Ç' => 'c',
                'ñ' | 'Ñ' => 'n',
                other => other,
            };
            folded.to_lowercase()
        })
        .collect()
}

/// Imprime o resultado sem as quebras de linha finais, ou o aviso de busca vazia.
fn print_or_empty(output: &str) {
    let output = output.trim_end_matches('\n');
    if output.is_empty() {
        println!("Nenhuma entrada encontrada.");
    } else {
        println!("{output}");
    }
}

/// `render` linha a linha, como o `awk` dos scripts: toda linha termina com `\n`.
fn render_lines(template: &str, values: &HashMap<&str, String>) -> String {
    join_records(&records(&render(template, values)))
}

/// Troca o `updated_at` do front-matter preservando o `\r` das outras linhas.
fn set_updated_at(path: &Path, value: &str) -> Result<()> {
    let content = read(path)?;
    let mut inside = false;
    let mut found = false;
    let lines: Vec<String> = records(&content)
        .into_iter()
        .enumerate()
        .map(|(index, line)| {
            let plain = line.strip_suffix('\r').unwrap_or(line);
            if index == 0 && plain == "---" {
                inside = true;
            } else if inside && plain == "---" {
                inside = false;
            } else if inside && line.starts_with("updated_at:") {
                found = true;
                return format!("updated_at: {value}");
            }
            line.to_string()
        })
        .collect();
    if !found {
        fail(format!(
            "Chave 'updated_at' não encontrada no front-matter de {}",
            path.display()
        ));
    }
    write(path, &join_records(&lines))
}

fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("não foi possível ler {}", path.display()))
}

fn write(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content).with_context(|| format!("não foi possível gravar {}", path.display()))
}
