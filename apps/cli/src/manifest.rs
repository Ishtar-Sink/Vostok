//! Manifesto do harness instalado (`vostok/.manifest`): a versão do binário que instalou e o
//! SHA-256 de cada arquivo que veio dele e que o `vostok update` regrava (RF-08, RN-01).

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};

use crate::embedded;
use crate::harness::Harness;
use crate::support::{config_get, sha256_hex};

pub const FILE_NAME: &str = ".manifest";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Pastas embutidas que passam a ser do projeto depois de instaladas (RN-01).
const PROJECT_DIRS: [&str; 4] = ["config/", "rules/project/", "skills/custom/", "memory/"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub version: String,
    /// Caminho relativo à raiz do harness e o SHA-256 do conteúdo.
    pub files: BTreeMap<String, String>,
}

impl Manifest {
    /// Manifesto do binário atual para os arquivos embutidos que ele regrava no projeto.
    pub fn current(harness: &Harness) -> Result<Self> {
        let files = binary_files(harness)?
            .into_iter()
            .map(|path| {
                let hash = sha256_hex(embedded::file(&path).unwrap_or_default());
                (path, hash)
            })
            .collect();
        Ok(Self {
            version: VERSION.to_string(),
            files,
        })
    }

    pub fn path(harness: &Harness) -> PathBuf {
        harness.root.join(FILE_NAME)
    }

    pub fn read(harness: &Harness) -> Result<Self> {
        let path = Self::path(harness);
        let content = fs::read_to_string(&path).with_context(|| {
            format!(
                "manifesto não encontrado em {}; instale o harness com `vostok init`",
                path.display()
            )
        })?;
        Self::parse(&content).with_context(|| format!("manifesto inválido: {}", path.display()))
    }

    /// Grava o manifesto se o conteúdo mudou e diz se gravou.
    pub fn write(&self, harness: &Harness) -> Result<bool> {
        let path = Self::path(harness);
        let content = self.render();
        if fs::read_to_string(&path).is_ok_and(|current| current == content) {
            return Ok(false);
        }
        fs::write(&path, content)
            .with_context(|| format!("não foi possível gravar {}", path.display()))?;
        Ok(true)
    }

    pub fn parse(content: &str) -> Result<Self> {
        let mut lines = content.lines();
        let Some(version) = lines
            .next()
            .and_then(|line| line.strip_prefix("version: "))
            .map(str::trim)
            .filter(|version| !version.is_empty())
        else {
            bail!("a primeira linha precisa ser 'version: <versão>'");
        };
        if lines.next() != Some("files:") {
            bail!("a segunda linha precisa ser 'files:'");
        }
        let mut files = BTreeMap::new();
        for line in lines {
            let entry = line
                .strip_prefix("  ")
                .and_then(|entry| entry.rsplit_once(": "))
                .filter(|(path, hash)| !path.is_empty() && !hash.trim().is_empty());
            let Some((path, hash)) = entry else {
                bail!("linha inválida: '{line}'; o formato é '  <caminho>: <sha256>'");
            };
            files.insert(path.to_string(), hash.trim().to_string());
        }
        Ok(Self {
            version: version.to_string(),
            files,
        })
    }

    pub fn render(&self) -> String {
        let files: String = self
            .files
            .iter()
            .map(|(path, hash)| format!("  {path}: {hash}\n"))
            .collect();
        format!("version: {}\nfiles:\n{files}", self.version)
    }
}

/// Arquivos embutidos que pertencem ao binário: tudo menos as pastas do projeto e o catálogo de
/// skills, que é regenerado a partir das skills instaladas (RF-12).
pub fn binary_files(harness: &Harness) -> Result<Vec<String>> {
    let catalog = catalog_path(harness)?;
    Ok(embedded::files()
        .into_iter()
        .filter(|path| !PROJECT_DIRS.iter().any(|dir| path.starts_with(dir)))
        .filter(|path| *path != catalog)
        .collect())
}

/// Caminho do catálogo de skills relativo à raiz do harness, como está em `config/skills.yml`.
pub fn catalog_path(harness: &Harness) -> Result<String> {
    let catalog = config_get(&harness.config_file("skills.yml"), "catalog")?;
    Ok(catalog.trim_start_matches("./").to_string())
}
