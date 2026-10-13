//! Comandos `spec`: specs: criação, refinamento, ajuste e aprovação.

use anyhow::Result;
use clap::Subcommand;

use super::not_implemented;
use crate::harness::Harness;

#[derive(Debug, Subcommand)]
pub enum SpecCommand {
    /// Cria uma spec em rascunho.
    New { title: String },
    /// Lista as specs, opcionalmente filtradas por status.
    List { status: Option<String> },
    /// Cria a spec refinada com as seções do tipo.
    Refine { spec_id: String, kind: String },
    /// Valida a spec refinada.
    Validate { spec_id: String },
    /// Grava o hash da refinada e marca o rascunho como refinado.
    MarkRefined { spec_id: String },
    /// Ajustes de uma spec refinada.
    #[command(subcommand)]
    Adjust(AdjustCommand),
    /// Aprovação de uma spec refinada.
    #[command(subcommand)]
    Approve(ApproveCommand),
}

#[derive(Debug, Subcommand)]
pub enum AdjustCommand {
    /// Registra edições feitas à mão.
    RecordManual { spec_id: String },
    /// Valida, sem alterar nada.
    Validate { spec_id: String },
    /// Registra o ajuste no histórico.
    Record { spec_id: String, summary: String },
}

#[derive(Debug, Subcommand)]
pub enum ApproveCommand {
    /// Lista os impedimentos, sem alterar nada.
    Validate { spec_id: String },
    /// Aprova a spec.
    Approve { spec_id: String },
}

pub fn run(command: SpecCommand) -> Result<()> {
    let _harness = Harness::from_current_dir()?;
    let name = match command {
        SpecCommand::New { .. } => "spec new",
        SpecCommand::List { .. } => "spec list",
        SpecCommand::Refine { .. } => "spec refine",
        SpecCommand::Validate { .. } => "spec validate",
        SpecCommand::MarkRefined { .. } => "spec mark-refined",
        SpecCommand::Adjust(_) => "spec adjust",
        SpecCommand::Approve(_) => "spec approve",
    };
    not_implemented(name)
}
