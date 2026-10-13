//! Comandos `skill`: criação de skills no formato padrão, redirecionamentos para as ferramentas e
//! catálogo. Tudo o que é configurável fica em `config/skills.yml`.
//!
//! `write_redirects` e `update_catalog` não dependem da linha de comando, para o `vostok update`
//! regenerar redirecionamentos e catálogo das skills instaladas (RF-12, RF-25).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};
use clap::Subcommand;

use crate::harness::Harness;
use crate::support::{
    config_get, config_list, frontmatter_get, frontmatter_lines, render, template_leftovers,
    yaml_escape,
};

// Limites da especificação de Agent Skills (agentskills.io) e recomendação da Anthropic para o
// SKILL.md continuar eficiente no contexto do agente.
const MAX_NAME_LENGTH: usize = 64;
const MAX_DESCRIPTION_LENGTH: usize = 1024;
const MAX_SKILL_LINES: usize = 500;
const RESERVED_WORDS: [&str; 2] = ["anthropic", "claude"];
/// Chaves de `metadata` com caminhos (relativos à raiz do harness) que precisam existir.
const METADATA_PATH_KEYS: [&str; 1] = ["templates"];
const SKILL_FILE: &str = "SKILL.md";

#[derive(Debug, Subcommand)]
pub enum SkillCommand {
    /// Cria o SKILL.md de uma skill nova.
    Create {
        /// Cria a skill entre as padrão do harness.
        #[arg(long)]
        default: bool,
        name: String,
    },
    /// Valida a skill e gera redirecionamentos e catálogo.
    Finalize { name: String },
}

/// Pasta de skills: as que vêm com o harness ou as criadas pelo usuário.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Default,
    Custom,
}

impl Scope {
    pub const ALL: [Scope; 2] = [Scope::Default, Scope::Custom];

    /// Chave da pasta em `dirs` e da seção em `catalog_sections` do `config/skills.yml`.
    pub fn key(self) -> &'static str {
        match self {
            Scope::Default => "default",
            Scope::Custom => "custom",
        }
    }
}

pub fn run(command: SkillCommand) -> Result<()> {
    let harness = Harness::from_current_dir()?;
    match command {
        SkillCommand::Create { default, name } => {
            let scope = if default {
                Scope::Default
            } else {
                Scope::Custom
            };
            create(&harness, scope, &name)
        }
        SkillCommand::Finalize { name } => finalize(&harness, &name),
    }
}

fn create(harness: &Harness, scope: Scope, name: &str) -> Result<()> {
    validate_name(name)?;
    if let Some(existing) = find_scope(harness, name)? {
        bail!(
            "a skill '{name}' já existe em {}",
            skill_dir(harness, existing, name)?.display()
        );
    }
    let file = skill_dir(harness, scope, name)?.join(SKILL_FILE);
    let values = HashMap::from([
        ("SKILL_NAME", name.to_string()),
        ("AUTHOR", yaml_escape(&author(harness))),
    ]);
    write_new(&file, &render_template(harness, "template", &values)?)?;
    add_context_entry(harness, name)?;
    println!("Skill '{name}' criada em {}", file.display());
    Ok(())
}

fn validate_name(name: &str) -> Result<()> {
    if !is_kebab_case(name) {
        bail!(
            "nome inválido '{name}': use minúsculas, números e hífens simples (ex.: review-code)"
        );
    }
    if name.len() > MAX_NAME_LENGTH {
        bail!("o nome '{name}' passa de {MAX_NAME_LENGTH} caracteres");
    }
    if RESERVED_WORDS.iter().any(|word| name.contains(word)) {
        bail!("o nome não pode conter as palavras reservadas 'anthropic' ou 'claude'");
    }
    Ok(())
}

/// Minúsculas, números e hífens simples, sem hífen no início ou no fim.
fn is_kebab_case(name: &str) -> bool {
    name.split('-').all(|part| {
        !part.is_empty()
            && part
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    })
}

fn author(harness: &Harness) -> String {
    Command::new("git")
        .arg("-C")
        .arg(&harness.root)
        .args(["config", "user.name"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .trim_end_matches('\n')
                .to_string()
        })
        .unwrap_or_else(|| std::env::var("USER").unwrap_or_default())
}

/// Acrescenta `  <nome>: []` ao bloco `skills:` do mapa de contexto, ou cria o bloco se não existir.
fn add_context_entry(harness: &Harness, name: &str) -> Result<()> {
    let path = harness.config_file("context.yml");
    if !path.is_file() {
        bail!("mapa de contexto não encontrado: {}", path.display());
    }
    let content = read(&path)?;
    let lines = split_lines(&content);
    let entry = format!("  {name}: []");
    let in_skills = skills_block_flags(&lines);
    let prefix = format!("  {name}:");
    if lines
        .iter()
        .zip(&in_skills)
        .any(|(line, inside)| *inside && line.starts_with(&prefix))
    {
        return Ok(());
    }
    let updated = if lines.iter().any(|line| line.starts_with("skills:")) {
        // Entra depois da última linha de conteúdo do bloco, antes de comentários e linhas em branco do fim.
        let last = lines
            .iter()
            .zip(&in_skills)
            .rposition(|(line, inside)| *inside && !is_blank_or_comment(line));
        let mut out: Vec<&str> = Vec::with_capacity(lines.len() + 1);
        for (index, line) in lines.iter().enumerate() {
            out.push(line);
            if Some(index) == last {
                out.push(&entry);
            }
        }
        join_lines(&out)
    } else {
        format!("{content}\n# Regras e arquivos carregados por cada skill (`vostok context load --skill <nome>`).\nskills:\n{entry}\n")
    };
    fs::write(&path, updated)
        .with_context(|| format!("não foi possível gravar {}", path.display()))?;
    println!("Mapa de contexto atualizado: {}", path.display());
    Ok(())
}

/// Para cada linha, se ela está no bloco de primeiro nível `skills:`.
fn skills_block_flags(lines: &[&str]) -> Vec<bool> {
    let mut inside = false;
    lines
        .iter()
        .map(|line| {
            if starts_top_level_key(line) {
                inside = line.starts_with("skills:");
            }
            inside
        })
        .collect()
}

fn starts_top_level_key(line: &str) -> bool {
    line.chars()
        .next()
        .is_some_and(|first| !matches!(first, ' ' | '\t' | '#'))
}

fn is_blank_or_comment(line: &str) -> bool {
    let trimmed = line.trim_start_matches([' ', '\t']);
    trimmed.is_empty() || trimmed.starts_with('#')
}

fn finalize(harness: &Harness, name: &str) -> Result<()> {
    let scope = require_scope(harness, name)?;
    let dir = skill_dir(harness, scope, name)?;
    let errors = skill_errors(harness, name, &dir)?;
    if !errors.is_empty() {
        bail!("a skill '{name}' tem problemas:\n{}", errors.join("\n"));
    }
    let file = dir.join(SKILL_FILE);
    for target in write_redirects(harness, name, &file)? {
        println!("Redirecionamento: {}", target.display());
    }
    let catalog = update_catalog(harness, name, scope, &file)?;
    println!("Catálogo atualizado: {}", catalog.display());
    println!("Skill '{name}' finalizada.");
    Ok(())
}

fn require_scope(harness: &Harness, name: &str) -> Result<Scope> {
    find_scope(harness, name)?.ok_or_else(|| anyhow!("skill '{name}' não encontrada"))
}

fn skills_config(harness: &Harness) -> PathBuf {
    harness.config_file("skills.yml")
}

/// Pasta de uma skill no escopo indicado, exista ela ou não.
pub fn skill_dir(harness: &Harness, scope: Scope, name: &str) -> Result<PathBuf> {
    let dir = config_get(&skills_config(harness), &format!("dirs.{}", scope.key()))?;
    Ok(harness.resolve(&dir).join(name))
}

/// Escopo em que a skill existe (com `SKILL.md`), procurando primeiro entre as padrão.
pub fn find_scope(harness: &Harness, name: &str) -> Result<Option<Scope>> {
    for scope in Scope::ALL {
        if skill_dir(harness, scope, name)?.join(SKILL_FILE).is_file() {
            return Ok(Some(scope));
        }
    }
    Ok(None)
}

/// Problemas da skill, um por item; vazio se ela estiver válida.
pub fn skill_errors(harness: &Harness, name: &str, dir: &Path) -> Result<Vec<String>> {
    let file = dir.join(SKILL_FILE);
    let content = read(&file)?;
    let mut errors = leftover_errors(dir);

    if frontmatter_get(&file, "name").ok().as_deref() != Some(name) {
        errors.push(format!(
            "o campo 'name' do front-matter precisa ser '{name}', igual ao nome da pasta"
        ));
    }
    errors.extend(description_errors(
        &frontmatter_get(&file, "description").unwrap_or_default(),
    ));
    let lines = content.matches('\n').count();
    if lines > MAX_SKILL_LINES {
        errors.push(format!("o SKILL.md tem {lines} linhas; o máximo é {MAX_SKILL_LINES} (mova detalhes para arquivos de referência)"));
    }
    let front_matter = frontmatter_lines(&content).unwrap_or_default();
    for key in METADATA_PATH_KEYS {
        let paths = nested_value(&front_matter, "metadata", key).unwrap_or_default();
        for path in paths.split_whitespace() {
            if !Path::new(&format!("{}/{path}", harness.root.display())).exists() {
                errors.push(format!(
                    "metadata.{key}: '{path}' não existe na raiz do harness"
                ));
            }
        }
    }
    for path in context_paths(harness, name)? {
        if !harness.resolve(&path).exists() {
            errors.push(format!("skills.{name}: '{path}' não existe"));
        }
    }
    Ok(errors)
}

fn description_errors(description: &str) -> Vec<String> {
    let mut errors = Vec::new();
    if description.is_empty() {
        errors.push("o campo 'description' está vazio ou ausente".to_string());
    }
    let length = description.chars().count();
    if length > MAX_DESCRIPTION_LENGTH {
        errors.push(format!(
            "o campo 'description' tem {length} caracteres; o máximo é {MAX_DESCRIPTION_LENGTH}"
        ));
    }
    if has_xml_tag(description) {
        errors.push("o campo 'description' não pode conter tags XML".to_string());
    }
    errors
}

fn has_xml_tag(text: &str) -> bool {
    text.as_bytes()
        .windows(2)
        .any(|pair| pair[0] == b'<' && (pair[1].is_ascii_alphabetic() || pair[1] == b'/'))
}

/// Variáveis de template sem preencher em todos os arquivos de texto da pasta da skill.
fn leftover_errors(dir: &Path) -> Vec<String> {
    let mut files = Vec::new();
    collect_files(dir, &mut files);
    let mut errors = Vec::new();
    for file in files {
        let Some(content) = fs::read(&file)
            .ok()
            .filter(|bytes| !bytes.contains(&0))
            .and_then(|bytes| String::from_utf8(bytes).ok())
        else {
            continue;
        };
        let relative = file.strip_prefix(dir).unwrap_or(&file).display();
        for (line, text) in template_leftovers(&content) {
            errors.push(format!("variável sem preencher: {relative}:{line}:{text}"));
        }
    }
    errors
}

/// Arquivos da pasta, em profundidade, com as entradas em ordem alfabética sem distinguir maiúsculas.
fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|entry| entry.path()).collect();
    paths.sort_by_key(|path| {
        path.file_name()
            .map(|name| name.to_string_lossy().to_lowercase())
    });
    for path in paths {
        if path.is_dir() {
            collect_files(&path, files);
        } else if path.is_file() {
            files.push(path);
        }
    }
}

/// Valor de `<pai>.<chave>` num YAML de dois níveis, sem aspas nem comentário.
fn nested_value(lines: &[&str], parent: &str, key: &str) -> Option<String> {
    let mut top = "";
    for raw in lines {
        if is_blank_or_comment(raw) {
            continue;
        }
        let line = raw.trim_start_matches(' ');
        let name = line.split(':').next().unwrap_or_default().trim_end();
        if line.len() == raw.len() {
            top = name;
            continue;
        }
        if top == parent && name == key {
            if let Some((_, value)) = line.split_once(':') {
                return Some(unquote(strip_comment(value).trim()).to_string());
            }
        }
    }
    None
}

fn strip_comment(value: &str) -> &str {
    let bytes = value.as_bytes();
    (1..bytes.len())
        .find(|&index| bytes[index] == b'#' && matches!(bytes[index - 1], b' ' | b'\t'))
        .map_or(value, |index| &value[..index])
}

fn unquote(value: &str) -> &str {
    ['"', '\'']
        .iter()
        .find_map(|quote| {
            value
                .strip_prefix(*quote)
                .and_then(|rest| rest.strip_suffix(*quote))
        })
        .unwrap_or(value)
}

/// Caminhos listados para a skill no bloco `skills:` do mapa de contexto.
fn context_paths(harness: &Harness, name: &str) -> Result<Vec<String>> {
    let path = harness.config_file("context.yml");
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let content = read(&path)?;
    let mut in_skills = false;
    let mut current = "";
    let mut paths = Vec::new();
    for line in content.lines() {
        if is_blank_or_comment(line) {
            continue;
        }
        if !line.starts_with([' ', '\t']) {
            in_skills = line.starts_with("skills:");
            current = "";
            continue;
        }
        if !in_skills {
            continue;
        }
        if let Some(entry) = line
            .strip_prefix("  ")
            .filter(|rest| !rest.starts_with([' ', '\t', '-']) && rest.contains(':'))
        {
            current = entry.split(':').next().unwrap_or_default().trim_end();
        } else if let Some(item) = line
            .trim_start_matches([' ', '\t'])
            .strip_prefix('-')
            .filter(|rest| rest.starts_with([' ', '\t']))
        {
            if current == name {
                let item = strip_comment(item.trim_start()).trim_end();
                paths.push(unquote(item).to_string());
            }
        }
    }
    Ok(paths)
}

/// Grava o redirecionamento da skill em cada pasta de `redirect_dirs` (RF-25) e devolve os arquivos gravados.
pub fn write_redirects(harness: &Harness, name: &str, skill_file: &Path) -> Result<Vec<PathBuf>> {
    let config = skills_config(harness);
    let keys = config_list(&config, "redirect_keys")?;
    let dirs = config_list(&config, "redirect_dirs")?;
    let root = project_root(harness)?;
    let content = read(skill_file)?;
    let front_matter = pick_front_matter(&content, &keys);
    let skill_path = skill_file
        .strip_prefix(&root)
        .unwrap_or(skill_file)
        .display()
        .to_string();
    let values = HashMap::from([
        ("FRONT_MATTER", format!("---\n{front_matter}\n---")),
        ("SKILL_PATH", skill_path),
    ]);
    let redirect = render_template(harness, "redirect_template", &values)?;
    let mut written = Vec::new();
    for dir in dirs.iter().filter(|dir| !dir.is_empty()) {
        let target = root.join(dir).join(name).join(SKILL_FILE);
        write_new(&target, &redirect)?;
        written.push(target);
    }
    Ok(written)
}

/// Linhas do front-matter das chaves pedidas, inclusive as de continuação, na ordem do arquivo.
fn pick_front_matter(content: &str, keys: &[String]) -> String {
    let mut picking = false;
    let mut picked = Vec::new();
    for line in frontmatter_lines(content).unwrap_or_default() {
        if let Some(key) = top_level_key(line) {
            picking = keys.iter().any(|wanted| wanted == key);
        }
        if picking {
            picked.push(line);
        }
    }
    picked.join("\n").trim_end_matches('\n').to_string()
}

fn top_level_key(line: &str) -> Option<&str> {
    let (key, _) = line.split_once(':')?;
    let valid = !key.is_empty()
        && key
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-');
    valid.then_some(key)
}

/// Insere ou atualiza a linha da skill na tabela da seção do seu escopo no catálogo e devolve o catálogo.
pub fn update_catalog(
    harness: &Harness,
    name: &str,
    scope: Scope,
    skill_file: &Path,
) -> Result<PathBuf> {
    let config = skills_config(harness);
    let catalog = harness.resolve(&config_get(&config, "catalog")?);
    let section = config_get(&config, &format!("catalog_sections.{}", scope.key()))?;
    let dir = config_get(&config, &format!("dirs.{}", scope.key()))?;
    let description = frontmatter_get(skill_file, "description")?.replace('|', "\\|");
    let row = format!(
        "| {name} | {description} | `{}{dir}/{name}` |",
        catalog_prefix(harness, &catalog)
    );

    let content = read(&catalog)?;
    let lines = split_lines(&content);
    let heading = format!("## {section}");
    let mut inside = false;
    let mut last = None;
    let mut found = None;
    let row_start = format!("| {name} |");
    for (index, line) in lines.iter().enumerate() {
        if line.starts_with("## ") {
            inside = *line == heading;
        }
        if !inside {
            continue;
        }
        if line.starts_with('|') {
            last = Some(index);
            if line.starts_with(&row_start) {
                found = Some(index);
            }
        }
    }
    let Some(last) = last else {
        bail!(
            "seção '{heading}' com tabela não encontrada em {}",
            catalog.display()
        );
    };
    let mut out: Vec<&str> = Vec::with_capacity(lines.len() + 1);
    for (index, line) in lines.iter().enumerate() {
        if Some(index) == found {
            out.push(&row);
            continue;
        }
        out.push(line);
        if found.is_none() && index == last {
            out.push(&row);
        }
    }
    fs::write(&catalog, join_lines(&out))
        .with_context(|| format!("não foi possível gravar {}", catalog.display()))?;
    Ok(catalog)
}

/// `../` repetido até a raiz do harness, a partir da pasta do catálogo.
fn catalog_prefix(harness: &Harness, catalog: &Path) -> String {
    let relative = catalog.strip_prefix(&harness.root).unwrap_or(catalog);
    let depth = relative
        .parent()
        .map_or(0, |parent| parent.components().count());
    "../".repeat(depth)
}

fn project_root(harness: &Harness) -> Result<PathBuf> {
    let root = harness.path("root")?;
    crate::support::existing_dir(&root)
        .with_context(|| format!("raiz do projeto inexistente: {}", root.display()))
}

fn render_template(harness: &Harness, key: &str, values: &HashMap<&str, String>) -> Result<String> {
    let name = config_get(&skills_config(harness), key)?;
    let template = read(&harness.path("templates")?.join(name))?;
    let mut rendered = render(&template, values);
    if !rendered.is_empty() && !rendered.ends_with('\n') {
        rendered.push('\n');
    }
    Ok(rendered)
}

fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("não foi possível ler {}", path.display()))
}

fn write_new(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("não foi possível criar {}", parent.display()))?;
    }
    fs::write(path, content).with_context(|| format!("não foi possível gravar {}", path.display()))
}

/// Linhas de um arquivo, sem a quebra do fim.
fn split_lines(content: &str) -> Vec<&str> {
    if content.is_empty() {
        return Vec::new();
    }
    content
        .strip_suffix('\n')
        .unwrap_or(content)
        .split('\n')
        .collect()
}

fn join_lines(lines: &[&str]) -> String {
    lines.iter().map(|line| format!("{line}\n")).collect()
}
