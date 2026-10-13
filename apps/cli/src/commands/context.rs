//! Comandos `context`: carregamento de contexto para o agente.

use anyhow::Result;
use clap::Subcommand;

use super::not_implemented;
use crate::harness::Harness;

#[derive(Debug, Subcommand)]
pub enum ContextCommand {
    /// Carrega o contexto obrigatório; com caminhos ou `--skill`, o contexto adicional.
    Load {
        /// Skill cuja entrada do mapa `skills` do `config/context.yml` é carregada.
        #[arg(long)]
        skill: Option<String>,
        /// Arquivos ou pastas, relativos à raiz do harness.
        paths: Vec<String>,
    },
}

pub fn run(command: ContextCommand) -> Result<()> {
    let _harness = Harness::from_current_dir()?;
    let name = match command {
        ContextCommand::Load { .. } => "context load",
    };
    not_implemented(name)
}
