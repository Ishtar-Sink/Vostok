//! Comandos `bug`: relato de defeito do próprio harness, validado e transformado em link de nova
//! issue no repositório de `harness.repository` (SPEC-008).

use std::fs;
use std::path::Path;

use anyhow::{bail, Result};
use clap::Subcommand;

use crate::harness::Harness;
use crate::support::{config_get, frontmatter_get, frontmatter_lines, template_leftovers};

const CHECK_USAGE: &str = "Uso: vostok bug check <arquivo>";
const LINK_USAGE: &str = "Uso: vostok bug link <arquivo>";
const TEMPLATE: &str = "bug/report.md";
const TITLE_PREFIX: &str = "[harness-bug]";
const FRONTMATTER_KEYS: [&str; 3] = ["issue", "title", "created_at"];
/// Acima disso o navegador ou o GitHub podem cortar o link.
pub(crate) const MAX_LINK: usize = 8000;

#[derive(Debug, Subcommand)]
pub enum BugCommand {
    /// Valida um relato de defeito do harness, sem alterar nada.
    Check { file: Option<String> },
    /// Imprime o link de nova issue com o título e o corpo do relato.
    Link { file: Option<String> },
}

pub fn run(command: BugCommand) -> Result<()> {
    let harness = Harness::from_current_dir()?;
    match command {
        BugCommand::Check { file } => {
            let file = file.ok_or_else(|| anyhow::Error::msg(CHECK_USAGE))?;
            check(&harness, &file)?;
            println!("Relato válido: {file}");
            Ok(())
        }
        BugCommand::Link { file } => {
            let file = file.ok_or_else(|| anyhow::Error::msg(LINK_USAGE))?;
            link(&harness, &file)
        }
    }
}

/// Relato validado: título e corpo, sem front-matter nem comentários.
struct Report {
    title: String,
    body: String,
}

fn check(harness: &Harness, file: &str) -> Result<Report> {
    let path = Path::new(file);
    let Ok(content) = fs::read_to_string(path) else {
        bail!("relato não encontrado: {file}");
    };
    let mut problems = Vec::new();
    match frontmatter_lines(&content) {
        None => problems.push("relato sem front-matter".to_string()),
        Some(lines) => {
            for key in FRONTMATTER_KEYS {
                if !lines
                    .iter()
                    .any(|line| line.starts_with(&format!("{key}:")))
                {
                    problems.push(format!("front-matter sem a chave '{key}'"));
                }
            }
        }
    }
    let title = frontmatter_get(path, "title").unwrap_or_default();
    let has_title = title
        .strip_prefix(TITLE_PREFIX)
        .is_some_and(|rest| !rest.trim().is_empty());
    if !has_title {
        problems.push(format!(
            "título precisa começar com '{TITLE_PREFIX}' e ter um resumo: '{title}'"
        ));
    }

    let body = without_comments(body_of(&content));
    let template = fs::read_to_string(harness.path("templates")?.join(TEMPLATE))?;
    for heading in template.lines().filter(|line| line.starts_with("## ")) {
        match section(&body, heading) {
            None => problems.push(format!("seção ausente: {heading}")),
            Some(text) if text.trim().is_empty() => {
                problems.push(format!("seção vazia: {heading}"));
            }
            Some(_) => {}
        }
    }
    for (line, text) in template_leftovers(&content) {
        problems.push(format!("variável sem preencher: linha {line}: {text}"));
    }
    if !problems.is_empty() {
        bail!(
            "o relato {file} tem problemas; nada foi alterado:\n- {}",
            problems.join("\n- ")
        );
    }
    Ok(Report {
        title,
        body: body.trim().to_string(),
    })
}

fn link(harness: &Harness, file: &str) -> Result<()> {
    let report = check(harness, file)?;
    let repository = config_get(&harness.config_file("main.yml"), "harness.repository")?;
    let base = format!(
        "https://github.com/{repository}/issues/new?title={}",
        percent_encode(&report.title)
    );
    let full = format!("{base}&body={}", percent_encode(&report.body));
    if full.len() <= MAX_LINK {
        println!("{full}");
    } else {
        println!("{base}");
        println!(
            "O relato passa do limite de {MAX_LINK} caracteres do link: abra o link e cole o corpo de {file} na issue."
        );
    }
    Ok(())
}

/// Conteúdo depois do front-matter.
pub(crate) fn body_of(content: &str) -> &str {
    content
        .strip_prefix("---\n")
        .and_then(|rest| rest.find("\n---\n").map(|end| &rest[end + 5..]))
        .unwrap_or(content)
}

pub(crate) fn without_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("<!--") {
        out.push_str(&rest[..start]);
        match rest[start..].find("-->") {
            Some(end) => rest = &rest[start + end + 3..],
            None => {
                rest = "";
                break;
            }
        }
    }
    out.push_str(rest);
    out
}

/// Texto da seção `heading` até a próxima seção `## `.
pub(crate) fn section<'a>(body: &'a str, heading: &str) -> Option<&'a str> {
    let start = body
        .match_indices(heading)
        .find(|(index, _)| {
            (*index == 0 || body[..*index].ends_with('\n'))
                && body[index + heading.len()..].starts_with(['\n', '\r'])
        })
        .map(|(index, _)| index + heading.len())?;
    let rest = &body[start..];
    let end = rest.find("\n## ").map_or(rest.len(), |end| end + 1);
    Some(&rest[..end])
}

/// Percent-encoding de URL: só os caracteres não reservados ficam como estão.
pub(crate) fn percent_encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 3);
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || b"-_.~".contains(&byte) {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}
