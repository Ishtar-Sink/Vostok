//! Comando `pedidos`: os pedidos que o painel grava, numa subpasta por estado, e o agente processa.

use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use clap::Subcommand;

use crate::harness::{Harness, QuestionFolders, RequestFolders};
use crate::support::{frontmatter_get, frontmatter_set, utc_datetime};

const PENDING: &str = "pendente";
const IN_PROGRESS: &str = "em_andamento";
const DONE: &str = "feito";
const REFUSED: &str = "recusado";
const RESULT_HEADING: &str = "## Resultado";
const HISTORY_HEADING: &str = "## Histórico";
pub const OPEN: &str = "aberta";
pub const ANSWERED: &str = "respondida";
const CODEX: &str = "codex";
const LEFT_PENDING: &str =
    "o codex terminou sem processar o pedido; veja a saída do vostok pedidos vigiar";

#[derive(Debug, Subcommand)]
pub enum PedidosCommand {
    /// Lista os pedidos, em ordem de ID.
    Listar {
        /// Mostra só os pedidos neste estado.
        #[arg(long)]
        estado: Option<String>,
    },
    /// Leva um pedido de `pendente` a `em_andamento`.
    Iniciar { id: String },
    /// Conclui um pedido `em_andamento`; o resultado vem da entrada padrão.
    Concluir { id: String },
    /// Recusa um pedido `em_andamento`; o motivo vem da entrada padrão.
    Recusar { id: String },
    /// Confere a pasta no intervalo e dispara o agente para cada pedido pendente, um por vez.
    Vigiar {
        /// Agente que processa os pedidos; hoje, só `codex`.
        #[arg(long)]
        agente: String,
        /// Segundos entre uma conferência e outra quando não há pedido pendente.
        #[arg(long, default_value_t = 15)]
        intervalo: u64,
    },
}

pub fn run(command: PedidosCommand) -> Result<()> {
    let harness = Harness::from_current_dir()?;
    let requests = Requests::open(&harness)?;
    match command {
        PedidosCommand::Listar { estado } => list(&requests, estado.as_deref()),
        PedidosCommand::Iniciar { id } => {
            print_transition(&requests, &id, &[PENDING], IN_PROGRESS, None)
        }
        PedidosCommand::Concluir { id } => {
            print_transition(&requests, &id, &[IN_PROGRESS], DONE, Some(read_stdin()?))
        }
        PedidosCommand::Recusar { id } => {
            print_transition(&requests, &id, &[IN_PROGRESS], REFUSED, Some(read_stdin()?))
        }
        PedidosCommand::Vigiar { agente, intervalo } => {
            watch(&harness, &requests, &agente, intervalo)
        }
    }
}

fn is_request_id(id: &str) -> bool {
    id.strip_prefix("PED-")
        .is_some_and(|number| number.len() >= 3 && number.chars().all(|c| c.is_ascii_digit()))
}

fn is_question_id(id: &str) -> bool {
    id.strip_prefix("P-")
        .is_some_and(|number| !number.is_empty() && number.chars().all(|c| c.is_ascii_digit()))
}

/// Arquivos `<ID>.md` de uma pasta, sem descer nas subpastas, em ordem de nome.
fn id_files(dir: &Path, is_id: fn(&str) -> bool) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| {
                    path.is_file()
                        && path
                            .file_stem()
                            .and_then(|stem| stem.to_str())
                            .is_some_and(is_id)
                        && path.extension().is_some_and(|extension| extension == "md")
                })
                .collect()
        })
        .unwrap_or_default();
    files.sort();
    files
}

fn subfolders(dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| path.is_dir())
                .collect()
        })
        .unwrap_or_default()
}

/// Move para a subpasta do seu estado os arquivos soltos e os de pastas fora da configuração, uma
/// vez; estado desconhecido fica onde está. As pastas configuradas não são lidas (RNF-01).
fn migrate(
    dir: &Path,
    configured: &[&str],
    key: &str,
    is_id: fn(&str) -> bool,
    folder_of: impl Fn(&str) -> Option<PathBuf>,
) -> Result<()> {
    let stray: Vec<PathBuf> = subfolders(dir)
        .into_iter()
        .filter(|folder| {
            folder
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| !configured.contains(&name))
        })
        .collect();
    let candidates = id_files(dir, is_id)
        .into_iter()
        .chain(stray.iter().flat_map(|folder| id_files(folder, is_id)));
    for file in candidates {
        let Some(target) = frontmatter_get(&file, key)
            .ok()
            .and_then(|state| folder_of(&state))
        else {
            continue;
        };
        fs::create_dir_all(&target)?;
        let name = file.file_name().unwrap_or_default();
        fs::rename(&file, target.join(name))
            .with_context(|| format!("mover {} para {}", file.display(), target.display()))?;
    }
    for folder in stray {
        // Só some a pasta antiga que ficou vazia; outra com arquivos que não são do harness fica.
        let _ = fs::remove_dir(folder);
    }
    Ok(())
}

/// Os pedidos do painel, numa subpasta por estado (`paths.requests` e `folders.requests`).
pub struct Requests {
    dir: PathBuf,
    folders: RequestFolders,
}

impl Requests {
    pub fn open(harness: &Harness) -> Result<Self> {
        let requests = Self {
            dir: harness.requests_dir(),
            folders: harness.folders()?.requests,
        };
        requests.migrate()?;
        Ok(requests)
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    fn states(&self) -> [(&'static str, &str); 4] {
        [
            (PENDING, self.folders.pending.as_str()),
            (IN_PROGRESS, self.folders.in_progress.as_str()),
            (DONE, self.folders.done.as_str()),
            (REFUSED, self.folders.refused.as_str()),
        ]
    }

    fn migrate(&self) -> Result<()> {
        let names: Vec<&str> = self.states().iter().map(|(_, folder)| *folder).collect();
        migrate(&self.dir, &names, "estado", is_request_id, |state| {
            self.folder_of(state)
        })
    }

    /// Subpasta de um estado de pedido; `None` para um estado que o harness não conhece.
    pub fn folder_of(&self, state: &str) -> Option<PathBuf> {
        self.states()
            .iter()
            .find(|(name, _)| *name == state)
            .map(|(_, folder)| self.dir.join(folder))
    }

    /// O arquivo do pedido, na subpasta em que estiver.
    pub fn find(&self, id: &str) -> Option<PathBuf> {
        self.states()
            .iter()
            .map(|(_, folder)| self.dir.join(folder).join(format!("{id}.md")))
            .find(|path| path.is_file())
    }

    /// Pedidos de um estado (só a pasta dele) ou de todos, em ordem de ID.
    pub fn files(&self, state: Option<&str>) -> Vec<PathBuf> {
        let mut files: Vec<PathBuf> = self
            .states()
            .iter()
            .filter(|(name, _)| state.is_none_or(|wanted| wanted == *name))
            .flat_map(|(_, folder)| id_files(&self.dir.join(folder), is_request_id))
            .collect();
        files.sort_by_key(|path| path.file_name().map(|name| name.to_os_string()));
        files
    }

    /// O maior `PED-NNN` das subpastas e da raiz, mais um.
    pub fn next_id(&self) -> String {
        let highest = self
            .states()
            .iter()
            .map(|(_, folder)| self.dir.join(folder))
            .chain([self.dir.clone()])
            .flat_map(|dir| {
                fs::read_dir(dir)
                    .into_iter()
                    .flatten()
                    .flatten()
                    .filter_map(|entry| {
                        entry
                            .file_name()
                            .to_str()?
                            .strip_prefix("PED-")?
                            .strip_suffix(".md")?
                            .parse::<u32>()
                            .ok()
                    })
            })
            .max()
            .unwrap_or(0);
        format!("PED-{:03}", highest + 1)
    }

    /// Cria o próximo pedido em `pendente/` sem sobrescrever: se outro levou o número, tenta o seguinte.
    pub fn create(&self, content: impl Fn(&str) -> String) -> io::Result<(String, PathBuf)> {
        let dir = self.dir.join(&self.folders.pending);
        fs::create_dir_all(&dir)?;
        loop {
            let id = self.next_id();
            let path = dir.join(format!("{id}.md"));
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(mut file) => {
                    file.write_all(content(&id).as_bytes())?;
                    return Ok((id, path));
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
    }

    /// Muda o estado: grava o resultado, a linha de histórico e o estado, e só então move o arquivo.
    pub fn transition(
        &self,
        id: &str,
        allowed_from: &[&str],
        to: &str,
        result: Option<&str>,
    ) -> Result<String> {
        if !is_request_id(id) {
            bail!("ID de pedido inválido '{id}': use o formato PED-NNN");
        }
        let Some(path) = self.find(id) else {
            bail!("Pedido {id} não encontrado em {}", self.dir.display());
        };
        let current = frontmatter_get(&path, "estado")?;
        if !allowed_from.contains(&current.as_str()) {
            bail!(
                "Pedido {id} está '{current}': só sai de {} para '{to}'",
                allowed_from.join(" ou ")
            );
        }
        let target = self
            .folder_of(to)
            .with_context(|| format!("estado de pedido desconhecido '{to}'"))?;
        let now = utc_datetime();
        let mut content = fs::read_to_string(&path)?;
        if let Some(text) = result {
            content = with_result_section(&content, text);
        }
        content = with_history_line(&content, &now, &current, to);
        fs::write(&path, content).with_context(|| format!("gravar {}", path.display()))?;
        frontmatter_set(&path, "estado", to)?;
        frontmatter_set(&path, "atualizado_em", &now)?;
        fs::create_dir_all(&target)?;
        fs::rename(&path, target.join(format!("{id}.md")))
            .with_context(|| format!("mover o pedido {id} para {}", target.display()))?;
        Ok(format!("Pedido {id}: {current} -> {to}"))
    }
}

/// As perguntas do agente, numa subpasta por status (`paths.questions` e `folders.questions`).
pub struct Questions {
    dir: PathBuf,
    folders: QuestionFolders,
}

impl Questions {
    pub fn open(harness: &Harness) -> Result<Self> {
        let questions = Self {
            dir: harness.questions_dir(),
            folders: harness.folders()?.questions,
        };
        let names = [
            questions.folders.open.as_str(),
            questions.folders.answered.as_str(),
        ];
        migrate(&questions.dir, &names, "status", is_question_id, |status| {
            questions.folder_of(status)
        })?;
        Ok(questions)
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Subpasta de um status de pergunta; `None` para um status que o harness não conhece.
    pub fn folder_of(&self, status: &str) -> Option<PathBuf> {
        match status {
            OPEN => Some(self.dir.join(&self.folders.open)),
            ANSWERED => Some(self.dir.join(&self.folders.answered)),
            _ => None,
        }
    }

    /// Perguntas de um status, em ordem de ID.
    pub fn files(&self, status: &str) -> Vec<PathBuf> {
        self.folder_of(status)
            .map(|folder| id_files(&folder, is_question_id))
            .unwrap_or_default()
    }

    /// Grava o status e move a pergunta para a subpasta dele.
    pub fn move_to(&self, id: &str, status: &str) -> Result<PathBuf> {
        let target = self
            .folder_of(status)
            .with_context(|| format!("status de pergunta desconhecido '{status}'"))?;
        let Some(path) = [OPEN, ANSWERED]
            .iter()
            .filter_map(|known| self.folder_of(known))
            .map(|folder| folder.join(format!("{id}.md")))
            .find(|path| path.is_file())
        else {
            bail!("Pergunta {id} não encontrada em {}", self.dir.display());
        };
        frontmatter_set(&path, "status", status)?;
        fs::create_dir_all(&target)?;
        let moved = target.join(format!("{id}.md"));
        fs::rename(&path, &moved)
            .with_context(|| format!("mover a pergunta {id} para {}", target.display()))?;
        Ok(moved)
    }
}

fn list(requests: &Requests, state: Option<&str>) -> Result<()> {
    println!("| ID | Ação | Spec | Tarefa | Estado |");
    println!("|---|---|---|---|---|");
    for path in requests.files(state) {
        let field = |key| frontmatter_get(&path, key).unwrap_or_default();
        let id = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("");
        println!(
            "| {id} | {} | {} | {} | {} |",
            field("acao"),
            field("spec"),
            field("tarefa"),
            field("estado")
        );
    }
    Ok(())
}

fn print_transition(
    requests: &Requests,
    id: &str,
    allowed_from: &[&str],
    to: &str,
    result: Option<String>,
) -> Result<()> {
    println!(
        "{}",
        requests.transition(id, allowed_from, to, result.as_deref())?
    );
    Ok(())
}

/// Laço do Codex, que não tem loop dentro da sessão: um `codex exec` por pedido pendente, em
/// ordem de ID e um por vez, para duas execuções não disputarem os mesmos arquivos.
fn watch(harness: &Harness, requests: &Requests, agent: &str, interval: u64) -> Result<()> {
    if agent != CODEX {
        bail!("agente '{agent}' não suportado: use --agente {CODEX}");
    }
    if !in_path(CODEX) {
        bail!(
            "o `{CODEX}` não está no PATH; instale o Codex CLI (npm install -g @openai/codex) e rode de novo"
        );
    }
    let project = harness.path("root")?;
    println!(
        "Vigiando {} a cada {interval} s com o {CODEX}.",
        requests.dir().display()
    );
    loop {
        match requests.files(Some(PENDING)).first() {
            Some(path) => process_with_codex(&project, requests, path)?,
            None => thread::sleep(Duration::from_secs(interval)),
        }
    }
}

fn in_path(program: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|paths| std::env::split_paths(&paths).any(|dir| dir.join(program).is_file()))
}

fn process_with_codex(project: &Path, requests: &Requests, path: &Path) -> Result<()> {
    let id = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default()
        .to_string();
    let shown = path.strip_prefix(project).unwrap_or(path);
    let instruction = format!(
        "Use a skill process-requests para processar o pedido {id} ({}) e só ele.",
        shown.display()
    );
    let status = Command::new(CODEX)
        .args(["exec", "--full-auto", &instruction])
        .current_dir(project)
        .status()
        .with_context(|| format!("executar o {CODEX} para o pedido {id}"))?;
    // Sem isso, um pedido que o codex não conseguiu pegar seria disparado de novo para sempre.
    let still_pending = requests
        .find(&id)
        .is_some_and(|file| frontmatter_get(&file, "estado").is_ok_and(|state| state == PENDING));
    if still_pending {
        requests.transition(&id, &[PENDING], REFUSED, Some(LEFT_PENDING))?;
    }
    let state = requests
        .find(&id)
        .and_then(|file| frontmatter_get(&file, "estado").ok())
        .unwrap_or_default();
    println!("{id}: {state} (codex saiu com {status})");
    Ok(())
}

fn read_stdin() -> Result<String> {
    let mut text = String::new();
    std::io::stdin().read_to_string(&mut text)?;
    Ok(text.trim().to_string())
}

/// Acrescenta a linha `| data | de | para |` à seção `## Histórico`, criada antes de `## Resultado`
/// (ou no fim) com o cabeçalho na primeira transição.
fn with_history_line(content: &str, now: &str, from: &str, to: &str) -> String {
    let line = format!("| {now} | {from} | {to} |");
    let section_end = |start: usize| {
        content[start + HISTORY_HEADING.len()..]
            .find("\n## ")
            .map_or(content.len(), |offset| {
                start + HISTORY_HEADING.len() + offset
            })
    };
    if let Some(start) = content.find(HISTORY_HEADING) {
        let end = section_end(start);
        let head = content[..end].trim_end();
        return format!("{head}\n{line}\n{}", &content[end..]);
    }
    let table = format!("{HISTORY_HEADING}\n\n| Data | De | Para |\n|---|---|---|\n{line}\n\n");
    match content.find(RESULT_HEADING) {
        Some(index) => format!("{}{table}{}", &content[..index], &content[index..]),
        None => format!("{}\n\n{}", content.trim_end(), table.trim_end()) + "\n",
    }
}

/// Troca o conteúdo de `## Resultado` (ou a cria no fim) pelo texto.
fn with_result_section(content: &str, text: &str) -> String {
    let head = match content.find(RESULT_HEADING) {
        Some(index) => &content[..index],
        None => content,
    };
    format!("{}\n\n{RESULT_HEADING}\n\n{text}\n", head.trim_end())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn harness() -> (TempDir, Harness) {
        let dir = TempDir::new().unwrap();
        crate::embedded::extract_to(dir.path()).unwrap();
        let harness = Harness {
            root: dir.path().to_path_buf(),
        };
        (dir, harness)
    }

    fn question(id: &str, status: &str) -> String {
        format!("---\nid: {id}\npedido: PED-001\nspec: SPEC-001\ntask: \"\"\nstatus: {status}\ncreated_at: 2026-01-01T00:00:00Z\n---\n\n## Pergunta\n\nQual?\n")
    }

    #[test]
    fn questions_migrate_loose_files() {
        let (_dir, harness) = harness();
        let base = harness.questions_dir();
        fs::create_dir_all(&base).unwrap();
        fs::write(base.join("P-1.md"), question("P-1", OPEN)).unwrap();
        fs::write(base.join("P-2.md"), question("P-2", ANSWERED)).unwrap();
        fs::write(base.join("P-3.md"), question("P-3", "outro")).unwrap();

        let questions = Questions::open(&harness).unwrap();

        assert_eq!(questions.files(OPEN), [base.join("aberta/P-1.md")]);
        assert_eq!(questions.files(ANSWERED), [base.join("respondida/P-2.md")]);
        assert!(base.join("P-3.md").is_file());
        assert_eq!(
            fs::read_to_string(base.join("aberta/P-1.md")).unwrap(),
            question("P-1", OPEN)
        );
    }

    #[test]
    fn questions_move_to_answered() {
        let (_dir, harness) = harness();
        let open = harness.questions_dir().join("aberta");
        fs::create_dir_all(&open).unwrap();
        fs::write(open.join("P-1.md"), question("P-1", OPEN)).unwrap();
        let questions = Questions::open(&harness).unwrap();

        let moved = questions.move_to("P-1", ANSWERED).unwrap();

        assert_eq!(moved, harness.questions_dir().join("respondida/P-1.md"));
        assert!(!open.join("P-1.md").exists());
        assert!(fs::read_to_string(&moved)
            .unwrap()
            .contains("status: respondida"));
        assert!(questions.move_to("P-1", ANSWERED).is_ok());
        assert!(questions.move_to("P-9", ANSWERED).is_err());
        assert!(questions.move_to("P-1", "outro").is_err());
    }
}
