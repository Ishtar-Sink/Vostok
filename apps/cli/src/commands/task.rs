//! Comandos `task`: tarefas: aprovação, ajuste, execução e conclusão.

use anyhow::Result;
use clap::Subcommand;

use super::not_implemented;
use crate::harness::Harness;

#[derive(Debug, Subcommand)]
pub enum TaskCommand {
    /// Imprime a seção de execução da tarefa.
    Load { spec_id: String, task_id: String },
    /// Confere os critérios de conclusão.
    CheckDone { spec_id: String, task_id: String },
    /// Aprovação de tarefas.
    #[command(subcommand)]
    Approve(ApproveCommand),
    /// Ajustes de uma tarefa.
    #[command(subcommand)]
    Adjust(AdjustCommand),
    /// Inicia a execução.
    Start { spec_id: String, task_id: String },
    /// Coleta o baseline.
    Baseline { spec_id: String, task_id: String },
    /// Executa a verificação.
    Tests { spec_id: String, task_id: String },
    /// Compara com o baseline.
    Regression { spec_id: String, task_id: String },
    /// Grava as evidências.
    Evidence { spec_id: String, task_id: String },
    /// Conclui a tarefa.
    Finish { spec_id: String, task_id: String },
    /// Retoma uma tarefa bloqueada com a decisão do usuário.
    Resume {
        spec_id: String,
        task_id: String,
        decision: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum AdjustCommand {
    /// Registra edições feitas à mão.
    RecordManual { spec_id: String, task_id: String },
    /// Valida, sem alterar nada.
    Validate { spec_id: String, task_id: String },
    /// Registra o ajuste no histórico.
    Record {
        spec_id: String,
        task_id: String,
        summary: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum ApproveCommand {
    /// Lista as tarefas que podem ser aprovadas.
    Validate {
        spec_id: String,
        task_ids: Vec<String>,
        #[arg(long)]
        all: bool,
    },
    /// Aprova as tarefas, todas ou nenhuma.
    Approve {
        spec_id: String,
        task_ids: Vec<String>,
        #[arg(long)]
        all: bool,
    },
}

pub fn run(command: TaskCommand) -> Result<()> {
    let _harness = Harness::from_current_dir()?;
    let name = match command {
        TaskCommand::Load { .. } => "task load",
        TaskCommand::CheckDone { .. } => "task check-done",
        TaskCommand::Approve(_) => "task approve",
        TaskCommand::Adjust(_) => "task adjust",
        TaskCommand::Start { .. } => "task start",
        TaskCommand::Baseline { .. } => "task baseline",
        TaskCommand::Tests { .. } => "task tests",
        TaskCommand::Regression { .. } => "task regression",
        TaskCommand::Evidence { .. } => "task evidence",
        TaskCommand::Finish { .. } => "task finish",
        TaskCommand::Resume { .. } => "task resume",
    };
    not_implemented(name)
}
