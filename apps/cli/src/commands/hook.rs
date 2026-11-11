//! Comandos `hook`: hooks das ferramentas de agente (SPEC-050).

use std::fs;

use anyhow::Result;
use clap::Subcommand;

use crate::harness::Harness;

/// Regras que precisam valer antes do primeiro comando do agente, relativas à raiz do harness.
const CORE_FILES: [&str; 2] = ["rules/agent/boundaries.md", "rules/project/security.md"];

/// Limite de caracteres que o Claude Code mostra inline na saída de um hook.
const HOOK_LIMIT: usize = 10_000;

#[derive(Debug, Subcommand)]
pub enum HookCommand {
    /// Imprime o núcleo curto do contexto obrigatório (limites, segurança e a ordem de rodar
    /// `vostok context load`) para o hook de início de sessão.
    SessionStart,
}

pub fn run(command: HookCommand) -> Result<()> {
    match command {
        HookCommand::SessionStart => {
            let mut text = core(&Harness::from_current_dir()?);
            let count = text.chars().count();
            if count >= HOOK_LIMIT {
                let warning = format!(
                    "Aviso: o núcleo tem {count} caracteres, acima do limite de 10.000 do hook; \
                     rode `vostok context load` agora."
                );
                eprintln!("{warning}");
                text.push_str(&format!("\n{warning}\n"));
            }
            print!("{text}");
            Ok(())
        }
    }
}

fn core(harness: &Harness) -> String {
    let mut text = format!(
        "# Núcleo do contexto obrigatório do harness\n\nRaiz do harness: `{}`\n",
        harness.root.display()
    );
    for item in CORE_FILES {
        let Ok(content) = fs::read_to_string(harness.root.join(item)) else {
            continue;
        };
        text.push_str(&format!("\n--- arquivo: {item} ---\n\n{content}"));
        if !content.ends_with('\n') {
            text.push('\n');
        }
    }
    text.push_str(
        "\n## Próximo passo obrigatório\n\nRode agora `vostok context load` e leia a saída \
         inteira, sem redirecionar, filtrar nem capturar: ela traz as demais regras, os templates \
         de resposta e a persona. Rode de novo depois de cada compactação ou limpeza da conversa.\n",
    );
    text
}
