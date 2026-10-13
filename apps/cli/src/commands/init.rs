//! Comando `init`: instala o harness no projeto.

use anyhow::Result;
use clap::Args;

use super::not_implemented;

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Usa os valores padrão, sem perguntas.
    #[arg(long)]
    pub yes: bool,
    /// Ferramentas que recebem roteamento, separadas por vírgula (ex.: claude-code,codex).
    #[arg(long)]
    pub tools: Option<String>,
}

pub fn run(_args: InitArgs) -> Result<()> {
    not_implemented("init")
}
