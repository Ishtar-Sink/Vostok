//! Comandos `finalize`: finalização de specs implementadas.

use anyhow::Result;
use clap::Subcommand;

use super::not_implemented;
use crate::harness::Harness;

#[derive(Debug, Subcommand)]
pub enum FinalizeCommand {
    /// Imprime as fontes da síntese.
    Sources { spec_id: String },
    /// Cria o conteúdo dos documentos.
    Create { spec_id: String },
    /// Valida o conteúdo.
    Validate { spec_id: String },
    /// Gera os documentos HTML.
    Build { spec_id: String },
    /// Arquiva a spec e publica os documentos.
    Archive { spec_id: String },
}

pub fn run(command: FinalizeCommand) -> Result<()> {
    let _harness = Harness::from_current_dir()?;
    let name = match command {
        FinalizeCommand::Sources { .. } => "finalize sources",
        FinalizeCommand::Create { .. } => "finalize create",
        FinalizeCommand::Validate { .. } => "finalize validate",
        FinalizeCommand::Build { .. } => "finalize build",
        FinalizeCommand::Archive { .. } => "finalize archive",
    };
    not_implemented(name)
}
