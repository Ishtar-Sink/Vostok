//! Comando `pedidos`: os pedidos que o painel grava em `work/pedidos/` e o agente processa.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::Subcommand;

use crate::harness::Harness;
use crate::support::{frontmatter_get, frontmatter_set, utc_datetime};

const PENDING: &str = "pendente";
const IN_PROGRESS: &str = "em_andamento";
const DONE: &str = "feito";
const REFUSED: &str = "recusado";
const RESULT_HEADING: &str = "## Resultado";

#[derive(Debug, Subcommand)]
pub enum PedidosCommand {
    /// Lista os pedidos, em ordem de ID.
    Listar {
        /// Mostra só os pedidos neste estado.
        #[arg(long)]
        estado: Option<String>,
    },
    /// Leva um pedido de `pendente` a `em_andamento`.
    Iniciar { id: String },
    /// Conclui um pedido `em_andamento`; o resultado vem da entrada padrão.
    Concluir { id: String },
    /// Recusa um pedido `em_andamento`; o motivo vem da entrada padrão.
    Recusar { id: String },
}

pub fn run(command: PedidosCommand) -> Result<()> {
    let dir = Harness::from_current_dir()?
        .root
        .join("work")
        .join("pedidos");
    match command {
        PedidosCommand::Listar { estado } => list(&dir, estado.as_deref()),
        PedidosCommand::Iniciar { id } => transition(&dir, &id, &[PENDING], IN_PROGRESS, false),
        PedidosCommand::Concluir { id } => transition(&dir, &id, &[IN_PROGRESS], DONE, true),
        PedidosCommand::Recusar { id } => transition(&dir, &id, &[IN_PROGRESS], REFUSED, true),
    }
}

fn is_request_id(id: &str) -> bool {
    id.strip_prefix("PED-")
        .is_some_and(|number| number.len() >= 3 && number.chars().all(|c| c.is_ascii_digit()))
}

fn request_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| {
                    path.file_stem()
                        .and_then(|stem| stem.to_str())
                        .is_some_and(is_request_id)
                        && path.extension().is_some_and(|extension| extension == "md")
                })
                .collect()
        })
        .unwrap_or_default();
    files.sort();
    files
}

fn list(dir: &Path, state: Option<&str>) -> Result<()> {
    println!("| ID | Ação | Spec | Tarefa | Estado |");
    println!("|---|---|---|---|---|");
    for path in request_files(dir) {
        let field = |key| frontmatter_get(&path, key).unwrap_or_default();
        let current = field("estado");
        if state.is_some_and(|wanted| wanted != current) {
            continue;
        }
        let id = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("");
        println!(
            "| {id} | {} | {} | {} | {current} |",
            field("acao"),
            field("spec"),
            field("tarefa")
        );
    }
    Ok(())
}

fn transition(
    dir: &Path,
    id: &str,
    allowed_from: &[&str],
    to: &str,
    with_result: bool,
) -> Result<()> {
    if !is_request_id(id) {
        bail!("ID de pedido inválido '{id}': use o formato PED-NNN");
    }
    let path = dir.join(format!("{id}.md"));
    if !path.is_file() {
        bail!("Pedido {id} não encontrado em {}", dir.display());
    }
    let current = frontmatter_get(&path, "estado")?;
    if !allowed_from.contains(&current.as_str()) {
        bail!(
            "Pedido {id} está '{current}': só sai de {} para '{to}'",
            allowed_from.join(" ou ")
        );
    }
    let result = if with_result {
        Some(read_stdin()?)
    } else {
        None
    };
    if let Some(text) = result {
        let content = fs::read_to_string(&path)?;
        fs::write(&path, with_result_section(&content, &text))
            .with_context(|| format!("gravar {}", path.display()))?;
    }
    frontmatter_set(&path, "estado", to)?;
    frontmatter_set(&path, "atualizado_em", &utc_datetime())?;
    println!("Pedido {id}: {current} -> {to}");
    Ok(())
}

fn read_stdin() -> Result<String> {
    let mut text = String::new();
    std::io::stdin().read_to_string(&mut text)?;
    Ok(text.trim().to_string())
}

/// Troca o conteúdo de `## Resultado` (ou a cria no fim) pelo texto.
fn with_result_section(content: &str, text: &str) -> String {
    let head = match content.find(RESULT_HEADING) {
        Some(index) => &content[..index],
        None => content,
    };
    format!("{}\n\n{RESULT_HEADING}\n\n{text}\n", head.trim_end())
}
