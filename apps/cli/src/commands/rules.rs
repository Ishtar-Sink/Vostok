//! Comandos `rules`: regras do projeto e mapa de carregamento.

use anyhow::Result;
use clap::Subcommand;

use super::not_implemented;
use crate::harness::Harness;

#[derive(Debug, Subcommand)]
pub enum RulesCommand {
    /// Valida o mapa de contexto e os arquivos de regras de projeto.
    Validate,
    /// Lista os sinais do projeto que indicam categorias de regras.
    Detect,
    /// Cria ou atualiza uma categoria a partir de um arquivo de regras.
    Write {
        category: String,
        rules_file: String,
    },
    /// Mostra o que o `write` mudaria, sem gravar.
    Preview {
        category: String,
        rules_file: String,
    },
    /// Registra a categoria em `required` (`always`) ou nas skills indicadas.
    Load { category: String, target: String },
    /// Retira a categoria de `required` (`always`) ou das skills indicadas.
    Unload { category: String, target: String },
    /// Imprime o próximo ID livre da categoria.
    NextId { category: String },
    /// Marca uma regra como removida.
    Remove { category: String, number: String },
    /// Cria um arquivo de categoria vazio pelo template.
    CreateCategory {
        category: String,
        title: String,
        purpose: String,
    },
}

pub fn run(command: RulesCommand) -> Result<()> {
    let _harness = Harness::from_current_dir()?;
    let name = match command {
        RulesCommand::Validate => "rules validate",
        RulesCommand::Detect => "rules detect",
        RulesCommand::Write { .. } => "rules write",
        RulesCommand::Preview { .. } => "rules preview",
        RulesCommand::Load { .. } => "rules load",
        RulesCommand::Unload { .. } => "rules unload",
        RulesCommand::NextId { .. } => "rules next-id",
        RulesCommand::Remove { .. } => "rules remove",
        RulesCommand::CreateCategory { .. } => "rules create-category",
    };
    not_implemented(name)
}
