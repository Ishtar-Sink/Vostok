//! Comandos `plan`: planos de tarefas: criação, níveis, ajuste e aprovação.

use anyhow::Result;
use clap::Subcommand;

use super::not_implemented;
use crate::harness::Harness;

#[derive(Debug, Subcommand)]
pub enum PlanCommand {
    /// Cria o plano de uma spec aprovada.
    Create { spec_id: String },
    /// Calcula e grava os níveis de esforço.
    Score { spec_id: String },
    /// Valida o plano.
    Validate { spec_id: String },
    /// Ajustes do plano.
    #[command(subcommand)]
    Adjust(AdjustCommand),
    /// Aprovação do plano e criação das tarefas.
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
    /// Cria os arquivos das tarefas.
    CreateTasks { spec_id: String },
    /// Valida as tarefas criadas.
    Validate { spec_id: String },
    /// Marca a spec como planejada e remove o plano.
    MarkPlanned { spec_id: String },
}

pub fn run(command: PlanCommand) -> Result<()> {
    let _harness = Harness::from_current_dir()?;
    let name = match command {
        PlanCommand::Create { .. } => "plan create",
        PlanCommand::Score { .. } => "plan score",
        PlanCommand::Validate { .. } => "plan validate",
        PlanCommand::Adjust(_) => "plan adjust",
        PlanCommand::Approve(_) => "plan approve",
    };
    not_implemented(name)
}
