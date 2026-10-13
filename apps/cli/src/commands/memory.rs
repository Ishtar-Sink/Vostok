//! Comandos `memory`: memória do projeto: decisões, issues e conhecimento.

use anyhow::Result;
use clap::Subcommand;

use super::not_implemented;
use crate::harness::Harness;

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
    let _harness = Harness::from_current_dir()?;
    let name = match command {
        MemoryCommand::Init => "memory init",
        MemoryCommand::Add { .. } => "memory add",
        MemoryCommand::Search { .. } => "memory search",
        MemoryCommand::Update { .. } => "memory update",
    };
    not_implemented(name)
}
