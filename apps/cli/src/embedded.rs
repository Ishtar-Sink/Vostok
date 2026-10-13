//! Conteúdo distribuível do harness, embutido no binário em tempo de compilação (`build.rs`).

use std::fs;
use std::io;
use std::path::Path;

use include_dir::{include_dir, Dir, DirEntry};

static HARNESS: Dir<'static> = include_dir!("$OUT_DIR/harness");

/// Caminhos relativos de todos os arquivos embutidos, em ordem alfabética.
pub fn files() -> Vec<String> {
    let mut paths = Vec::new();
    collect(&HARNESS, &mut paths);
    paths.sort();
    paths
}

/// Conteúdo de um arquivo embutido, pelo caminho relativo à raiz do harness.
pub fn file(path: &str) -> Option<&'static [u8]> {
    HARNESS.get_file(path).map(|file| file.contents())
}

/// Grava todos os arquivos embutidos em `target`, criando as pastas necessárias.
pub fn extract_to(target: &Path) -> io::Result<()> {
    for path in files() {
        let destination = target.join(&path);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(destination, file(&path).unwrap_or_default())?;
    }
    Ok(())
}

fn collect(dir: &Dir<'static>, paths: &mut Vec<String>) {
    for entry in dir.entries() {
        match entry {
            DirEntry::Dir(child) => collect(child, paths),
            DirEntry::File(file) => paths.push(file.path().to_string_lossy().into_owned()),
        }
    }
}
