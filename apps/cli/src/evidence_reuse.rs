//! Índice de evidências reaproveitáveis: chave, entradas e cópias por SHA-256 (SPEC-075).

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};

use crate::support::sha256_hex;

pub const INDEX_DIR: &str = "evidence-index";
const INDEX_FILE: &str = "index.yml";
const OBJECTS_DIR: &str = "objects";
pub const KINDS: [&str; 3] = ["log", "print", "video"];

/// SHA-256 do comando exato da evidência.
pub fn command_hash(command: &str) -> String {
    sha256_hex(command.as_bytes())
}

/// Chave de reaproveitamento: o comando exato e a impressão das entradas, separados por um byte nulo
/// para que a fronteira entre os dois não seja ambígua.
pub fn reuse_key(command: &str, inputs: &str) -> String {
    sha256_hex(format!("{command}\0{inputs}").as_bytes())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Chave de reaproveitamento (`reuse_key`).
    pub key: String,
    /// SHA-256 do comando exato.
    pub command_hash: String,
    /// Caminhos do `Depende:`; vazio quando a chave usou a árvore inteira.
    pub depends: Vec<String>,
    /// Impressão das entradas usada na chave.
    pub inputs: String,
    /// SHA-256 do conteúdo guardado em `objects/`.
    pub content_sha256: String,
    /// Tipo da evidência: `log`, `print` ou `video`.
    pub kind: String,
    /// Código de saída do comando que gravou a evidência.
    pub exit_code: i32,
    /// Spec de origem.
    pub spec: String,
    /// Tarefa de origem.
    pub task: String,
    /// ID da evidência na tarefa de origem (EV-NN).
    pub evidence_id: String,
    /// Data da gravação, em UTC.
    pub date: String,
}

impl Entry {
    fn to_json(&self) -> Value {
        json!({
            "key": self.key,
            "command_hash": self.command_hash,
            "depends": self.depends,
            "inputs": self.inputs,
            "content_sha256": self.content_sha256,
            "kind": self.kind,
            "exit_code": self.exit_code,
            "spec": self.spec,
            "task": self.task,
            "evidence_id": self.evidence_id,
            "date": self.date,
        })
    }

    fn from_json(value: &Value) -> Option<Entry> {
        let text = |field: &str| value.get(field)?.as_str().map(str::to_string);
        let depends = value
            .get("depends")?
            .as_array()?
            .iter()
            .map(|path| path.as_str().map(str::to_string))
            .collect::<Option<Vec<String>>>()?;
        let exit_code = i32::try_from(value.get("exit_code")?.as_i64()?).ok()?;
        Some(Entry {
            key: text("key")?,
            command_hash: text("command_hash")?,
            depends,
            inputs: text("inputs")?,
            content_sha256: text("content_sha256")?,
            kind: text("kind")?,
            exit_code,
            spec: text("spec")?,
            task: text("task")?,
            evidence_id: text("evidence_id")?,
            date: text("date")?,
        })
    }
}

pub struct ReuseIndex {
    dir: PathBuf,
}

impl ReuseIndex {
    pub fn at(work_dir: &Path) -> ReuseIndex {
        ReuseIndex {
            dir: work_dir.join(INDEX_DIR),
        }
    }

    fn index_path(&self) -> PathBuf {
        self.dir.join(INDEX_FILE)
    }

    fn object_path(&self, sha256: &str) -> PathBuf {
        self.dir.join(OBJECTS_DIR).join(sha256)
    }

    fn entries(&self) -> Result<Vec<Entry>> {
        let path = self.index_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let text = fs::read_to_string(&path)
            .with_context(|| format!("não consegui ler {}", path.display()))?;
        let parsed = serde_json::from_str::<Value>(&text).ok().and_then(|doc| {
            doc.get("entries")?
                .as_array()?
                .iter()
                .map(Entry::from_json)
                .collect::<Option<Vec<Entry>>>()
        });
        match parsed {
            Some(entries) => Ok(entries),
            None => bail!("índice de reaproveitamento inválido: {}", path.display()),
        }
    }

    fn write_entries(&self, entries: &mut [Entry]) -> Result<()> {
        entries.sort_by(|a, b| a.key.cmp(&b.key));
        let doc = json!({ "entries": entries.iter().map(Entry::to_json).collect::<Vec<Value>>() });
        let text = format!("{}\n", serde_json::to_string_pretty(&doc)?);
        write_atomic(&self.index_path(), text.as_bytes())
    }

    /// Grava a entrada e a cópia do conteúdo; a entrada de mesma chave é substituída.
    pub fn record(&self, entry: Entry, content: &[u8]) -> Result<()> {
        if !KINDS.contains(&entry.kind.as_str()) {
            bail!("tipo de evidência inválido: {}", entry.kind);
        }
        if sha256_hex(content) != entry.content_sha256 {
            bail!(
                "o SHA-256 do conteúdo não confere com a entrada {}",
                entry.key
            );
        }
        let mut entries = self.entries()?;
        let objects = self.dir.join(OBJECTS_DIR);
        fs::create_dir_all(&objects)
            .with_context(|| format!("não consegui criar {}", objects.display()))?;
        let object = self.object_path(&entry.content_sha256);
        let intact = fs::read(&object)
            .map(|bytes| sha256_hex(&bytes) == entry.content_sha256)
            .unwrap_or(false);
        if !intact {
            write_atomic(&object, content)?;
        }
        entries.retain(|existing| existing.key != entry.key);
        entries.push(entry);
        self.write_entries(&mut entries)
    }

    /// A entrada da chave e o conteúdo, só quando a cópia existe e confere com o SHA-256 gravado.
    pub fn lookup(&self, key: &str) -> Result<Option<(Entry, Vec<u8>)>> {
        let Some(entry) = self.entries()?.into_iter().find(|entry| entry.key == key) else {
            return Ok(None);
        };
        let Ok(bytes) = fs::read(self.object_path(&entry.content_sha256)) else {
            return Ok(None);
        };
        if sha256_hex(&bytes) != entry.content_sha256 {
            return Ok(None);
        }
        Ok(Some((entry, bytes)))
    }

    /// As entradas gravadas com o mesmo comando, na ordem da chave.
    pub fn by_command(&self, command_hash: &str) -> Result<Vec<Entry>> {
        Ok(self
            .entries()?
            .into_iter()
            .filter(|entry| entry.command_hash == command_hash)
            .collect())
    }
}

fn write_atomic(path: &Path, content: &[u8]) -> Result<()> {
    let temporary = path.with_extension(match path.extension() {
        Some(extension) => format!("{}.tmp", extension.to_string_lossy()),
        None => "tmp".to_string(),
    });
    fs::write(&temporary, content)
        .with_context(|| format!("não consegui gravar {}", temporary.display()))?;
    fs::rename(&temporary, path).with_context(|| format!("não consegui gravar {}", path.display()))
}
