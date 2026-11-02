//! Aval do usuário: leitura do `avais.log`, da chave `approval_lock` e dos pedidos do painel.

use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::harness::Harness;
use crate::support::{config_get, frontmatter_get, sha256_hex};

const AVAIS_LOG: &str = "avais.log";
const DEFAULT_ACTION: &str = "aprovar";
const PANEL_ORIGIN: &str = "painel";

pub struct Aval {
    pub date: String,
    pub id: String,
    pub hash: String,
    pub origin: String,
    pub action: String,
}

/// Lê o `avais.log` (4 colunas: aprovar; 5 colunas: com a ação). Linhas malformadas vão à segunda lista.
pub fn read(work: &Path) -> Result<(Vec<Aval>, Vec<String>)> {
    let path = work.join(AVAIS_LOG);
    if !path.is_file() {
        return Ok((Vec::new(), Vec::new()));
    }
    let content = fs::read_to_string(&path).with_context(|| format!("lendo {}", path.display()))?;
    let mut avais = Vec::new();
    let mut malformed = Vec::new();
    for (index, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let columns: Vec<&str> = line.split('\t').collect();
        match columns.as_slice() {
            [date, id, hash, origin] => avais.push(aval_of(date, id, hash, origin, DEFAULT_ACTION)),
            [date, id, hash, origin, action] => avais.push(aval_of(date, id, hash, origin, action)),
            _ => malformed.push(format!("linha {} malformada", index + 1)),
        }
    }
    Ok((avais, malformed))
}

fn aval_of(date: &str, id: &str, hash: &str, origin: &str, action: &str) -> Aval {
    Aval {
        date: date.to_string(),
        id: id.to_string(),
        hash: hash.to_string(),
        origin: origin.to_string(),
        action: action.to_string(),
    }
}

/// A trava vale só com `approval_lock: true`; chave ausente conta como desligada.
pub fn lock_enabled(harness: &Harness) -> Result<bool> {
    let value = config_get(&harness.config_file("main.yml"), "approval_lock").unwrap_or_default();
    Ok(value == "true")
}

/// Com a trava ligada, exige um aval do painel para a ação, o ID e o hash.
pub fn require(harness: &Harness, id: &str, action: &str, hash: &str) -> Result<()> {
    if !lock_enabled(harness)? {
        return Ok(());
    }
    let work = harness.root.join("work");
    let (avais, _) = read(&work)?;
    let matches_panel =
        |aval: &Aval| aval.origin == PANEL_ORIGIN && aval.id == id && aval.action == action;
    if avais
        .iter()
        .any(|aval| matches_panel(aval) && aval.hash == hash)
    {
        return Ok(());
    }
    let mut message =
        format!("sem o aval do painel para {action} de {id} (hash {hash}); peça pelo painel (vostok painel)");
    if let Some(other) = avais.iter().rev().find(|aval| matches_panel(aval)) {
        message.push_str(&format!("; o aval existente é do hash {}", other.hash));
    }
    bail!(message)
}

/// Há aval `aprovar` do painel para o ID e o hash.
pub fn has_panel_approval(work: &Path, id: &str, hash: &str) -> Result<bool> {
    let (avais, _) = read(work)?;
    Ok(avais.iter().any(|aval| {
        aval.origin == PANEL_ORIGIN
            && aval.action == DEFAULT_ACTION
            && aval.id == id
            && aval.hash == hash
    }))
}

/// Estado e ação do pedido, lidos do front-matter de `pedidos/<ID>.md`.
pub fn request_state(work: &Path, request_id: &str) -> Result<(String, String)> {
    let path = work.join("pedidos").join(format!("{request_id}.md"));
    Ok((
        frontmatter_get(&path, "estado")?,
        frontmatter_get(&path, "acao")?,
    ))
}

/// SHA-256 do texto da seção `## Pedido`, sem espaços nas pontas.
pub fn request_text_hash(work: &Path, request_id: &str) -> Result<String> {
    let path = work.join("pedidos").join(format!("{request_id}.md"));
    let content = fs::read_to_string(&path).with_context(|| format!("lendo {}", path.display()))?;
    let section = content
        .split("\n## Pedido\n")
        .nth(1)
        .with_context(|| format!("{} não tem a seção Pedido", path.display()))?;
    let text = section.split("\n## ").next().unwrap_or(section);
    Ok(sha256_hex(text.trim().as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn work_with_log(log: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(AVAIS_LOG), log).unwrap();
        dir
    }

    fn harness_with_main(main: &str) -> (tempfile::TempDir, Harness) {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("config")).unwrap();
        fs::write(dir.path().join("config/main.yml"), main).unwrap();
        let harness = Harness {
            root: dir.path().to_path_buf(),
        };
        (dir, harness)
    }

    #[test]
    fn four_columns_read_as_aprovar() {
        let dir = work_with_log("2026-11-02T10:00:00Z\tSPEC-001\tabc\tpainel\n");
        let (avais, malformed) = read(dir.path()).unwrap();
        assert!(malformed.is_empty());
        assert_eq!(avais[0].action, "aprovar");
        assert_eq!(avais[0].origin, "painel");
    }

    #[test]
    fn five_columns_keep_the_action() {
        let dir = work_with_log("2026-11-02T10:00:00Z\tSPEC-001\tabc\tpainel\tcommit\n");
        let (avais, _) = read(dir.path()).unwrap();
        assert_eq!(avais[0].action, "commit");
        assert_eq!(avais[0].hash, "abc");
    }

    #[test]
    fn malformed_line_is_reported_not_matched() {
        let dir = work_with_log("so\tduas\nok\tSPEC-001\tabc\tpainel\n");
        let (avais, malformed) = read(dir.path()).unwrap();
        assert_eq!(avais.len(), 1);
        assert_eq!(malformed, vec!["linha 1 malformada".to_string()]);
    }

    #[test]
    fn missing_log_reads_empty() {
        let dir = tempfile::tempdir().unwrap();
        let (avais, malformed) = read(dir.path()).unwrap();
        assert!(avais.is_empty() && malformed.is_empty());
    }

    #[test]
    fn lock_is_off_without_the_key() {
        let (_dir, harness) = harness_with_main("");
        assert!(!lock_enabled(&harness).unwrap());
        assert!(require(&harness, "SPEC-001", "concluir", "abc").is_ok());
    }

    #[test]
    fn lock_reads_true() {
        let (_dir, harness) = harness_with_main("approval_lock: true\n");
        assert!(lock_enabled(&harness).unwrap());
        let error = require(&harness, "SPEC-001", "concluir", "abc")
            .unwrap_err()
            .to_string();
        assert!(error.contains("sem o aval do painel para concluir de SPEC-001"));
    }

    #[test]
    fn lock_accepts_matching_panel_approval_and_names_other_hash() {
        let (dir, harness) = harness_with_main("approval_lock: true\n");
        let work = dir.path().join("work");
        fs::create_dir_all(&work).unwrap();
        fs::write(work.join(AVAIS_LOG), "d\tSPEC-001\told\tpainel\tconcluir\n").unwrap();
        let error = require(&harness, "SPEC-001", "concluir", "new")
            .unwrap_err()
            .to_string();
        assert!(error.contains("o aval existente é do hash old"));
        fs::write(work.join(AVAIS_LOG), "d\tSPEC-001\tnew\tpainel\tconcluir\n").unwrap();
        assert!(require(&harness, "SPEC-001", "concluir", "new").is_ok());
    }

    #[test]
    fn request_state_reads_estado() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("pedidos")).unwrap();
        fs::write(
            dir.path().join("pedidos/PED-001.md"),
            "---\nid: PED-001\nacao: ajustar-spec\nestado: pendente\n---\n\n## Pedido\n\n  Texto  \n\n## Resultado\n\n",
        )
        .unwrap();
        let (state, action) = request_state(dir.path(), "PED-001").unwrap();
        assert_eq!(
            (state.as_str(), action.as_str()),
            ("pendente", "ajustar-spec")
        );
        assert_eq!(
            request_text_hash(dir.path(), "PED-001").unwrap(),
            sha256_hex(b"Texto")
        );
    }
}
