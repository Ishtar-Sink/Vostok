//! `vostok perguntas`: perguntas do agente ao usuário, gravadas para o painel responder.

use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::Subcommand;

use crate::commands::pedidos::{Questions, Requests, ANSWERED, OPEN};
use crate::harness::Harness;
use crate::support::{frontmatter_get, utc_datetime};

#[derive(Debug, Subcommand)]
pub enum PerguntasCommand {
    /// Grava uma pergunta aberta do agente para o pedido; o texto vem da entrada padrão.
    Abrir { pedido: String },
}

pub fn run(command: PerguntasCommand) -> Result<()> {
    let harness = Harness::from_current_dir()?;
    match command {
        PerguntasCommand::Abrir { pedido } => {
            let mut text = String::new();
            std::io::stdin().read_to_string(&mut text)?;
            open(&harness, &pedido, &text)
        }
    }
}

fn is_request_id(id: &str) -> bool {
    id.strip_prefix("PED-")
        .is_some_and(|number| number.len() >= 3 && number.chars().all(|c| c.is_ascii_digit()))
}

fn open(harness: &Harness, request_id: &str, text: &str) -> Result<()> {
    if !is_request_id(request_id) {
        bail!("ID de pedido inválido '{request_id}': use o formato PED-NNN");
    }
    let requests = Requests::open(harness)?;
    let Some(request) = requests.find(request_id) else {
        bail!(
            "Pedido {request_id} não encontrado em {}",
            requests.dir().display()
        );
    };
    let text = text.trim();
    if text.is_empty() {
        bail!("o texto da pergunta veio vazio: envie as perguntas e as opções pela entrada padrão");
    }
    let spec = frontmatter_get(&request, "spec").unwrap_or_default();
    let task = frontmatter_get(&request, "tarefa").unwrap_or_default();
    let questions = Questions::open(harness)?;
    let open_dir = questions
        .folder_of(OPEN)
        .context("pasta das perguntas abertas")?;
    let mut number = next_question_number(harness, &questions)?;
    fs::create_dir_all(&open_dir).with_context(|| format!("criando {}", open_dir.display()))?;
    loop {
        let id = format!("P-{number:03}");
        let path = open_dir.join(format!("{id}.md"));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                write!(
                    file,
                    "---\nid: {id}\npedido: {request_id}\nspec: {spec}\ntask: {task}\nstatus: {OPEN}\ncreated_at: {}\n---\n\n## Pergunta\n\n{text}\n",
                    utc_datetime()
                )
                .with_context(|| format!("gravando {}", path.display()))?;
                println!(
                    "Pergunta {id} aberta para o pedido {request_id}: {}",
                    path.display()
                );
                return Ok(());
            }
            Err(error) if error.kind() == ErrorKind::AlreadyExists => number += 1,
            Err(error) => {
                return Err(error).with_context(|| format!("gravando {}", path.display()))
            }
        }
    }
}

/// O número seguinte ao maior `P-NNN` entre as abertas, as respondidas e as arquivadas com as specs.
fn next_question_number(harness: &Harness, questions: &Questions) -> Result<u32> {
    let mut dirs: Vec<PathBuf> = [OPEN, ANSWERED]
        .iter()
        .filter_map(|status| questions.folder_of(status))
        .collect();
    if let (Ok(archive), Some(name)) = (harness.path("archive"), questions.dir().file_name()) {
        if let Ok(specs) = fs::read_dir(archive) {
            dirs.extend(specs.flatten().map(|spec| spec.path().join(name)));
        }
    }
    let highest = dirs
        .iter()
        .flat_map(|dir| numbers_in(dir))
        .max()
        .unwrap_or(0);
    Ok(highest + 1)
}

fn numbers_in(dir: &Path) -> Vec<u32> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().into_string().ok()?;
            name.strip_suffix(".md")?.strip_prefix("P-")?.parse().ok()
        })
        .collect()
}
