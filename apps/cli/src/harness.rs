//! Localização do harness de um projeto e resolução dos caminhos da configuração.

use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

use crate::support::config_get;

/// Pastas, relativas à raiz do projeto, onde o harness pode estar instalado.
/// `packages/harness` só vale enquanto este repositório não migra para `vostok/` (SPEC-005, TASK-11).
const INSTALL_DIRS: &[&str] = &["vostok", "packages/harness"];

/// O harness encontrado: a pasta que contém `config/main.yml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Harness {
    pub root: PathBuf,
}

impl Harness {
    /// Procura o harness a partir de `start`, subindo até a raiz do sistema de arquivos (RF-14).
    pub fn locate(start: &Path) -> Result<Self> {
        for dir in start.ancestors() {
            for install in INSTALL_DIRS {
                let candidate = dir.join(install);
                if candidate.join("config/main.yml").is_file() {
                    return Ok(Self { root: candidate });
                }
            }
        }
        bail!(
            "nenhum harness encontrado a partir de {}; execute `vostok init` na raiz do projeto",
            start.display()
        )
    }

    /// Procura o harness a partir do diretório atual.
    pub fn from_current_dir() -> Result<Self> {
        Self::locate(&std::env::current_dir()?)
    }

    /// Caminho de um arquivo de `config/`.
    pub fn config_file(&self, name: &str) -> PathBuf {
        self.root.join("config").join(name)
    }

    /// Resolve uma chave de `paths` do `config/main.yml`: absoluta ou relativa à raiz do harness.
    pub fn path(&self, key: &str) -> Result<PathBuf> {
        let value = config_get(&self.config_file("main.yml"), &format!("paths.{key}"))?;
        Ok(self.resolve(&value))
    }

    /// Resolve um caminho absoluto ou relativo à raiz do harness.
    pub fn resolve(&self, value: &str) -> PathBuf {
        let path = Path::new(value);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.root.join(value.trim_start_matches("./"))
        }
    }
}
