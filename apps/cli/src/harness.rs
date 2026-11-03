//! Localização do harness de um projeto e resolução dos caminhos da configuração.

use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

use crate::support::config_get;

/// Pastas, relativas à raiz do projeto, onde o harness pode estar instalado.
/// `packages/harness` só vale enquanto este repositório não migra para `vostok/` (SPEC-005, TASK-11).
const INSTALL_DIRS: &[&str] = &["vostok", "packages/harness"];

/// Padrões das pastas de comunicação entre o painel e o agente, para projetos sem as chaves novas.
const DEFAULT_REQUESTS_DIR: &str = "./work/pedidos";
const DEFAULT_QUESTIONS_DIR: &str = "./work/perguntas";
const DEFAULT_APPROVALS_DIR: &str = "./work/avais";
const DEFAULT_APPROVALS_WITHOUT_SPEC: &str = "sem-spec.log";
const DISPATCH_AGENTS: &[&str] = &["codex", "claude"];

/// Subpasta de cada estado de um pedido.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestFolders {
    pub pending: String,
    pub in_progress: String,
    pub done: String,
    pub refused: String,
}

/// Subpasta de cada status de uma pergunta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestionFolders {
    pub open: String,
    pub answered: String,
}

/// Nomes configuráveis das pastas e do arquivo de avais sem spec (`folders` do `config/main.yml`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Folders {
    pub requests: RequestFolders,
    pub questions: QuestionFolders,
    pub approvals_without_spec: String,
}

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
        Self::locate(&crate::support::current_dir()?)
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

    /// Pasta dos pedidos do painel ao agente.
    pub fn requests_dir(&self) -> PathBuf {
        self.path_or("requests", DEFAULT_REQUESTS_DIR)
    }

    /// Pasta das perguntas do agente ao usuário.
    pub fn questions_dir(&self) -> PathBuf {
        self.path_or("questions", DEFAULT_QUESTIONS_DIR)
    }

    /// Pasta dos avais do painel.
    pub fn approvals_dir(&self) -> PathBuf {
        self.path_or("approvals", DEFAULT_APPROVALS_DIR)
    }

    /// Chave de `paths`, com o padrão quando ela falta ou está vazia (projeto da v0.3 sem `vostok update`).
    fn path_or(&self, key: &str, default: &str) -> PathBuf {
        let value = config_get(&self.config_file("main.yml"), &format!("paths.{key}"))
            .ok()
            .filter(|value| !value.is_empty());
        self.resolve(value.as_deref().unwrap_or(default))
    }

    /// Nomes das subpastas de cada estado e do arquivo de avais sem spec, validados.
    pub fn folders(&self) -> Result<Folders> {
        let requests = self.folder_group(
            "requests",
            &[
                ("pending", "pendente"),
                ("in_progress", "em_andamento"),
                ("done", "feito"),
                ("refused", "recusado"),
            ],
        )?;
        let questions = self.folder_group(
            "questions",
            &[("open", "aberta"), ("answered", "respondida")],
        )?;
        let approvals_without_spec = self.folder_name(
            "folders.approvals_without_spec",
            DEFAULT_APPROVALS_WITHOUT_SPEC,
        )?;
        Ok(Folders {
            requests: RequestFolders {
                pending: requests[0].clone(),
                in_progress: requests[1].clone(),
                done: requests[2].clone(),
                refused: requests[3].clone(),
            },
            questions: QuestionFolders {
                open: questions[0].clone(),
                answered: questions[1].clone(),
            },
            approvals_without_spec,
        })
    }

    /// Nomes de um grupo de `folders`, na ordem dos estados, sem dois estados na mesma pasta.
    fn folder_group(&self, group: &str, states: &[(&str, &str)]) -> Result<Vec<String>> {
        let mut names: Vec<String> = Vec::new();
        for (state, default) in states {
            let name = self.folder_name(&format!("folders.{group}.{state}"), default)?;
            if let Some(other) = names.iter().position(|taken| *taken == name) {
                bail!(
                    "folders.{group}: os estados {} e {state} usam a mesma pasta '{name}'",
                    states[other].0
                );
            }
            names.push(name);
        }
        Ok(names)
    }

    /// Nome de pasta ou arquivo de `key`: o padrão quando a chave falta; vazio ou com caminho, recusado.
    fn folder_name(&self, key: &str, default: &str) -> Result<String> {
        let Ok(name) = config_get(&self.config_file("main.yml"), key) else {
            return Ok(default.to_string());
        };
        let unsafe_name = name.is_empty()
            || name.contains('/')
            || name.contains('\\')
            || name == "."
            || name.contains("..");
        if unsafe_name {
            bail!("{key}: nome de pasta inválido '{name}'");
        }
        Ok(name)
    }

    /// Agente do disparo sob demanda: vazio (desligado), `codex` ou `claude`.
    pub fn request_dispatch(&self) -> Result<String> {
        let agent =
            config_get(&self.config_file("main.yml"), "request_dispatch").unwrap_or_default();
        if !agent.is_empty() && !DISPATCH_AGENTS.contains(&agent.as_str()) {
            bail!("request_dispatch: agente '{agent}' desconhecido; use \"\", codex ou claude");
        }
        Ok(agent)
    }
}
