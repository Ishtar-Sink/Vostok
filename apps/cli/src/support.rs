//! Utilitários comuns aos comandos: configuração, front-matter, Markdown, templates e hash.
//! Reproduzem as libs bash `config.sh`, `frontmatter.sh`, `markdown.sh` e `text.sh`.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};
use sha2::{Digest, Sha256};

// --- Configuração ---
// Subconjunto de YAML dos arquivos de `config/`: `chave: valor` em até dois níveis (`pai.filho`),
// blocos literais de primeiro nível (`chave: |`) e listas de primeiro nível.

fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("não foi possível ler {}", path.display()))
}

fn is_blank_or_comment(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.is_empty() || trimmed.starts_with('#')
}

/// Caminho `pai.filho` de cada linha de chave, com a indentação e o resto da linha.
struct KeyLine<'a> {
    path: String,
    indent: usize,
    line: &'a str,
}

fn key_lines(content: &str) -> impl Iterator<Item = Option<KeyLine<'_>>> {
    let mut top = String::new();
    content.lines().map(move |raw| {
        if is_blank_or_comment(raw) {
            return None;
        }
        let indent = raw.len() - raw.trim_start_matches(' ').len();
        let line = &raw[indent..];
        let key = line.split(':').next().unwrap_or("").trim_end().to_string();
        if indent == 0 {
            top = key.clone();
        }
        let path = if indent == 0 {
            key
        } else {
            format!("{top}.{key}")
        };
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
            return Ok(unescape(&value[1..value.len() - 1]));
        }
        return Ok(unquote(value).to_string());
    }
    bail!(
        "Chave '{key}' não encontrada no front-matter de {}",
        path.display()
    )
}

fn unescape(value: &str) -> String {
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
