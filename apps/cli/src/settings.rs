//! Configurações das evidências liberadas no painel: lista fechada, leitura, validação e gravação (SPEC-052, DEC-007).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Map, Value};

use crate::harness::Harness;
use crate::support::{sha256_hex, yaml_escape, yaml_unescape};

const TASKS: &str = "tasks.yml";
const DOCUMENTS: &str = "documents.yml";

/// Valor de uma configuração: número inteiro, texto ou lista de textos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingValue {
    Int(i64),
    Text(String),
    List(Vec<String>),
}

impl SettingValue {
    pub fn to_json(&self) -> Value {
        match self {
            SettingValue::Int(number) => json!(number),
            SettingValue::Text(text) => json!(text),
            SettingValue::List(items) => json!(items),
        }
    }

    /// Inteiro, texto ou lista só de textos; qualquer outro valor JSON é recusado.
    pub fn from_json(value: &Value) -> Option<SettingValue> {
        match value {
            Value::Number(number) => number.as_i64().map(SettingValue::Int),
            Value::String(text) => Some(SettingValue::Text(text.clone())),
            Value::Array(items) => items
                .iter()
                .map(|item| item.as_str().map(str::to_string))
                .collect::<Option<Vec<_>>>()
                .map(SettingValue::List),
            _ => None,
        }
    }
}

/// Valores das chaves da lista fechada e o sha256 de cada arquivo lido.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsSnapshot {
    pub files: BTreeMap<String, String>,
    pub values: BTreeMap<String, SettingValue>,
}

impl SettingsSnapshot {
    pub fn to_json(&self) -> Value {
        let values: Map<String, Value> = self
            .values
            .iter()
            .map(|(key, value)| (key.clone(), value.to_json()))
            .collect();
        json!({ "arquivos": self.files, "valores": values })
    }
}

/// Pedido de gravação: os valores novos e o sha256 lido de cada arquivo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsRequest {
    pub values: BTreeMap<String, SettingValue>,
    pub files: BTreeMap<String, String>,
}

impl SettingsRequest {
    pub fn from_json(body: &Value) -> Result<SettingsRequest, SettingsError> {
        let mut errors = BTreeMap::new();
        let mut values = BTreeMap::new();
        let mut files = BTreeMap::new();
        match body.get("valores").and_then(Value::as_object) {
            Some(object) => {
                for (key, value) in object {
                    match SettingValue::from_json(value) {
                        Some(value) => {
                            values.insert(key.clone(), value);
                        }
                        None => {
                            errors.insert(key.clone(), "tipo de valor não aceito".to_string());
                        }
                    }
                }
            }
            None => {
                errors.insert(
                    "valores".to_string(),
                    "o corpo deve ter o objeto valores".to_string(),
                );
            }
        }
        match body.get("arquivos").and_then(Value::as_object) {
            Some(object) => {
                for (name, sha) in object {
                    match sha.as_str() {
                        Some(sha) => {
                            files.insert(name.clone(), sha.to_string());
                        }
                        None => {
                            errors.insert(
                                format!("arquivos.{name}"),
                                "o sha256 deve ser um texto".to_string(),
                            );
                        }
                    }
                }
            }
            None => {
                errors.insert(
                    "arquivos".to_string(),
                    "o corpo deve ter o objeto arquivos com o sha256 de cada arquivo lido"
                        .to_string(),
                );
            }
        }
        if errors.is_empty() {
            Ok(SettingsRequest { values, files })
        } else {
            Err(SettingsError::Invalid(errors))
        }
    }
}

#[derive(Debug)]
pub enum SettingsError {
    /// Mensagem de erro por chave; nada foi gravado.
    Invalid(BTreeMap<String, String>),
    /// O arquivo mudou depois da leitura; nada foi gravado.
    Conflict {
        file: String,
    },
    Io(anyhow::Error),
}

impl From<anyhow::Error> for SettingsError {
    fn from(error: anyhow::Error) -> Self {
        SettingsError::Io(error)
    }
}

impl From<std::io::Error> for SettingsError {
    fn from(error: std::io::Error) -> Self {
        SettingsError::Io(error.into())
    }
}

// --- Lista fechada ---

enum Rule {
    Range(i64, i64),
    ReviewKinds,
    PersistDir,
    Readme,
    Snippets,
    Extensions,
}

#[derive(PartialEq)]
enum Kind {
    Int,
    Text,
    List,
}

impl Rule {
    fn kind(&self) -> Kind {
        match self {
            Rule::Range(..) => Kind::Int,
            Rule::PersistDir | Rule::Readme => Kind::Text,
            Rule::ReviewKinds | Rule::Snippets | Rule::Extensions => Kind::List,
        }
    }
}

struct KeySpec {
    name: &'static str,
    file: &'static str,
    key: &'static str,
    rule: Rule,
}

const fn spec(name: &'static str, file: &'static str, key: &'static str, rule: Rule) -> KeySpec {
    KeySpec {
        name,
        file,
        key,
        rule,
    }
}

const KEYS: [KeySpec; 12] = [
    spec(
        "tasks.evidence_review_kinds",
        TASKS,
        "evidence_review_kinds",
        Rule::ReviewKinds,
    ),
    spec(
        "tasks.evidence_baseline_timeout",
        TASKS,
        "evidence_baseline_timeout",
        Rule::Range(0, 3600),
    ),
    spec(
        "tasks.evidence_title_max",
        TASKS,
        "evidence_title_max",
        Rule::Range(20, 200),
    ),
    spec("tasks.persist_dir", TASKS, "persist_dir", Rule::PersistDir),
    spec(
        "tasks.persist_max_mb",
        TASKS,
        "persist_max_mb",
        Rule::Range(0, 1024),
    ),
    spec(
        "tasks.persist_readme",
        TASKS,
        "persist_readme",
        Rule::Readme,
    ),
    spec(
        "tasks.regression_ignore",
        TASKS,
        "regression_ignore",
        Rule::Snippets,
    ),
    spec(
        "tasks.regression_alert",
        TASKS,
        "regression_alert",
        Rule::Snippets,
    ),
    spec(
        "documents.embed_max_mb",
        DOCUMENTS,
        "embed_max_mb",
        Rule::Range(0, 100),
    ),
    spec(
        "documents.text_extensions",
        DOCUMENTS,
        "text_extensions",
        Rule::Extensions,
    ),
    spec(
        "documents.image_extensions",
        DOCUMENTS,
        "image_extensions",
        Rule::Extensions,
    ),
    spec(
        "documents.video_extensions",
        DOCUMENTS,
        "video_extensions",
        Rule::Extensions,
    ),
];

fn key_spec(name: &str) -> Option<&'static KeySpec> {
    KEYS.iter().find(|spec| spec.name == name)
}

// --- Leitura do subconjunto de YAML ---

enum Style {
    Scalar { quoted: bool },
    Inline,
    Block { indent: usize },
}

/// Linhas `start..=end` de uma chave de primeiro nível, o valor lido e o comentário da linha.
struct Entry {
    start: usize,
    end: usize,
    style: Style,
    raw: String,
    items: Vec<String>,
    comment: String,
}

fn key_line(lines: &[&str], key: &str) -> Option<usize> {
    lines.iter().position(|line| {
        line.strip_prefix(key)
            .is_some_and(|rest| rest.starts_with(':'))
    })
}

/// Separa o valor do comentário (espaço seguido de `#`, fora de aspas duplas); o comentário
/// guarda os espaços que o antecedem.
fn split_comment(rest: &str) -> (&str, &str) {
    let bytes = rest.as_bytes();
    let mut quoted = false;
    for index in 0..bytes.len() {
        match bytes[index] {
            b'"' => quoted = !quoted,
            b'#' if !quoted && index > 0 && matches!(bytes[index - 1], b' ' | b'\t') => {
                let value = rest[..index].trim_end();
                let value_end = value.len();
                return (value.trim_start(), &rest[value_end..]);
            }
            _ => {}
        }
    }
    (rest.trim(), "")
}

/// Texto sem as aspas; `true` quando estava entre aspas duplas.
fn unquote(value: &str) -> (String, bool) {
    let value = value.trim();
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        return (yaml_unescape(&value[1..value.len() - 1]), true);
    }
    if value.len() >= 2 && value.starts_with('\'') && value.ends_with('\'') {
        return (value[1..value.len() - 1].to_string(), false);
    }
    (value.to_string(), false)
}

/// Itens de uma lista na linha (`[a, "b, c", d]`), separados por vírgula fora de aspas duplas.
fn inline_items(value: &str) -> Vec<String> {
    let inner = value
        .trim()
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
        .unwrap_or("");
    let mut items = Vec::new();
    let mut item = String::new();
    let mut quoted = false;
    let mut escaped = false;
    for character in inner.chars() {
        if escaped {
            escaped = false;
        } else if quoted && character == '\\' {
            escaped = true;
        } else if character == '"' {
            quoted = !quoted;
        } else if character == ',' && !quoted {
            items.push(std::mem::take(&mut item));
            continue;
        }
        item.push(character);
    }
    items.push(item);
    items
        .iter()
        .map(|item| unquote(item).0)
        .filter(|item| !item.is_empty())
        .collect()
}

fn is_blank_or_comment(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.is_empty() || trimmed.starts_with('#')
}

fn block_item(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    trimmed
        .strip_prefix("- ")
        .or_else(|| trimmed.strip_prefix("-\t"))
}

fn parse_entry(lines: &[&str], index: usize) -> Entry {
    let line = lines[index];
    let rest = line.split_once(':').map_or("", |(_, rest)| rest);
    let (value, comment) = split_comment(rest);
    let mut entry = Entry {
        start: index,
        end: index,
        style: Style::Inline,
        raw: String::new(),
        items: Vec::new(),
        comment: comment.to_string(),
    };
    if value.starts_with('[') {
        entry.items = inline_items(value);
    } else if value.is_empty() {
        let mut indent = None;
        for (offset, next) in lines.iter().enumerate().skip(index + 1) {
            if is_blank_or_comment(next) {
                continue;
            }
            let Some(item) = block_item(next) else {
                break;
            };
            indent.get_or_insert(next.len() - next.trim_start().len());
            entry.end = offset;
            entry.items.push(unquote(split_comment(item).0).0);
        }
        entry.style = Style::Block {
            indent: indent.unwrap_or(2),
        };
    } else {
        let (raw, quoted) = unquote(value);
        entry.raw = raw;
        entry.style = Style::Scalar { quoted };
    }
    entry
}

fn find_entry(content: &str, key: &str) -> Option<Entry> {
    let lines: Vec<&str> = content.lines().collect();
    key_line(&lines, key).map(|index| parse_entry(&lines, index))
}

/// Conteúdo do arquivo de `config/` embutido no binário: os valores padrão.
fn defaults(file: &str) -> Result<String> {
    let bytes = crate::embedded::file(&format!("config/{file}"))
        .ok_or_else(|| anyhow!("config/{file} não está embutido no binário"))?;
    Ok(String::from_utf8_lossy(bytes).into_owned())
}

fn entry_value(spec: &KeySpec, entry: &Entry) -> Option<SettingValue> {
    match (spec.rule.kind(), &entry.style) {
        (Kind::Int, Style::Scalar { .. }) => entry.raw.parse().ok().map(SettingValue::Int),
        (Kind::Text, Style::Scalar { .. }) => Some(SettingValue::Text(entry.raw.clone())),
        (Kind::Text, Style::Block { .. }) if entry.items.is_empty() => {
            Some(SettingValue::Text(String::new()))
        }
        (Kind::List, Style::Inline | Style::Block { .. }) => {
            Some(SettingValue::List(entry.items.clone()))
        }
        _ => None,
    }
}

fn empty_value(spec: &KeySpec) -> SettingValue {
    match spec.rule.kind() {
        Kind::Int => SettingValue::Int(0),
        Kind::Text => SettingValue::Text(String::new()),
        Kind::List => SettingValue::List(Vec::new()),
    }
}

/// Valor do projeto; sem a chave, ou com valor que não se lê, o do arquivo embutido.
fn value_of(spec: &KeySpec, project_content: &str, default_content: &str) -> SettingValue {
    [project_content, default_content]
        .iter()
        .find_map(|content| {
            find_entry(content, spec.key).and_then(|entry| entry_value(spec, &entry))
        })
        .unwrap_or_else(|| empty_value(spec))
}

/// Os cinco tipos de evidência que podem pedir aval, lidos do `tasks.yml` do projeto.
fn review_kind_options(tasks_content: &str) -> Vec<String> {
    [
        ("evidence_automatic", "automática"),
        ("evidence_from_verify", "automática, da verificação"),
        ("evidence_diff", "diff"),
        ("evidence_manual", "manual"),
        ("evidence_print", "print"),
    ]
    .iter()
    .map(|(key, default)| {
        find_entry(tasks_content, key)
            .filter(|entry| matches!(entry.style, Style::Scalar { .. }))
            .map(|entry| entry.raw)
            .filter(|raw| !raw.is_empty())
            .unwrap_or_else(|| default.to_string())
    })
    .collect()
}

/// Conteúdo e sha256 de um arquivo de `config/`.
struct ConfigFile {
    path: PathBuf,
    content: String,
    sha: String,
}

fn read_config(harness: &Harness, name: &str) -> Result<ConfigFile> {
    let path = harness.config_file(name);
    let bytes =
        fs::read(&path).with_context(|| format!("não foi possível ler {}", path.display()))?;
    let content = String::from_utf8(bytes.clone())
        .with_context(|| format!("{} não está em UTF-8", path.display()))?;
    Ok(ConfigFile {
        path,
        sha: sha256_hex(&bytes),
        content,
    })
}

struct Current {
    files: BTreeMap<&'static str, ConfigFile>,
    values: BTreeMap<String, SettingValue>,
}

fn read_current(harness: &Harness) -> Result<Current> {
    let mut files = BTreeMap::new();
    let mut defaults_by_file = BTreeMap::new();
    for name in [TASKS, DOCUMENTS] {
        files.insert(name, read_config(harness, name)?);
        defaults_by_file.insert(name, defaults(name)?);
    }
    let values = KEYS
        .iter()
        .map(|spec| {
            let value = value_of(
                spec,
                &files[spec.file].content,
                &defaults_by_file[spec.file],
            );
            (spec.name.to_string(), value)
        })
        .collect();
    Ok(Current { files, values })
}

/// Valores das 12 chaves da lista fechada e o sha256 dos dois arquivos.
pub fn read_evidence_settings(harness: &Harness) -> Result<SettingsSnapshot> {
    let current = read_current(harness)?;
    Ok(SettingsSnapshot {
        files: current
            .files
            .iter()
            .map(|(name, file)| (name.to_string(), file.sha.clone()))
            .collect(),
        values: current.values,
    })
}

// --- Validação ---

fn type_error(kind: Kind) -> String {
    match kind {
        Kind::Int => "o valor deve ser um número inteiro",
        Kind::Text => "o valor deve ser um texto",
        Kind::List => "o valor deve ser uma lista de textos",
    }
    .to_string()
}

fn first_repeated(items: &[String]) -> Option<&String> {
    let mut seen = BTreeSet::new();
    items.iter().find(|item| !seen.insert(item.as_str()))
}

fn check_review_kinds(items: &[String], options: &[String]) -> Result<Vec<String>, String> {
    if let Some(unknown) = items.iter().find(|item| !options.contains(item)) {
        return Err(format!("tipo de evidência desconhecido: {unknown}"));
    }
    if let Some(repeated) = first_repeated(items) {
        return Err(format!("tipo repetido: {repeated}"));
    }
    Ok(items.to_vec())
}

fn check_persist_dir(dir: &str) -> Result<String, String> {
    if dir.is_empty() {
        return Ok(String::new());
    }
    let mut chars = dir.chars();
    let drive = chars.next().is_some_and(|c| c.is_ascii_alphabetic()) && chars.next() == Some(':');
    if dir.starts_with(['/', '\\']) || drive {
        return Err("o caminho deve ser relativo à raiz do projeto".to_string());
    }
    if dir.split(['/', '\\']).any(|segment| segment == "..") {
        return Err("o caminho não pode ter ..".to_string());
    }
    if !dir.contains("<SPEC>") {
        return Err("o caminho deve ter <SPEC>".to_string());
    }
    Ok(dir.to_string())
}

fn check_readme(name: &str) -> Result<String, String> {
    let valid = name.len() > ".md".len() && name.ends_with(".md") && !name.contains(['/', '\\']);
    if valid {
        Ok(name.to_string())
    } else {
        Err("o nome deve ser um arquivo .md, sem barra".to_string())
    }
}

fn check_snippets(items: &[String]) -> Result<Vec<String>, String> {
    let items: Vec<String> = items.iter().map(|item| item.trim().to_string()).collect();
    if items.iter().any(String::is_empty) {
        return Err("os trechos não podem ser vazios".to_string());
    }
    if items
        .iter()
        .any(|item| item.contains('"') || item.contains(" #"))
    {
        return Err("o trecho não pode ter aspas duplas nem espaço seguido de #".to_string());
    }
    if let Some(repeated) = first_repeated(&items) {
        return Err(format!("trecho repetido: {repeated}"));
    }
    Ok(items)
}

fn check_extensions(items: &[String]) -> Result<Vec<String>, String> {
    let mut normalized = Vec::new();
    for item in items {
        let trimmed = item.trim();
        let extension = trimmed.strip_prefix('.').unwrap_or(trimmed).to_lowercase();
        let valid = !extension.is_empty()
            && extension
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
        if !valid {
            return Err(format!(
                "extensão inválida: {trimmed}; use só letras minúsculas e números"
            ));
        }
        normalized.push(extension);
    }
    if let Some(repeated) = first_repeated(&normalized) {
        return Err(format!("extensão repetida: {repeated}"));
    }
    Ok(normalized)
}

fn check(spec: &KeySpec, value: &SettingValue, options: &[String]) -> Result<SettingValue, String> {
    match (&spec.rule, value) {
        (Rule::Range(min, max), SettingValue::Int(number)) => {
            if (*min..=*max).contains(number) {
                Ok(value.clone())
            } else {
                Err(format!("o valor deve estar entre {min} e {max}"))
            }
        }
        (Rule::PersistDir, SettingValue::Text(dir)) => {
            check_persist_dir(dir).map(SettingValue::Text)
        }
        (Rule::Readme, SettingValue::Text(name)) => check_readme(name).map(SettingValue::Text),
        (Rule::ReviewKinds, SettingValue::List(items)) => {
            check_review_kinds(items, options).map(SettingValue::List)
        }
        (Rule::Snippets, SettingValue::List(items)) => {
            check_snippets(items).map(SettingValue::List)
        }
        (Rule::Extensions, SettingValue::List(items)) => {
            check_extensions(items).map(SettingValue::List)
        }
        _ => Err(type_error(spec.rule.kind())),
    }
}

/// Recusa a extensão que já está em outra das três listas, com as listas do pedido sobre as atuais.
fn check_cross_extensions(
    normalized: &BTreeMap<String, SettingValue>,
    current: &BTreeMap<String, SettingValue>,
    errors: &mut BTreeMap<String, String>,
) {
    let extension_keys: Vec<&str> = KEYS
        .iter()
        .filter(|spec| matches!(spec.rule, Rule::Extensions))
        .map(|spec| spec.name)
        .collect();
    let effective = |key: &str| match normalized.get(key).or_else(|| current.get(key)) {
        Some(SettingValue::List(items)) => items.clone(),
        _ => Vec::new(),
    };
    for key in &extension_keys {
        let Some(SettingValue::List(items)) = normalized.get(*key) else {
            continue;
        };
        let clash = items.iter().find_map(|item| {
            extension_keys
                .iter()
                .filter(|other| *other != key)
                .find(|other| effective(other).contains(item))
                .map(|other| format!("extensão repetida: {item}, já está em {other}"))
        });
        if let Some(message) = clash {
            errors.insert(key.to_string(), message);
        }
    }
}

/// Confere todas as chaves do pedido e devolve os valores normalizados que diferem dos atuais.
fn validate(
    request: &SettingsRequest,
    current: &BTreeMap<String, SettingValue>,
    tasks_content: &str,
) -> Result<BTreeMap<String, SettingValue>, BTreeMap<String, String>> {
    let options = review_kind_options(tasks_content);
    let mut errors = BTreeMap::new();
    let mut normalized = BTreeMap::new();
    for (name, value) in &request.values {
        let Some(spec) = key_spec(name) else {
            errors.insert(
                name.clone(),
                "chave fora da lista de configurações liberadas".to_string(),
            );
            continue;
        };
        match check(spec, value, &options) {
            Ok(value) => {
                normalized.insert(name.clone(), value);
            }
            Err(message) => {
                errors.insert(name.clone(), message);
            }
        }
    }
    check_cross_extensions(&normalized, current, &mut errors);
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(normalized
        .into_iter()
        .filter(|(name, value)| current.get(name) != Some(value))
        .collect())
}

// --- Gravação ---

/// Texto entre aspas quando o YAML lido pelo vostok o confundiria com outra coisa.
fn needs_quotes(item: &str) -> bool {
    item.is_empty()
        || item.starts_with([' ', '-', '?'])
        || item.ends_with(' ')
        || item.contains([
            ':', '#', ',', '[', ']', '{', '}', '"', '\'', '&', '*', '!', '|', '>', '%', '@', '`',
        ])
}

fn quoted(item: &str) -> String {
    format!("\"{}\"", yaml_escape(item))
}

fn list_item(item: &str) -> String {
    if needs_quotes(item) {
        quoted(item)
    } else {
        item.to_string()
    }
}

fn scalar_text(value: &SettingValue, keep_quotes: bool) -> String {
    match value {
        SettingValue::Int(number) => number.to_string(),
        SettingValue::Text(text) if keep_quotes || needs_quotes(text) => quoted(text),
        SettingValue::Text(text) => text.clone(),
        SettingValue::List(items) => format!(
            "[{}]",
            items
                .iter()
                .map(|item| list_item(item))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn block_lines(key: &str, indent: usize, comments: Vec<String>, items: &[String]) -> Vec<String> {
    let pad = " ".repeat(indent);
    std::iter::once(format!("{key}:"))
        .chain(comments)
        .chain(
            items
                .iter()
                .map(|item| format!("{pad}- {}", list_item(item))),
        )
        .collect()
}

fn entry_lines(spec: &KeySpec, entry: &Entry, lines: &[&str], value: &SettingValue) -> Vec<String> {
    let key = spec.key;
    match (&entry.style, value) {
        (Style::Block { indent }, SettingValue::List(items)) => {
            let comments = lines[entry.start + 1..=entry.end]
                .iter()
                .filter(|line| line.trim_start().starts_with('#'))
                .map(|line| line.to_string())
                .collect();
            block_lines(key, *indent, comments, items)
        }
        (Style::Scalar { quoted }, _) => {
            vec![format!(
                "{key}: {}{}",
                scalar_text(value, *quoted),
                entry.comment
            )]
        }
        _ => vec![format!(
            "{key}: {}{}",
            scalar_text(value, false),
            entry.comment
        )],
    }
}

fn appended_lines(spec: &KeySpec, value: &SettingValue) -> Vec<String> {
    match value {
        SettingValue::List(items) => block_lines(spec.key, 2, Vec::new(), items),
        _ => vec![format!("{}: {}", spec.key, scalar_text(value, false))],
    }
}

/// Troca só as linhas da chave, mantendo o fim de linha e a quebra final; a chave ausente vai
/// para o fim do arquivo.
fn render(content: &str, spec: &KeySpec, value: &SettingValue) -> String {
    let eol = if content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let lines: Vec<&str> = content.lines().collect();
    let mut trailing = content.ends_with('\n');
    let mut out: Vec<String>;
    match key_line(&lines, spec.key) {
        Some(index) => {
            let entry = parse_entry(&lines, index);
            out = lines[..entry.start].iter().map(|l| l.to_string()).collect();
            out.extend(entry_lines(spec, &entry, &lines, value));
            out.extend(lines[entry.end + 1..].iter().map(|l| l.to_string()));
        }
        None => {
            out = lines.iter().map(|l| l.to_string()).collect();
            if out.last().is_some_and(|line| !line.trim().is_empty()) {
                out.push(String::new());
            }
            out.extend(appended_lines(spec, value));
            trailing = true;
        }
    }
    let mut rendered = out.join(eol);
    if trailing {
        rendered.push_str(eol);
    }
    rendered
}

fn tmp_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".tmp");
    path.with_file_name(name)
}

/// Grava cada arquivo num `.tmp` e só depois renomeia; numa falha, restaura os já renomeados.
fn commit_files(writes: &[(&ConfigFile, String)]) -> Result<(), SettingsError> {
    for (file, content) in writes {
        if let Err(error) = fs::write(tmp_path(&file.path), content) {
            for (file, _) in writes {
                let _ = fs::remove_file(tmp_path(&file.path));
            }
            return Err(error.into());
        }
    }
    for (index, (file, _)) in writes.iter().enumerate() {
        if let Err(error) = fs::rename(tmp_path(&file.path), &file.path) {
            for (renamed, _) in &writes[..index] {
                let _ = fs::write(&renamed.path, &renamed.content);
            }
            for (pending, _) in &writes[index..] {
                let _ = fs::remove_file(tmp_path(&pending.path));
            }
            return Err(error.into());
        }
    }
    Ok(())
}

/// Grava as chaves do pedido, tudo ou nada, só sobre o conteúdo lido (sha256 do pedido).
pub fn write_evidence_settings(
    harness: &Harness,
    request: &SettingsRequest,
) -> Result<SettingsSnapshot, SettingsError> {
    let current = read_current(harness)?;
    let changes = validate(request, &current.values, &current.files[TASKS].content)
        .map_err(SettingsError::Invalid)?;
    let changed_files: BTreeSet<&'static str> = changes
        .keys()
        .filter_map(|name| key_spec(name))
        .map(|spec| spec.file)
        .collect();
    let missing: BTreeMap<String, String> = changed_files
        .iter()
        .filter(|file| !request.files.contains_key(**file))
        .map(|file| {
            (
                format!("arquivos.{file}"),
                "falta o sha256 lido do arquivo".to_string(),
            )
        })
        .collect();
    if !missing.is_empty() {
        return Err(SettingsError::Invalid(missing));
    }
    if changes.is_empty() {
        return Ok(read_evidence_settings(harness)?);
    }
    if let Some(file) = changed_files
        .iter()
        .find(|file| request.files[**file] != current.files[**file].sha)
    {
        return Err(SettingsError::Conflict {
            file: file.to_string(),
        });
    }
    let writes: Vec<(&ConfigFile, String)> = changed_files
        .iter()
        .map(|name| {
            let file = &current.files[name];
            let content = changes
                .iter()
                .filter_map(|(key, value)| key_spec(key).map(|spec| (spec, value)))
                .filter(|(spec, _)| spec.file == *name)
                .fold(file.content.clone(), |content, (spec, value)| {
                    render(&content, spec, value)
                });
            (file, content)
        })
        .collect();
    commit_files(&writes)?;
    Ok(read_evidence_settings(harness)?)
}
