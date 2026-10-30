//! Comandos `suggest`: sugestão de melhoria ou de recurso do próprio harness, validada e
//! transformada em link de nova issue no repositório de `harness.repository` (SPEC-014).

use std::fs;
use std::path::Path;

use anyhow::{bail, Result};
use clap::Subcommand;

use crate::commands::bug::{body_of, percent_encode, section, without_comments, MAX_LINK};
use crate::harness::Harness;
use crate::support::{config_get, frontmatter_get, frontmatter_lines, template_leftovers};

const CHECK_USAGE: &str = "Uso: vostok suggest check <arquivo>";
const LINK_USAGE: &str = "Uso: vostok suggest link <arquivo>";
const FRONTMATTER_KEYS: [&str; 4] = ["type", "issue", "title", "created_at"];
/// Tipos aceitos e o prefixo do título de cada um.
const KINDS: [(&str, &str); 2] = [
    ("improvement", "[harness-improvement]"),
    ("feature", "[harness-feature]"),
];

#[derive(Debug, Subcommand)]
pub enum SuggestCommand {
    /// Valida uma sugestão de melhoria ou de recurso do harness, sem alterar nada.
    Check { file: Option<String> },
    /// Imprime o link de nova issue com o título e o corpo da sugestão.
    Link { file: Option<String> },
}

pub fn run(command: SuggestCommand) -> Result<()> {
    let harness = Harness::from_current_dir()?;
    match command {
        SuggestCommand::Check { file } => {
            let file = file.ok_or_else(|| anyhow::Error::msg(CHECK_USAGE))?;
            check(&harness, &file)?;
            println!("Sugestão válida: {file}");
            Ok(())
        }
        SuggestCommand::Link { file } => {
            let file = file.ok_or_else(|| anyhow::Error::msg(LINK_USAGE))?;
            link(&harness, &file)
        }
    }
}

/// Sugestão validada: título e corpo, sem front-matter nem comentários.
struct Suggestion {
    title: String,
    body: String,
}

fn check(harness: &Harness, file: &str) -> Result<Suggestion> {
    let path = Path::new(file);
    let Ok(content) = fs::read_to_string(path) else {
        bail!("sugestão não encontrada: {file}");
    };
    let mut problems = Vec::new();
    match frontmatter_lines(&content) {
        None => problems.push("sugestão sem front-matter".to_string()),
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
    let kind = frontmatter_get(path, "type").unwrap_or_default();
    let body = without_comments(body_of(&content));
    match KINDS.iter().find(|(name, _)| *name == kind) {
        None => {
            problems.push(format!(
                "tipo inválido '{kind}': use 'improvement' ou 'feature'"
            ));
        }
        Some((name, prefix)) => {
            let has_title = title
                .strip_prefix(prefix)
                .is_some_and(|rest| !rest.trim().is_empty());
            if !has_title {
                problems.push(format!(
                    "título precisa começar com '{prefix}' e ter um resumo: '{title}'"
                ));
            }
            let template = fs::read_to_string(
                harness
                    .path("templates")?
                    .join(format!("suggestion/{name}.md")),
            )?;
            for heading in template.lines().filter(|line| line.starts_with("## ")) {
                match section(&body, heading) {
                    None => problems.push(format!("seção ausente: {heading}")),
                    Some(text) if text.trim().is_empty() => {
                        problems.push(format!("seção vazia: {heading}"));
                    }
                    Some(_) => {}
                }
            }
        }
    }
    for (line, text) in template_leftovers(&content) {
        problems.push(format!("variável sem preencher: linha {line}: {text}"));
    }
    if !problems.is_empty() {
        bail!(
            "a sugestão {file} tem problemas; nada foi alterado:\n- {}",
            problems.join("\n- ")
        );
    }
    Ok(Suggestion {
        title,
        body: body.trim().to_string(),
    })
}

fn link(harness: &Harness, file: &str) -> Result<()> {
    let suggestion = check(harness, file)?;
    let repository = config_get(&harness.config_file("main.yml"), "harness.repository")?;
    let base = format!(
        "https://github.com/{repository}/issues/new?title={}",
        percent_encode(&suggestion.title)
    );
    let full = format!("{base}&body={}", percent_encode(&suggestion.body));
    if full.len() <= MAX_LINK {
        println!("{full}");
    } else {
        println!("{base}");
        println!(
            "A sugestão passa do limite de {MAX_LINK} caracteres do link: abra o link e cole o corpo de {file} na issue."
        );
    }
    Ok(())
}
