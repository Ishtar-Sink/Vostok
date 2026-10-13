//! Comandos `skill`: criação de skills.

use anyhow::Result;
use clap::Subcommand;

use super::not_implemented;
use crate::harness::Harness;

#[derive(Debug, Subcommand)]
pub enum SkillCommand {
    /// Cria o SKILL.md de uma skill nova.
    Create {
        /// Cria a skill entre as padrão do harness.
        #[arg(long)]
        default: bool,
        name: String,
    },
    /// Cria um script na pasta da skill.
    Script { name: String, script: String },
    /// Valida a skill e gera redirecionamentos e catálogo.
    Finalize { name: String },
}

pub fn run(command: SkillCommand) -> Result<()> {
    let _harness = Harness::from_current_dir()?;
    let name = match command {
        SkillCommand::Create { .. } => "skill create",
        SkillCommand::Script { .. } => "skill script",
        SkillCommand::Finalize { .. } => "skill finalize",
    };
    not_implemented(name)
}
