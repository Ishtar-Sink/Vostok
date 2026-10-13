//! Helpers dos testes de integração: projeto temporário com o harness e execução do binário.

#![allow(dead_code)]

pub mod golden;

use std::path::{Path, PathBuf};

use assert_cmd::Command;
use tempfile::TempDir;

/// Projeto temporário com o harness embutido extraído em `vostok/`.
pub struct Project {
    pub dir: TempDir,
}

impl Project {
    pub fn new() -> Self {
        let dir = TempDir::new().expect("criar pasta temporária");
        vostok::embedded::extract_to(&dir.path().join("vostok")).expect("extrair o harness");
        Self { dir }
    }

    pub fn root(&self) -> &Path {
        self.dir.path()
    }

    pub fn harness(&self) -> PathBuf {
        self.root().join("vostok")
    }

    /// Comando `vostok` executado a partir de `relative` (relativo à raiz do projeto).
    pub fn vostok_in(&self, relative: &str) -> Command {
        vostok_at(&self.root().join(relative))
    }

    pub fn vostok(&self) -> Command {
        vostok_at(self.root())
    }
}

/// Comando `vostok` executado em `dir`.
pub fn vostok_at(dir: &Path) -> Command {
    let mut command = Command::cargo_bin("vostok").expect("binário vostok");
    command.current_dir(dir).env("PWD", dir);
    command
}

/// Saída de erro de um comando que deve falhar.
pub fn failure_stderr(command: &mut Command) -> String {
    let output = command.assert().failure().get_output().stderr.clone();
    String::from_utf8(output).expect("stderr em UTF-8")
}
