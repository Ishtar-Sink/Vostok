//! Aval do usuário: os logs de avais por spec, a chave `approval_lock` e os pedidos do painel.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::harness::Harness;
use crate::support::{config_get, frontmatter_get, sha256_hex, utc_datetime};

/// Log único da v0.3, em `work/`, dividido por spec na primeira leitura ou gravação.
const LEGACY_LOG: &str = "avais.log";
const LOG_EXTENSION: &str = "log";
const DEFAULT_ACTION: &str = "aprovar";
const PANEL_ORIGIN: &str = "painel";

pub struct Aval {
    pub date: String,
    pub id: String,
    pub hash: String,
    pub origin: String,
    pub action: String,
}

/// A spec do ID (`SPEC-001` de `SPEC-001/TASK-01`), ou nada para pedidos e IDs sem spec.
fn spec_of(id: &str) -> Option<&str> {
    let head = id.split('/').next()?;
    let digits = head.strip_prefix("SPEC-")?;
    (!digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())).then_some(head)
}

/// O log onde fica o aval do ID: o da spec, ou o arquivo sem spec.
fn log_of(harness: &Harness, id: &str) -> Result<PathBuf> {
    let name = match spec_of(id) {
        Some(spec) => format!("{spec}.{LOG_EXTENSION}"),
        None => harness.folders()?.approvals_without_spec,
    };
    Ok(harness.approvals_dir().join(name))
}

/// Lê as linhas de um log (4 colunas: aprovar; 5 colunas: com a ação). Linhas malformadas vão a `malformed`.
fn parse(content: &str, source: &str, avais: &mut Vec<Aval>, malformed: &mut Vec<String>) {
    for (index, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let columns: Vec<&str> = line.split('\t').collect();
        match columns.as_slice() {
            [date, id, hash, origin] => avais.push(aval_of(date, id, hash, origin, DEFAULT_ACTION)),
            [date, id, hash, origin, action] => avais.push(aval_of(date, id, hash, origin, action)),
            _ => malformed.push(format!("{source}: linha {} malformada", index + 1)),
        }
    }
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

fn append_line(path: &Path, line: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("criando {}", parent.display()))?;
    }
    let mut log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("abrindo {}", path.display()))?;
    writeln!(log, "{line}").with_context(|| format!("gravando {}", path.display()))
}

/// Divide o `avais.log` antigo pelos logs de cada spec, sem mudar as linhas, e só então o apaga.
pub fn migrate_legacy(harness: &Harness) -> Result<()> {
    let legacy = harness.root.join("work").join(LEGACY_LOG);
    if !legacy.is_file() {
        return Ok(());
    }
    let content =
        fs::read_to_string(&legacy).with_context(|| format!("lendo {}", legacy.display()))?;
    for line in content.lines().filter(|line| !line.trim().is_empty()) {
        let columns: Vec<&str> = line.split('\t').collect();
        let id = match columns.len() {
            4 | 5 => columns[1],
            _ => "",
        };
        append_line(&log_of(harness, id)?, line)?;
    }
    fs::remove_file(&legacy).with_context(|| format!("removendo {}", legacy.display()))
}

/// Acrescenta o aval ao log do ID (RN-05: só acréscimo).
pub fn append(harness: &Harness, id: &str, hash: &str, origin: &str, action: &str) -> Result<()> {
    migrate_legacy(harness)?;
    append_line(
        &log_of(harness, id)?,
        &format!("{}\t{id}\t{hash}\t{origin}\t{action}", utc_datetime()),
    )
}

fn read_log(path: &Path, avais: &mut Vec<Aval>, malformed: &mut Vec<String>) -> Result<()> {
    if !path.is_file() {
        return Ok(());
    }
    let content = fs::read_to_string(path).with_context(|| format!("lendo {}", path.display()))?;
    let source = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    parse(&content, &source, avais, malformed);
    Ok(())
}

/// Os avais do log do ID (o da spec, com as tarefas dela, ou o sem spec).
pub fn read_for(harness: &Harness, id: &str) -> Result<Vec<Aval>> {
    migrate_legacy(harness)?;
    let mut avais = Vec::new();
    read_log(&log_of(harness, id)?, &mut avais, &mut Vec::new())?;
    Ok(avais)
}

/// Os avais de todos os logs da pasta, em ordem de nome. Linhas malformadas vão à segunda lista.
pub fn read_all(harness: &Harness) -> Result<(Vec<Aval>, Vec<String>)> {
    migrate_legacy(harness)?;
    let dir = harness.approvals_dir();
    let mut avais = Vec::new();
    let mut malformed = Vec::new();
    if !dir.is_dir() {
        return Ok((avais, malformed));
    }
    let mut logs: Vec<PathBuf> = fs::read_dir(&dir)
        .with_context(|| format!("lendo {}", dir.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_file())
        .filter(|path| path.extension().is_some_and(|ext| ext == LOG_EXTENSION))
        .collect();
    logs.sort();
    for log in logs {
        read_log(&log, &mut avais, &mut malformed)?;
    }
    Ok((avais, malformed))
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
    let avais = read_for(harness, id)?;
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
    message.push_str(&format!("; consulte com vostok avais {id}"));
    bail!(message)
}

/// Há aval `aprovar` do painel para o ID e o hash.
pub fn has_panel_approval(harness: &Harness, id: &str, hash: &str) -> Result<bool> {
    Ok(read_for(harness, id)?.iter().any(|aval| {
        aval.origin == PANEL_ORIGIN
            && aval.action == DEFAULT_ACTION
            && aval.id == id
            && aval.hash == hash
    }))
}

/// Estado e ação do pedido, lidos do front-matter do arquivo dele.
pub fn request_state(path: &Path) -> Result<(String, String)> {
    Ok((
        frontmatter_get(path, "estado")?,
        frontmatter_get(path, "acao")?,
    ))
}

/// SHA-256 do texto da seção `## Pedido`, sem espaços nas pontas.
pub fn request_text_hash(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path).with_context(|| format!("lendo {}", path.display()))?;
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

    fn harness_with_main(main: &str) -> (tempfile::TempDir, Harness) {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("config")).unwrap();
        fs::write(dir.path().join("config/main.yml"), main).unwrap();
        let harness = Harness {
            root: dir.path().to_path_buf(),
        };
        (dir, harness)
    }

    fn harness_with_log(name: &str, log: &str) -> (tempfile::TempDir, Harness) {
        let (dir, harness) = harness_with_main("");
        fs::create_dir_all(harness.approvals_dir()).unwrap();
        fs::write(harness.approvals_dir().join(name), log).unwrap();
        (dir, harness)
    }

    #[test]
    fn four_columns_read_as_aprovar() {
        let (_dir, harness) = harness_with_log(
            "SPEC-001.log",
            "2026-11-02T10:00:00Z\tSPEC-001\tabc\tpainel\n",
        );
        let (avais, malformed) = read_all(&harness).unwrap();
        assert!(malformed.is_empty());
        assert_eq!(avais[0].action, "aprovar");
        assert_eq!(avais[0].origin, "painel");
    }

    #[test]
    fn five_columns_keep_the_action() {
        let (_dir, harness) = harness_with_log(
            "SPEC-001.log",
            "2026-11-02T10:00:00Z\tSPEC-001\tabc\tpainel\tcommit\n",
        );
        let avais = read_for(&harness, "SPEC-001").unwrap();
        assert_eq!(avais[0].action, "commit");
        assert_eq!(avais[0].hash, "abc");
    }

    #[test]
    fn malformed_line_is_reported_not_matched() {
        let (_dir, harness) =
            harness_with_log("SPEC-001.log", "so\tduas\nok\tSPEC-001\tabc\tpainel\n");
        let (avais, malformed) = read_all(&harness).unwrap();
        assert_eq!(avais.len(), 1);
        assert_eq!(
            malformed,
            vec!["SPEC-001.log: linha 1 malformada".to_string()]
        );
    }

    #[test]
    fn missing_log_reads_empty() {
        let (_dir, harness) = harness_with_main("");
        let (avais, malformed) = read_all(&harness).unwrap();
        assert!(avais.is_empty() && malformed.is_empty());
        assert!(read_for(&harness, "SPEC-001").unwrap().is_empty());
    }

    #[test]
    fn spec_of_reads_only_spec_ids() {
        assert_eq!(spec_of("SPEC-001/TASK-01"), Some("SPEC-001"));
        assert_eq!(spec_of("SPEC-012"), Some("SPEC-012"));
        assert_eq!(spec_of("PED-001"), None);
        assert_eq!(spec_of("SPEC-"), None);
        assert_eq!(spec_of("SPEC-1a"), None);
        assert_eq!(spec_of(""), None);
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
        assert!(error.ends_with("; consulte com vostok avais SPEC-001"));
    }

    #[test]
    fn lock_accepts_matching_panel_approval_and_names_other_hash() {
        let (_dir, harness) = harness_with_main("approval_lock: true\n");
        append(&harness, "SPEC-001", "old", "painel", "concluir").unwrap();
        let error = require(&harness, "SPEC-001", "concluir", "new")
            .unwrap_err()
            .to_string();
        assert!(error.contains("o aval existente é do hash old"));
        append(&harness, "SPEC-001", "new", "painel", "concluir").unwrap();
        assert!(require(&harness, "SPEC-001", "concluir", "new").is_ok());
    }

    #[test]
    fn legacy_log_is_kept_when_a_write_fails() {
        let (_dir, harness) = harness_with_main("");
        let work = harness.root.join("work");
        fs::create_dir_all(&work).unwrap();
        fs::write(work.join(LEGACY_LOG), "d\tSPEC-001\th\tpainel\n").unwrap();
        fs::create_dir_all(harness.approvals_dir().join("SPEC-001.log")).unwrap();
        assert!(migrate_legacy(&harness).is_err());
        assert!(work.join(LEGACY_LOG).is_file());
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
        let path = dir.path().join("pedidos/PED-001.md");
        let (state, action) = request_state(&path).unwrap();
        assert_eq!(
            (state.as_str(), action.as_str()),
            ("pendente", "ajustar-spec")
        );
        assert_eq!(request_text_hash(&path).unwrap(), sha256_hex(b"Texto"));
    }
}
