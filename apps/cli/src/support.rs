//! Utilitários comuns aos comandos: configuração, front-matter, Markdown, templates e hash.
//! Reproduzem as libs bash `config.sh`, `frontmatter.sh`, `markdown.sh` e `text.sh`.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};
use sha2::{Digest, Sha256};

// --- Configuração ---
// Subconjunto de YAML dos arquivos de `config/`: `chave: valor` em níveis aninhados (`pai.filho.neto`),
// blocos literais de primeiro nível (`chave: |`) e listas de primeiro nível.

fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("não foi possível ler {}", path.display()))
}

fn is_blank_or_comment(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.is_empty() || trimmed.starts_with('#')
}

/// Caminho `pai.filho.neto` de cada linha de chave, com a indentação e o resto da linha.
struct KeyLine<'a> {
    path: String,
    indent: usize,
    line: &'a str,
}

fn key_lines(content: &str) -> impl Iterator<Item = Option<KeyLine<'_>>> {
    let mut parents: Vec<(usize, String)> = Vec::new();
    content.lines().map(move |raw| {
        if is_blank_or_comment(raw) {
            return None;
        }
        let indent = raw.len() - raw.trim_start_matches(' ').len();
        let line = &raw[indent..];
        let key = line.split(':').next().unwrap_or("").trim_end().to_string();
        while parents.last().is_some_and(|(level, _)| *level >= indent) {
            parents.pop();
        }
        let path = parents
            .iter()
            .map(|(_, parent)| parent.as_str())
            .chain([key.as_str()])
            .collect::<Vec<_>>()
            .join(".");
        parents.push((indent, key));
        Some(KeyLine { path, indent, line })
    })
}

fn strip_comment(value: &str) -> &str {
    let bytes = value.as_bytes();
    for index in 1..bytes.len() {
        if bytes[index] == b'#' && (bytes[index - 1] == b' ' || bytes[index - 1] == b'\t') {
            return &value[..index];
        }
    }
    value
}

fn unquote(value: &str) -> &str {
    let quoted = value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')));
    if quoted {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

/// Valor escalar de uma chave (`pai.filho`). Falha se a chave não existir.
pub fn config_get(path: &Path, key: &str) -> Result<String> {
    let content = read(path)?;
    for entry in key_lines(&content).flatten() {
        if entry.path == key {
            if let Some((_, value)) = entry.line.split_once(':') {
                return Ok(unquote(strip_comment(value).trim()).to_string());
            }
        }
    }
    bail!("Configuração '{key}' não encontrada em {}", path.display())
}

/// Itens de uma lista de primeiro nível. Falha se a lista não existir.
pub fn config_list(path: &Path, key: &str) -> Result<Vec<String>> {
    let content = read(path)?;
    let header = format!("{key}:");
    let mut lines = content.lines();
    if !lines.any(|line| line.trim_end() == header) {
        bail!("Lista '{key}' não encontrada em {}", path.display());
    }
    let mut items = Vec::new();
    for line in lines {
        if is_blank_or_comment(line) {
            continue;
        }
        let Some(item) = line
            .trim_start()
            .strip_prefix("- ")
            .or_else(|| line.trim_start().strip_prefix("-\t"))
        else {
            break;
        };
        items.push(unquote(strip_comment(item.trim_start()).trim_end()).to_string());
    }
    Ok(items)
}

/// Conteúdo de um bloco literal de primeiro nível (`chave: |`), sem a indentação.
pub fn config_block(path: &Path, key: &str) -> Result<String> {
    let content = read(path)?;
    let mut lines = content.lines();
    let opens = |line: &str| {
        line.strip_prefix(key)
            .and_then(|rest| rest.strip_prefix(':'))
            .is_some_and(|rest| rest.trim_start().starts_with('|'))
    };
    if !lines.any(opens) {
        bail!("Bloco '{key}' não encontrado em {}", path.display());
    }
    let mut indent = None;
    let mut out = Vec::new();
    for line in lines {
        if !line.is_empty() && !line.starts_with([' ', '\t']) {
            break;
        }
        if indent.is_none() && !line.trim().is_empty() {
            indent = Some(line.len() - line.trim_start().len());
        }
        out.push(line.get(indent.unwrap_or(0)..).unwrap_or(""));
    }
    Ok(out.join("\n"))
}

/// Troca o valor escalar de uma chave existente, preservando o resto do arquivo.
pub fn config_set(path: &Path, key: &str, value: &str) -> Result<()> {
    let content = read(path)?;
    let mut found = false;
    let lines: Vec<String> = content
        .lines()
        .zip(key_lines(&content))
        .map(|(raw, entry)| match entry {
            Some(entry) if entry.path == key && entry.line.contains(':') => {
                found = true;
                let name = &entry.line[..entry.line.find(':').unwrap_or(0)];
                format!("{}{name}: {value}", " ".repeat(entry.indent))
            }
            _ => raw.to_string(),
        })
        .collect();
    if !found {
        bail!("Configuração '{key}' não encontrada em {}", path.display());
    }
    write_lines(path, &lines, content.ends_with('\n'))
}

fn write_lines(path: &Path, lines: &[String], trailing_newline: bool) -> Result<()> {
    let mut out = lines.join("\n");
    if trailing_newline {
        out.push('\n');
    }
    fs::write(path, out).with_context(|| format!("não foi possível gravar {}", path.display()))
}

// --- Front-matter ---

/// Linhas do front-matter (entre as duas primeiras linhas `---`), sem os delimitadores.
pub fn frontmatter_lines(content: &str) -> Option<Vec<&str>> {
    let mut lines = content.lines().map(|line| line.trim_end_matches('\r'));
    if lines.next()? != "---" {
        return None;
    }
    let mut out = Vec::new();
    for line in lines {
        if line == "---" {
            return Some(out);
        }
        out.push(line);
    }
    None
}

/// Valor de uma chave do front-matter, sem aspas e com escapes resolvidos.
/// Valores em bloco (`>` ou `|`) saem numa linha só, unidos por espaço.
pub fn frontmatter_get(path: &Path, key: &str) -> Result<String> {
    let content = read(path)?;
    let lines = frontmatter_lines(&content)
        .ok_or_else(|| anyhow!("{} não tem front-matter", path.display()))?;
    let prefix = format!("{key}:");
    let mut iter = lines.iter();
    while let Some(line) = iter.next() {
        let Some(rest) = line.strip_prefix(&prefix) else {
            continue;
        };
        let value = rest.trim();
        if matches!(value, ">" | "|" | ">-" | "|-" | ">+" | "|+") {
            let block: Vec<&str> = iter
                .take_while(|next| next.is_empty() || next.starts_with([' ', '\t']))
                .map(|next| next.trim())
                .filter(|next| !next.is_empty())
                .collect();
            return Ok(block.join(" "));
        }
        if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
            return Ok(yaml_unescape(&value[1..value.len() - 1]));
        }
        return Ok(unquote(value).to_string());
    }
    bail!(
        "Chave '{key}' não encontrada no front-matter de {}",
        path.display()
    )
}

/// Desfaz o escape de `yaml_escape` dentro de aspas duplas.
pub fn yaml_unescape(value: &str) -> String {
    let mut out = String::new();
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            out.push(chars.next().unwrap_or('\\'));
        } else {
            out.push(c);
        }
    }
    out
}

/// Troca o valor de uma chave existente do front-matter, sem tocar no resto do arquivo.
pub fn frontmatter_set(path: &Path, key: &str, value: &str) -> Result<()> {
    let content = read(path)?;
    let prefix = format!("{key}:");
    let mut inside = false;
    let mut found = false;
    let lines: Vec<String> = content
        .lines()
        .enumerate()
        .map(|(index, raw)| {
            let plain = raw.trim_end_matches('\r');
            if index == 0 && plain == "---" {
                inside = true;
            } else if inside && plain == "---" {
                inside = false;
            } else if inside && !found && raw.starts_with(&prefix) {
                found = true;
                return format!("{key}: {value}");
            }
            raw.to_string()
        })
        .collect();
    if !found {
        bail!(
            "Chave '{key}' não encontrada no front-matter de {}",
            path.display()
        );
    }
    write_lines(path, &lines, content.ends_with('\n'))
}

// --- Markdown ---

/// Conteúdo de uma seção `## <título>` até a próxima `## `. Títulos em blocos de código não contam.
pub fn markdown_section(content: &str, title: &str) -> Option<String> {
    let heading = format!("## {title}");
    let mut in_code = false;
    let mut inside = false;
    let mut found = false;
    let mut out = Vec::new();
    for line in content.lines().map(|line| line.trim_end_matches('\r')) {
        if line.trim_start().starts_with("```") {
            in_code = !in_code;
            if inside {
                out.push(line);
            }
            continue;
        }
        if !in_code && line == heading {
            inside = true;
            found = true;
            continue;
        }
        if !in_code && inside && line.starts_with("## ") {
            break;
        }
        if inside {
            out.push(line);
        }
    }
    found.then(|| out.join("\n"))
}

/// Conteúdo de uma subseção `### <subtítulo>` dentro da seção `## <título>`.
pub fn markdown_subsection(content: &str, title: &str, subtitle: &str) -> Option<String> {
    let heading = format!("## {title}");
    let sub_heading = format!("### {subtitle}");
    let mut in_code = false;
    let mut in_section = false;
    let mut inside = false;
    let mut found = false;
    let mut out = Vec::new();
    for line in content.lines().map(|line| line.trim_end_matches('\r')) {
        if line.trim_start().starts_with("```") {
            in_code = !in_code;
            if inside {
                out.push(line);
            }
            continue;
        }
        if !in_code && line.starts_with("## ") {
            in_section = line == heading;
            inside = false;
            continue;
        }
        if !in_code && in_section && line.starts_with("### ") {
            inside = line == sub_heading;
            found |= inside;
            continue;
        }
        if inside {
            out.push(line);
        }
    }
    found.then(|| out.join("\n"))
}

fn is_separator(line: &str) -> bool {
    let cells: Vec<&str> = line.trim().trim_matches('|').split('|').collect();
    !cells.is_empty()
        && cells.iter().all(|cell| {
            let cell = cell.trim().trim_matches(':');
            cell.len() >= 3 && cell.chars().all(|c| c == '-')
        })
}

/// Linhas de dados das tabelas do texto, sem cabeçalho nem separador.
pub fn markdown_table_rows(content: &str) -> Vec<String> {
    let mut header_done = false;
    let mut rows = Vec::new();
    for line in content.lines() {
        if !line.starts_with('|') {
            header_done = false;
            continue;
        }
        if is_separator(line) {
            header_done = true;
        } else if header_done {
            rows.push(line.to_string());
        }
    }
    rows
}

/// Valor de uma coluna (a partir de 1) de uma linha de tabela, sem espaços nas pontas.
pub fn markdown_cell(row: &str, column: usize) -> String {
    row.split('|').nth(column).unwrap_or("").trim().to_string()
}

// --- Templates e texto ---

/// Substitui cada `#{NOME}` pelo valor de `values`; variáveis sem valor ficam como estão.
pub fn render(template: &str, values: &HashMap<&str, String>) -> String {
    let mut out = String::new();
    let mut rest = template;
    while let Some(start) = rest.find("#{") {
        let after = &rest[start + 2..];
        let name_len = after
            .find(|c: char| !(c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'))
            .unwrap_or(after.len());
        let closes = after[name_len..].starts_with('}') && name_len > 0;
        out.push_str(&rest[..start]);
        match (closes, values.get(&after[..name_len])) {
            (true, Some(value)) => {
                out.push_str(value);
                rest = &after[name_len + 1..];
            }
            _ => {
                out.push_str("#{");
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

/// Variáveis `#{NOME}` ainda não preenchidas, como `(linha, texto da linha)`.
/// Variáveis precedidas de crase são menções em documentação e não contam.
pub fn template_leftovers(content: &str) -> Vec<(usize, String)> {
    content
        .lines()
        .enumerate()
        .filter(|(_, line)| has_leftover(line))
        .map(|(index, line)| (index + 1, line.to_string()))
        .collect()
}

fn has_leftover(line: &str) -> bool {
    let bytes = line.as_bytes();
    let mut index = 0;
    while let Some(offset) = line[index..].find("#{") {
        let start = index + offset;
        let name: String = line[start + 2..]
            .chars()
            .take_while(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == '_')
            .collect();
        let closes = line[start + 2 + name.len()..].starts_with('}');
        if !name.is_empty() && closes && (start == 0 || bytes[start - 1] != b'`') {
            return true;
        }
        index = start + 2;
    }
    false
}

/// Título limpo para virar cabeçalho: sem espaços nas pontas, não vazio, numa linha e sem `#{`.
pub fn clean_title(title: &str) -> Result<String> {
    let title = title.trim();
    if title.is_empty() {
        bail!("O título não pode ser vazio");
    }
    if title.contains('\n') || title.contains('\r') {
        bail!("O título não pode ter quebra de linha");
    }
    if title.contains("#{") {
        bail!("O título não pode conter '#{{', reservado para variáveis de template");
    }
    Ok(title.to_string())
}

/// Escapa um texto para uso dentro de aspas duplas em YAML.
pub fn yaml_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

/// SHA-256 em hexadecimal minúsculo.
pub fn sha256_hex(content: &[u8]) -> String {
    Sha256::digest(content)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Slug: minúsculas, sem acentos, palavras separadas por hífen.
pub fn slugify(text: &str) -> String {
    let mut slug = String::new();
    let mut pending_dash = false;
    for c in text.chars().map(fold_accent) {
        if c.is_ascii_alphanumeric() {
            if pending_dash && !slug.is_empty() {
                slug.push('-');
            }
            pending_dash = false;
            slug.push(c.to_ascii_lowercase());
        } else {
            pending_dash = true;
        }
    }
    slug
}

fn fold_accent(c: char) -> char {
    match c {
        'á' | 'à' | 'â' | 'ã' | 'ä' | 'Á' | 'À' | 'Â' | 'Ã' | 'Ä' => 'a',
        'é' | 'è' | 'ê' | 'ë' | 'É' | 'È' | 'Ê' | 'Ë' => 'e',
        'í' | 'ì' | 'î' | 'ï' | 'Í' | 'Ì' | 'Î' | 'Ï' => 'i',
        'ó' | 'ò' | 'ô' | 'õ' | 'ö' | 'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' => 'o',
        'ú' | 'ù' | 'û' | 'ü' | 'Ú' | 'Ù' | 'Û' | 'Ü' => 'u',
        'ç' | 'Ç' => 'c',
        'ñ' | 'Ñ' => 'n',
        other => other,
    }
}

// --- Saída, títulos e datas ---

/// Encerra com código 1 sem o prefixo `Erro:` do `main`, para mensagens que os scripts imprimem
/// sem ele (ou que já foram impressas).
pub fn exit_failure() -> ! {
    let _ = std::io::Write::flush(&mut std::io::stdout());
    std::process::exit(1)
}

/// Imprime a mensagem na saída de erro, como está, e encerra com código 1.
pub fn fail(message: impl std::fmt::Display) -> ! {
    eprintln!("{message}");
    exit_failure()
}

/// Linhas como o `awk` as lê: separadas só por `\n` (um `\r` final fica na linha) e sem a vazia
/// depois do último `\n`.
pub fn records(content: &str) -> Vec<&str> {
    let mut lines: Vec<&str> = content.split('\n').collect();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    lines
}

/// Junta linhas como o `awk` as imprime: cada uma seguida de `\n`.
pub fn join_records<S: AsRef<str>>(lines: &[S]) -> String {
    lines.iter().fold(String::new(), |mut out, line| {
        out.push_str(line.as_ref());
        out.push('\n');
        out
    })
}

/// `clean_title` com a mensagem de título vazio dos scripts (`text.sh`).
pub fn script_title(title: &str) -> Result<String> {
    if title.trim().is_empty() {
        bail!("O título está vazio");
    }
    clean_title(title)
}

/// Nome em kebab-case: `^[a-z0-9]+(-[a-z0-9]+)*$`.
pub fn is_kebab_case(name: &str) -> bool {
    !name.is_empty()
        && name.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
}

/// Data atual em UTC, `AAAA-MM-DD`.
pub fn utc_date() -> String {
    utc_datetime()[..10].to_string()
}

/// Data e hora atuais em UTC, `AAAA-MM-DDTHH:MM:SSZ`.
pub fn utc_datetime() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());
    let (year, month, day) = civil_from_days((seconds / 86_400) as i64);
    let time = seconds % 86_400;
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        time / 3600,
        time % 3600 / 60,
        time % 60
    )
}

/// Data do calendário gregoriano a partir dos dias desde 1970-01-01 (algoritmo de Howard Hinnant).
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * month_index + 2) / 5 + 1) as u32;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    } as u32;
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

/// Diretório atual como o shell o mostra: o `$PWD`, quando ele aponta para o mesmo diretório que o
/// sistema informa. Assim um link no caminho (ex.: `/var` → `/private/var` no macOS) não aparece
/// resolvido nas mensagens.
pub fn current_dir() -> std::io::Result<std::path::PathBuf> {
    let physical = std::env::current_dir()?;
    let logical = std::env::var_os("PWD")
        .map(std::path::PathBuf::from)
        .filter(|pwd| pwd.is_absolute() && same_dir(pwd, &physical));
    Ok(logical.map_or(physical, |pwd| lexical(&pwd)))
}

#[cfg(unix)]
fn same_dir(a: &Path, b: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    match (fs::metadata(a), fs::metadata(b)) {
        (Ok(a), Ok(b)) => a.dev() == b.dev() && a.ino() == b.ino(),
        _ => false,
    }
}

#[cfg(not(unix))]
fn same_dir(a: &Path, b: &Path) -> bool {
    a == b
}

/// Resolve `.` e `..` sem seguir links, como o `cd <caminho> && pwd` do shell.
pub fn lexical(path: &Path) -> std::path::PathBuf {
    use std::path::Component;
    let mut out = std::path::PathBuf::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    out.push(part);
                }
            }
            other => out.push(other),
        }
    }
    out
}

/// Caminho lógico de uma pasta que precisa existir.
pub fn existing_dir(path: &Path) -> Result<std::path::PathBuf> {
    let logical = lexical(path);
    if !logical.is_dir() {
        bail!("pasta {} não encontrada", logical.display());
    }
    Ok(logical)
}

pub const EVIDENCE_PROOF_FIELD: &str = "Prova";
pub const EVIDENCE_SHOWS_FIELD: &str = "Mostra";

/// Subitem obrigatório que outro subitem satisfaz: com as duas, vale Mostra (SPEC-055, CB-01).
const EVIDENCE_ALTERNATIVES: &[(&str, &str)] = &[(EVIDENCE_PROOF_FIELD, EVIDENCE_SHOWS_FIELD)];

/// Nomes dos subitens de cada evidência (`evidence_fields` e `evidence_optional_fields`).
pub struct EvidenceFields {
    pub required: Vec<String>,
    pub optional: Vec<String>,
}

/// Linha recuada de subitem de evidência, `  - Campo: texto`, como (campo, texto), com o campo de
/// até três palavras.
pub fn evidence_subitem(line: &str) -> Option<(&str, &str)> {
    if !is_indented(line) {
        return None;
    }
    let (field, text) = line.trim_start().strip_prefix("- ")?.split_once(": ")?;
    let valid = !field.is_empty()
        && field.trim() == field
        && !field.contains("  ")
        && !field.contains('\t')
        && field.split(' ').count() <= 3;
    valid.then_some((field, text))
}

pub fn is_indented(line: &str) -> bool {
    line.starts_with([' ', '\t'])
}

/// Problemas dos subitens de "Evidências": subitem fora do formato, sem evidência acima ou
/// desconhecido e, com `require`, evidência sem um subitem obrigatório. Sem `fields`, só o formato
/// é conferido.
pub fn evidence_subitem_errors(
    text: &str,
    section: &str,
    fields: Option<&EvidenceFields>,
    require: bool,
) -> Vec<String> {
    let mut errors = Vec::new();
    let mut evidences: Vec<(&str, Vec<&str>)> = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        if !is_indented(line) {
            evidences.push((line, Vec::new()));
            continue;
        }
        let Some((field, _)) = evidence_subitem(line) else {
            errors.push(format!(
                "'{section}' com subitem fora do formato: {line} (use: \"  - Campo: texto\")"
            ));
            continue;
        };
        let Some((_, seen)) = evidences.last_mut() else {
            errors.push(format!(
                "'{section}' com subitem antes de qualquer evidência: {line}"
            ));
            continue;
        };
        if let Some(fields) = fields {
            if !fields
                .required
                .iter()
                .chain(&fields.optional)
                .any(|known| known == field)
            {
                errors.push(format!(
                    "'{section}' com subitem desconhecido '{field}' (use: {})",
                    fields
                        .required
                        .iter()
                        .chain(&fields.optional)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }
        seen.push(field);
    }
    if let (Some(fields), true) = (fields, require) {
        for (evidence, seen) in &evidences {
            for field in &fields.required {
                let alternative = EVIDENCE_ALTERNATIVES
                    .iter()
                    .find(|(required, _)| required == field)
                    .map(|(_, alternative)| *alternative);
                let present = seen.contains(&field.as_str())
                    || alternative.is_some_and(|alternative| seen.contains(&alternative));
                if present {
                    continue;
                }
                errors.push(match alternative {
                    Some(alternative) => format!(
                        "'{section}' com evidência sem o subitem '{field}' ou '{alternative}': {evidence}"
                    ),
                    None => format!(
                        "'{section}' com evidência sem o subitem '{field}': {evidence}"
                    ),
                });
            }
        }
    }
    errors
}

const LOG_HEADER: [&str; 4] = ["$ ", "# data:", "# commit:", "# código de saída:"];

/// Textos entre crases do Trecho ou da Falha, na ordem em que aparecem.
pub fn snippets_of(value: &str) -> Vec<String> {
    let mut texts = Vec::new();
    let mut rest = value;
    while let Some(open) = rest.find('`') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('`') else { break };
        if close > 0 {
            texts.push(after[..close].to_string());
            rest = &after[close + 1..];
        } else {
            rest = after;
        }
    }
    texts
}

pub const FAILURE_IN_EXCERPT: &str = "está no Trecho";
pub const FAILURE_CONDITION: &str = "condição sobre o valor";

const CONDITION_WORDS: [&str; 8] = [
    "diferente de",
    "maior que",
    "menor que",
    "acima de",
    "abaixo de",
    "outro",
    "outra",
    "sem",
];

/// Primeiro texto do Trecho que contém o texto da Falha, pela mesma busca do log.
pub fn excerpt_holding(trecho: &str, text: &str) -> Option<String> {
    snippets_of(trecho)
        .into_iter()
        .find(|excerpt| !find_lines(excerpt, text).is_empty())
}

/// Palavra de condição junto do texto na Falha, na mesma oração: as três últimas palavras de antes
/// ou as três primeiras de depois.
pub fn condition_near(falha: &str, text: &str) -> Option<&'static str> {
    let quoted = format!("`{text}`");
    let is_break = |c: char| matches!(c, ';' | '.' | ',' | '`');
    falha.match_indices(&quoted).find_map(|(start, _)| {
        let before = &falha[..start];
        let before = &before[before.rfind(is_break).map_or(0, |i| i + 1)..];
        let after = &falha[start + quoted.len()..];
        let after = &after[..after.find(is_break).unwrap_or(after.len())];
        let last: Vec<&str> = before.split_whitespace().rev().take(3).collect();
        let last: Vec<&str> = last.into_iter().rev().collect();
        let first: Vec<&str> = after.split_whitespace().take(3).collect();
        condition_word(&last.join(" ")).or_else(|| condition_word(&first.join(" ")))
    })
}

fn condition_word(clause: &str) -> Option<&'static str> {
    let lower = clause.to_lowercase();
    CONDITION_WORDS.into_iter().find(|word| {
        lower.match_indices(word).any(|(at, _)| {
            let before = lower[..at].chars().next_back();
            let after = lower[at + word.len()..].chars().next();
            !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
        })
    })
}

/// Motivo de um texto da Falha ser suspeito: já está no Trecho ou vem com condição sobre o valor.
pub fn failure_reason(trecho: &str, falha: &str, text: &str) -> &'static str {
    if excerpt_holding(trecho, text).is_some() {
        FAILURE_IN_EXCERPT
    } else if condition_near(falha, text).is_some() {
        FAILURE_CONDITION
    } else {
        ""
    }
}

/// Motivo de um texto não ser buscado, ou vazio quando dá para buscar.
pub fn unsearchable(snippet: &str) -> &'static str {
    // a barra na ponta é o que sobra de uma crase escapada, que cortou o texto no meio
    if snippet.starts_with('\\') || snippet.ends_with('\\') {
        return "crase dentro do texto";
    }
    let compact: String = snippet.split_whitespace().collect();
    // pontuação solta, como `+` ou `|`, acharia quase qualquer linha do log
    if !compact.is_empty() && compact.chars().all(|c| c.is_ascii_punctuation()) {
        return "símbolo solto";
    }
    ""
}

fn strip_ansi(text: &str) -> String {
    let mut clean = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' || chars.peek() != Some(&'[') {
            clean.push(c);
            continue;
        }
        chars.next();
        while chars.next_if(|p| ('\x30'..='\x3f').contains(p)).is_some() {}
        while chars.next_if(|p| ('\x20'..='\x2f').contains(p)).is_some() {}
        chars.next_if(|p| ('\x40'..='\x7e').contains(p));
    }
    clean
}

fn normalize(text: &str) -> String {
    strip_ansi(text)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn header_length(lines: &[&str]) -> usize {
    LOG_HEADER
        .iter()
        .zip(lines)
        .take_while(|(prefix, line)| line.starts_with(**prefix))
        .count()
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric()
}

/// Primeira posição de `part` em `text` a partir de `from` cujos vizinhos respeitam os limites de palavra pedidos.
fn find_part(
    text: &str,
    part: &str,
    from: usize,
    word_before: bool,
    word_after: bool,
) -> Option<usize> {
    let mut start = from;
    while let Some(offset) = text[start..].find(part) {
        let at = start + offset;
        let end = at + part.len();
        let before_ok = !word_before || !text[..at].chars().next_back().is_some_and(is_word_char);
        let after_ok = !word_after || !text[end..].chars().next().is_some_and(is_word_char);
        if before_ok && after_ok {
            return Some(at);
        }
        start = at + text[at..].chars().next().map_or(1, char::len_utf8);
    }
    None
}

fn line_matches(text: &str, wanted: &[&str], word_before: bool, word_after: bool) -> bool {
    let last = wanted.len() - 1;
    let first_needs_after = last == 0 && word_after;
    let Some(first) = find_part(text, wanted[0], 0, word_before, first_needs_after) else {
        return false;
    };
    if last == 0 {
        return true;
    }
    let mut cursor = first + wanted[0].len();
    for part in &wanted[1..last] {
        match find_part(text, part, cursor, false, false) {
            Some(at) => cursor = at + part.len(),
            None => return false,
        }
    }
    find_part(text, wanted[last], cursor, false, word_after).is_some()
}

/// Índices das linhas do log que o texto do Trecho ou da Falha aponta, pela mesma regra do painel (RF-22).
pub fn find_lines(log: &str, snippet: &str) -> Vec<usize> {
    let parts: Vec<String> = snippet
        .split("...")
        .flat_map(|p| p.split('…'))
        .map(normalize)
        .collect();
    let wanted: Vec<&str> = parts
        .iter()
        .map(String::as_str)
        .filter(|p| !p.is_empty())
        .collect();
    if wanted.is_empty() {
        return Vec::new();
    }
    let raw: Vec<&str> = log.split('\n').collect();
    let body: Vec<(usize, String)> = raw
        .iter()
        .enumerate()
        .skip(header_length(&raw))
        .map(|(index, line)| (index, normalize(line)))
        .collect();
    let word_before = parts.first().is_some_and(|p| !p.is_empty());
    let word_after = parts.last().is_some_and(|p| !p.is_empty());

    if parts.len() == 1 {
        let equal: Vec<usize> = body
            .iter()
            .filter(|(_, text)| text == wanted[0])
            .map(|(i, _)| *i)
            .collect();
        if !equal.is_empty() {
            return equal;
        }
    }
    body.iter()
        .filter(|(_, text)| line_matches(text, &wanted, word_before, word_after))
        .map(|(index, _)| *index)
        .collect()
}

#[cfg(test)]
mod evidence_tests {
    use super::*;

    fn fields() -> EvidenceFields {
        EvidenceFields {
            required: vec!["Prova".into(), "Trecho".into(), "Falha".into()],
            optional: vec![
                "Artefato".into(),
                "Mostra".into(),
                "Cobre".into(),
                "Onde olhar".into(),
                "Reproduzir".into(),
            ],
        }
    }

    const COMPLETE: &str =
        "- `a.txt` (automática): `true`\n  - Prova: p\n  - Trecho: t\n  - Falha: f\n";

    #[test]
    fn subitems_follow_the_format() {
        assert_eq!(evidence_subitem("  - Prova: algo"), Some(("Prova", "algo")));
        assert_eq!(evidence_subitem("- Prova: algo"), None);
        assert_eq!(evidence_subitem("  - Prova algo"), None);
        assert_eq!(evidence_subitem("  - Quatro palavras no nome: x"), None);
        assert_eq!(
            evidence_subitem("  - Onde olhar: linha 3"),
            Some(("Onde olhar", "linha 3"))
        );
        assert_eq!(evidence_subitem("  - Onde  olhar: x"), None);
    }

    #[test]
    fn complete_evidence_has_no_errors() {
        assert!(evidence_subitem_errors(COMPLETE, "Evidências", Some(&fields()), true).is_empty());
        let with_optional = format!("{COMPLETE}  - Artefato: v1\n");
        assert!(
            evidence_subitem_errors(&with_optional, "Evidências", Some(&fields()), true).is_empty()
        );
    }

    #[test]
    fn missing_required_subitem_is_reported_only_when_required() {
        let text = "- `a.txt` (automática): `true`\n  - Prova: p\n";
        let errors = evidence_subitem_errors(text, "Evidências", Some(&fields()), true);
        assert_eq!(errors.len(), 2, "{errors:?}");
        assert!(errors[0].contains("sem o subitem 'Trecho'"));
        assert!(evidence_subitem_errors(text, "Evidências", Some(&fields()), false).is_empty());
    }

    #[test]
    fn evidence_without_subitems_is_accepted_when_not_required() {
        let text = "- `a.txt` (automática): `true`\n";
        assert!(evidence_subitem_errors(text, "Evidências", Some(&fields()), false).is_empty());
        assert!(evidence_subitem_errors(text, "Evidências", None, true).is_empty());
    }

    #[test]
    fn unknown_and_malformed_subitems_are_reported() {
        let text = format!("{COMPLETE}  - Outro: x\n  - sem dois pontos\n");
        let errors = evidence_subitem_errors(&text, "Evidências", Some(&fields()), false);
        assert_eq!(errors.len(), 2, "{errors:?}");
        assert!(errors[0].contains("desconhecido 'Outro'"));
        assert!(errors[1].contains("fora do formato"));
    }

    #[test]
    fn subitem_before_any_evidence_is_reported() {
        let errors = evidence_subitem_errors("  - Prova: p\n", "Evidências", None, false);
        assert!(errors[0].contains("antes de qualquer evidência"));
    }

    #[test]
    fn new_fields_mostra_replaces_prova() {
        let text = "- `a.txt` (automática): `true`\n  - Mostra: m\n  - Trecho: t\n  - Falha: f\n";
        assert!(evidence_subitem_errors(text, "Evidências", Some(&fields()), true).is_empty());
    }

    #[test]
    fn new_fields_mostra_and_prova_together_are_accepted() {
        let text = format!("{COMPLETE}  - Mostra: m\n");
        assert!(evidence_subitem_errors(&text, "Evidências", Some(&fields()), true).is_empty());
    }

    #[test]
    fn new_fields_missing_prova_and_mostra_names_both() {
        let text = "- `a.txt` (automática): `true`\n  - Trecho: t\n  - Falha: f\n";
        let errors = evidence_subitem_errors(text, "Evidências", Some(&fields()), true);
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].contains("sem o subitem 'Prova' ou 'Mostra'"));
    }

    #[test]
    fn new_fields_all_optional_fields_are_known() {
        let text = format!(
            "{COMPLETE}  - Mostra: m\n  - Cobre: RF-01, CT-01\n  - Onde olhar: 0:05, o botão muda\n  - Reproduzir: `vostok painel`\n"
        );
        for require in [false, true] {
            let errors = evidence_subitem_errors(&text, "Evidências", Some(&fields()), require);
            assert!(errors.is_empty(), "{errors:?}");
        }
    }

    #[test]
    fn new_fields_unknown_multi_word_field_is_reported() {
        let text = format!("{COMPLETE}  - Onde ver: x\n");
        let errors = evidence_subitem_errors(&text, "Evidências", Some(&fields()), false);
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].contains("desconhecido 'Onde ver'"));
    }
}

#[cfg(test)]
mod snippet_tests {
    use super::*;

    const CASES: &str = include_str!("../../painel/src/features/task/snippet-cases.json");
    const HEADER: &str = "$ cmd\n# data: d\n# commit: c\n# código de saída: 0\n";

    #[test]
    fn snippet_shared_cases_match_the_painel() {
        let cases: serde_json::Value = serde_json::from_str(CASES).unwrap();
        let cases = cases.as_array().unwrap();
        assert!(!cases.is_empty());
        for case in cases {
            let name = case["name"].as_str().unwrap();
            let log = case["log"].as_str().unwrap();
            let snippet = case["snippet"].as_str().unwrap();
            let reason = case["unsearchable"].as_str().unwrap();
            assert_eq!(unsearchable(snippet), reason, "caso: {name}");
            if reason.is_empty() {
                let lines: Vec<usize> = case["lines"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|line| line.as_u64().unwrap() as usize)
                    .collect();
                assert_eq!(find_lines(log, snippet), lines, "caso: {name}");
            }
        }
    }

    #[test]
    fn snippet_snippets_of_reads_the_backtick_texts() {
        assert_eq!(snippets_of("veja `a b` e `c`"), vec!["a b", "c"]);
        assert!(snippets_of("sem crases").is_empty());
        assert!(snippets_of("``").is_empty());
    }

    #[test]
    fn snippet_header_and_ansi_are_ignored() {
        let log = format!("{HEADER}\u{1b}[32mok\u{1b}[39m");
        assert_eq!(find_lines(&log, "ok"), vec![4]);
        assert!(find_lines(&log, "cmd").is_empty());
        assert!(find_lines(&log, "32m").is_empty());
    }
}

#[cfg(test)]
mod falha_tests {
    use super::*;

    #[test]
    fn falha_reason_nome_contido_no_trecho() {
        let reason = failure_reason(
            "`mp4_referencia=152`",
            "se `mp4_referencia` diferente de 152",
            "mp4_referencia",
        );
        assert_eq!(reason, FAILURE_IN_EXCERPT);
    }

    #[test]
    fn falha_reason_ausencia_repete_o_trecho() {
        let reason = failure_reason("`resultado=ok`", "sem `resultado=ok`", "resultado=ok");
        assert_eq!(reason, FAILURE_IN_EXCERPT);
    }

    #[test]
    fn falha_reason_texto_maior_que_o_trecho_nao_e_contradicao() {
        assert_eq!(failure_reason("`rc`", "se aparecer `rc=1`", "rc=1"), "");
    }

    #[test]
    fn falha_reason_condicao_depois() {
        let falha = "se `rm_na_referencia` for maior que 0";
        assert_eq!(
            failure_reason("", falha, "rm_na_referencia"),
            FAILURE_CONDITION
        );
    }

    #[test]
    fn falha_reason_condicao_antes() {
        let falha = "outra versão em `versao_instalada`";
        assert_eq!(
            failure_reason("", falha, "versao_instalada"),
            FAILURE_CONDITION
        );
    }

    #[test]
    fn falha_reason_condicao_em_outra_oracao_nao_conta() {
        let falha = "se aparecer `FAILED`; sem isso, tudo certo";
        assert_eq!(failure_reason("", falha, "FAILED"), "");
    }

    #[test]
    fn falha_reason_valor_literal_passa() {
        let falha = "se aparecer `rm_na_referencia=1`";
        assert_eq!(failure_reason("", falha, "rm_na_referencia=1"), "");
    }

    #[test]
    fn falha_shared_cases_match_the_painel() {
        let cases: serde_json::Value =
            serde_json::from_str(include_str!("../../painel/src/features/task/snippet-cases.json"))
                .unwrap();
        let mut checked = 0;
        for case in cases.as_array().unwrap() {
            let Some(falha) = case["falha"].as_str() else { continue };
            let name = case["name"].as_str().unwrap();
            let trecho = case["trecho"].as_str().unwrap();
            let snippet = case["snippet"].as_str().unwrap();
            let expected = case["failure"].as_str().unwrap();
            assert_eq!(failure_reason(trecho, falha, snippet), expected, "caso: {name}");
            checked += 1;
        }
        assert!(checked > 0);
    }
}
