//! Comando `update`: atualiza o harness instalado com o conteúdo do binário.

use anyhow::Result;

use super::not_implemented;
use crate::harness::Harness;

pub fn run() -> Result<()> {
    let _harness = Harness::from_current_dir()?;
    not_implemented("update")
}
