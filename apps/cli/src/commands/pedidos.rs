//! Comando `pedidos`: os pedidos que o painel grava, numa subpasta por estado, e o agente processa.

use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use clap::Subcommand;
use sha2::{Digest, Sha256};

use crate::harness::{Harness, QuestionFolders, RequestFolders};
use crate::support::{
    config_get, frontmatter_get, frontmatter_set, utc_datetime, utc_datetime_days_ago,
    utc_datetime_minutes_ago, yaml_escape,
};

pub const PENDING: &str = "pendente";
pub const IN_PROGRESS: &str = "em_andamento";
pub const DONE: &str = "feito";
pub const REFUSED: &str = "recusado";
pub const NOTIFICATION_UNREAD: &str = "nao_lida";
pub const NOTIFICATION_READ: &str = "lida";
pub const FREE_QUESTION: &str = "perguntar-agente";
/// Ações que o painel grava como pedido ao agente. Aprovar spec e tarefa não está aqui: o painel
/// aprova direto, para que o agente não fique entre o clique e a aprovação (RN-04).
pub const REQUEST_ACTIONS: &[&str] = &[
    "aprovar-plano",
    "planejar-spec",
    "ajustar-spec",
    "ajustar-plano",
    "ajustar-tarefa",
    "executar-tarefa",
    "finalizar-tarefa",
    "desbloquear-tarefa",
    "finalizar-spec",
    "responder-pergunta",
    "fazer-commit",
    "criar-spec",
    "refinar-spec",
    "executar-onda",
    "liberar-commits",
    "desfazer-commits",
    FREE_QUESTION,
];
pub const CONCLUDED: &str = "concluido";
pub const AWAITING_QUESTION: &str = "aguardando-pergunta";
pub const PAUSED: &str = "pausado";
const RESULT_HEADING: &str = "## Resultado";
const HISTORY_HEADING: &str = "## Histórico";
const REQUEST_HEADING: &str = "## Pedido";
const REQUEST_TEXT_ENDS: &[&str] = &["## Anexos", "## Notas", HISTORY_HEADING, RESULT_HEADING];
const NOTES_HEADING: &str = "## Notas";
const NOTE_MAX_CHARS: usize = 200;
const NOTE_DETAIL_MAX_CHARS: usize = 2000;
const NOTE_WARNING_KEY: &str = "request_note_warning_minutes";
const DEFAULT_NOTE_WARNING_MINUTES: u64 = 15;
pub const OPEN: &str = "aberta";
pub const ANSWERED: &str = "respondida";
const CODEX: &str = "codex";
const CLAUDE: &str = "claude";
const LOOP_FILE: &str = ".loop";
pub const DEFAULT_ROUND_INTERVAL: u64 = 60;
/// Intervalo do disparo sob demanda entre uma conferência e outra quando não há pedido pendente.
const DISPATCH_INTERVAL: Duration = Duration::from_secs(2);

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
    Concluir {
        id: String,
        /// Como o pedido terminou: concluido (padrão), aguardando-pergunta ou pausado.
        #[arg(long)]
        desfecho: Option<String>,
        /// A pergunta aberta; obrigatória com --desfecho aguardando-pergunta.
        #[arg(long)]
        pergunta: Option<String>,
        /// Ação que o usuário pode pedir em seguida; repetível.
        #[arg(long = "saida")]
        saidas: Vec<String>,
    },
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
    /// Grava a nota do agente num pedido `em_andamento`, sem mudar o estado nem `atualizado_em`.
    Nota {
        id: String,
        /// Linha de até 200 caracteres.
        #[arg(allow_hyphen_values = true)]
        texto: String,
        /// Até 2000 caracteres, gravados em `## Notas`.
        #[arg(long, allow_hyphen_values = true)]
        detalhe: Option<String>,
    },
    /// Grava em work/pedidos/.loop a hora desta rodada do loop, o intervalo e o agente.
    Rodada {
        /// Segundos entre um ciclo do loop e outro.
        #[arg(long, default_value_t = DEFAULT_ROUND_INTERVAL)]
        intervalo: u64,
        /// Agente que roda o loop.
        #[arg(long, default_value = CLAUDE)]
        agente: String,
    },
    /// Lista os pedidos e as perguntas que podem ser apagados; só apaga com --confirmar.
    Prune {
        #[arg(long)]
        confirmar: bool,
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
        PedidosCommand::Concluir {
            id,
            desfecho,
            pergunta,
            saidas,
        } => {
            let outcome = Outcome::parse(desfecho.as_deref(), pergunta.as_deref(), &saidas)?;
            let text = read_stdin()?;
            println!(
                "{}",
                requests.transition(&id, &[IN_PROGRESS], DONE, Some(&text), &outcome.fields())?
            );
            Ok(())
        }
        PedidosCommand::Recusar { id } => {
            print_transition(&requests, &id, &[IN_PROGRESS], REFUSED, Some(read_stdin()?))
        }
        PedidosCommand::Vigiar { agente, intervalo } => {
            watch(&harness, &requests, &agente, intervalo)
        }
        PedidosCommand::Nota { id, texto, detalhe } => {
            requests.note(&id, &texto, detalhe.as_deref())?;
            println!("Nota gravada em {id}.");
            Ok(())
        }
        PedidosCommand::Rodada { intervalo, agente } => round(&requests, intervalo, &agente),
        PedidosCommand::Prune { confirmar } => prune(&harness, confirmar),
    }
}

// --- anexos ---

pub const MAX_ATTACHMENTS: usize = 10;
const DRAFTS_FOLDER: &str = ".rascunho";
const DRAFT_MAX_AGE_HOURS: u64 = 24;
pub const DEFAULT_ATTACHMENT_MAX_MB: u64 = 500;
const ATTACHMENT_NAME_MAX_CHARS: usize = 100;
const ATTACHMENT_EXTENSION_MAX_CHARS: usize = 16;
const DRAFT_ID_MAX_CHARS: usize = 64;
const HASH_BUFFER_BYTES: usize = 64 * 1024;
const ATTACHMENT_LIMIT_KEY: &str = "manual_attachment_max_mb";

/// Nomes de arquivo de segredo que o anexo recusa: `*.x` é sufixo, `x.*` é prefixo seguido de
/// ponto e o resto é o nome inteiro.
pub const REFUSED_ATTACHMENT_NAMES: &[&str] = &[
    ".env",
    ".env.*",
    "*.pem",
    "*.key",
    "*.p12",
    "*.pfx",
    "*.jks",
    "*.keystore",
    "*.crt",
    "*.cer",
    "*.der",
    "*.kdbx",
    "*.ovpn",
    "id_rsa",
    "id_dsa",
    "id_ecdsa",
    "id_ed25519",
    "credentials",
    "credentials.*",
    "secrets",
    "secrets.*",
    ".netrc",
    ".npmrc",
    ".pgpass",
];
pub const ACCEPTED_ATTACHMENT_NAMES: &[&str] = &[".env.example"];

/// O último trecho de um nome com caminho de qualquer sistema.
fn last_path_segment(original: &str) -> &str {
    original.rsplit(['/', '\\']).next().unwrap_or(original)
}

pub fn is_refused_attachment_name(original: &str) -> bool {
    let name: String = last_path_segment(original)
        .chars()
        .filter(|c| !c.is_control())
        .collect();
    let name = name
        .trim_end_matches(|c: char| c == '.' || c.is_whitespace())
        .to_lowercase();
    if ACCEPTED_ATTACHMENT_NAMES.contains(&name.as_str()) {
        return false;
    }
    REFUSED_ATTACHMENT_NAMES.iter().any(|pattern| {
        if let Some(suffix) = pattern.strip_prefix('*') {
            name.ends_with(suffix)
        } else if let Some(prefix) = pattern.strip_suffix('*') {
            name.starts_with(prefix)
        } else {
            name == *pattern
        }
    })
}

/// Só `[A-Za-z0-9._-]`, sem sequências de `_` e sem `.`, `_` nem `-` nas pontas.
fn clean_name_part(part: &str) -> String {
    let mut cleaned = String::new();
    for c in part.chars() {
        let kept = if c.is_ascii_alphanumeric() || matches!(c, '.' | '-') {
            c
        } else {
            '_'
        };
        if kept != '_' || !cleaned.ends_with('_') {
            cleaned.push(kept);
        }
    }
    cleaned
        .trim_matches(|c| matches!(c, '.' | '_' | '-'))
        .to_string()
}

/// `NN-nome.ext` só com caracteres seguros e em até 100 caracteres, cortando só o nome.
pub fn clean_attachment_name(number: usize, original: &str) -> String {
    let segment = last_path_segment(original);
    let (stem, extension) = match segment.rfind('.') {
        Some(dot) if dot > 0 => (&segment[..dot], &segment[dot + 1..]),
        _ => (segment, ""),
    };
    let mut stem = clean_name_part(stem);
    if stem.is_empty() {
        stem = "arquivo".to_string();
    }
    let extension: String = clean_name_part(extension)
        .chars()
        .take(ATTACHMENT_EXTENSION_MAX_CHARS)
        .collect();
    let prefix = format!("{number:02}-");
    let suffix = if extension.is_empty() {
        String::new()
    } else {
        format!(".{extension}")
    };
    let room = ATTACHMENT_NAME_MAX_CHARS.saturating_sub(prefix.len() + suffix.len());
    let stem: String = stem.chars().take(room).collect();
    format!("{prefix}{stem}{suffix}")
}

pub fn is_draft_id(id: &str) -> bool {
    (1..=DRAFT_ID_MAX_CHARS).contains(&id.len())
        && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

/// O limite da soma dos anexos de um pedido, em bytes.
pub fn attachment_limit_bytes(harness: &Harness) -> u64 {
    let megabytes = config_get(&harness.config_file("tasks.yml"), ATTACHMENT_LIMIT_KEY)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_ATTACHMENT_MAX_MB);
    megabytes * 1024 * 1024
}

/// O sha256 em hexadecimal, lendo por um buffer para não carregar o arquivo.
pub fn file_sha256(path: &Path) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; HASH_BUFFER_BYTES];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentFile {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
}

/// Os arquivos de uma pasta de anexos, em ordem de nome, sem os que começam por ponto.
pub fn attachment_files(dir: &Path) -> Vec<AttachmentFile> {
    sorted_entries(dir)
        .into_iter()
        .filter_map(|path| {
            let name = file_name_of(&path);
            let metadata = fs::metadata(&path).ok()?;
            (metadata.is_file() && !name.starts_with('.')).then_some(AttachmentFile {
                name,
                size: metadata.len(),
                path,
            })
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttachmentRefusal {
    SecretName,
    TooMany,
    Empty,
    OverLimit,
}

impl AttachmentRefusal {
    pub fn message(&self) -> String {
        match self {
            Self::SecretName => "O nome desse arquivo parece o de um arquivo de segredo, então ele não foi anexado. Para o agente ler esse arquivo, cite-o na própria pergunta.".to_string(),
            Self::TooMany => format!("Um pedido aceita até {MAX_ATTACHMENTS} anexos."),
            Self::Empty => "O arquivo está vazio e não foi anexado.".to_string(),
            Self::OverLimit => "Os anexos passam do tamanho máximo permitido para um pedido.".to_string(),
        }
    }
}

// --- limpeza ---

/// Dias depois de concluído para um pedido ou pergunta sem spec poder ser apagado.
const PRUNE_DAYS: u64 = 30;
const NO_SPEC: &str = "sem spec";

/// Um arquivo que o `prune` pode apagar, com a spec dele (vazia sem spec) e o motivo.
pub struct PruneItem {
    pub path: PathBuf,
    pub spec: String,
    pub reason: String,
    /// A pasta de anexos de um pedido arquivado, dentro do arquivo morto da spec.
    pub in_archive: bool,
}

/// Os pedidos e as perguntas que podem ser apagados; nunca um log de avais.
pub fn prune_candidates(harness: &Harness) -> Result<Vec<PruneItem>> {
    let requests = Requests::open(harness)?;
    let questions = Questions::open(harness)?;
    let archive = harness.path("archive")?;
    let mut archived = Vec::new();
    let names = [requests.dir().file_name(), questions.dir().file_name()];
    let requests_name = requests.dir().file_name().unwrap_or_default();
    let attachments_name = requests.folders.attachments.clone();
    for spec_dir in sorted_entries(&archive)
        .into_iter()
        .filter(|path| path.is_dir())
    {
        let spec = file_name_of(&spec_dir);
        for name in names.iter().flatten() {
            for path in sorted_entries(&spec_dir.join(name)) {
                if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
                    let attachments = (*name == requests_name)
                        .then(|| {
                            let folder = spec_dir.join(name).join(&attachments_name);
                            let id = path.file_stem().unwrap_or_default();
                            attachments_item(folder.join(id), &spec, true)
                        })
                        .flatten();
                    archived.push(item(path, &spec, "arquivado"));
                    archived.extend(attachments);
                }
            }
        }
    }
    let finished: Vec<(PathBuf, &str)> = [DONE, REFUSED]
        .iter()
        .flat_map(|state| requests.files(Some(state)))
        .map(|path| (path, "atualizado_em"))
        .chain(
            questions
                .files(ANSWERED)
                .into_iter()
                .map(|path| (path, "answered_at")),
        )
        .collect();
    let cutoff = utc_datetime_days_ago(PRUNE_DAYS);
    let mut old = Vec::new();
    let mut of_archived_spec = Vec::new();
    for (path, date_key) in finished {
        let spec = frontmatter_get(&path, "spec").unwrap_or_default();
        if spec.is_empty() {
            let date = frontmatter_get(&path, date_key)
                .ok()
                .filter(|date| !date.is_empty())
                .or_else(|| frontmatter_get(&path, "created_at").ok())
                .unwrap_or_default();
            if !date.is_empty() && date < cutoff {
                let attachments = request_attachments_item(&requests, &path, "");
                old.push(item(path, "", "concluído há mais de 30 dias"));
                old.extend(attachments);
            }
        } else if archive.join(&spec).is_dir() {
            let attachments = request_attachments_item(&requests, &path, &spec);
            of_archived_spec.push(item(path, &spec, "spec arquivada"));
            of_archived_spec.extend(attachments);
        }
    }
    let drafts: Vec<PruneItem> = requests
        .old_drafts()
        .into_iter()
        .map(|path| item(path, "", OLD_DRAFT_REASON))
        .collect();
    let mut candidates = Vec::new();
    for mut group in [archived, old, of_archived_spec, drafts] {
        group.sort_by(|left, right| left.path.cmp(&right.path));
        candidates.extend(group);
    }
    Ok(candidates)
}

/// Apaga os arquivos listados pelo `prune_candidates` e devolve quantos foram apagados.
pub fn prune_delete(items: &[PruneItem]) -> Result<usize> {
    for item in items {
        if item.path.is_dir() {
            fs::remove_dir_all(&item.path)
        } else {
            fs::remove_file(&item.path)
        }
        .with_context(|| format!("apagando {}", item.path.display()))?;
        if item.reason == "arquivado" {
            remove_if_empty(item.path.parent())?;
        } else if item.in_archive {
            // A pasta de anexos e a de pedidos do arquivo morto, de dentro para fora.
            let attachments = item.path.parent();
            remove_if_empty(attachments)?;
            remove_if_empty(attachments.and_then(Path::parent))?;
        }
    }
    Ok(items.len())
}

/// Só a pasta `pedidos/`, `perguntas/` ou `anexos/` do arquivo morto, quando fica vazia.
fn remove_if_empty(dir: Option<&Path>) -> Result<()> {
    if let Some(dir) = dir {
        if fs::read_dir(dir).is_ok_and(|mut entries| entries.next().is_none()) {
            fs::remove_dir(dir).with_context(|| format!("removendo {}", dir.display()))?;
        }
    }
    Ok(())
}

fn item(path: PathBuf, spec: &str, reason: &str) -> PruneItem {
    PruneItem {
        path,
        spec: spec.to_string(),
        reason: reason.to_string(),
        in_archive: false,
    }
}

const ATTACHMENTS_REASON: &str = "anexos do pedido";
const OLD_DRAFT_REASON: &str = "rascunho com mais de 24 horas";

/// A pasta de anexos do pedido concluído, quando o arquivo é de um pedido e a pasta existe.
fn request_attachments_item(requests: &Requests, request: &Path, spec: &str) -> Option<PruneItem> {
    let id = request.file_stem()?.to_str()?;
    if !is_request_id(id) {
        return None;
    }
    attachments_item(requests.request_attachments_dir(id), spec, false)
}

/// A pasta de anexos de um pedido, quando existe.
fn attachments_item(folder: PathBuf, spec: &str, in_archive: bool) -> Option<PruneItem> {
    folder.is_dir().then(|| PruneItem {
        in_archive,
        ..item(folder, spec, ATTACHMENTS_REASON)
    })
}

fn sorted_entries(dir: &Path) -> Vec<PathBuf> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .map(|entries| entries.flatten().map(|entry| entry.path()).collect())
        .unwrap_or_default();
    entries.sort();
    entries
}

fn file_name_of(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn prune(harness: &Harness, confirm: bool) -> Result<()> {
    let items = prune_candidates(harness)?;
    if items.is_empty() {
        println!("Não há o que limpar.");
        return Ok(());
    }
    println!("| Arquivo | Spec | Motivo |");
    println!("|---|---|---|");
    for item in &items {
        let path = item.path.strip_prefix(&harness.root).unwrap_or(&item.path);
        let spec = if item.spec.is_empty() {
            "-"
        } else {
            &item.spec
        };
        println!("| {} | {spec} | {} |", path.display(), item.reason);
    }
    let mut specs: Vec<&str> = items
        .iter()
        .map(|item| {
            if item.spec.is_empty() {
                NO_SPEC
            } else {
                item.spec.as_str()
            }
        })
        .collect();
    specs.sort();
    specs.dedup();
    let specs = specs.join(", ");
    println!();
    if confirm {
        let deleted = prune_delete(&items)?;
        println!("{deleted} arquivo(s) apagado(s), de {specs}; os avais ficam.");
    } else {
        println!(
            "{} arquivo(s), de {specs}. Nada foi apagado: rode vostok pedidos prune --confirmar para apagar.",
            items.len()
        );
    }
    Ok(())
}

pub fn is_request_id(id: &str) -> bool {
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

/// O resultado de `Requests::mark_read`.
#[derive(Debug, PartialEq)]
pub enum MarkRead {
    Missing,
    NotAnswered,
    Marked(String),
}

/// Os pedidos do painel, numa subpasta por estado (`paths.requests` e `folders.requests`).
pub struct Requests {
    dir: PathBuf,
    folders: RequestFolders,
    note_warning_minutes: u64,
}

impl Requests {
    pub fn open(harness: &Harness) -> Result<Self> {
        let requests = Self {
            dir: harness.requests_dir(),
            folders: harness.folders()?.requests,
            note_warning_minutes: note_warning_minutes(harness),
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
        let mut names: Vec<&str> = self.states().iter().map(|(_, folder)| *folder).collect();
        names.push(self.folders.attachments.as_str());
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

    pub fn attachments_dir(&self) -> PathBuf {
        self.dir.join(&self.folders.attachments)
    }

    pub fn draft_dir(&self, draft_id: &str) -> PathBuf {
        self.attachments_dir().join(DRAFTS_FOLDER).join(draft_id)
    }

    pub fn request_attachments_dir(&self, id: &str) -> PathBuf {
        self.attachments_dir().join(id)
    }

    /// Confere um arquivo a anexar ao rascunho, nesta ordem: nome, tamanho zero, décimo primeiro
    /// arquivo e soma acima do limite.
    pub fn check_attachment(
        &self,
        draft_id: &str,
        original: &str,
        size: u64,
        max_bytes: u64,
    ) -> Result<(), AttachmentRefusal> {
        if is_refused_attachment_name(original) {
            return Err(AttachmentRefusal::SecretName);
        }
        if size == 0 {
            return Err(AttachmentRefusal::Empty);
        }
        let existing = attachment_files(&self.draft_dir(draft_id));
        if existing.len() >= MAX_ATTACHMENTS {
            return Err(AttachmentRefusal::TooMany);
        }
        let total: u64 = existing.iter().map(|file| file.size).sum();
        if total + size > max_bytes {
            return Err(AttachmentRefusal::OverLimit);
        }
        Ok(())
    }

    /// O nome limpo do próximo anexo do rascunho: o maior `NN` da pasta mais um.
    pub fn next_attachment_name(&self, draft_id: &str, original: &str) -> String {
        let highest = sorted_entries(&self.draft_dir(draft_id))
            .iter()
            .filter_map(|path| {
                let name = file_name_of(path);
                let (number, _) = name.split_once('-')?;
                number.parse::<usize>().ok()
            })
            .max()
            .unwrap_or(0);
        clean_attachment_name(highest + 1, original)
    }

    /// As pastas de rascunho com mais de 24 horas.
    pub fn old_drafts(&self) -> Vec<PathBuf> {
        let limit = Duration::from_secs(DRAFT_MAX_AGE_HOURS * 3600);
        sorted_entries(&self.attachments_dir().join(DRAFTS_FOLDER))
            .into_iter()
            .filter(|path| path.is_dir())
            .filter(|path| {
                fs::metadata(path)
                    .and_then(|metadata| metadata.modified())
                    .ok()
                    .and_then(|modified| modified.elapsed().ok())
                    .is_some_and(|age| age > limit)
            })
            .collect()
    }

    /// Apaga os rascunhos com mais de 24 horas e devolve quantos eram.
    pub fn sweep_drafts(&self) -> io::Result<usize> {
        let old = self.old_drafts();
        for draft in &old {
            fs::remove_dir_all(draft)?;
        }
        Ok(old.len())
    }

    /// Cria o próximo pedido levando antes a pasta do rascunho para a do pedido; se o número já
    /// tem pasta ou arquivo, tenta o seguinte. Em falha, a pasta volta ao rascunho.
    pub fn create_from_draft(
        &self,
        draft_id: &str,
        content: impl Fn(&str) -> String,
    ) -> io::Result<(String, PathBuf)> {
        let pending = self.dir.join(&self.folders.pending);
        fs::create_dir_all(&pending)?;
        fs::create_dir_all(self.attachments_dir())?;
        let draft = self.draft_dir(draft_id);
        let mut number: u32 = self
            .next_id()
            .strip_prefix("PED-")
            .and_then(|digits| digits.parse().ok())
            .unwrap_or(1);
        loop {
            let id = format!("PED-{number:03}");
            let folder = self.request_attachments_dir(&id);
            if folder.exists() {
                number += 1;
                continue;
            }
            fs::rename(&draft, &folder)?;
            let path = pending.join(format!("{id}.md"));
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(mut file) => match file.write_all(content(&id).as_bytes()) {
                    Ok(()) => return Ok((id, path)),
                    Err(error) => {
                        let _ = fs::remove_file(&path);
                        let _ = fs::rename(&folder, &draft);
                        return Err(error);
                    }
                },
                Err(error) => {
                    let _ = fs::rename(&folder, &draft);
                    if error.kind() == io::ErrorKind::AlreadyExists {
                        number += 1;
                        continue;
                    }
                    return Err(error);
                }
            }
        }
    }

    /// Marca a resposta de uma pergunta livre como lida, uma vez só, sem mexer no estado nem em
    /// `atualizado_em`.
    pub fn mark_read(&self, id: &str) -> Result<MarkRead> {
        let Some(path) = is_request_id(id).then(|| self.find(id)).flatten() else {
            return Ok(MarkRead::Missing);
        };
        let state = frontmatter_get(&path, "estado")?;
        let answered = state == DONE || state == REFUSED;
        if frontmatter_get(&path, "acao")? != FREE_QUESTION || !answered {
            return Ok(MarkRead::NotAnswered);
        }
        if frontmatter_get(&path, "notificacao").unwrap_or_default() == NOTIFICATION_UNREAD {
            frontmatter_set(&path, "notificacao", NOTIFICATION_READ)?;
            crate::commands::spec::frontmatter_put(&path, "notificacao_lida_em", &utc_datetime())?;
        }
        let already = frontmatter_get(&path, "lida_em").unwrap_or_default();
        if !already.is_empty() {
            return Ok(MarkRead::Marked(already));
        }
        let now = utc_datetime();
        crate::commands::spec::frontmatter_put(&path, "lida_em", &now)?;
        Ok(MarkRead::Marked(now))
    }

    /// Grava a nota no front-matter e no histórico de `## Notas`; só um pedido em andamento aceita.
    pub fn note(&self, id: &str, text: &str, detail: Option<&str>) -> Result<()> {
        let text = checked_note(text)?;
        let detail = checked_detail(detail)?;
        if !is_request_id(id) {
            bail!("ID de pedido inválido '{id}': use o formato PED-NNN");
        }
        let Some(path) = self.find(id) else {
            bail!("Pedido {id} não encontrado em {}", self.dir.display());
        };
        let state = frontmatter_get(&path, "estado")?;
        if state != IN_PROGRESS {
            bail!("Pedido {id} está '{state}': só aceita nota em '{IN_PROGRESS}'");
        }
        let now = utc_datetime();
        let content = fs::read_to_string(&path)?;
        let tmp = path.with_file_name(format!(".{id}.nota.tmp"));
        fs::write(&tmp, with_note(&content, &now, text, detail))
            .with_context(|| format!("gravar {}", tmp.display()))?;
        let quoted = format!("\"{}\"", yaml_escape(text));
        crate::commands::spec::frontmatter_put(&tmp, "nota", &quoted)?;
        crate::commands::spec::frontmatter_put(&tmp, "nota_em", &now)?;
        fs::rename(&tmp, &path).with_context(|| format!("gravar {}", path.display()))
    }

    /// O primeiro pedido em andamento sem nota nova há mais do tempo configurado, com a hora da
    /// última nota ou, sem nenhuma, a de início.
    fn note_warning(&self) -> Option<(String, String)> {
        let cutoff = utc_datetime_minutes_ago(self.note_warning_minutes);
        self.files(Some(IN_PROGRESS)).iter().find_map(|path| {
            let field = |key| {
                frontmatter_get(path, key)
                    .ok()
                    .filter(|value: &String| !value.is_empty())
            };
            let since = field("nota_em").or_else(|| field("atualizado_em"))?;
            (since < cutoff).then(|| (field("id").unwrap_or_default(), since))
        })
    }

    /// Pedidos já respondidos que carregam o campo `notificacao`, lida ou não.
    pub fn notification_files(&self) -> Vec<PathBuf> {
        let mut files = self.files(Some(DONE));
        files.extend(self.files(Some(REFUSED)));
        files.retain(|file| frontmatter_get(file, "notificacao").is_ok_and(|v| !v.is_empty()));
        files
    }

    /// Marca como lidas as notificações não lidas (as dos `ids`, ou todas sem a lista) sem mexer
    /// no estado nem em `atualizado_em`; devolve quantas gravou.
    pub fn mark_notifications_read(&self, ids: Option<&[&str]>) -> Result<usize> {
        let mut marked = 0;
        for file in self.notification_files() {
            let unread = frontmatter_get(&file, "notificacao")? == NOTIFICATION_UNREAD;
            let wanted = ids.is_none_or(|list| {
                let id = file.file_stem().and_then(|name| name.to_str());
                id.is_some_and(|id| list.contains(&id))
            });
            if !unread || !wanted {
                continue;
            }
            frontmatter_set(&file, "notificacao", NOTIFICATION_READ)?;
            crate::commands::spec::frontmatter_put(&file, "notificacao_lida_em", &utc_datetime())?;
            marked += 1;
        }
        Ok(marked)
    }

    /// Muda o estado: grava o resultado, a linha de histórico e o estado, e só então move o arquivo.
    pub fn transition(
        &self,
        id: &str,
        allowed_from: &[&str],
        to: &str,
        result: Option<&str>,
        fields: &[(&str, String)],
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
        for (key, value) in fields {
            crate::commands::spec::frontmatter_put(&path, key, value)?;
        }
        if to == DONE || to == REFUSED {
            crate::commands::spec::frontmatter_put(&path, "notificacao", NOTIFICATION_UNREAD)?;
        }
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

/// Como o agente terminou um pedido: o desfecho, a pergunta aberta e as ações seguintes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub desfecho: String,
    pub pergunta: String,
    pub saidas: Vec<String>,
}

impl Outcome {
    /// Valida as opções do `concluir`; sem desfecho, o pedido é `concluido`.
    pub fn parse(
        desfecho: Option<&str>,
        pergunta: Option<&str>,
        saidas: &[String],
    ) -> Result<Self> {
        let desfecho = desfecho.unwrap_or(CONCLUDED);
        if ![CONCLUDED, AWAITING_QUESTION, PAUSED].contains(&desfecho) {
            bail!("desfecho inválido '{desfecho}': use concluido, aguardando-pergunta ou pausado");
        }
        let pergunta = match (desfecho == AWAITING_QUESTION, pergunta) {
            (true, None) => bail!("--desfecho aguardando-pergunta exige --pergunta P-NNN"),
            (false, Some(_)) => bail!("--pergunta só vale com --desfecho aguardando-pergunta"),
            (true, Some(id)) if !is_question_id(id) => {
                bail!("ID de pergunta inválido '{id}': use o formato P-NNN")
            }
            (_, id) => id.unwrap_or_default(),
        };
        for saida in saidas {
            if !REQUEST_ACTIONS.contains(&saida.as_str()) {
                bail!(
                    "ação de saída desconhecida '{saida}': use uma de {}",
                    REQUEST_ACTIONS.join(", ")
                );
            }
        }
        Ok(Self {
            desfecho: desfecho.to_string(),
            pergunta: pergunta.to_string(),
            saidas: saidas.to_vec(),
        })
    }

    /// Lê o desfecho de um pedido; um pedido sem o campo (formato antigo) é `concluido`.
    pub fn read(path: &Path) -> Self {
        let get = |key| frontmatter_get(path, key).unwrap_or_default();
        let desfecho = get("desfecho");
        Self {
            desfecho: if desfecho.is_empty() {
                CONCLUDED.to_string()
            } else {
                desfecho
            },
            pergunta: get("pergunta"),
            saidas: get("saidas")
                .trim_matches(|c| c == '[' || c == ']')
                .split(',')
                .map(|item| item.trim().to_string())
                .filter(|item| !item.is_empty())
                .collect(),
        }
    }

    fn fields(&self) -> Vec<(&'static str, String)> {
        let mut fields = vec![("desfecho", self.desfecho.clone())];
        if self.desfecho == AWAITING_QUESTION {
            fields.push(("pergunta", self.pergunta.clone()));
        }
        fields.push(("saidas", format!("[{}]", self.saidas.join(", "))));
        fields
    }
}

/// A última rodada do agente em loop, em `work/pedidos/.loop` (RF-19).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoopRound {
    pub ultima_rodada: String,
    pub intervalo: u64,
    pub agente: String,
    /// Pedido em andamento sem nota nova e a hora da falta; vazios sem aviso.
    pub sem_nota_pedido: String,
    pub sem_nota_desde: String,
}

impl LoopRound {
    /// Lê o registro; sem o arquivo ou sem a hora da rodada, `None`. Nunca falha.
    pub fn read(requests_dir: &Path) -> Option<Self> {
        let content = fs::read_to_string(requests_dir.join(LOOP_FILE)).ok()?;
        let value = |key: &str| {
            content.lines().find_map(|line| {
                let (name, value) = line.split_once(": ")?;
                (name.trim() == key).then(|| value.trim().to_string())
            })
        };
        let ultima_rodada = value("ultima_rodada").filter(|value| !value.is_empty())?;
        let intervalo = value("intervalo")
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(DEFAULT_ROUND_INTERVAL);
        let agente = value("agente")
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| CLAUDE.to_string());
        Some(Self {
            ultima_rodada,
            intervalo,
            agente,
            sem_nota_pedido: value("sem_nota_pedido").unwrap_or_default(),
            sem_nota_desde: value("sem_nota_desde").unwrap_or_default(),
        })
    }

    /// Grava a rodada agora, por arquivo temporário e rename, para ninguém ler pela metade.
    pub fn record(requests: &Requests, intervalo: u64, agente: &str) -> Result<()> {
        let requests_dir = requests.dir();
        fs::create_dir_all(requests_dir)?;
        let tmp = requests_dir.join(".loop.tmp");
        let mut content = format!(
            "ultima_rodada: {}\nintervalo: {intervalo}\nagente: {agente}\n",
            utc_datetime()
        );
        if let Some((id, since)) = requests.note_warning() {
            content.push_str(&format!("sem_nota_pedido: {id}\nsem_nota_desde: {since}\n"));
        }
        fs::write(&tmp, content).with_context(|| format!("gravar {}", tmp.display()))?;
        fs::rename(&tmp, requests_dir.join(LOOP_FILE))
            .with_context(|| format!("gravar {}", requests_dir.join(LOOP_FILE).display()))
    }
}

fn round(requests: &Requests, interval: u64, agent: &str) -> Result<()> {
    if interval == 0 {
        bail!("--intervalo deve ser maior que zero");
    }
    let agent = agent.trim();
    if agent.is_empty() {
        bail!("--agente não pode ser vazio");
    }
    LoopRound::record(requests, interval, agent)?;
    println!("Rodada gravada ({interval} s, {agent}).");
    Ok(())
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
        requests.transition(id, allowed_from, to, result.as_deref(), &[])?
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
    run_loop(CODEX, &project, requests, Duration::from_secs(interval))
}

/// Processa os pendentes um por vez, em ordem de ID, e dorme quando não há nenhum (RN-04).
fn run_loop(agent: &str, project: &Path, requests: &Requests, interval: Duration) -> Result<()> {
    loop {
        if let Err(error) = LoopRound::record(requests, interval.as_secs().max(1), agent) {
            eprintln!("Não foi possível gravar a rodada do loop: {error:#}");
        }
        match requests.files(Some(PENDING)).first() {
            Some(path) => process_with(agent, project, requests, path)?,
            None => thread::sleep(interval),
        }
    }
}

/// Liga o disparo sob demanda do `vostok painel` (RF-13): uma única thread com o laço do agente
/// de `request_dispatch`. Desligado ou sem o agente no PATH, não faz nada e devolve `None`.
pub fn start_dispatch(harness: &Harness) -> Result<Option<thread::JoinHandle<()>>> {
    // O leitor da configuração já recusa um agente fora de "", codex e claude.
    let agent = harness.request_dispatch()?;
    if agent.is_empty() {
        return Ok(None);
    }
    if !in_path(&agent) {
        eprintln!(
            "Disparo sob demanda desligado: o {agent} não está no PATH; os pedidos ficam pendentes até um agente em loop (vostok pedidos vigiar ou /process-requests)."
        );
        return Ok(None);
    }
    let project = harness.path("root")?;
    let requests = Requests::open(harness)?;
    eprintln!(
        "Disparo sob demanda com o {agent} a cada {} s.",
        DISPATCH_INTERVAL.as_secs()
    );
    Ok(Some(thread::spawn(move || {
        if let Err(error) = run_loop(&agent, &project, &requests, DISPATCH_INTERVAL) {
            eprintln!("Disparo sob demanda parado: {error:#}");
        }
    })))
}

fn in_path(program: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|paths| std::env::split_paths(&paths).any(|dir| dir.join(program).is_file()))
}

/// O comando não interativo do agente para a instrução.
fn agent_command(agent: &str, instruction: &str) -> Command {
    let mut command = Command::new(agent);
    if agent == CLAUDE {
        command.args([
            "-p",
            instruction,
            "--permission-mode",
            "acceptEdits",
            "--allowedTools",
            "Bash",
        ]);
    } else {
        command.args(["exec", "--full-auto", instruction]);
    }
    command
}

fn left_pending(agent: &str) -> String {
    format!(
        "o {agent} terminou sem processar o pedido; veja a saída do vostok pedidos vigiar ou do vostok painel"
    )
}

fn process_with(agent: &str, project: &Path, requests: &Requests, path: &Path) -> Result<()> {
    let id = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default()
        .to_string();
    let instruction =
        format!("Use a skill process-requests para processar o pedido {id} e só ele.");
    let status = agent_command(agent, &instruction)
        .current_dir(project)
        .status()
        .with_context(|| format!("executar o {agent} para o pedido {id}"))?;
    // Sem isso, um pedido que o agente não conseguiu pegar seria disparado de novo para sempre.
    let still_pending = requests
        .find(&id)
        .is_some_and(|file| frontmatter_get(&file, "estado").is_ok_and(|state| state == PENDING));
    if still_pending {
        requests.transition(&id, &[PENDING], REFUSED, Some(&left_pending(agent)), &[])?;
    }
    let state = requests
        .find(&id)
        .and_then(|file| frontmatter_get(&file, "estado").ok())
        .unwrap_or_default();
    println!("{id}: {state} ({agent} saiu com {status})");
    Ok(())
}

fn read_stdin() -> Result<String> {
    let mut text = String::new();
    std::io::stdin().read_to_string(&mut text)?;
    Ok(text.trim().to_string())
}

/// Início do título `## ...` no começo de uma linha: o texto de uma nota, no front-matter ou
/// recuado no detalhe, nunca vale como seção.
fn heading_start(content: &str, heading: &str) -> Option<usize> {
    if content.starts_with(heading) {
        return Some(0);
    }
    content.find(&format!("\n{heading}")).map(|index| index + 1)
}

fn note_warning_minutes(harness: &Harness) -> u64 {
    config_get(&harness.config_file("main.yml"), NOTE_WARNING_KEY)
        .ok()
        .and_then(|value| value.parse().ok())
        .filter(|minutes| *minutes > 0)
        .unwrap_or(DEFAULT_NOTE_WARNING_MINUTES)
}

fn checked_note(text: &str) -> Result<&str> {
    if text.contains(['\n', '\r']) {
        bail!("a nota deve ter uma linha só");
    }
    let text = text.trim();
    if text.is_empty() {
        bail!("a nota não pode ser vazia");
    }
    if text.chars().count() > NOTE_MAX_CHARS {
        bail!("a nota passa de {NOTE_MAX_CHARS} caracteres");
    }
    Ok(text)
}

fn checked_detail(detail: Option<&str>) -> Result<Option<&str>> {
    let detail = detail.map(str::trim).filter(|detail| !detail.is_empty());
    if detail.is_some_and(|detail| detail.chars().count() > NOTE_DETAIL_MAX_CHARS) {
        bail!("o detalhe passa de {NOTE_DETAIL_MAX_CHARS} caracteres");
    }
    Ok(detail)
}

/// A entrada da nota em `## Notas`. Cada linha do detalhe vai recuada, para não criar seção, e a
/// cerca de código leva uma barra, porque uma cerca aberta esconderia o `## Resultado` da leitura.
fn note_entry(now: &str, text: &str, detail: Option<&str>) -> String {
    let mut entry = format!("- {now} {text}\n");
    for line in detail.unwrap_or_default().lines() {
        let escape = if line.trim_start().starts_with("```") {
            "\\"
        } else {
            ""
        };
        entry.push_str(&format!("  {escape}{line}\n"));
    }
    entry
}

/// Acrescenta a entrada ao fim de `## Notas`, criada antes de `## Resultado` (ou no fim).
fn with_note(content: &str, now: &str, text: &str, detail: Option<&str>) -> String {
    let entry = note_entry(now, text, detail);
    if let Some(start) = heading_start(content, NOTES_HEADING) {
        let after = start + NOTES_HEADING.len();
        let end = content[after..]
            .find("\n## ")
            .map_or(content.len(), |offset| after + offset);
        let head = content[..end].trim_end();
        return format!("{head}\n{entry}{}", &content[end..]);
    }
    let section = format!("{NOTES_HEADING}\n\n{entry}\n");
    match heading_start(content, RESULT_HEADING) {
        Some(index) => format!("{}{section}{}", &content[..index], &content[index..]),
        None => format!("{}\n\n{section}", content.trim_end()),
    }
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
    if let Some(start) = heading_start(content, HISTORY_HEADING) {
        let end = section_end(start);
        let head = content[..end].trim_end();
        return format!("{head}\n{line}\n{}", &content[end..]);
    }
    let table = format!("{HISTORY_HEADING}\n\n| Data | De | Para |\n|---|---|---|\n{line}\n\n");
    match heading_start(content, RESULT_HEADING) {
        Some(index) => format!("{}{table}{}", &content[..index], &content[index..]),
        None => format!("{}\n\n{}", content.trim_end(), table.trim_end()) + "\n",
    }
}

/// O texto do usuário em `## Pedido`, inteiro: títulos próprios dele ficam, e só as seções do
/// harness (anexos, notas, histórico e resultado) encerram.
pub fn request_text(content: &str) -> String {
    let mut lines = content.lines().map(|line| line.trim_end_matches('\r'));
    let text: Vec<&str> = lines
        .by_ref()
        .skip_while(|line| *line != REQUEST_HEADING)
        .skip(1)
        .take_while(|line| !REQUEST_TEXT_ENDS.contains(line))
        .collect();
    text.join("\n").trim().to_string()
}

/// Tudo depois de `## Resultado`, com os títulos próprios da resposta.
pub fn request_result(content: &str) -> String {
    heading_start(content, RESULT_HEADING)
        .map(|index| content[index + RESULT_HEADING.len()..].trim().to_string())
        .unwrap_or_default()
}

/// Troca o conteúdo de `## Resultado` (ou a cria no fim) pelo texto.
fn with_result_section(content: &str, text: &str) -> String {
    let head = match heading_start(content, RESULT_HEADING) {
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

    #[test]
    fn pergunta_livre_texto_com_titulo_proprio() {
        let content = "---\nid: PED-001\n---\n\n## Pedido\n\nPrimeira.\n\n## Seção própria\n\nMais.\n\n## Notas\n\n- n\n\n## Resultado\n\nok\n";
        assert_eq!(
            request_text(content),
            "Primeira.\n\n## Seção própria\n\nMais."
        );
        assert_eq!(request_text("---\n---\n\nsem seção\n"), "");
    }

    #[test]
    fn pergunta_livre_resultado_com_titulo_proprio() {
        let content = "## Pedido\n\nP.\n\n## Resultado\n\nCurta.\n\n## Detalhe\n\nLonga.\n";
        assert_eq!(request_result(content), "Curta.\n\n## Detalhe\n\nLonga.");
        assert_eq!(request_result("## Pedido\n\nP.\n"), "");
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

    #[test]
    fn desfecho_de_pedido_antigo_e_concluido() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("PED-001.md");
        fs::write(
            &path,
            "---\nid: PED-001\nestado: feito\n---\n\n## Resultado\n\nOk.\n",
        )
        .unwrap();

        let outcome = Outcome::read(&path);

        assert_eq!(outcome.desfecho, "concluido");
        assert_eq!(outcome.pergunta, "");
        assert!(outcome.saidas.is_empty());
    }

    #[test]
    fn desfecho_le_a_lista_de_saidas() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("PED-001.md");
        fs::write(
            &path,
            "---\nid: PED-001\nestado: feito\ndesfecho: pausado\nsaidas: [criar-spec, ajustar-spec]\n---\n",
        )
        .unwrap();

        let outcome = Outcome::read(&path);

        assert_eq!(outcome.desfecho, "pausado");
        assert_eq!(outcome.saidas, ["criar-spec", "ajustar-spec"]);
    }

    #[test]
    fn rodada_le_o_registro() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join(".loop"),
            "ultima_rodada: 2026-11-03T10:00:00Z\nintervalo: 90\nagente: codex\n",
        )
        .unwrap();

        assert_eq!(
            LoopRound::read(dir.path()),
            Some(LoopRound {
                ultima_rodada: "2026-11-03T10:00:00Z".to_string(),
                intervalo: 90,
                agente: "codex".to_string(),
                sem_nota_pedido: String::new(),
                sem_nota_desde: String::new(),
            })
        );
    }

    #[test]
    fn rodada_le_o_aviso() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join(".loop"),
            "ultima_rodada: 2026-11-03T10:00:00Z\nintervalo: 90\nagente: codex\nsem_nota_pedido: PED-002\nsem_nota_desde: 2026-11-03T09:00:00Z\n",
        )
        .unwrap();

        let round = LoopRound::read(dir.path()).unwrap();

        assert_eq!(round.sem_nota_pedido, "PED-002");
        assert_eq!(round.sem_nota_desde, "2026-11-03T09:00:00Z");
    }

    #[test]
    fn rodada_antiga_le_sem_aviso() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join(".loop"),
            "ultima_rodada: 2026-11-03T10:00:00Z\nintervalo: 90\nagente: codex\n",
        )
        .unwrap();

        let round = LoopRound::read(dir.path()).unwrap();

        assert!(round.sem_nota_pedido.is_empty());
        assert!(round.sem_nota_desde.is_empty());
    }

    #[test]
    fn rodada_sem_registro_e_none() {
        let dir = TempDir::new().unwrap();

        assert_eq!(LoopRound::read(dir.path()), None);
    }

    #[test]
    fn rodada_com_campos_faltando_usa_os_padroes() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join(".loop"),
            "ultima_rodada: 2026-11-03T10:00:00Z\nintervalo: x\n",
        )
        .unwrap();

        let round = LoopRound::read(dir.path()).unwrap();

        assert_eq!(round.intervalo, 60);
        assert_eq!(round.agente, "claude");
    }

    #[test]
    fn anexo_nome_limpo_tira_caminho_html_controle_e_acento() {
        assert_eq!(clean_attachment_name(1, "../../etc/passwd"), "01-passwd");
        assert_eq!(clean_attachment_name(1, "a\\b.txt"), "01-b.txt");
        assert_eq!(
            clean_attachment_name(1, "<img src=x onerror=alert(1)>.png"),
            "01-img_src_x_onerror_alert_1.png"
        );
        assert_eq!(clean_attachment_name(1, "a\u{0}b.txt"), "01-a_b.txt");
        assert_eq!(
            clean_attachment_name(1, "relatório.pdf"),
            "01-relat_rio.pdf"
        );
    }

    #[test]
    fn anexo_nome_sem_sobra_vira_arquivo() {
        assert_eq!(clean_attachment_name(1, ".."), "01-arquivo");
        assert_eq!(clean_attachment_name(1, "日本語"), "01-arquivo");
        assert_eq!(clean_attachment_name(1, "<>.png"), "01-arquivo.png");
    }

    #[test]
    fn anexo_nome_fica_em_ate_100_caracteres_com_a_extensao() {
        let long = clean_attachment_name(1, &format!("{}.pdf", "a".repeat(200)));
        assert_eq!(long.chars().count(), 100);
        assert!(long.ends_with(".pdf"), "{long}");

        let extension = clean_attachment_name(1, &format!("a.{}", "b".repeat(40)));
        assert_eq!(extension, format!("01-a.{}", "b".repeat(16)));
    }

    #[test]
    fn anexo_numera_sem_sobrescrever() {
        let (_dir, harness) = harness();
        let requests = Requests::open(&harness).unwrap();
        let draft = requests.draft_dir("rasc1");
        fs::create_dir_all(&draft).unwrap();

        let first = requests.next_attachment_name("rasc1", "image.png");
        assert_eq!(first, "01-image.png");
        fs::write(draft.join(&first), "a").unwrap();

        assert_eq!(
            requests.next_attachment_name("rasc1", "image.png"),
            "02-image.png"
        );
    }

    #[test]
    fn anexo_nomes_recusados_sem_diferenciar_maiusculas() {
        for name in REFUSED_ATTACHMENT_NAMES {
            let sample = name.replace('*', "x");
            for variant in [sample.to_lowercase(), sample.to_uppercase()] {
                assert!(is_refused_attachment_name(&variant), "{variant}");
            }
        }
        for name in [
            ".env",
            ".ENV.local",
            "id_rsa",
            "chave.PEM",
            "credentials.json",
            "dir/.env",
            ".env ",
            ".env.",
        ] {
            assert!(is_refused_attachment_name(name), "{name}");
        }
        for name in [
            ".env.example",
            ".ENV.EXAMPLE",
            "environment.txt",
            "chaves.txt",
        ] {
            assert!(!is_refused_attachment_name(name), "{name}");
        }
    }

    #[test]
    fn anexo_limites_recusam_decimo_primeiro_vazio_e_soma() {
        let (_dir, harness) = harness();
        let requests = Requests::open(&harness).unwrap();
        let draft = requests.draft_dir("rasc1");
        fs::create_dir_all(&draft).unwrap();

        assert_eq!(requests.check_attachment("rasc1", "a.png", 5, 100), Ok(()));
        assert_eq!(
            requests.check_attachment("rasc1", ".env", 5, 100),
            Err(AttachmentRefusal::SecretName)
        );
        assert_eq!(
            requests.check_attachment("rasc1", "a.png", 0, 100),
            Err(AttachmentRefusal::Empty)
        );

        fs::write(draft.join("01-a.bin"), vec![0u8; 60]).unwrap();
        assert_eq!(requests.check_attachment("rasc1", "b.png", 40, 100), Ok(()));
        assert_eq!(
            requests.check_attachment("rasc1", "b.png", 41, 100),
            Err(AttachmentRefusal::OverLimit)
        );

        for number in 2..=10 {
            fs::write(draft.join(format!("{number:02}-a.bin")), "x").unwrap();
        }
        assert_eq!(
            requests.check_attachment("rasc1", "b.png", 1, 100),
            Err(AttachmentRefusal::TooMany)
        );
        assert!(AttachmentRefusal::SecretName
            .message()
            .contains("na própria pergunta"));
    }

    #[test]
    fn anexo_rascunho_com_id_invalido() {
        for id in ["../x", "a/b", "", ".", &"a".repeat(65)] {
            assert!(!is_draft_id(id), "{id}");
        }
        assert!(is_draft_id("a1-b2"));
        assert!(is_draft_id(&"a".repeat(64)));
    }

    #[test]
    fn anexo_rascunho_vai_para_a_pasta_antes_do_pedido() {
        let (_dir, harness) = harness();
        let requests = Requests::open(&harness).unwrap();
        let draft = requests.draft_dir("rasc1");
        fs::create_dir_all(&draft).unwrap();
        fs::write(draft.join("01-a.txt"), "x").unwrap();

        let (id, path) = requests
            .create_from_draft("rasc1", |id| {
                let folder = requests.request_attachments_dir(id);
                assert!(folder.join("01-a.txt").is_file());
                assert!(!requests.draft_dir("rasc1").exists());
                let pending = requests.folder_of(PENDING).unwrap();
                let empty = fs::read_dir(pending)
                    .unwrap()
                    .flatten()
                    .all(|entry| entry.metadata().unwrap().len() == 0);
                assert!(empty);
                format!("pedido {id}\n")
            })
            .unwrap();

        assert_eq!(id, "PED-001");
        assert_eq!(fs::read_to_string(path).unwrap(), "pedido PED-001\n");
    }

    #[test]
    fn anexo_rascunho_salta_o_numero_com_pasta_que_ja_existe() {
        let (_dir, harness) = harness();
        let requests = Requests::open(&harness).unwrap();
        fs::create_dir_all(requests.request_attachments_dir("PED-001")).unwrap();
        let draft = requests.draft_dir("rasc1");
        fs::create_dir_all(&draft).unwrap();
        fs::write(draft.join("01-a.txt"), "x").unwrap();

        let (id, _) = requests
            .create_from_draft("rasc1", |_| String::new())
            .unwrap();

        assert_eq!(id, "PED-002");
        assert!(requests
            .request_attachments_dir("PED-002")
            .join("01-a.txt")
            .is_file());
        assert!(requests.request_attachments_dir("PED-001").is_dir());
    }
}
