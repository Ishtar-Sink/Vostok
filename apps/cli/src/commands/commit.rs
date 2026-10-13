//! Comandos `commit`: commits liberados pelo usuário.

use anyhow::Result;
use clap::Subcommand;

use super::not_implemented;
use crate::harness::Harness;

#[derive(Debug, Subcommand)]
pub enum CommitCommand {
    /// Recusa se houver commits sem liberação.
    Check,
    /// Lista os commits sem liberação e o código de liberação.
    Show,
    /// Libera os commits apontados.
    Approve { code: String },
    /// Desfaz os commits novos da branch atual.
    Revert { code: String },
    /// Faz um commit já liberado, com a mensagem pela entrada padrão.
    Commit { paths: Vec<String> },
}

pub fn run(command: CommitCommand) -> Result<()> {
    let _harness = Harness::from_current_dir()?;
    let name = match command {
        CommitCommand::Check => "commit check",
        CommitCommand::Show => "commit show",
        CommitCommand::Approve { .. } => "commit approve",
        CommitCommand::Revert { .. } => "commit revert",
        CommitCommand::Commit { .. } => "commit commit",
    };
    not_implemented(name)
}
