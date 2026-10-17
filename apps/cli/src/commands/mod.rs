//! Um módulo por grupo de comandos.

pub mod bug;
pub mod commit;
pub mod context;
pub mod finalize;
pub mod init;
pub mod memory;
pub mod plan;
pub mod rules;
pub mod skill;
pub mod spec;
pub mod task;
pub mod update;

use anyhow::{bail, Result};

/// Resposta dos subcomandos que ainda não foram reescritos.
pub fn not_implemented(name: &str) -> Result<()> {
    bail!("'vostok {name}' ainda não foi implementado")
}
