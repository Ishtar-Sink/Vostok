//! Um módulo por grupo de comandos.

pub mod avais;
pub mod bug;
pub mod commit;
pub mod context;
pub mod documents;
pub mod finalize;
pub mod init;
pub mod memory;
pub mod painel;
pub mod pedidos;
pub mod perguntas;
pub mod plan;
pub mod rules;
pub mod skill;
pub mod spec;
pub mod suggest;
pub mod task;
pub mod update;

use anyhow::{bail, Result};

/// Resposta dos subcomandos que ainda não foram reescritos.
pub fn not_implemented(name: &str) -> Result<()> {
    bail!("'vostok {name}' ainda não foi implementado")
}
