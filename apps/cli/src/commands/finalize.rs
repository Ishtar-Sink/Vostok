//! Comandos `finalize`: finalização de specs implementadas. Reproduzem `finalize-spec.sh` e
//! `scripts/lib/finalize.sh`: fontes da síntese, conteúdo, validação, documentos HTML e arquivamento.

use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use clap::Subcommand;

use crate::commands::task::{accepted_diff_name, accepted_regressions};
use crate::harness::Harness;
use crate::support::{
    config_get, config_list, frontmatter_get, frontmatter_set, markdown_section,
    markdown_subsection, render, template_leftovers,
};

#[derive(Debug, Subcommand)]
pub enum FinalizeCommand {
    /// Imprime as fontes da síntese.
    Sources { spec_id: String },
    /// Cria o conteúdo dos documentos.
    Create { spec_id: String },
    /// Valida o conteúdo.
    Validate { spec_id: String },
    /// Gera os documentos HTML.
    Build { spec_id: String },
    /// Arquiva a spec e publica os documentos.
    Archive { spec_id: String },
}

const CONTENT_FILE: &str = "content.md";
const EVIDENCE_HOW: &str = "Como chegar";
const EVIDENCE_READ: &str = "Como ler";
const EVIDENCE_PROOF: &str = "Prova";
/// Seção que marca o template de conteúdo do estudo de conteúdo (SPEC-025); sem ela, vale o modelo anterior.
const COVERAGE_SECTION: &str = "Mapa de cobertura";
const NOT_APPLICABLE: &str = "Não se aplica.";
/// Máximo de palavras da frase "Resultado" do executivo.
const MAX_RESULT_WORDS: usize = 30;
const MAX_NUMBERS: usize = 3;
const MAX_LIST_ITEM_CHARS: usize = 300;
const EXECUTIVE_WORDS_WARNING: usize = 1000;
const EXECUTIVE_TEMPLATE: &str = "finalize/executive.html";
const TECHNICAL_TEMPLATE: &str = "finalize/technical.html";
const EVIDENCE_TEMPLATE: &str = "finalize/evidence.html";
const EVIDENCE_ITEM_TEMPLATE: &str = "finalize/evidence-item.html";
const ASSETS_DIR: &str = "assets";
const DIFFS_MARKER: &str = "@@FINALIZE_DIFFS@@";
const MEDIA_MARKER: &str = "@@FINALIZE_MEDIA@@";
const ITEMS_MARKER: &str = "@@FINALIZE_ITEMS@@";
const NO_DATE: &str = "data não registrada";
/// Nível do `<h>` de um `#` no conteúdo: os títulos seguem o Markdown.
const SECTION_HEADING: usize = 1;
/// Nível do `<h>` de um `#` no técnico de uma tarefa, abaixo do `<h3>` com o título dela: `##` vira `<h4>`.
const TASK_HEADING: usize = 3;
/// Fontes embutidas nos documentos, em `templates/finalize/fonts/`: família e arquivo.
const FONTS: [(&str, &str); 2] = [("Inter", "Inter.woff2"), ("Outfit", "Outfit.woff2")];

pub fn run(command: FinalizeCommand) -> Result<()> {
    let result = dispatch(command);
    if let Some(Reported(stderr)) = result.as_ref().err().and_then(|e| e.downcast_ref()) {
        eprint!("{stderr}");
        std::process::exit(1);
    }
    result
}

fn dispatch(command: FinalizeCommand) -> Result<()> {
    let harness = Harness::from_current_dir()?;
    match command {
        FinalizeCommand::Sources { spec_id } => Finalization::open(harness, &spec_id)?.sources(),
        FinalizeCommand::Create { spec_id } => Finalization::open(harness, &spec_id)?.create(),
        FinalizeCommand::Validate { spec_id } => Finalization::open(harness, &spec_id)?.validate(),
        FinalizeCommand::Build { spec_id } => Finalization::open(harness, &spec_id)?.build(),
        FinalizeCommand::Archive { spec_id } => Finalization::open(harness, &spec_id)?.archive(),
    }
}

/// Falha cuja saída de erro já está completa: sai sem o prefixo `Erro:` do `main`, como os scripts.
#[derive(Debug)]
struct Reported(String);

impl fmt::Display for Reported {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0.trim_end())
    }
}

impl std::error::Error for Reported {}

/// Lista de problemas no formato dos scripts: um título e um item `  - ` por problema.
fn problem_list(title: &str, problems: &[String]) -> String {
    let items: Vec<String> = problems.iter().map(|p| format!("  - {p}")).collect();
    format!("{title}\n{}", items.join("\n"))
}

// --- Spec e tarefas ---

struct Task {
    path: PathBuf,
    content: String,
    /// Arquivo de estado da tarefa em pasta; `None` no arquivo único.
    state: Option<PathBuf>,
    executive: String,
    technical: String,
}

impl Task {
    fn field(&self, key: &str) -> Option<String> {
        match &self.state {
            Some(state) => state_field(state, key),
            None => frontmatter_get(&self.path, key).ok(),
        }
    }

    fn is_folder(&self) -> bool {
        self.state.is_some()
    }

    fn id(&self) -> String {
        self.field("id").unwrap_or_default()
    }

    fn title(&self) -> String {
        self.field("title").unwrap_or_default()
    }

    fn status(&self) -> String {
        self.field("status").unwrap_or_default()
    }

    fn stem(&self) -> String {
        file_stem(&self.path)
    }
}

/// Nomes dos arquivos e dos títulos da tarefa em pasta, lidos de `config/tasks.yml`.
struct TaskFiles {
    state: String,
    execution: String,
    verification: String,
    criteria: String,
    executive: String,
    technical: String,
    record: String,
    evidence_dir: String,
    execution_section: String,
    description_section: String,
    executive_heading: String,
    technical_heading: String,
}

impl TaskFiles {
    fn read(config: &Path) -> Self {
        let get = |key: &str, default: &str| {
            config_get(config, key)
                .ok()
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| default.to_string())
        };
        Self {
            state: get("task_files.state", "estado.yml"),
            execution: get("task_files.execution", "execucao.md"),
            verification: get("task_files.verification", "verificacao.md"),
            criteria: get("task_files.criteria", "criterios.md"),
            executive: get("task_files.executive", "executivo.md"),
            technical: get("task_files.technical", "tecnico.md"),
            record: get("task_files.record", "registro.md"),
            evidence_dir: get("task_evidence_dir", "evidencias"),
            execution_section: get("execution_section", "Execução"),
            description_section: get("description_section", "Descrição"),
            executive_heading: get("executive_summary_subsection", "Resumo executivo"),
            technical_heading: get("technical_details_subsection", "Detalhes técnicos"),
        }
    }

    /// Tarefa em pasta: tem o arquivo de estado.
    fn is_folder(&self, path: &Path) -> bool {
        path.join(&self.state).is_file()
    }
}

/// Tarefas de uma spec, em ordem alfabética: arquivos `TASK-NN.md` ou pastas `TASK-NN/`.
fn load_tasks(dir: &Path, files: &TaskFiles) -> Result<Vec<Task>> {
    let mut folders: Vec<PathBuf> = fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| files.is_folder(path))
                .collect()
        })
        .unwrap_or_default();
    let documents = md_files(dir);
    if !documents.is_empty() && !folders.is_empty() {
        let spec = file_name(dir);
        bail!("a spec {spec} tem tarefas nos dois formatos; execute: vostok task migrate {spec}");
    }
    folders.sort();
    if folders.is_empty() {
        return Ok(documents.into_iter().map(load_task_file).collect());
    }
    Ok(folders
        .into_iter()
        .map(|folder| load_task_folder(folder, files))
        .collect())
}

fn load_task_file(path: PathBuf) -> Task {
    let content = fs::read_to_string(&path).unwrap_or_default();
    Task {
        path,
        content,
        state: None,
        executive: String::new(),
        technical: String::new(),
    }
}

/// Monta o texto da tarefa em pasta com os mesmos títulos do arquivo único, para as leituras serem
/// iguais nos dois formatos.
fn load_task_folder(folder: PathBuf, files: &TaskFiles) -> Task {
    let part = |name: &str| {
        let text = fs::read_to_string(folder.join(name)).unwrap_or_default();
        text.trim_matches('\n').to_string()
    };
    let state_path = folder.join(&files.state);
    let executive = part(&files.executive);
    let technical = part(&files.technical);
    let title = state_field(&state_path, "title").unwrap_or_default();
    let content = format!(
        "---\n{}\n---\n\n# {}: {title}\n\n## {}\n\n{}\n\n{}\n\n{}\n\n## {}\n\n### {}\n\n{executive}\n\n### {}\n\n{technical}\n\n{}\n",
        part(&files.state),
        file_name(&folder),
        files.execution_section,
        part(&files.execution),
        part(&files.verification),
        part(&files.criteria),
        files.description_section,
        files.executive_heading,
        files.technical_heading,
        part(&files.record),
    );
    Task {
        path: folder,
        content,
        state: Some(state_path),
        executive,
        technical,
    }
}

/// Valor de uma linha `chave: valor` do estado, sem aspas, como no front-matter.
fn state_field(state: &Path, key: &str) -> Option<String> {
    let prefix = format!("{key}:");
    let text = fs::read_to_string(state).ok()?;
    text.lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .map(|rest| {
            let value = rest.trim();
            if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
                return value[1..value.len() - 1]
                    .replace("\\\"", "\"")
                    .replace("\\\\", "\\");
            }
            value.trim_matches('\'').to_string()
        })
}

/// Uma spec que pode ser finalizada, com as tarefas e os caminhos da configuração.
struct Finalization {
    harness: Harness,
    root: PathBuf,
    id: String,
    spec: PathBuf,
    tasks_dir: PathBuf,
    tasks: Vec<Task>,
}

impl Finalization {
    /// Confere o ID, acha a spec refinada e recusa se ela não puder ser finalizada.
    fn open(harness: Harness, id: &str) -> Result<Self> {
        let prefix = config_get(&harness.config_file("specs.yml"), "id.prefix")?;
        if !is_spec_id(id, &prefix) {
            bail!("ID de spec inválido: '{id}'");
        }
        let refined = harness.path("specs")?.join(config_get(
            &harness.config_file("specs.yml"),
            "refined_dir",
        )?);
        let Some(spec) = md_files(&refined)
            .into_iter()
            .find(|file| frontmatter_get(file, "id").is_ok_and(|value| value == id))
        else {
            bail!("spec {id} não encontrada entre as refinadas");
        };
        let tasks_dir = harness.path("tasks")?.join(id);
        let files = TaskFiles::read(&harness.config_file("tasks.yml"));
        let tasks = load_tasks(&tasks_dir, &files)?;
        let root = harness.path("root")?;
        let root = crate::support::lexical(&root);
        let finalization = Self {
            harness,
            root,
            id: id.to_string(),
            spec,
            tasks_dir,
            tasks,
        };
        let problems = finalization.check()?;
        if !problems.is_empty() {
            bail!(problem_list(
                &format!("a spec {id} não pode ser finalizada:"),
                &problems
            ));
        }
        Ok(finalization)
    }

    fn check(&self) -> Result<Vec<String>> {
        let implemented = self.spec_setting("status.implemented")?;
        let done = self.task_setting("status.done")?;
        let mut problems = Vec::new();
        let status = frontmatter_get(&self.spec, "status").unwrap_or_default();
        if status != implemented {
            problems.push(format!(
                "a spec está com status '{status}'; só specs com status '{implemented}' são finalizadas"
            ));
        }
        for task in &self.tasks {
            let status = task.status();
            if status != done {
                let id = task.field("id").unwrap_or_else(|| task.stem());
                problems.push(format!("{id}: status '{status}', não '{done}'"));
            }
        }
        if self.tasks.is_empty() {
            problems.push(format!(
                "nenhuma tarefa encontrada em {}",
                self.tasks_dir.display()
            ));
        }
        Ok(problems)
    }

    fn spec_setting(&self, key: &str) -> Result<String> {
        config_get(&self.harness.config_file("specs.yml"), key)
    }

    fn task_setting(&self, key: &str) -> Result<String> {
        config_get(&self.harness.config_file("tasks.yml"), key)
    }

    fn documents_config(&self) -> PathBuf {
        self.harness.config_file("documents.yml")
    }

    /// Caminho relativo à raiz do projeto, quando está dentro dela.
    fn rel(&self, path: &Path) -> String {
        let path = path.display().to_string();
        let root = format!("{}/", self.root.display());
        path.strip_prefix(&root).unwrap_or(&path).to_string()
    }

    fn content_dir(&self) -> Result<PathBuf> {
        Ok(self.harness.path("finalize")?.join(&self.id))
    }

    fn content_path(&self) -> Result<PathBuf> {
        Ok(self.content_dir()?.join(CONTENT_FILE))
    }

    fn templates_dir(&self) -> Result<PathBuf> {
        self.harness.path("templates")
    }

    fn content_template(&self) -> Result<PathBuf> {
        Ok(self
            .templates_dir()?
            .join(self.spec_setting("finalize_content_template")?))
    }

    fn document_path(&self, kind: &str) -> Result<PathBuf> {
        Ok(self
            .content_dir()?
            .join(self.spec_setting(&format!("finalize_documents.{kind}"))?))
    }

    fn archive_dir(&self) -> Result<PathBuf> {
        Ok(self.harness.path("archive")?.join(&self.id))
    }

    fn spec_docs_dir(&self) -> Result<PathBuf> {
        let base = self.harness.path("spec_docs")?;
        let base = if base.is_dir() {
            crate::support::lexical(&base)
        } else {
            base
        };
        Ok(base.join(&self.id))
    }

    fn approved_at(&self, spec: &Path) -> String {
        let Ok(field) = self.spec_setting("approved_at_field") else {
            return String::new();
        };
        match frontmatter_get(spec, &field) {
            Ok(value) if value != "null" => value,
            _ => String::new(),
        }
    }

    fn task_evidence_dir(&self, task: &Task) -> Result<PathBuf> {
        if task.is_folder() {
            let files = TaskFiles::read(&self.harness.config_file("tasks.yml"));
            return Ok(task.path.join(files.evidence_dir));
        }
        let parent = task.path.parent().unwrap_or(Path::new(""));
        Ok(parent
            .join(self.task_setting("evidence_dir")?)
            .join(task.stem()))
    }

    fn task_subsection(&self, task: &Task, key: &str) -> Result<String> {
        let section = self.task_setting("execution_section")?;
        let subsection = self.task_setting(key)?;
        Ok(markdown_subsection(&task.content, &section, &subsection).unwrap_or_default())
    }

    /// Evidências listadas em "Evidências" da tarefa: nome e tipo (automática ou manual).
    fn evidence_items(&self, task: &Task) -> Result<Vec<(String, String)>> {
        Ok(self
            .task_subsection(task, "evidence_subsection")?
            .lines()
            .filter_map(evidence_item)
            .collect())
    }

    /// Subitens de cada evidência de "Evidências" da tarefa, na ordem de `evidence_items`.
    fn evidence_notes(&self, task: &Task) -> Result<Vec<EvidenceNotes>> {
        let mut notes: Vec<EvidenceNotes> = Vec::new();
        for line in self
            .task_subsection(task, "evidence_subsection")?
            .lines()
            .map(strip_cr)
        {
            if evidence_item(line).is_some() {
                notes.push(EvidenceNotes::default());
            } else if let Some(note) = notes.last_mut() {
                note.fill(line);
            }
        }
        Ok(notes)
    }

    /// Itens de "Mudar" de todas as tarefas, sem repetição: ação e caminho.
    fn changed_items(&self) -> Result<Vec<(String, String)>> {
        let mut items: Vec<(String, String)> = Vec::new();
        for task in &self.tasks {
            for item in self
                .task_subsection(task, "change_subsection")?
                .lines()
                .filter_map(change_item)
            {
                if !items.contains(&item) {
                    items.push(item);
                }
            }
        }
        Ok(items)
    }

    /// Data da última linha "conclusão" do registro de execução, vazia se não houver.
    fn task_done_date(&self, task: &Task) -> Result<String> {
        let record = self.task_setting("record_section")?;
        let mut date = String::new();
        for line in markdown_section(&task.content, &record)
            .unwrap_or_default()
            .lines()
        {
            if !is_numbered_row(line) {
                continue;
            }
            let cells: Vec<&str> = line.split('|').collect();
            if awk_trim(cells.get(3).copied().unwrap_or("")) == "conclusão" {
                date = awk_trim(cells.get(2).copied().unwrap_or("")).to_string();
            }
        }
        Ok(date)
    }

    fn last_done_date(&self) -> Result<String> {
        let mut last = String::new();
        for task in &self.tasks {
            let date = self.task_done_date(task)?;
            if date > last {
                last = date;
            }
        }
        Ok(last)
    }

    fn git(&self, args: &[&str]) -> Option<String> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(args)
            .output()
            .ok()?;
        output
            .status
            .success()
            .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
    }

    fn inside_git(&self) -> bool {
        self.git(&["rev-parse", "--is-inside-work-tree"]).is_some()
    }
}

fn is_spec_id(id: &str, prefix: &str) -> bool {
    id.strip_prefix(prefix)
        .and_then(|rest| rest.strip_prefix('-'))
        .is_some_and(|number| !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit()))
}

/// Arquivos `*.md` visíveis de uma pasta, em ordem alfabética.
fn md_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            name.ends_with(".md") && !name.starts_with('.') && path.is_file()
        })
        .collect();
    files.sort();
    files
}

fn file_stem(path: &Path) -> String {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Item "- `nome` (tipo)" da subseção "Evidências".
fn evidence_item(line: &str) -> Option<(String, String)> {
    let (name, rest) = backticked_item(line)?;
    let kind = rest.strip_prefix(" (")?;
    let end = kind.find(')')?;
    (end > 0).then(|| (name.to_string(), kind[..end].to_string()))
}

/// Subitens que o documento de evidências herda da tarefa.
#[derive(Default)]
struct EvidenceNotes {
    title: String,
    shows: String,
    proof: String,
    where_to_look: String,
    snippet: String,
    failure: String,
}

impl EvidenceNotes {
    /// Lê um subitem recuado "  - Campo: texto". `support::evidence_subitem` não serve: recusa nome de
    /// campo com espaço, como "Onde olhar".
    fn fill(&mut self, line: &str) {
        if !line.starts_with([' ', '\t']) {
            return;
        }
        let Some((name, value)) = line
            .trim_start()
            .strip_prefix("- ")
            .and_then(|item| item.split_once(": "))
        else {
            return;
        };
        let field = match name {
            "Título" => &mut self.title,
            "Mostra" => &mut self.shows,
            "Prova" => &mut self.proof,
            "Onde olhar" => &mut self.where_to_look,
            "Trecho" => &mut self.snippet,
            "Falha" => &mut self.failure,
            _ => return,
        };
        *field = value.trim().to_string();
    }
}

/// Comando da primeira linha do log de uma evidência (`$ comando`); `None` sem log legível nesse formato.
fn log_command(path: &Path) -> Option<String> {
    let text = fs::read_to_string(path).ok()?;
    let command = text.lines().next()?.strip_prefix("$ ")?.trim();
    (!command.is_empty()).then(|| command.to_string())
}

/// Item "- `caminho` (ação)" da subseção "Mudar", como ação e caminho.
fn change_item(line: &str) -> Option<(String, String)> {
    let (path, rest) = backticked_item(line)?;
    let action = rest.strip_prefix(" (")?;
    let end = action.find(')')?;
    let action = &action[..end];
    let valid = !action.is_empty() && action.bytes().all(|b| b.is_ascii_lowercase());
    valid.then(|| (action.to_string(), path.to_string()))
}

fn backticked_item(line: &str) -> Option<(&str, &str)> {
    let rest = line.strip_prefix("- `")?;
    let end = rest.find('`')?;
    (end > 0).then(|| (&rest[..end], &rest[end + 1..]))
}

/// Linha de tabela que começa com um número: `| 3 | ...`.
fn is_numbered_row(line: &str) -> bool {
    let Some(rest) = line.strip_prefix('|') else {
        return false;
    };
    let rest = rest.trim_start_matches(' ');
    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
    digits > 0 && rest[digits..].trim_start_matches(' ').starts_with('|')
}

fn awk_trim(text: &str) -> &str {
    text.trim_matches([' ', '\t'])
}

fn strip_cr(line: &str) -> &str {
    line.strip_suffix('\r').unwrap_or(line)
}

/// Texto como sai de uma substituição de comando do shell: sem as quebras de linha do fim.
fn captured(text: &str) -> String {
    text.trim_end_matches('\n').to_string()
}

/// Campos de uma linha lida por `IFS=$'\t' read -r` com `count` variáveis: tabulações nas pontas somem,
/// tabulações seguidas contam como um separador e a última variável fica com o resto.
fn read_fields(line: &str, count: usize) -> Vec<String> {
    let mut rest = line.trim_matches('\t');
    let mut fields = Vec::new();
    while fields.len() + 1 < count {
        let (field, tail) = rest.split_once('\t').unwrap_or((rest, ""));
        fields.push(field.to_string());
        rest = tail.trim_start_matches('\t');
    }
    fields.push(rest.to_string());
    fields
}

/// Linha que é só um comentário HTML: `<!-- ... -->`.
fn is_comment_line(line: &str) -> bool {
    let line = line.trim_end_matches([' ', '\t']);
    line.len() >= 7 && line.starts_with("<!--") && line.ends_with("-->")
}

// --- Fontes ---

impl Finalization {
    fn sources(&self) -> Result<()> {
        let mut out = String::new();
        self.sources_spec(&mut out)?;
        if self.study_model() {
            out.push_str("\n## Métricas\n\n");
            for line in metrics_lines(&self.metrics()?) {
                push_line(&mut out, &line);
            }
        }
        out.push('\n');
        self.sources_tasks(&mut out)?;
        out.push('\n');
        self.sources_paths(&mut out)?;
        out.push('\n');
        self.sources_memory(&mut out)?;
        out.push('\n');
        out.push_str("## Commits\n\n");
        for line in self.commit_lines()? {
            push_line(&mut out, &line);
        }
        print!("{out}");
        Ok(())
    }

    fn sources_spec(&self, out: &mut String) -> Result<()> {
        let approved = self.approved_at(&self.spec);
        let field = |key| frontmatter_get(&self.spec, key).unwrap_or_default();
        push_line(out, "## Spec");
        push_line(out, "");
        push_line(out, &format!("- **ID:** {}", field("id")));
        push_line(out, &format!("- **Título:** {}", field("title")));
        push_line(out, &format!("- **Arquivo:** `{}`", self.rel(&self.spec)));
        let approved = if approved.is_empty() {
            "sem data de aprovação"
        } else {
            &approved
        };
        push_line(out, &format!("- **Aprovada em:** {approved}"));
        Ok(())
    }

    fn sources_tasks(&self, out: &mut String) -> Result<()> {
        let record = self.task_setting("record_section")?;
        let history = self.adjustment_history_heading()?;
        push_line(out, "## Tarefas");
        for task in &self.tasks {
            push_line(out, "");
            push_line(out, &format!("### {}: {}", task.id(), task.title()));
            push_line(out, "");
            push_line(out, &format!("- **Status:** {}", task.status()));
            if task.is_folder() {
                self.sources_task_texts(out, task);
            }
            for heading in [&record, &history] {
                push_line(out, "");
                push_line(out, &format!("**{heading}**"));
                push_line(out, "");
                let section = markdown_section(&task.content, heading).unwrap_or_default();
                for row in section.lines().filter(|line| line.starts_with('|')) {
                    push_line(out, row);
                }
            }
            push_line(out, "");
            push_line(out, "**Evidências**");
            push_line(out, "");
            let evidence_dir = self.task_evidence_dir(task)?;
            let items = self.evidence_items(task)?;
            for (name, kind) in &items {
                let path = evidence_dir.join(name);
                let state = if path.is_file() { "existe" } else { "ausente" };
                push_line(out, &format!("- `{}` ({kind}): {state}", self.rel(&path)));
            }
            if items.is_empty() {
                push_line(out, "Não se aplica.");
            }
        }
        Ok(())
    }

    /// Executivo e técnico da tarefa em pasta, com o conteúdo dos arquivos.
    fn sources_task_texts(&self, out: &mut String, task: &Task) {
        let files = TaskFiles::read(&self.harness.config_file("tasks.yml"));
        for (heading, text) in [
            (&files.executive_heading, &task.executive),
            (&files.technical_heading, &task.technical),
        ] {
            push_line(out, "");
            push_line(out, &format!("**{heading}**"));
            push_line(out, "");
            push_line(
                out,
                if text.is_empty() {
                    "Não se aplica."
                } else {
                    text
                },
            );
        }
    }

    fn adjustment_history_heading(&self) -> Result<String> {
        let template = self
            .templates_dir()?
            .join(self.spec_setting("sections_dir")?)
            .join(format!(
                "{}.md",
                self.spec_setting("adjustment_history_section")?
            ));
        let content = fs::read_to_string(&template).unwrap_or_default();
        let first = content.lines().next().unwrap_or("");
        Ok(first.strip_prefix("## ").unwrap_or(first).to_string())
    }

    fn sources_paths(&self, out: &mut String) -> Result<()> {
        push_line(out, "## Caminhos alterados");
        push_line(out, "");
        for (action, path) in self.changed_items()? {
            push_line(out, &format!("- `{path}` ({action})"));
        }
        Ok(())
    }

    fn sources_memory(&self, out: &mut String) -> Result<()> {
        push_line(out, "## Memória do período");
        push_line(out, "");
        let approved = self.approved_at(&self.spec);
        if approved.is_empty() {
            push_line(out, "Spec sem data de aprovação.");
            return Ok(());
        }
        let start = approved.split('T').next().unwrap_or("");
        let today = utc_date();
        let memory_config = self.harness.config_file("memory.yml");
        let memory_dir = self.harness.path("agent_memory")?;
        let mut found = false;
        for kind in config_list(&memory_config, "types")? {
            let file = memory_dir.join(config_get(&memory_config, &format!("{kind}.file"))?);
            let Ok(content) = fs::read_to_string(&file) else {
                continue;
            };
            for line in memory_entries(&content, start, &today) {
                push_line(out, &line);
                found = true;
            }
        }
        if !found {
            push_line(out, "Nenhuma entrada de memória no período.");
        }
        Ok(())
    }

    /// Linhas do bloco "## Commits": commits desde a aprovação da spec, até a da spec seguinte, que
    /// alteram os caminhos das tarefas, ou o aviso de por que não há.
    fn commit_lines(&self) -> Result<Vec<String>> {
        if !self.inside_git() {
            return Ok(vec!["Fora de um repositório git.".to_string()]);
        }
        let approved = self.approved_at(&self.spec);
        if approved.is_empty() {
            return Ok(vec!["Spec sem data de aprovação.".to_string()]);
        }
        let mut paths: Vec<String> = Vec::new();
        for (_, path) in self.changed_items()? {
            if !paths.contains(&path) {
                paths.push(path);
            }
        }
        let none = vec!["Nenhum commit encontrado.".to_string()];
        if paths.is_empty() {
            return Ok(none);
        }
        let since = format!("--since={approved}");
        let until = self
            .next_approved_at(&approved)?
            .map(|v| format!("--until={v}"));
        let mut args = vec!["log", since.as_str()];
        args.extend(until.as_deref());
        args.extend(["--format=- `%h` %ad %s", "--date=short", "--"]);
        args.extend(paths.iter().map(String::as_str));
        let log = captured(&self.git(&args).unwrap_or_default());
        if log.is_empty() {
            return Ok(none);
        }
        Ok(log.lines().map(str::to_string).collect())
    }

    /// O menor `approved_at` maior que o da spec, entre as refinadas e as arquivadas.
    fn next_approved_at(&self, approved: &str) -> Result<Option<String>> {
        let refined = self
            .harness
            .path("specs")?
            .join(self.spec_setting("refined_dir")?);
        let archive = self.harness.path("archive")?;
        let mut candidates = md_files(&refined);
        let mut archived: Vec<PathBuf> = fs::read_dir(&archive)
            .map(|entries| {
                entries
                    .flatten()
                    .filter(|entry| !entry.file_name().to_string_lossy().starts_with('.'))
                    .map(|entry| entry.path().join("spec.md"))
                    .filter(|path| path.is_file())
                    .collect()
            })
            .unwrap_or_default();
        archived.sort();
        candidates.extend(archived);
        Ok(candidates
            .iter()
            .map(|file| self.approved_at(file))
            .filter(|value| !value.is_empty() && value.as_str() > approved)
            .min())
    }
}

fn push_line(out: &mut String, line: &str) {
    out.push_str(line);
    out.push('\n');
}

/// Entradas de memória com a data entre `start` e `end`: "- `ID` título (status, data)".
fn memory_entries(content: &str, start: &str, end: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let (mut id, mut title, mut date, mut status) =
        (String::new(), String::new(), String::new(), String::new());
    let mut flush = |id: &mut String, title: &str, date: &mut String, status: &mut String| {
        if !id.is_empty() && date.as_str() >= start && date.as_str() <= end {
            lines.push(format!("- `{id}` {title} ({status}, {date})"));
        }
        id.clear();
        date.clear();
        status.clear();
    };
    for line in content.lines().map(strip_cr) {
        if let Some((entry_id, entry_title)) = memory_heading(line) {
            flush(&mut id, &title, &mut date, &mut status);
            id = entry_id.to_string();
            title = entry_title.to_string();
            continue;
        }
        if let Some(value) = line.strip_prefix("- **Data:** ") {
            date = value.to_string();
        }
        if let Some(value) = line.strip_prefix("- **Status:** ") {
            status = value.to_string();
        }
    }
    flush(&mut id, &title, &mut date, &mut status);
    lines
}

/// Cabeçalho `## <PREFIXO>-<número>: <título>` de uma entrada de memória.
fn memory_heading(line: &str) -> Option<(&str, &str)> {
    let rest = line.strip_prefix("## ")?;
    let letters = rest.bytes().take_while(u8::is_ascii_uppercase).count();
    let after = rest[letters..].strip_prefix('-')?;
    let digits = after.bytes().take_while(u8::is_ascii_digit).count();
    let title = after[digits..].strip_prefix(": ")?;
    (letters > 0 && digits > 0).then(|| (&rest[..letters + 1 + digits], title))
}

// --- Métricas ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Situation {
    Green,
    Amber,
    Red,
}

impl Situation {
    fn label(self) -> &'static str {
        match self {
            Self::Green => "verde",
            Self::Amber => "âmbar",
            Self::Red => "vermelho",
        }
    }

    fn class(self) -> &'static str {
        match self {
            Self::Green => "green",
            Self::Amber => "amber",
            Self::Red => "red",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CriterionResult {
    Met,
    Partial,
    NotMet,
    Pending,
}

struct CycleMark {
    label: &'static str,
    seconds: i64,
}

enum Volume {
    OutsideGit,
    NoApproval,
    Counted {
        commits: usize,
        files: Option<usize>,
        added: usize,
        removed: usize,
    },
}

struct SpecMetrics {
    cycle: Vec<CycleMark>,
    approval_to_implementation: Option<i64>,
    tasks: usize,
    waves: usize,
    adjustments: usize,
    extra_attempts: usize,
    blocks: usize,
    refusals: usize,
    criteria_total: usize,
    criteria: Vec<CriterionResult>,
    coverage_found: bool,
    evidence_texts: usize,
    evidence_prints: usize,
    evidence_videos: usize,
    evidence_accepted: usize,
    evidence_refused: usize,
    evidence_to_check: usize,
    volume: Volume,
    situation: Situation,
}

/// Última data de um status no histórico `<data> <status>; ...` da spec.
fn status_time(history: &str, status: &str) -> Option<i64> {
    history
        .split(';')
        .filter_map(|entry| {
            let (date, name) = entry.trim().split_once(' ')?;
            (name.trim() == status).then(|| seconds_of(date))?
        })
        .next_back()
}

/// IDs `CA-NN` dos critérios de aceite da spec: linhas `- [ ] CA-NN:` ou `- [x] CA-NN:`.
fn criterion_ids(spec: &str) -> Vec<String> {
    spec.lines()
        .filter_map(|line| {
            let rest = line
                .strip_prefix("- [ ] ")
                .or_else(|| line.strip_prefix("- [x] "))?;
            let (id, _) = rest.split_once(':')?;
            let number = id.strip_prefix("CA-")?;
            (!number.is_empty() && number.bytes().all(|b| b.is_ascii_digit()))
                .then(|| id.to_string())
        })
        .collect()
}

fn criterion_result(text: &str) -> CriterionResult {
    let text = text.trim().to_lowercase();
    if text.starts_with("não atendido") {
        CriterionResult::NotMet
    } else if text.starts_with("parcial") {
        CriterionResult::Partial
    } else if text.starts_with("atendido") {
        CriterionResult::Met
    } else {
        CriterionResult::Pending
    }
}

impl Finalization {
    /// Números da finalização, calculados das fontes sem estimar nada (RN-03).
    fn metrics(&self) -> Result<SpecMetrics> {
        let history = frontmatter_get(&self.spec, "status_history").unwrap_or_default();
        let field_time = |key: &str| {
            frontmatter_get(&self.spec, key)
                .ok()
                .and_then(|value| seconds_of(&value))
        };
        let approved = status_time(&history, "approved").or_else(|| {
            let value = self.approved_at(&self.spec);
            seconds_of(&value)
        });
        let implemented = status_time(&history, "implemented");
        let marks = [
            ("criada", field_time("created_at")),
            ("aprovada", approved),
            ("planejada", status_time(&history, "planned")),
            ("implementada", implemented),
            ("arquivada", status_time(&history, "archived")),
        ];
        let cycle = marks
            .into_iter()
            .filter_map(|(label, seconds)| {
                Some(CycleMark {
                    label,
                    seconds: seconds?,
                })
            })
            .collect();
        let approval_to_implementation = approved.zip(implemented).map(|(from, to)| to - from);

        let record = self.task_setting("record_section")?;
        let (mut adjustments, mut extra_attempts, mut blocks, mut refusals) = (0, 0, 0, 0);
        let mut waves: Vec<String> = Vec::new();
        let (mut texts, mut prints, mut videos) = (0, 0, 0);
        let (mut accepted, mut refused, mut to_check) = (0, 0, 0);
        let mut regression_accepted = false;
        let config = self.documents_config();
        for task in &self.tasks {
            if let Some(wave) = task.field("wave").filter(|wave| !wave.is_empty()) {
                if !waves.contains(&wave) {
                    waves.push(wave);
                }
            }
            extra_attempts += task
                .field("attempts")
                .map_or(0, |value| leading_number(&value)) as usize;
            let mut record_reviews: Vec<String> = Vec::new();
            for line in markdown_section(&task.content, &record)
                .unwrap_or_default()
                .lines()
                .filter(|line| is_numbered_row(line))
            {
                let cells: Vec<&str> = line.split('|').collect();
                let cell = |index: usize| awk_trim(cells.get(index).copied().unwrap_or(""));
                let (step, result, detail) = (cell(3), cell(4), cell(5));
                if step == "ajuste" {
                    adjustments += 1;
                }
                if result == "blocked" {
                    blocks += 1;
                }
                if step == "rejeição" || (step == "conferência" && detail.contains(" recusada"))
                {
                    refusals += 1;
                }
                if step == "conferência" {
                    record_reviews.push(detail.to_string());
                }
            }
            let mut states: HashMap<String, String> = HashMap::new();
            let reviews_file = task.path.join("conferencias.md");
            if !reviews_file.is_file() {
                states = legacy_review_states(&record_reviews, &self.evidence_items(task)?);
            }
            let reviews = fs::read_to_string(reviews_file).unwrap_or_default();
            for line in reviews.lines() {
                let cells: Vec<&str> = line.split('|').collect();
                if cells.len() > 3 {
                    states.insert(
                        awk_trim(cells[2]).to_string(),
                        awk_trim(cells[3]).to_string(),
                    );
                }
            }
            for (name, _) in self.evidence_items(task)? {
                match media_type(&config, &name, &mut String::new()) {
                    MediaType::Image => prints += 1,
                    MediaType::Video => videos += 1,
                    _ => texts += 1,
                }
                match states.get(&name).map(String::as_str) {
                    Some("conferida") => accepted += 1,
                    Some("recusada") => refused += 1,
                    _ => to_check += 1,
                }
            }
            let evidence_dir = self.task_evidence_dir(task)?;
            if !accepted_regressions(&evidence_dir.join("validation/regression")).is_empty() {
                regression_accepted = true;
            }
        }

        let ids = criterion_ids(&fs::read_to_string(&self.spec).unwrap_or_default());
        let content = fs::read_to_string(self.content_path()?).ok();
        let coverage = content
            .as_deref()
            .map(|content| section_of(&content_part(content, "Evidências"), COVERAGE_SECTION))
            .unwrap_or_default();
        let criteria: Vec<CriterionResult> = ids
            .iter()
            .map(|id| {
                coverage
                    .lines()
                    .find_map(|line| {
                        let cells: Vec<&str> = line.split('|').collect();
                        (awk_trim(cells.get(1).copied().unwrap_or("")) == id)
                            .then(|| criterion_result(cells.get(2).copied().unwrap_or("")))
                    })
                    .unwrap_or(CriterionResult::Pending)
            })
            .collect();

        let volume = self.volume()?;
        let situation = if criteria.contains(&CriterionResult::NotMet) {
            Situation::Red
        } else if blocks > 0
            || refusals > 0
            || regression_accepted
            || criteria
                .iter()
                .any(|c| matches!(c, CriterionResult::Partial | CriterionResult::Pending))
        {
            Situation::Amber
        } else {
            Situation::Green
        };
        Ok(SpecMetrics {
            cycle,
            approval_to_implementation,
            tasks: self.tasks.len(),
            waves: waves.len(),
            adjustments,
            extra_attempts,
            blocks,
            refusals,
            criteria_total: ids.len(),
            criteria,
            coverage_found: content.is_some(),
            evidence_texts: texts,
            evidence_prints: prints,
            evidence_videos: videos,
            evidence_accepted: accepted,
            evidence_refused: refused,
            evidence_to_check: to_check,
            volume,
            situation,
        })
    }

    /// Commits, arquivos e linhas desde a aprovação da spec.
    fn volume(&self) -> Result<Volume> {
        if !self.inside_git() {
            return Ok(Volume::OutsideGit);
        }
        if self.approved_at(&self.spec).is_empty() {
            return Ok(Volume::NoApproval);
        }
        let commits = self
            .commit_lines()?
            .iter()
            .filter(|line| commit_line(line).is_some())
            .count();
        let Ok(base) = self.diff_base() else {
            return Ok(Volume::Counted {
                commits,
                files: None,
                added: 0,
                removed: 0,
            });
        };
        let mut paths: Vec<String> = Vec::new();
        for (_, path) in self.changed_items()? {
            if !paths.contains(&path) {
                paths.push(path);
            }
        }
        let mut args = vec![
            "-c",
            "core.quotepath=off",
            "diff",
            "--numstat",
            "--no-color",
            "--no-ext-diff",
            base.as_str(),
            "--",
        ];
        args.extend(paths.iter().map(String::as_str));
        let (mut files, mut added, mut removed) = (0, 0, 0);
        for line in self.git(&args).unwrap_or_default().lines() {
            let mut cells = line.split('\t');
            let (Some(plus), Some(minus)) = (cells.next(), cells.next()) else {
                continue;
            };
            files += 1;
            added += plus.parse::<usize>().unwrap_or(0);
            removed += minus.parse::<usize>().unwrap_or(0);
        }
        Ok(Volume::Counted {
            commits,
            files: Some(files),
            added,
            removed,
        })
    }
}

/// `1 ajuste`, `2 ajustes`.
fn plural(count: usize, one: &str, many: &str) -> String {
    format!("{count} {}", if count == 1 { one } else { many })
}

/// Linhas do bloco "## Métricas", no formato do contrato.
fn metrics_lines(metrics: &SpecMetrics) -> Vec<String> {
    let mut lines = Vec::new();
    let mut marks: Vec<String> = Vec::new();
    let mut previous_day = String::new();
    for mark in metrics.cycle.iter().filter(|m| m.label != "planejada") {
        let day = date_of(mark.seconds);
        let hour = hour_of(mark.seconds);
        marks.push(if day == previous_day {
            format!("{} {hour}", mark.label)
        } else {
            format!("{} {day} {hour}", mark.label)
        });
        previous_day = day;
    }
    lines.push(if marks.is_empty() {
        "- Ciclo: sem datas registradas".to_string()
    } else {
        format!("- Ciclo: {}", marks.join(", "))
    });
    lines.push(format!(
        "- Da aprovação à implementação: {}",
        metrics
            .approval_to_implementation
            .map_or("sem registro de hora".to_string(), duration_text)
    ));
    lines.push(if metrics.waves == 0 {
        format!("- Tarefas: {}", metrics.tasks)
    } else {
        format!(
            "- Tarefas: {} em {}",
            metrics.tasks,
            plural(metrics.waves, "onda", "ondas")
        )
    });
    lines.push(format!(
        "- Retrabalho: {} em execução, {}, {}, {}",
        plural(metrics.adjustments, "ajuste", "ajustes"),
        plural(
            metrics.extra_attempts,
            "tentativa extra",
            "tentativas extras"
        ),
        plural(metrics.blocks, "bloqueio", "bloqueios"),
        plural(metrics.refusals, "recusa", "recusas"),
    ));
    lines.push(if metrics.criteria_total == 0 {
        "- Critérios de aceite: nenhum na spec".to_string()
    } else if !metrics.coverage_found {
        format!(
            "- Critérios de aceite: {} a conferir no mapa de cobertura",
            metrics.criteria_total
        )
    } else {
        let count =
            |wanted: CriterionResult| metrics.criteria.iter().filter(|c| **c == wanted).count();
        let mut text = format!(
            "- Critérios de aceite: {} de {} atendidos",
            count(CriterionResult::Met),
            metrics.criteria_total
        );
        let partial = count(CriterionResult::Partial);
        if partial > 0 {
            text.push_str(&format!(", {}", plural(partial, "parcial", "parciais")));
        }
        let not_met = count(CriterionResult::NotMet);
        if not_met > 0 {
            text.push_str(&format!(
                ", {}",
                plural(not_met, "não atendido", "não atendidos")
            ));
        }
        let pending = count(CriterionResult::Pending);
        if pending > 0 {
            text.push_str(&format!(", {pending} a conferir"));
        }
        text
    });
    let total = metrics.evidence_texts + metrics.evidence_prints + metrics.evidence_videos;
    lines.push(if total == 0 {
        "- Evidências: nenhuma".to_string()
    } else {
        let kinds: Vec<String> = [
            plural(metrics.evidence_prints, "print", "prints"),
            plural(metrics.evidence_texts, "texto", "textos"),
            plural(metrics.evidence_videos, "vídeo", "vídeos"),
        ]
        .into_iter()
        .filter(|kind| !kind.starts_with("0 "))
        .collect();
        let mut text = format!(
            "- Evidências: {total} ({}); {} conferidas, {} recusadas",
            kinds.join(", "),
            metrics.evidence_accepted,
            metrics.evidence_refused
        );
        if metrics.evidence_to_check > 0 {
            text.push_str(&format!(", {} a conferir", metrics.evidence_to_check));
        }
        text
    });
    lines.push(match &metrics.volume {
        Volume::OutsideGit => "- Volume: fora de um repositório git".to_string(),
        Volume::NoApproval => "- Volume: sem data de aprovação".to_string(),
        Volume::Counted {
            commits,
            files: Some(files),
            added,
            removed,
        } => format!(
            "- Volume: {}, {}, +{added} / -{removed} linhas",
            plural(*commits, "commit", "commits"),
            plural(*files, "arquivo", "arquivos")
        ),
        Volume::Counted {
            commits,
            files: None,
            ..
        } => format!(
            "- Volume: {}; sem base de comparação para arquivos e linhas",
            plural(*commits, "commit", "commits")
        ),
    });
    lines.push(format!("- Situação: {}", metrics.situation.label()));
    lines
}

// --- Topo do executivo ---

/// Células de uma linha de tabela Markdown, sem espaços nas pontas.
fn table_cells(line: &str) -> Vec<&str> {
    line.trim()
        .trim_start_matches('|')
        .trim_end_matches('|')
        .split('|')
        .map(str::trim)
        .collect()
}

const VALUE_INDICATOR_HEADER: [&str; 4] = ["Valor", "Antes", "Depois", "Rótulo"];

/// Linha de dados do "Indicador de valor": valor, antes, depois e rótulo.
fn value_indicator(part: &str) -> Option<[String; 4]> {
    let section = section_of(part, "Indicador de valor");
    let rows: Vec<&str> = prose_lines(&section)
        .filter(|line| line.starts_with('|') && !is_table_separator(line))
        .collect();
    let cells = table_cells(rows.get(1)?);
    (table_cells(rows.first()?) == VALUE_INDICATOR_HEADER && cells.len() == 4)
        .then(|| [cells[0], cells[1], cells[2], cells[3]].map(str::to_string))
}

/// Erros do "Resultado" e do "Indicador de valor" do executivo, só quando o conteúdo já tem o "Resultado".
fn decision_errors(part: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let words: usize = prose_lines(&section_of(part, "Resultado"))
        .map(|line| line.split_whitespace().count())
        .sum();
    if words > MAX_RESULT_WORDS {
        errors.push(format!(
            "Resultado (Executivo): {words} palavras, o máximo é {MAX_RESULT_WORDS}"
        ));
    }
    let indicator = section_of(part, "Indicador de valor");
    let lines: Vec<&str> = prose_lines(&indicator).collect();
    let valid = lines == [NOT_APPLICABLE]
        || (lines.len() == 3
            && table_cells(lines[0]) == VALUE_INDICATOR_HEADER
            && is_table_separator(lines[1])
            && lines[2].starts_with('|')
            && table_cells(lines[2]).len() == 4
            && table_cells(lines[2]).iter().all(|cell| !cell.is_empty()));
    if !lines.is_empty() && !valid {
        errors.push(
            "Indicador de valor (Executivo): use a tabela \"| Valor | Antes | Depois | Rótulo |\" com uma linha, ou \"Não se aplica.\""
                .to_string(),
        );
    }
    errors
}

/// Selo de situação: a cor pela classe e o texto por extenso.
fn badge_html(metrics: &SpecMetrics) -> String {
    let text = match metrics.situation {
        Situation::Green => "Concluída sem pendências",
        Situation::Amber => "Concluída com ressalvas",
        Situation::Red => "Critério de aceite não atendido",
    };
    format!(
        "<p class=\"badge {}\" role=\"status\"><span aria-hidden=\"true\"></span>Situação: {} · {text}</p>",
        metrics.situation.class(),
        metrics.situation.label()
    )
}

/// Primeira entrega de "O que foi entregue": o primeiro trecho em negrito, sem o ponto final.
fn first_delivery(part: &str) -> Option<String> {
    let section = section_of(part, "O que foi entregue");
    let start = section.find("**")? + 2;
    let end = start + section[start..].find("**")?;
    let name = section[start..end].trim().trim_end_matches('.').trim();
    (!name.is_empty()).then(|| name.to_string())
}

fn card_html(title: &str, value: &str, detail: &str, note: Option<String>) -> String {
    let note = note.map_or(String::new(), |note| {
        format!("<p class=\"card-note\">{}</p>", html_escape(&note))
    });
    format!(
        "<article class=\"card\"><h2 class=\"card-title\">{title}</h2><b class=\"card-value\">{}</b><p>{}</p>{note}</article>\n",
        html_escape(value),
        html_escape(detail)
    )
}

/// Os quatro cartões: resultado, tempo, qualidade e esforço.
fn cards_html(metrics: &SpecMetrics, part: &str) -> String {
    let none = "sem métrica de valor nas fontes";
    let result = match (value_indicator(part), first_delivery(part)) {
        (Some([value, before, after, label]), _) => card_html(
            "Resultado",
            &value,
            &label,
            Some(format!("antes: {before} · depois: {after}")),
        ),
        (None, Some(name)) => card_html("Resultado", &name, none, None),
        (None, None) => card_html("Resultado", "Entrega concluída", none, None),
    };
    let time = match metrics.approval_to_implementation {
        Some(seconds) => card_html(
            "Tempo",
            &duration_text(seconds),
            "da aprovação à implementação",
            None,
        ),
        None => card_html("Tempo", "—", "sem registro de hora", None),
    };
    let met = metrics
        .criteria
        .iter()
        .filter(|c| **c == CriterionResult::Met)
        .count();
    let checked = metrics.evidence_accepted;
    let total = checked + metrics.evidence_refused + metrics.evidence_to_check;
    let quality = card_html(
        "Qualidade",
        &if metrics.criteria_total == 0 || !metrics.coverage_found {
            "a conferir".to_string()
        } else {
            format!("{met} de {}", metrics.criteria_total)
        },
        &format!(
            "{checked} de {total} evidências conferidas, {}",
            plural(metrics.evidence_refused, "recusa", "recusas")
        ),
        None,
    );
    let mut effort = Vec::new();
    if metrics.waves > 0 {
        effort.push(plural(metrics.waves, "onda", "ondas"));
    }
    effort.push(plural(metrics.adjustments, "ajuste", "ajustes"));
    effort.push(plural(
        metrics.extra_attempts,
        "tentativa extra",
        "tentativas extras",
    ));
    effort.push(plural(metrics.blocks, "bloqueio", "bloqueios"));
    let effort = card_html(
        "Esforço",
        &plural(metrics.tasks, "tarefa", "tarefas"),
        &effort.join(", "),
        None,
    );
    format!("{result}{time}{quality}{effort}")
}

/// Marcas do ciclo para a barra: a data só aparece quando o dia muda em relação à marca anterior.
fn cycle_mark_texts(metrics: &SpecMetrics) -> Vec<String> {
    let mut previous = metrics
        .cycle
        .first()
        .map(|mark| date_of(mark.seconds))
        .unwrap_or_default();
    metrics
        .cycle
        .iter()
        .map(|mark| {
            let day = date_of(mark.seconds);
            let text = if day == previous {
                format!("{} {}", mark.label, hour_of(mark.seconds))
            } else {
                format!("{} {day} {}", mark.label, hour_of(mark.seconds))
            };
            previous = day;
            text
        })
        .collect()
}

fn marks_html(list: &str, items: &[String]) -> String {
    let items: String = items
        .iter()
        .map(|item| format!("<li>{}</li>", html_escape(item)))
        .collect();
    format!("<{list} class=\"marks\">{items}</{list}>")
}

/// Barra do ciclo em SVG embutido, com um trecho por intervalo entre marcas.
fn cycle_bar_html(metrics: &SpecMetrics) -> String {
    let caption = metrics
        .approval_to_implementation
        .map_or("Ciclo da entrega".to_string(), |seconds| {
            format!("{} da aprovação à implementação", duration_text(seconds))
        });
    let caption = html_escape(&caption);
    let marks = &metrics.cycle;
    if marks.len() < 2 {
        return format!(
            "<figure class=\"bar\" id=\"barra-ciclo\"><figcaption>{caption}</figcaption><p class=\"na\">Sem horas suficientes para desenhar o ciclo.</p></figure>"
        );
    }
    let texts = cycle_mark_texts(metrics);
    let start = marks[0].seconds;
    let span = (marks[marks.len() - 1].seconds - start).max(1) as f64;
    let rects: String = marks
        .windows(2)
        .map(|pair| {
            let width = (((pair[1].seconds - pair[0].seconds) as f64 / span) * 1000.0)
                .round()
                .max(10.0);
            let x = (((pair[0].seconds - start) as f64 / span) * 1000.0)
                .round()
                .min(1000.0 - width);
            let class = if pair[1].label == "implementada" {
                "seg exec"
            } else {
                "seg"
            };
            format!("<rect class=\"{class}\" x=\"{x}\" y=\"0\" width=\"{width}\" height=\"14\"/>")
        })
        .collect();
    format!(
        "<figure class=\"bar\" id=\"barra-ciclo\"><figcaption>{caption}</figcaption><svg role=\"img\" aria-label=\"{}\" viewBox=\"0 0 1000 14\" preserveAspectRatio=\"none\">{rects}</svg>{}</figure>",
        html_escape(&format!("Ciclo: {}", texts.join(", "))),
        marks_html("ol", &texts)
    )
}

/// Barra das evidências por estado: conferidas, recusadas e a conferir.
fn evidence_bar_html(metrics: &SpecMetrics) -> String {
    let counts = [
        ("seg ok", metrics.evidence_accepted),
        ("seg bad", metrics.evidence_refused),
        ("seg todo", metrics.evidence_to_check),
    ];
    let total: usize = counts.iter().map(|(_, count)| count).sum();
    if total == 0 {
        return "<figure class=\"bar\" id=\"barra-evidencias\"><p class=\"na\">Nenhuma evidência.</p></figure>".to_string();
    }
    let last = counts
        .iter()
        .rposition(|(_, count)| *count > 0)
        .unwrap_or(0);
    let (mut x, mut rects) = (0, String::new());
    for (index, (class, count)) in counts.iter().enumerate() {
        if *count == 0 {
            continue;
        }
        let width = if index == last {
            1000 - x
        } else {
            count * 1000 / total
        };
        rects.push_str(&format!(
            "<rect class=\"{class}\" x=\"{x}\" y=\"0\" width=\"{width}\" height=\"14\"/>"
        ));
        x += width;
    }
    let (accepted, refused, to_check) = (
        metrics.evidence_accepted,
        metrics.evidence_refused,
        metrics.evidence_to_check,
    );
    format!(
        "<figure class=\"bar\" id=\"barra-evidencias\"><figcaption>{accepted} de {total} evidências conferidas</figcaption><svg role=\"img\" aria-label=\"Evidências: {accepted} conferidas, {refused} recusadas, {to_check} a conferir\" viewBox=\"0 0 1000 14\" preserveAspectRatio=\"none\">{rects}</svg>{}</figure>",
        marks_html(
            "ul",
            &[
                format!("{accepted} conferidas"),
                format!("{refused} recusadas"),
                format!("{to_check} a conferir"),
            ]
        )
    )
}

/// Bloco "Como foi executado": tarefas e ondas, retrabalho e volume.
fn execution_html(metrics: &SpecMetrics) -> String {
    let mut items = vec![if metrics.waves == 0 {
        plural(metrics.tasks, "tarefa", "tarefas")
    } else {
        format!(
            "{} em {}",
            plural(metrics.tasks, "tarefa", "tarefas"),
            plural(metrics.waves, "onda", "ondas")
        )
    }];
    let lines = metrics_lines(metrics);
    for prefix in ["- Retrabalho: ", "- Volume: "] {
        items.extend(
            lines
                .iter()
                .find_map(|line| line.strip_prefix(prefix))
                .map(str::to_string),
        );
    }
    let items: String = items
        .iter()
        .map(|item| format!("<li>{}</li>\n", html_escape(item)))
        .collect();
    format!("<ul class=\"facts-list\">\n{items}</ul>")
}

/// Primeira linha de prosa de uma seção, em HTML de uma linha, dentro do elemento dado (vazio sem texto).
fn first_line_html(part: &str, title: &str, wrap: impl Fn(&str) -> String) -> String {
    let section = section_of(part, title);
    let first = prose_lines(&section).next();
    first.map_or(String::new(), |line| wrap(&inline_html(line)))
}

// --- Conteúdo ---

impl Finalization {
    fn create(&self) -> Result<()> {
        let file = self.content_path()?;
        if file.exists() {
            return Err(Reported(format!(
                "o conteúdo de {} já existe: {}\n",
                self.id,
                self.rel(&file)
            ))
            .into());
        }
        let template = fs::read_to_string(self.content_template()?)
            .context("não foi possível ler o template do conteúdo")?;
        let values = HashMap::from([("SPEC_ID", self.id.clone())]);
        let mut content = render_lines(&template, &values);
        let mut blocks = 0;
        let study = self.study_model();
        for task in &self.tasks {
            let task_id = task.id();
            let evidence_dir = self.task_evidence_dir(task)?;
            for (index, (name, _)) in self.evidence_items(task)?.into_iter().enumerate() {
                blocks += 1;
                content.push_str(&format!(
                    "\n{}\n\n",
                    evidence_heading(&task_id, index, &name)
                ));
                if study {
                    content.push_str(&format!(
                        "- **{EVIDENCE_PROOF}:** #{{EVIDENCE_{blocks:02}_PROOF}}\n"
                    ));
                }
                let how = match log_command(&evidence_dir.join(&name)).filter(|_| study) {
                    Some(command) => format!("`{command}`"),
                    None => format!("#{{EVIDENCE_{blocks:02}_HOW}}"),
                };
                content.push_str(&format!(
                    "- **{EVIDENCE_HOW}:** {how}\n- **{EVIDENCE_READ}:** #{{EVIDENCE_{blocks:02}_READ}}\n"
                ));
            }
        }
        fs::create_dir_all(self.content_dir()?)?;
        fs::write(&file, content)
            .with_context(|| format!("não foi possível gravar {}", file.display()))?;
        println!(
            "Conteúdo de {} criado em {}: {blocks} bloco(s) de evidência.",
            self.id,
            self.rel(&file)
        );
        Ok(())
    }

    fn validate(&self) -> Result<()> {
        self.require_valid_content()?;
        println!(
            "Conteúdo de {} válido: {}",
            self.id,
            self.rel(&self.content_path()?)
        );
        if self.study_model() {
            let content = fs::read_to_string(self.content_path()?).unwrap_or_default();
            let words = prose_lines(&content_part(&content, "Executivo"))
                .flat_map(str::split_whitespace)
                .count();
            if words > EXECUTIVE_WORDS_WARNING {
                println!(
                    "Aviso: o executivo tem {words} palavras; o estudo de conteúdo recomenda de 400 a 800."
                );
            }
        }
        Ok(())
    }

    /// O template de conteúdo instalado segue o estudo de conteúdo: tem o mapa de cobertura. Um harness
    /// anterior ao `vostok update` continua com o modelo antigo, nas regras e nos documentos.
    fn study_model(&self) -> bool {
        self.content_template()
            .ok()
            .and_then(|path| fs::read_to_string(path).ok())
            .is_some_and(|template| {
                template
                    .lines()
                    .any(|line| line == format!("## {COVERAGE_SECTION}"))
            })
    }

    /// IDs dos requisitos funcionais e não funcionais da spec, na ordem das tabelas.
    fn requirement_ids(&self) -> Vec<String> {
        let spec = fs::read_to_string(&self.spec).unwrap_or_default();
        spec.lines()
            .filter_map(|line| line.strip_prefix("| ")?.split(' ').next())
            .filter(|id| {
                ["RF-", "RNF-"].iter().any(|prefix| {
                    id.strip_prefix(prefix)
                        .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
                })
            })
            .map(str::to_string)
            .collect()
    }

    fn require_valid_content(&self) -> Result<()> {
        let errors = self.content_errors()?;
        if !errors.is_empty() {
            bail!(problem_list(
                &format!("o conteúdo de {} tem problemas:", self.id),
                &errors
            ));
        }
        Ok(())
    }

    /// Problemas do arquivo de conteúdo, na ordem dos scripts. Vazio se ele estiver válido.
    fn content_errors(&self) -> Result<Vec<String>> {
        let file = self.content_path()?;
        let Ok(content) = fs::read_to_string(&file) else {
            return Ok(vec![format!(
                "conteúdo não encontrado: {}; crie com vostok finalize create",
                self.rel(&file)
            )]);
        };
        let mut errors: Vec<String> = template_leftovers(&content)
            .into_iter()
            .map(|(line, text)| format!("variável sem preencher: linha {line}: {text}"))
            .collect();
        let template = fs::read_to_string(self.content_template()?).unwrap_or_default();
        for heading in template
            .lines()
            .filter(|line| line.starts_with("# ") || line.starts_with("## "))
        {
            if !content.lines().any(|line| line == heading) {
                errors.push(format!("seção ausente: {heading}"));
            }
        }
        let study = self.study_model();
        errors.extend(section_errors(
            &content,
            &self.task_setting("id.prefix")?,
            study,
        ));
        if study {
            errors.extend(study_errors(&content, &self.requirement_ids()));
        } else {
            let deliveries = part_section(&content_part(&content, "Executivo"), "Entregas");
            errors.extend(delivery_errors(&deliveries));
        }
        for task in &self.tasks {
            let task_id = task.id();
            let evidence_dir = self.task_evidence_dir(task)?;
            for (index, (name, _)) in self.evidence_items(task)?.into_iter().enumerate() {
                let path = evidence_dir.join(&name);
                if !path.is_file() {
                    errors.push(format!("{task_id}: evidência ausente: {}", self.rel(&path)));
                }
                if evidence_heading_in(&content, &task_id, index, &name).is_none() {
                    let block = evidence_heading(&task_id, index, &name);
                    errors.push(format!("evidência sem bloco: {block}"));
                }
            }
        }
        Ok(errors)
    }
}

/// Seções `## ` vazias e blocos de evidência sem "Como chegar" ou "Como ler" (e sem "Prova", com
/// `require_proof`).
fn section_errors(content: &str, task_prefix: &str, require_proof: bool) -> Vec<String> {
    struct Block {
        heading: String,
        evidence: bool,
        lines: usize,
        proof: bool,
        how: bool,
        read: bool,
    }
    let flush = |block: Option<Block>, errors: &mut Vec<String>| {
        let Some(block) = block else {
            return;
        };
        if block.evidence && require_proof && !block.proof {
            errors.push(format!(
                "bloco de evidência sem \"{EVIDENCE_PROOF}\": {}",
                block.heading
            ));
        }
        if block.evidence {
            if !block.how {
                errors.push(format!(
                    "bloco de evidência sem \"{EVIDENCE_HOW}\": {}",
                    block.heading
                ));
            }
            if !block.read {
                errors.push(format!(
                    "bloco de evidência sem \"{EVIDENCE_READ}\": {}",
                    block.heading
                ));
            }
        } else if block.lines == 0 {
            errors.push(format!("seção vazia: {}", block.heading));
        }
    };
    let mut errors = Vec::new();
    let mut current: Option<Block> = None;
    for line in content.lines().map(strip_cr) {
        if is_comment_line(line) {
            continue;
        }
        if line.starts_with("# ") {
            flush(current.take(), &mut errors);
            continue;
        }
        if line.starts_with("## ") {
            flush(current.take(), &mut errors);
            current = Some(Block {
                heading: line.to_string(),
                evidence: is_evidence_heading(line, task_prefix),
                lines: 0,
                proof: false,
                how: false,
                read: false,
            });
            continue;
        }
        if let Some(block) = current.as_mut() {
            if line.contains(|c| c != ' ' && c != '\t') {
                block.lines += 1;
                block.proof |= is_filled(line, EVIDENCE_PROOF);
                block.how |= is_filled(line, EVIDENCE_HOW);
                block.read |= is_filled(line, EVIDENCE_READ);
            }
        }
    }
    flush(current, &mut errors);
    errors
}

/// Linhas de texto de um trecho, sem comentários e sem linhas em branco.
fn prose_lines(text: &str) -> impl Iterator<Item = &str> {
    text.lines()
        .map(strip_cr)
        .filter(|line| !is_comment_line(line) && !awk_trim(line).is_empty())
}

/// Linhas de tabela que não são o separador.
fn table_rows(section: &str) -> usize {
    prose_lines(section)
        .filter(|line| line.starts_with('|') && !is_table_separator(line))
        .count()
}

/// O texto cita o ID sozinho (`RF-1` não conta como `RF-10`).
fn cites_id(text: &str, id: &str) -> bool {
    text.match_indices(id).any(|(index, _)| {
        let before = text[..index].chars().next_back();
        let after = text[index + id.len()..].chars().next();
        !before.is_some_and(|c| c.is_alphanumeric() || c == '-')
            && !after.is_some_and(|c| c.is_ascii_digit())
    })
}

/// Regras do estudo de conteúdo que a validação confere: "Em números", as tabelas do técnico, o mapa
/// de cobertura, o executivo sem código e itens de lista que são parágrafos.
fn study_errors(content: &str, requirements: &[String]) -> Vec<String> {
    let mut errors = Vec::new();
    for part_title in ["Executivo", "Técnico"] {
        let part = content_part(content, part_title);
        let numbers = section_of(&part, "Em números");
        let lines: Vec<&str> = prose_lines(&numbers).collect();
        if lines != [NOT_APPLICABLE] {
            if lines.len() > MAX_NUMBERS {
                errors.push(format!(
                    "Em números ({part_title}): {} itens, o máximo é {MAX_NUMBERS}",
                    lines.len()
                ));
            }
            for line in &lines {
                let valid = line
                    .strip_prefix("- **")
                    .and_then(|rest| rest.split_once("**"))
                    .is_some_and(|(value, label)| !value.is_empty() && !awk_trim(label).is_empty());
                if !valid {
                    errors.push(format!(
                        "Em números ({part_title}): linha fora do formato \"- **<valor>** <rótulo>\": {line}"
                    ));
                }
            }
        }
        for line in prose_lines(&part) {
            let item = line.strip_prefix("- ").or_else(|| ordered_item(line));
            if item.is_some_and(|item| item.chars().count() > MAX_LIST_ITEM_CHARS) {
                let start: String = line.chars().take(60).collect();
                errors.push(format!(
                    "item de lista com mais de {MAX_LIST_ITEM_CHARS} caracteres ({part_title}); transforme em parágrafo: {start}…"
                ));
            }
        }
    }
    let executive = content_part(content, "Executivo");
    if executive.lines().any(|line| line == "## Resultado") {
        errors.extend(decision_errors(&executive));
    }
    for line in prose_lines(&executive) {
        if line.contains('`') {
            errors.push(format!(
                "código no executivo (caminho, comando ou função): {line}"
            ));
        }
    }
    let technical = content_part(content, "Técnico");
    if table_rows(&section_of(&technical, "Mapa do código")) < 2 {
        errors.push(
            "Mapa do código (Técnico): falta a tabela \"| Caminho | Papel | Testes |\" com pelo menos uma linha"
                .to_string(),
        );
    }
    let operation = section_of(&technical, "Operação e diagnóstico");
    if !prose_lines(&operation).any(|line| ordered_item(line).is_some())
        && table_rows(&operation) < 2
    {
        errors.push(
            "Operação e diagnóstico (Técnico): falta um passo numerado ou uma tabela de sintomas"
                .to_string(),
        );
    }
    let coverage = section_of(&content_part(content, "Evidências"), COVERAGE_SECTION);
    for id in requirements {
        if !cites_id(&coverage, id) {
            errors.push(format!("{COVERAGE_SECTION} (Evidências): falta {id}"));
        }
    }
    errors
}

/// Título `## <TASK>-<número> / EV-<número> / <arquivo>` do bloco de uma evidência, pela posição dela na
/// tarefa.
fn evidence_heading(task_id: &str, index: usize, name: &str) -> String {
    format!("## {task_id} / EV-{:02} / {name}", index + 1)
}

/// Título `## <TASK>-<número> / <arquivo>`, do formato anterior ao ID da evidência.
fn legacy_evidence_heading(task_id: &str, name: &str) -> String {
    format!("## {task_id} / {name}")
}

/// Título do bloco da evidência no conteúdo: o com ID, se existe, ou o antigo, aceito para conteúdos
/// criados antes do ID.
fn evidence_heading_in(content: &str, task_id: &str, index: usize, name: &str) -> Option<String> {
    [
        evidence_heading(task_id, index, name),
        legacy_evidence_heading(task_id, name),
    ]
    .into_iter()
    .find(|heading| content.lines().any(|line| line == heading))
}

/// Título `## <TASK>-<número> / ` de um bloco de evidência.
fn is_evidence_heading(line: &str, task_prefix: &str) -> bool {
    line.strip_prefix("## ")
        .and_then(|rest| rest.strip_prefix(task_prefix))
        .and_then(|rest| rest.strip_prefix('-'))
        .is_some_and(|rest| {
            let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
            digits > 0 && rest[digits..].starts_with(" / ")
        })
}

/// Campo "- **Rótulo:** valor" com algum texto no valor.
fn is_filled(line: &str, label: &str) -> bool {
    line.strip_prefix(&format!("- **{label}:** "))
        .is_some_and(|rest| rest.contains(|c| c != ' ' && c != '\t'))
}

/// Linhas de uma parte (`# Título`) do conteúdo, sem o título, até a próxima parte.
fn content_part(content: &str, title: &str) -> String {
    let heading = format!("# {title}");
    let mut inside = false;
    let mut out = String::new();
    for line in content.lines().map(strip_cr) {
        if line.starts_with("# ") {
            inside = line == heading;
        } else if inside {
            push_line(&mut out, line);
        }
    }
    out
}

/// Linhas de uma seção (`## Título`) de uma parte, sem o título, até a próxima seção.
fn part_section(part: &str, title: &str) -> String {
    let heading = format!("## {title}");
    let mut inside = false;
    let mut out = String::new();
    for line in part.lines() {
        if line.starts_with("## ") {
            inside = line == heading;
        } else if inside {
            push_line(&mut out, line);
        }
    }
    out
}

/// Seção de uma parte já capturada, como no `printf '%s\n' "$part" | finalize_part_section`.
fn section_of(part: &str, title: &str) -> String {
    part_section(&format!("{part}\n"), title)
}

// --- Entregas do executivo ---

enum Delivery {
    Entry(String),
    Field(String, String),
    Text(String),
}

/// Entregas da seção: `### ` abre uma entrega, "- **Rótulo:** valor" é um campo e o resto é texto solto.
/// As linhas passam pelo mesmo `read` com tabulação dos scripts.
fn deliveries(section: &str) -> Vec<Delivery> {
    let mut raw = Vec::new();
    let mut open = false;
    for line in section.lines().map(strip_cr) {
        if is_comment_line(line) {
            continue;
        }
        if let Some(title) = line.strip_prefix("### ") {
            open = true;
            raw.push(format!(
                "E\t{}",
                title.trim_start_matches(' ').trim_end_matches([' ', '\t'])
            ));
        } else if !open {
            continue;
        } else if let Some((label, value)) = delivery_field(line) {
            raw.push(format!("F\t{label}\t{}", awk_trim(value)));
        } else {
            raw.push(format!("T\t{line}"));
        }
    }
    raw.iter()
        .filter_map(|line| {
            let mut fields = read_fields(line, 3).into_iter();
            let kind = fields.next()?;
            let a = fields.next().unwrap_or_default();
            let b = fields.next().unwrap_or_default();
            match kind.as_str() {
                "E" => Some(Delivery::Entry(a)),
                "F" => Some(Delivery::Field(a, b)),
                "T" => Some(Delivery::Text(a)),
                _ => None,
            }
        })
        .collect()
}

/// Campo "- **Rótulo:** valor" de uma entrega: rótulo e o resto da linha.
fn delivery_field(line: &str) -> Option<(&str, &str)> {
    let rest = line.strip_prefix("- **")?;
    let star = rest.find('*')?;
    let label = rest[..star].strip_suffix(':')?;
    let value = rest[star..].strip_prefix("**")?;
    (!label.is_empty()).then_some((label, value))
}

fn impact_class(impact: &str) -> &'static str {
    match impact {
        "alto" => "alto",
        "médio" | "medio" => "medio",
        "baixo" => "baixo",
        _ => "",
    }
}

fn impact_tag(impact: &str) -> String {
    if impact.is_empty() {
        return String::new();
    }
    format!(
        "<span class=\"tag {}\">Impacto {}</span>",
        impact_class(impact),
        html_escape(impact)
    )
}

/// Problemas das entregas: nenhuma entrega, entrega sem "O que mudou" ou "Valor" e impacto inválido.
fn delivery_errors(section: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let mut current: Option<(String, bool, bool)> = None;
    let finish = |current: Option<(String, bool, bool)>, errors: &mut Vec<String>| {
        if let Some((title, changed, value)) = current {
            if !changed {
                errors.push(format!("entrega sem \"O que mudou\": ### {title}"));
            }
            if !value {
                errors.push(format!("entrega sem \"Valor\": ### {title}"));
            }
        }
    };
    let mut count = 0;
    for delivery in deliveries(section) {
        match delivery {
            Delivery::Entry(title) => {
                finish(current.take(), &mut errors);
                current = Some((title, false, false));
                count += 1;
            }
            Delivery::Field(label, value) => {
                let title = current.as_ref().map_or("", |(title, _, _)| title.as_str());
                match label.as_str() {
                    "O que mudou" if !value.is_empty() => {
                        if let Some(entry) = current.as_mut() {
                            entry.1 = true;
                        }
                    }
                    "Valor" if !value.is_empty() => {
                        if let Some(entry) = current.as_mut() {
                            entry.2 = true;
                        }
                    }
                    "Impacto" if impact_class(&value).is_empty() => errors.push(format!(
                        "impacto inválido em ### {title}: '{value}' (use alto, médio ou baixo)"
                    )),
                    _ => {}
                }
            }
            Delivery::Text(_) => {}
        }
    }
    finish(current, &mut errors);
    if count == 0 {
        errors.push("entregas sem nenhuma entrega: use um \"### \" por entrega".to_string());
    }
    errors
}

/// Tabela-resumo das entregas: nome, impacto e valor.
fn deliveries_table_html(section: &str) -> String {
    let mut out = String::from(
        "<div class=\"tw\"><table>\n<thead><tr><th>Entrega</th><th>Impacto</th><th>Valor</th></tr></thead>\n<tbody>\n",
    );
    let mut row: Option<(String, String, String)> = None;
    let emit = |row: Option<(String, String, String)>, out: &mut String| {
        if let Some((title, impact, value)) = row.filter(|(title, _, _)| !title.is_empty()) {
            out.push_str(&format!(
                "<tr><td class=\"strong\">{}</td><td>{}</td><td>{}</td></tr>\n",
                inline_html(&title),
                impact_tag(&impact),
                inline_html(&value)
            ));
        }
    };
    for delivery in deliveries(section) {
        match delivery {
            Delivery::Entry(title) => {
                emit(row.take(), &mut out);
                row = Some((title, String::new(), String::new()));
            }
            Delivery::Field(label, value) => {
                if let Some((_, impact, row_value)) = row.as_mut() {
                    match label.as_str() {
                        "Impacto" => *impact = value,
                        "Valor" => *row_value = value,
                        _ => {}
                    }
                }
            }
            Delivery::Text(_) => {}
        }
    }
    emit(row, &mut out);
    out.push_str("</tbody>\n</table></div>\n");
    out
}

/// Cartão expansível de uma entrega, montado aos poucos.
#[derive(Default)]
struct Card {
    title: String,
    impact: String,
    body: String,
    text: String,
}

impl Card {
    fn flush_text(&mut self) {
        if self.text.is_empty() {
            return;
        }
        self.body
            .push_str(&captured(&block_html(&format!("{}\n", self.text))));
        self.body.push('\n');
        self.text.clear();
    }

    fn emit(mut self, out: &mut String) {
        if self.title.is_empty() {
            return;
        }
        self.flush_text();
        out.push_str(&format!(
            "<details class=\"card impact-{}\">\n<summary><b>{}</b>{}</summary>\n<div class=\"card-body\">\n{}</div>\n</details>\n",
            impact_class(&self.impact),
            inline_html(&self.title),
            impact_tag(&self.impact),
            self.body
        ));
    }
}

/// Cartões expansíveis das entregas: título e impacto no resumo e os outros campos no corpo.
fn deliveries_cards_html(section: &str) -> String {
    let mut out = String::new();
    let mut card = Card::default();
    for delivery in deliveries(section) {
        match delivery {
            Delivery::Entry(title) => {
                std::mem::take(&mut card).emit(&mut out);
                card.title = title;
            }
            Delivery::Field(label, value) if label == "Impacto" => card.impact = value,
            Delivery::Field(label, value) => {
                card.flush_text();
                card.body.push_str(&format!(
                    "<p><b>{}</b>{}</p>\n",
                    html_escape(&label),
                    inline_html(&value)
                ));
            }
            Delivery::Text(text) => {
                card.text.push_str(&text);
                card.text.push('\n');
            }
        }
    }
    card.emit(&mut out);
    out
}

/// Faixa de indicadores de valor a partir da tabela "| Valor | Indicador |" da seção.
fn value_kpis_html(section: &str) -> String {
    let mut rows = Vec::new();
    let mut after_separator = false;
    for line in section.lines().map(strip_cr) {
        if !line.starts_with('|') {
            continue;
        }
        if is_table_separator(line) {
            after_separator = true;
            continue;
        }
        if !after_separator {
            continue;
        }
        let inner = line.strip_prefix('|').unwrap_or(line);
        let inner = strip_trailing_pipe(inner);
        let cells = awk_split(inner, '|');
        if cells.len() < 2 {
            continue;
        }
        rows.push(format!("{}\t{}", awk_trim(cells[0]), awk_trim(cells[1])));
    }
    if rows.is_empty() {
        return String::new();
    }
    let mut out = String::from("<div class=\"kpis\">\n");
    for row in rows {
        let fields = read_fields(&row, 2);
        out.push_str(&format!(
            "<div class=\"kpi\"><b>{}</b><span>{}</span></div>\n",
            inline_html(&fields[0]),
            inline_html(&fields[1])
        ));
    }
    out.push_str("</div>\n");
    out
}

/// Remove um `|` final seguido só de espaços ou tabulações.
fn strip_trailing_pipe(text: &str) -> &str {
    let trimmed = text.trim_end_matches([' ', '\t']);
    trimmed.strip_suffix('|').unwrap_or(text)
}

/// `split` do awk com separador de um caractere: texto vazio não tem campos.
fn awk_split(text: &str, separator: char) -> Vec<&str> {
    if text.is_empty() {
        Vec::new()
    } else {
        text.split(separator).collect()
    }
}

/// Separador de tabela: só `|`, `:`, `-`, espaços e tabulações, entre barras, com `---`.
fn is_table_separator(line: &str) -> bool {
    let Some(inner) = line.strip_prefix('|') else {
        return false;
    };
    let inner = inner.trim_end_matches([' ', '\t']);
    let Some(cells) = inner.strip_suffix('|') else {
        return false;
    };
    !cells.is_empty()
        && cells
            .chars()
            .all(|c| matches!(c, ' ' | '\t' | ':' | '|' | '-'))
        && line.contains("---")
}

// --- Markdown restrito para HTML ---

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn html_escape_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    for byte in bytes {
        match byte {
            b'&' => out.extend_from_slice(b"&amp;"),
            b'<' => out.extend_from_slice(b"&lt;"),
            b'>' => out.extend_from_slice(b"&gt;"),
            b'"' => out.extend_from_slice(b"&quot;"),
            b'\'' => out.extend_from_slice(b"&#39;"),
            other => out.push(*other),
        }
    }
    out
}

/// `**negrito**` de um texto já escapado.
fn bold(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some((start, end)) = bold_span(rest) {
        out.push_str(&rest[..start]);
        out.push_str("<strong>");
        out.push_str(&rest[start + 2..end]);
        out.push_str("</strong>");
        rest = &rest[end + 2..];
    }
    out.push_str(rest);
    out
}

/// Posição do primeiro `**texto**` sem `*` dentro: início do `**` de abertura e do de fechamento.
fn bold_span(text: &str) -> Option<(usize, usize)> {
    let bytes = text.as_bytes();
    (0..bytes.len().saturating_sub(1)).find_map(|start| {
        if &bytes[start..start + 2] != b"**" {
            return None;
        }
        let close = start + 2 + bytes[start + 2..].iter().position(|b| *b == b'*')?;
        (close > start + 2 && bytes.get(close + 1) == Some(&b'*')).then_some((start, close))
    })
}

/// `código` e **negrito** de um texto já escapado. Crase sem par fica como está.
fn inline(text: &str) -> String {
    let parts: Vec<&str> = text.split('`').collect();
    if parts.len().is_multiple_of(2) {
        return bold(text);
    }
    parts
        .iter()
        .enumerate()
        .map(|(index, part)| {
            if index % 2 == 0 {
                bold(part)
            } else {
                format!("<code>{part}</code>")
            }
        })
        .collect()
}

fn table_row(line: &str, tag: &str) -> String {
    let line = awk_trim(line);
    let line = line.strip_prefix('|').unwrap_or(line);
    let line = line.strip_suffix('|').unwrap_or(line);
    let cells: String = awk_split(line, '|')
        .into_iter()
        .map(|cell| format!("<{tag}>{}</{tag}>", inline(&html_escape(awk_trim(cell)))))
        .collect();
    format!("<tr>{cells}</tr>")
}

/// Bloco aberto durante a conversão do Markdown.
enum Block {
    None,
    Paragraph(String),
    List(&'static str),
    Table { rows: Vec<String>, header: bool },
    Code(String),
}

fn close_block(block: &mut Block, out: &mut String) {
    match std::mem::replace(block, Block::None) {
        Block::None => {}
        Block::Paragraph(text) => {
            push_line(out, &format!("<p>{}</p>", inline(&html_escape(&text))));
        }
        Block::List(tag) => push_line(out, &format!("</{tag}>")),
        Block::Code(code) => {
            push_line(
                out,
                &format!("<pre><code>{}</code></pre>", html_escape(&code)),
            );
        }
        Block::Table { rows, header } => {
            push_line(out, "<table>");
            if header {
                push_line(out, "<thead>");
                push_line(out, &table_row(&rows[0], "th"));
                push_line(out, "</thead>");
            }
            push_line(out, "<tbody>");
            for row in rows.iter().skip(usize::from(header)) {
                push_line(out, &table_row(row, "td"));
            }
            push_line(out, "</tbody>");
            push_line(out, "</table>");
        }
    }
}

/// Nível e texto de um título Markdown de `#` a `###`.
fn heading(line: &str) -> Option<(usize, &str)> {
    let depth = line.bytes().take_while(|byte| *byte == b'#').count();
    if !(1..=3).contains(&depth) {
        return None;
    }
    line[depth..]
        .strip_prefix(' ')
        .map(|title| (depth, awk_trim(title)))
}

fn ordered_item(line: &str) -> Option<&str> {
    let digits = line.bytes().take_while(u8::is_ascii_digit).count();
    (digits > 0)
        .then(|| line[digits..].strip_prefix(". "))
        .flatten()
}

fn open_list(block: &mut Block, tag: &'static str, out: &mut String) {
    if !matches!(block, Block::List(open) if *open == tag) {
        close_block(block, out);
        *block = Block::List(tag);
        push_line(out, &format!("<{tag}>"));
    }
}

/// Converte o Markdown restrito do conteúdo em HTML: títulos, parágrafos, listas, tabelas, blocos de
/// código, `código` e **negrito**. Todo o texto sai escapado; linhas que são só um comentário são descartadas.
fn markdown_html(markdown: &str) -> String {
    markdown_html_at(markdown, SECTION_HEADING)
}

/// Como `markdown_html`, com `#` virando `<h{level}>` e cada `#` a mais descendo um nível, até `<h6>`.
fn markdown_html_at(markdown: &str, level: usize) -> String {
    let mut out = String::new();
    let mut block = Block::None;
    for line in markdown.lines().map(strip_cr) {
        if let Block::Code(code) = &mut block {
            if line.trim_start().starts_with("```") {
                close_block(&mut block, &mut out);
            } else {
                if !code.is_empty() {
                    code.push('\n');
                }
                code.push_str(line);
            }
            continue;
        }
        if line.trim_start().starts_with("```") {
            close_block(&mut block, &mut out);
            block = Block::Code(String::new());
            continue;
        }
        if is_comment_line(line) {
            continue;
        }
        if let Some((depth, title)) = heading(line) {
            close_block(&mut block, &mut out);
            let tag = (level + depth - 1).min(6);
            push_line(
                &mut out,
                &format!("<h{tag}>{}</h{tag}>", inline(&html_escape(title))),
            );
            continue;
        }
        if awk_trim(line).is_empty() {
            close_block(&mut block, &mut out);
            continue;
        }
        if line.starts_with('|') {
            if !matches!(block, Block::Table { .. }) {
                close_block(&mut block, &mut out);
                block = Block::Table {
                    rows: Vec::new(),
                    header: false,
                };
            }
            if let Block::Table { rows, header } = &mut block {
                if is_table_separator(line) {
                    *header |= rows.len() == 1;
                } else {
                    rows.push(line.to_string());
                }
            }
            continue;
        }
        if let Some(item) = line.strip_prefix("- ") {
            open_list(&mut block, "ul", &mut out);
            push_line(
                &mut out,
                &format!("<li>{}</li>", inline(&html_escape(item))),
            );
            continue;
        }
        if let Some(item) = ordered_item(line) {
            open_list(&mut block, "ol", &mut out);
            push_line(
                &mut out,
                &format!("<li>{}</li>", inline(&html_escape(item))),
            );
            continue;
        }
        let text = awk_trim(line);
        match &mut block {
            Block::Paragraph(para) => {
                para.push(' ');
                para.push_str(text);
            }
            _ => {
                close_block(&mut block, &mut out);
                block = Block::Paragraph(text.to_string());
            }
        }
    }
    close_block(&mut block, &mut out);
    out
}

/// HTML de uma seção, com cada tabela num contêiner que rola na horizontal.
fn block_html(markdown: &str) -> String {
    block_html_at(markdown, SECTION_HEADING)
}

/// Como `block_html`, com os títulos a partir de `<h{level}>`.
fn block_html_at(markdown: &str, level: usize) -> String {
    markdown_html_at(markdown, level)
        .lines()
        .map(|line| match line {
            "<table>" => "<div class=\"tw\"><table>\n".to_string(),
            "</table>" => "</table></div>\n".to_string(),
            other => format!("{other}\n"),
        })
        .collect()
}

/// HTML de uma linha de Markdown restrito, sem o parágrafo em volta.
fn inline_html(text: &str) -> String {
    let html: String = markdown_html(&format!("{text}\n"))
        .lines()
        .map(|line| {
            let line = line.strip_prefix("<p>").unwrap_or(line);
            let line = line.strip_suffix("</p>").unwrap_or(line);
            format!("{line}\n")
        })
        .collect();
    captured(&html)
}

/// CSS dos tokens de um `documents.yml` anterior aos tokens da organização (sem a chave `bg`), como era:
/// `:root` com o tema escuro e as fontes e `:root[data-theme="light"]` com o claro. Vale para o harness
/// que ainda não passou pelo `vostok update`, cujos templates também são os antigos.
fn legacy_css_tokens(
    dark: &[(String, String)],
    light: &[(String, String)],
    font: &[(String, String)],
) -> String {
    let variable =
        |(key, value): &(String, String)| format!("  --{}: {value};\n", key.replace('_', "-"));
    let mut css = String::from(":root {\n  color-scheme: dark;\n");
    css.extend(dark.iter().map(variable));
    css.extend(
        font.iter()
            .map(|(key, value)| format!("  --font-{}: {value};\n", key.replace('_', "-"))),
    );
    css.push_str("}\n:root[data-theme=\"light\"] {\n  color-scheme: light;\n");
    css.extend(light.iter().map(variable));
    css.push('}');
    css
}

/// Parágrafo "Prova" de um bloco de evidência; vazio no conteúdo anterior ao estudo, que não tem o campo.
fn proof_html(content: &str, heading: &str) -> String {
    let proof = evidence_field(content, heading, EVIDENCE_PROOF);
    if awk_trim(&proof).is_empty() {
        return String::new();
    }
    format!("    <p><b>{EVIDENCE_PROOF}</b>{}</p>", inline_html(&proof))
}

/// Estado de cada evidência pelas linhas "conferência" do registro, na tarefa sem `conferencias.md`:
/// a última linha que cita "<nome> conferida" ou "<nome> recusada" vale.
fn legacy_review_states(
    details: &[String],
    evidences: &[(String, String)],
) -> HashMap<String, String> {
    let mut states = HashMap::new();
    for detail in details {
        for (name, _) in evidences {
            for state in ["conferida", "recusada"] {
                if detail.contains(&format!("{name} {state}")) {
                    states.insert(name.clone(), state.to_string());
                }
            }
        }
    }
    states
}

/// Parágrafo `<p><b>rótulo</b>texto</p>` de um campo da evidência; vazio sem o texto.
fn note_html(label: &str, text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    format!("    <p><b>{label}</b>{}</p>", inline_html(text))
}

/// O que a evidência mostra: "Mostra" da tarefa, senão a "Prova" da tarefa, senão a "Prova" do conteúdo.
fn shows_html(note: &EvidenceNotes, content: &str, heading: &str) -> String {
    let shows = if note.shows.is_empty() {
        &note.proof
    } else {
        &note.shows
    };
    if shows.is_empty() {
        return proof_html(content, heading);
    }
    note_html("Mostra", shows)
}

/// Onde olhar na evidência: "Onde olhar" da tarefa, senão o "Trecho" como "O que olhar".
fn where_html(note: &EvidenceNotes) -> String {
    if note.where_to_look.is_empty() {
        note_html("O que olhar", &note.snippet)
    } else {
        note_html("Onde olhar", &note.where_to_look)
    }
}

/// Coluna de números da faixa do resumo: cada item `- **<valor>** <rótulo>` de `## Em números` vira um
/// bloco. Sem a seção ou sem itens nesse formato, fica vazia e a faixa ocupa a largura toda.
fn numbers_html(part: &str) -> String {
    let facts: String = section_of(part, "Em números")
        .lines()
        .filter_map(|line| {
            let rest = line.strip_prefix("- **")?;
            let (value, label) = rest.split_once("**")?;
            Some(format!(
                "<div class=\"fact\"><b>{}</b><span>{}</span></div>\n",
                html_escape(value),
                inline(&html_escape(awk_trim(label)))
            ))
        })
        .collect();
    if facts.is_empty() {
        return String::new();
    }
    captured(&format!("<div class=\"facts\">\n{facts}</div>\n"))
}

/// HTML capturado de uma seção da parte, como `$(... | finalize_block_html)`.
fn section_html(part: &str, title: &str) -> String {
    captured(&block_html(&section_of(part, title)))
}

// --- Templates e montagem ---

/// Renderiza linha a linha, como o `render` dos scripts: cada linha sai com quebra no fim.
fn render_lines(template: &str, values: &HashMap<&str, String>) -> String {
    let mut out = render(template, values);
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

/// Troca cada linha que é só o marcador pelo conteúdo, seguido de uma quebra de linha.
fn splice(text: &str, marker: &str, content: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len() + content.len());
    for line in text.split_terminator('\n') {
        if line == marker {
            out.extend_from_slice(content);
        } else {
            out.extend_from_slice(line.as_bytes());
        }
        out.push(b'\n');
    }
    out
}

/// Mensagem do `awk` ao não conseguir abrir um arquivo.
fn awk_open_error(path: &Path, error: &io::Error) -> String {
    format!(
        "awk: fatal: cannot open file `{}' for reading: {}\n",
        path.display(),
        strerror(error)
    )
}

/// Texto do erro do sistema, sem o "(os error N)" do Rust.
fn strerror(error: &io::Error) -> String {
    let text = error.to_string();
    match text.find(" (os error ") {
        Some(index) => text[..index].to_string(),
        None => text,
    }
}

/// Lê um template; se não conseguir, registra a mensagem do `awk` e devolve um template vazio.
fn read_template(path: &Path, errors: &mut String) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| {
        errors.push_str(&awk_open_error(path, &error));
        String::new()
    })
}

// --- Documentos ---

/// Quantas evidências foram embutidas no documento e quantas foram para `assets/`.
#[derive(Default)]
struct MediaCounts {
    embedded: usize,
    assets: usize,
}

/// Valores comuns ao cabeçalho dos três documentos.
struct Header {
    title: String,
    approved: String,
    finished: String,
}

impl Finalization {
    fn build(&self) -> Result<()> {
        self.require_valid_content()?;
        let executive = self.document_path("executive")?;
        self.generate(&executive, |errors| self.executive_html(errors))?;
        let technical = self.document_path("technical")?;
        self.generate(&technical, |errors| self.technical_html(errors))?;
        let assets = self.build_evidence()?;
        let evidence = self.document_path("evidence")?;
        println!(
            "Documentos de {} gerados em {}:",
            self.id,
            self.rel(&self.content_dir()?)
        );
        for document in [&executive, &technical, &evidence] {
            println!("  - {}", file_name(document));
        }
        if assets > 0 {
            println!("  - {ASSETS_DIR}/ ({assets} evidência(s) fora do documento)");
        }
        Ok(())
    }

    /// Gera um documento: se a montagem registrar erros, mostra-os e mantém o documento anterior.
    fn generate(
        &self,
        out: &Path,
        build: impl FnOnce(&mut String) -> Result<Vec<u8>>,
    ) -> Result<()> {
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut errors = String::new();
        let document = build(&mut errors)?;
        if !errors.is_empty() {
            errors.push_str(&format!("Erro: falha ao gerar {}\n", file_name(out)));
            return Err(Reported(errors).into());
        }
        let temporary = PathBuf::from(format!("{}.new", out.display()));
        fs::write(&temporary, document)
            .with_context(|| format!("não foi possível gravar {}", temporary.display()))?;
        fs::rename(&temporary, out)
            .with_context(|| format!("não foi possível gravar {}", out.display()))?;
        Ok(())
    }

    fn header(&self) -> Result<Header> {
        let approved = self.approved_at(&self.spec);
        let approved = approved.split('T').next().unwrap_or("");
        let finished = self.last_done_date()?;
        let or_no_date = |value: &str| {
            if value.is_empty() {
                NO_DATE.to_string()
            } else {
                value.to_string()
            }
        };
        Ok(Header {
            title: html_escape(&frontmatter_get(&self.spec, "title").unwrap_or_default()),
            approved: or_no_date(approved),
            finished: or_no_date(&finished),
        })
    }

    fn header_values(
        &self,
        header: &Header,
        kind: &str,
        tokens: String,
        errors: &mut String,
    ) -> Vec<(&'static str, String)> {
        let (brand_name, brand_mark) = self.brand(errors);
        vec![
            (
                "DOC_TITLE",
                format!("{} · {kind} · {}", self.id, header.title),
            ),
            ("TOKENS", tokens),
            ("SPEC_ID", self.id.clone()),
            ("SPEC_TITLE", header.title.clone()),
            ("APPROVED", header.approved.clone()),
            ("FINISHED", header.finished.clone()),
            ("BRAND_NAME", brand_name),
            ("BRAND_MARK", brand_mark),
        ]
    }

    /// Nome e marca do topo dos documentos: os do grupo `brand` de `documents.yml` ou, sem eles, a
    /// identidade do Vostok.
    fn brand(&self, errors: &mut String) -> (String, String) {
        let config = self.documents_config();
        let group = config_group(&config, "brand", errors);
        let value = |key: &str| {
            group
                .iter()
                .find(|(name, _)| name == key)
                .map(|(_, value)| value.clone())
                .unwrap_or_default()
        };
        let name = value("name");
        let name = if name.is_empty() {
            "Vostok".to_string()
        } else {
            html_escape(&name)
        };
        let icon = value("icon");
        let mark = if icon.is_empty() {
            "<i></i>".to_string()
        } else {
            self.brand_icon(&icon).unwrap_or_else(|reason| {
                errors.push_str(&format!("Erro: ícone '{icon}' ({reason})\n"));
                String::new()
            })
        };
        (name, mark)
    }

    /// O ícone da marca embutido em base64, lido em relação à raiz do projeto.
    fn brand_icon(&self, icon: &str) -> std::result::Result<String, String> {
        let extension = icon
            .rsplit_once('.')
            .map_or("", |(_, ext)| ext)
            .to_ascii_lowercase();
        let mime = match extension.as_str() {
            "svg" => "image/svg+xml",
            "png" => "image/png",
            "webp" => "image/webp",
            _ => return Err("extensão não suportada; use svg, png, webp".to_string()),
        };
        let bytes = fs::read(self.root.join(icon))
            .map_err(|error| format!("não foi possível ler o arquivo: {}", strerror(&error)))?;
        if bytes.len() > BRAND_ICON_MAX_BYTES {
            return Err("acima do limite de 256 KB".to_string());
        }
        Ok(format!(
            "<img class=\"brand-mark\" alt=\"\" src=\"data:{mime};base64,{}\">",
            base64(&bytes)
        ))
    }

    fn content_part(&self, title: &str) -> Result<String> {
        let content = fs::read_to_string(self.content_path()?).unwrap_or_default();
        Ok(captured(&content_part(&content, title)))
    }

    fn executive_html(&self, errors: &mut String) -> Result<Vec<u8>> {
        let Some(tokens) = self.css_tokens(errors) else {
            return Ok(Vec::new());
        };
        let header = self.header()?;
        let part = self.content_part("Executivo")?;
        let deliveries_section = format!("{}\n", captured(&section_of(&part, "Entregas")));
        let count = deliveries(&deliveries_section)
            .iter()
            .filter(|delivery| matches!(delivery, Delivery::Entry(_)))
            .count();
        let metrics = self.metrics()?;
        let mut values = self.header_values(&header, "Resumo executivo", tokens, errors);
        values.extend([
            ("LEDE", section_html(&part, "Panorama")),
            ("NUMBERS", numbers_html(&part)),
            ("SUMMARY", section_html(&part, "Em resumo")),
            ("OBJECTIVE", section_html(&part, "Objetivo")),
            ("DELIVERED", section_html(&part, "O que foi entregue")),
            ("VALUE", section_html(&part, "Valor e impacto")),
            ("SECONDARY", section_html(&part, "Ganhos secundários")),
            ("RISKS_LIMITS", section_html(&part, "Riscos e limites")),
            (
                "VALUE_KPIS",
                captured(&value_kpis_html(&section_of(&part, "Indicadores de valor"))),
            ),
            ("SCOPE", section_html(&part, "Recorte")),
            ("DELIVERY_COUNT", count.to_string()),
            (
                "DELIVERY_TABLE",
                captured(&deliveries_table_html(&deliveries_section)),
            ),
            (
                "DELIVERIES",
                captured(&deliveries_cards_html(&deliveries_section)),
            ),
            ("BEFORE_AFTER", section_html(&part, "Antes e depois")),
            ("NEXT", section_html(&part, "Próximos passos")),
            ("RISKS", section_html(&part, "Riscos e limitações")),
            ("STATUS_BADGE", badge_html(&metrics)),
            (
                "RESULT",
                first_line_html(&part, "Resultado", |text| {
                    format!("<p class=\"result\">{text}</p>")
                }),
            ),
            (
                "DECISION",
                first_line_html(&part, "Decisão pedida", |text| {
                    format!("<p class=\"ask\"><b>Decisão pedida</b>{text}</p>")
                }),
            ),
            ("CARDS", captured(&cards_html(&metrics, &part))),
            ("CYCLE_BAR", cycle_bar_html(&metrics)),
            ("EVIDENCE_BAR", evidence_bar_html(&metrics)),
            ("EXECUTION", execution_html(&metrics)),
            ("INDICATORS", captured(&self.indicators_html()?)),
            ("TIMELINE", captured(&self.timeline_html()?)),
        ]);
        let template = read_template(&self.templates_dir()?.join(EXECUTIVE_TEMPLATE), errors);
        Ok(render_lines(&template, &values.into_iter().collect()).into_bytes())
    }

    fn technical_html(&self, errors: &mut String) -> Result<Vec<u8>> {
        let Some(tokens) = self.css_tokens(errors) else {
            return Ok(Vec::new());
        };
        let header = self.header()?;
        let part = self.content_part("Técnico")?;
        let diffs = self.diffs_html()?;
        let mut values = self.header_values(&header, "Documento técnico", tokens, errors);
        values.extend([
            ("OVERVIEW", section_html(&part, "Visão geral")),
            ("NUMBERS", numbers_html(&part)),
            ("SUMMARY", section_html(&part, "Em resumo")),
            ("CONTEXT", section_html(&part, "Contexto")),
            ("HOW", section_html(&part, "Como funciona")),
            ("OPERATION", section_html(&part, "Operação e diagnóstico")),
            ("CODE_MAP", section_html(&part, "Mapa do código")),
            ("LIMITS", section_html(&part, "Limites e dívidas")),
            (
                "ARCHITECTURE",
                section_html(&part, "Arquitetura e componentes"),
            ),
            ("DONE", self.done_html(&part)),
            ("REQUIREMENTS", captured(&self.requirements_html()?)),
            ("FILES", captured(&self.files_html()?)),
            ("DIFFS", DIFFS_MARKER.to_string()),
            ("COMMITS", captured(&self.commits_html()?)),
            ("DECISIONS", section_html(&part, "Decisões")),
            ("LEARNINGS", section_html(&part, "Aprendizados")),
            ("ATTENTION", section_html(&part, "Pontos de atenção")),
            ("REFERENCES", section_html(&part, "Referências")),
        ]);
        let template = read_template(&self.templates_dir()?.join(TECHNICAL_TEMPLATE), errors);
        let html = render_lines(&template, &values.into_iter().collect());
        Ok(splice(&html, DIFFS_MARKER, diffs.as_bytes()))
    }

    /// "O que foi feito" seguido do técnico de cada tarefa em pasta.
    fn done_html(&self, part: &str) -> String {
        let done = section_html(part, "O que foi feito");
        let tasks: String = self
            .tasks
            .iter()
            .filter(|task| task.is_folder() && !task.technical.is_empty())
            .map(|task| {
                format!(
                    "<h3>{}: {}</h3>\n{}",
                    html_escape(&task.id()),
                    html_escape(&task.title()),
                    block_html_at(&task.technical, TASK_HEADING)
                )
            })
            .collect();
        if tasks.is_empty() {
            return done;
        }
        format!("{done}\n{}", captured(&tasks))
    }

    /// CSS dos tokens e das fontes: `:root` com o tema claro, o escuro pela preferência do sistema e
    /// por `data-theme="dark"`, os gradientes de `calm_*` e `brand_*` e as fontes embutidas em base64.
    fn css_tokens(&self, errors: &mut String) -> Option<String> {
        let config = self.documents_config();
        let light = config_group(&config, "light", errors);
        let dark = config_group(&config, "dark", errors);
        let font = config_group(&config, "font", errors);
        if dark.is_empty() || light.is_empty() {
            errors.push_str("config/documents.yml sem os grupos 'dark' e 'light'\n");
            return None;
        }
        if !light.iter().any(|(key, _)| key == "bg") {
            return Some(legacy_css_tokens(&dark, &light, &font));
        }
        let theme = |pairs: &[(String, String)]| {
            let mut css: String = pairs
                .iter()
                .map(|(key, value)| format!("  --{}: {value};\n", key.replace('_', "-")))
                .collect();
            for gradient in ["calm", "brand"] {
                css.push_str(&format!(
                    "  --{gradient}: linear-gradient(135deg, var(--{gradient}-from), var(--{gradient}-to));\n"
                ));
            }
            css
        };
        let mut css = self.font_faces();
        css.push_str(":root {\n  color-scheme: light;\n");
        css.push_str(&theme(&light));
        css.extend(
            font.iter()
                .map(|(key, value)| format!("  --font-{}: {value};\n", key.replace('_', "-"))),
        );
        let dark = theme(&dark);
        css.push_str("}\n@media (prefers-color-scheme: dark) {\n:root:not([data-theme=\"light\"]) {\n  color-scheme: dark;\n");
        css.push_str(&dark);
        css.push_str("}\n}\n:root[data-theme=\"dark\"] {\n  color-scheme: dark;\n");
        css.push_str(&dark);
        css.push('}');
        Some(css)
    }

    /// `@font-face` de cada fonte de `templates/finalize/fonts/`, embutida em base64. Uma fonte que não
    /// está instalada (harness anterior ao `vostok update`) fica de fora, e vale a reserva do sistema.
    fn font_faces(&self) -> String {
        let Ok(dir) = self.templates_dir() else {
            return String::new();
        };
        let mut css = String::new();
        for (family, file) in FONTS {
            let path = dir.join("finalize/fonts").join(file);
            if let Ok(bytes) = fs::read(&path) {
                css.push_str(&format!(
                    "@font-face {{ font-family: \"{family}\"; src: url(data:font/woff2;base64,{}) format(\"woff2\"); font-weight: 100 900; font-style: normal; font-display: swap; }}\n",
                    base64(&bytes)
                ));
            }
        }
        css
    }

    /// Indicadores de execução: tarefas concluídas, tarefas com testes aprovados e evidências geradas.
    fn indicators_html(&self) -> Result<String> {
        let done = self.task_setting("status.done")?;
        let total = self.tasks.len();
        let mut finished = 0;
        let mut passed = 0;
        let mut evidences = 0;
        for task in &self.tasks {
            finished += usize::from(task.status() == done);
            passed += usize::from(task.field("tests_result").as_deref() == Some("passed"));
            evidences += self.evidence_items(task)?.len();
        }
        let kpi = |value: usize, total: usize, label: &str| {
            format!("<div class=\"kpi\"><b>{value}<small> / {total}</small></b><span>{label}</span></div>\n")
        };
        Ok(format!(
            "<div class=\"kpis\">\n{}{}{}</div>\n",
            kpi(finished, total, "Tarefas concluídas"),
            kpi(passed, total, "Tarefas com testes aprovados"),
            kpi(evidences, evidences, "Evidências geradas")
        ))
    }

    /// Linha do tempo: uma tarefa por item, com a data da conclusão tirada do registro.
    fn timeline_html(&self) -> Result<String> {
        let mut out = String::new();
        for task in &self.tasks {
            let date = self.task_done_date(task)?;
            let (class, date) = if date.is_empty() {
                (" class=\"pending\"", "sem data".to_string())
            } else {
                ("", date)
            };
            out.push_str(&format!(
                "      <li{class}><span class=\"date\">{date}</span><span class=\"task\"><span class=\"id\">{}</span>{}</span></li>\n",
                html_escape(&task.id()),
                html_escape(&task.title())
            ));
        }
        Ok(out)
    }

    /// Requisitos cobertos pelas tarefas, na ordem da primeira aparição, com as tarefas de cada um.
    fn requirement_map(&self) -> Vec<(String, Vec<String>)> {
        let mut map: Vec<(String, Vec<String>)> = Vec::new();
        for task in &self.tasks {
            let id = task.id();
            let covers = task
                .field("covers")
                .unwrap_or_default()
                .replace(['[', ']'], "");
            for requirement in covers
                .split(',')
                .map(|r| r.trim())
                .filter(|r| !r.is_empty())
            {
                match map.iter_mut().find(|(req, _)| req == requirement) {
                    Some((_, tasks)) => tasks.push(id.clone()),
                    None => map.push((requirement.to_string(), vec![id.clone()])),
                }
            }
        }
        map
    }

    fn requirements_html(&self) -> Result<String> {
        let map = self.requirement_map();
        if map.is_empty() {
            return Ok(
                "<p class=\"na\">Não se aplica. Nenhum requisito coberto pelas tarefas.</p>\n"
                    .to_string(),
            );
        }
        let spec = fs::read_to_string(&self.spec).unwrap_or_default();
        let mut out = String::from(
            "<div class=\"tw\"><table>\n<thead><tr><th>Requisito</th><th>Descrição</th><th>Tarefas</th></tr></thead>\n<tbody>\n",
        );
        for (requirement, tasks) in map {
            let chips: String = tasks
                .iter()
                .map(|task| format!("<span class=\"chip\">{}</span>", html_escape(task)))
                .collect();
            out.push_str(&format!(
                "<tr><td class=\"mono\">{}</td><td>{}</td><td>{chips}</td></tr>\n",
                html_escape(&requirement),
                inline_html(&requirement_text(&spec, &requirement))
            ));
        }
        out.push_str("</tbody>\n</table></div>\n");
        Ok(out)
    }

    fn files_html(&self) -> Result<String> {
        let items = self.changed_items()?;
        if items.is_empty() {
            return Ok("<p class=\"na\">Não se aplica. Nenhum arquivo alterado.</p>\n".to_string());
        }
        let mut out = String::from(
            "<div class=\"tw\"><table>\n<thead><tr><th>Ação</th><th>Caminho</th></tr></thead>\n<tbody>\n",
        );
        for (action, path) in items {
            let action = html_escape(&action);
            out.push_str(&format!(
                "<tr><td><span class=\"tag {action}\">{action}</span></td><td class=\"path\">{}</td></tr>\n",
                html_escape(&path)
            ));
        }
        out.push_str("</tbody>\n</table></div>\n");
        Ok(out)
    }

    fn commits_html(&self) -> Result<String> {
        let lines = self.commit_lines()?;
        let rows: Vec<String> = lines
            .iter()
            .filter_map(|line| commit_line(line))
            .map(|(hash, date, message)| {
                format!(
                    "<tr><td class=\"mono\">{hash}</td><td class=\"mono\">{date}</td><td>{}</td></tr>\n",
                    html_escape(message)
                )
            })
            .collect();
        if rows.is_empty() {
            let first = lines.first().map(String::as_str).unwrap_or("");
            return Ok(format!(
                "<p class=\"na\">Não se aplica. {}</p>\n",
                html_escape(first)
            ));
        }
        Ok(format!(
            "<div class=\"tw\"><table>\n<thead><tr><th>Commit</th><th>Data</th><th>Mensagem</th></tr></thead>\n<tbody>\n{}</tbody>\n</table></div>\n",
            rows.concat()
        ))
    }

    /// Último commit anterior à aprovação da spec, base do diff; ou o motivo de não haver.
    fn diff_base(&self) -> std::result::Result<String, &'static str> {
        if !self.inside_git() {
            return Err("fora de um repositório git");
        }
        let approved = self.approved_at(&self.spec);
        if approved.is_empty() {
            return Err("a spec não tem data de aprovação");
        }
        let before = format!("--before={approved}");
        let base = self
            .git(&["rev-list", "-1", &before, "HEAD"])
            .map(|out| out.trim_end_matches('\n').to_string())
            .unwrap_or_default();
        if base.is_empty() {
            return Err("nenhum commit anterior à aprovação da spec");
        }
        Ok(base)
    }

    /// Caminhos com ação `alterar` que nenhuma tarefa cria, sem repetição.
    fn modified_paths(&self) -> Result<Vec<String>> {
        let items = self.changed_items()?;
        let created: Vec<&String> = items
            .iter()
            .filter(|(action, _)| action == "criar")
            .map(|(_, path)| path)
            .collect();
        let mut paths: Vec<String> = Vec::new();
        for (action, path) in &items {
            if action == "alterar" && !paths.contains(path) {
                paths.push(path.clone());
            }
        }
        paths.retain(|path| !created.contains(&path));
        Ok(paths)
    }

    /// Diffs lado a lado de cada arquivo alterado, entre a base e o estado atual.
    fn diffs_html(&self) -> Result<String> {
        let base = match self.diff_base() {
            Ok(base) => base,
            Err(reason) => {
                return Ok(format!(
                    "<p class=\"na\">Não se aplica. Sem base de comparação: {}.</p>\n",
                    html_escape(reason)
                ))
            }
        };
        let paths = self.modified_paths()?;
        if paths.is_empty() {
            return Ok(
                "<p class=\"na\">Não se aplica. Nenhum arquivo alterado, só criados ou removidos.</p>\n"
                    .to_string(),
            );
        }
        let mut out = String::new();
        for path in paths {
            out.push_str(&self.file_diff_html(&base, &path));
        }
        Ok(out)
    }

    fn file_diff_html(&self, base: &str, path: &str) -> String {
        let stat = self
            .git(&[
                "-c",
                "core.quotepath=off",
                "diff",
                "--numstat",
                "--no-color",
                "--no-ext-diff",
                base,
                "--",
                path,
            ])
            .unwrap_or_default();
        let stat = stat.lines().next().unwrap_or("");
        let mut columns = stat.split('\t');
        let added = columns.next().unwrap_or("");
        let removed = columns.next().unwrap_or(added);
        let escaped = html_escape(path);
        let mut out = String::from("<details class=\"dfile\">\n");
        if stat.is_empty() {
            out.push_str(&format!("<summary><b class=\"path\">{escaped}</b><span class=\"tag\">sem mudanças</span></summary>\n"));
            out.push_str(
                "<div class=\"dbody\"><p>Sem mudanças desde a base de comparação.</p></div>\n",
            );
        } else if added == "-" {
            out.push_str(&format!("<summary><b class=\"path\">{escaped}</b><span class=\"tag\">binário</span></summary>\n"));
            out.push_str("<div class=\"dbody\"><p>Arquivo binário alterado.</p></div>\n");
        } else {
            out.push_str(&format!("<summary><b class=\"path\">{escaped}</b><span class=\"tag criar\">+{added}</span><span class=\"tag remover\">−{removed}</span></summary>\n"));
            out.push_str("<div class=\"dbody\"><div class=\"tw\"><table class=\"diff\">\n");
            out.push_str("<colgroup><col class=\"cln\"><col><col class=\"cln\"><col></colgroup>\n");
            let diff = self
                .git(&[
                    "-c",
                    "core.quotepath=off",
                    "diff",
                    "--no-color",
                    "--no-ext-diff",
                    "--no-textconv",
                    "-U3",
                    base,
                    "--",
                    path,
                ])
                .unwrap_or_default();
            out.push_str(&diff_rows(&diff));
            out.push_str("</table></div></div>\n");
        }
        out.push_str("</details>\n");
        out
    }
}

/// Pares chave e valor de um grupo de primeiro nível de um arquivo de configuração, na ordem do arquivo.
fn config_group(path: &Path, group: &str, errors: &mut String) -> Vec<(String, String)> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) => {
            errors.push_str(&awk_open_error(path, &error));
            return Vec::new();
        }
    };
    let mut pairs = Vec::new();
    let mut in_group = false;
    for line in content.lines() {
        let trimmed = line.trim_start_matches([' ', '\t']);
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if !line.starts_with([' ', '\t']) {
            in_group = opens_group(line, group);
            continue;
        }
        if !in_group {
            continue;
        }
        let key = trimmed
            .split(':')
            .next()
            .unwrap_or("")
            .trim_end_matches([' ', '\t']);
        let value = trimmed.split_once(':').map_or(trimmed, |(_, value)| value);
        pairs.push((key.to_string(), config_value(value)));
    }
    pairs
}

fn opens_group(line: &str, group: &str) -> bool {
    line.strip_prefix(group)
        .and_then(|rest| rest.strip_prefix(':'))
        .is_some_and(|rest| {
            let rest = rest.trim_start_matches([' ', '\t']);
            rest.is_empty() || rest.starts_with('#')
        })
}

/// Valor sem comentário no fim, sem espaços nas pontas e sem aspas externas.
fn config_value(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut end = raw.len();
    for (index, byte) in bytes.iter().enumerate() {
        if *byte == b'#' && index > 0 && matches!(bytes[index - 1], b' ' | b'\t') {
            end = raw[..index].trim_end_matches([' ', '\t']).len();
            break;
        }
    }
    let value = awk_trim(&raw[..end]);
    let quoted = value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')));
    if quoted {
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    }
}

/// Descrição de um requisito: a segunda célula da primeira linha cuja primeira célula é o ID.
fn requirement_text(spec: &str, requirement: &str) -> String {
    spec.lines()
        .map(strip_cr)
        .find_map(|line| {
            let cells: Vec<&str> = line.split('|').collect();
            (awk_trim(cells.get(1).copied().unwrap_or("")) == requirement)
                .then(|| awk_trim(cells.get(2).copied().unwrap_or("")).to_string())
        })
        .unwrap_or_default()
}

/// Linha "- `hash` data mensagem" do bloco de commits.
fn commit_line(line: &str) -> Option<(&str, &str, &str)> {
    let rest = line.strip_prefix("- `")?;
    let end = rest.find('`')?;
    let hash = &rest[..end];
    if hash.is_empty()
        || !hash
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return None;
    }
    let rest = rest[end + 1..].strip_prefix(' ')?;
    let date_len = rest
        .bytes()
        .take_while(|b| b.is_ascii_digit() || *b == b'-')
        .count();
    let message = rest[date_len..].strip_prefix(' ')?;
    (date_len > 0).then(|| (hash, &rest[..date_len], message))
}

/// Número no início do texto, como a conversão numérica do awk; zero sem dígitos.
fn leading_number(text: &str) -> i64 {
    let digits: String = text.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().unwrap_or(0)
}

fn diff_cell(number: i64, text: &str, class: &str) -> String {
    format!(
        "<td class=\"ln\">{number}</td><td class=\"code{class}\">{}</td>",
        html_escape(text)
    )
}

const EMPTY_DIFF_CELL: &str = "<td class=\"ln\"></td><td class=\"code empty\"></td>";

fn flush_changes(
    out: &mut String,
    removed: &mut Vec<(i64, String)>,
    added: &mut Vec<(i64, String)>,
) {
    for index in 0..removed.len().max(added.len()) {
        let left = removed
            .get(index)
            .map_or(EMPTY_DIFF_CELL.to_string(), |(n, t)| {
                diff_cell(*n, t, " del")
            });
        let right = added
            .get(index)
            .map_or(EMPTY_DIFF_CELL.to_string(), |(n, t)| {
                diff_cell(*n, t, " add")
            });
        push_line(out, &format!("<tr>{left}{right}</tr>"));
    }
    removed.clear();
    added.clear();
}

/// Texto do cabeçalho de um trecho depois de `@@ ... @@ `, ou a linha inteira se não houver o par.
fn hunk_context(line: &str) -> &str {
    let rest = &line[3..];
    match rest.find('@') {
        Some(at) if rest[at..].starts_with("@@") => {
            let after = &rest[at + 2..];
            after.strip_prefix(' ').unwrap_or(after)
        }
        _ => line,
    }
}

/// Converte um diff unificado em linhas de tabela de duas colunas (antes e depois), com remoções e
/// acréscimos seguidos lado a lado. Todo o código sai escapado.
fn diff_rows(diff: &str) -> String {
    let mut out = String::new();
    let mut in_hunk = false;
    let (mut old_line, mut new_line) = (0, 0);
    let mut removed = Vec::new();
    let mut added = Vec::new();
    for line in diff.lines().map(strip_cr) {
        if line.starts_with("@@ ") {
            flush_changes(&mut out, &mut removed, &mut added);
            in_hunk = true;
            let fields: Vec<&str> = line.split([' ', '\t']).filter(|f| !f.is_empty()).collect();
            let field = |index: usize| fields.get(index).copied().unwrap_or("");
            let old = field(1);
            old_line = leading_number(old.strip_prefix('-').unwrap_or(old));
            let new = field(2);
            new_line = leading_number(new.strip_prefix('+').unwrap_or(new));
            let context = hunk_context(line);
            let mut title = format!("{} {} {} {}", field(0), field(1), field(2), field(3));
            if !context.is_empty() {
                title.push(' ');
                title.push_str(context);
            }
            push_line(
                &mut out,
                &format!(
                    "<tr class=\"hunk\"><td colspan=\"4\">{}</td></tr>",
                    html_escape(&title)
                ),
            );
            continue;
        }
        if !in_hunk || line.starts_with('\\') {
            continue;
        }
        let text: &str = line.char_indices().nth(1).map_or("", |(i, _)| &line[i..]);
        if line.starts_with('-') {
            removed.push((old_line, text.to_string()));
            old_line += 1;
        } else if line.starts_with('+') {
            added.push((new_line, text.to_string()));
            new_line += 1;
        } else {
            flush_changes(&mut out, &mut removed, &mut added);
            push_line(
                &mut out,
                &format!(
                    "<tr>{}{}</tr>",
                    diff_cell(old_line, text, ""),
                    diff_cell(new_line, text, "")
                ),
            );
            old_line += 1;
            new_line += 1;
        }
    }
    flush_changes(&mut out, &mut removed, &mut added);
    out
}

// --- Documento de evidências ---

impl Finalization {
    /// Gera o documento de evidências; as que não são embutidas são montadas ao lado e só trocam
    /// `assets/` depois do documento gerado. Devolve quantas ficaram fora do documento.
    fn build_evidence(&self) -> Result<usize> {
        let out = self.document_path("evidence")?;
        let dir = out.parent().unwrap_or(Path::new("")).to_path_buf();
        let stage = dir.join(".assets.new");
        fs::create_dir_all(&dir)?;
        remove_path(&stage)?;
        let mut counts = MediaCounts::default();
        if let Err(error) = self.generate(&out, |errors| {
            self.evidence_html(&stage, &mut counts, errors)
        }) {
            remove_path(&stage)?;
            return Err(error);
        }
        let assets = dir.join(ASSETS_DIR);
        remove_path(&assets)?;
        if stage.join(ASSETS_DIR).is_dir() {
            fs::rename(stage.join(ASSETS_DIR), &assets)?;
        }
        remove_path(&stage)?;
        Ok(counts.assets)
    }

    fn evidence_html(
        &self,
        stage: &Path,
        counts: &mut MediaCounts,
        errors: &mut String,
    ) -> Result<Vec<u8>> {
        let content = fs::read_to_string(self.content_path()?).unwrap_or_default();
        let Some(tokens) = self.css_tokens(errors) else {
            return Ok(Vec::new());
        };
        let header = self.header()?;
        let templates = self.templates_dir()?;
        let mut items: Vec<u8> = Vec::new();
        let mut nav = String::new();
        let mut total = 0;
        for task in &self.tasks {
            let task_id = task.id();
            let task_title = task.title();
            let evidence_dir = self.task_evidence_dir(task)?;
            let slug = task_id.to_ascii_lowercase();
            nav.push_str(&format!(
                "    <a href=\"#p-{slug}\" role=\"tab\" data-tab=\"{slug}\">{}</a>\n",
                html_escape(&task_id)
            ));
            items.extend(format!(
                "<section class=\"panel\" id=\"p-{slug}\" role=\"tabpanel\" data-panel=\"{slug}\">\n<div class=\"sec-h\"><h2>{} — {}</h2></div>\n",
                html_escape(&task_id),
                html_escape(&task_title)
            ).into_bytes());
            let evidences = self.evidence_items(task)?;
            let mut notes = self.evidence_notes(task)?.into_iter();
            for (index, (name, kind)) in evidences.iter().enumerate() {
                total += 1;
                let note = notes.next().unwrap_or_default();
                let media = self.media_html(
                    &evidence_dir.join(name),
                    &task_id,
                    name,
                    stage,
                    counts,
                    errors,
                )?;
                let heading = evidence_heading_in(&content, &task_id, index, name)
                    .unwrap_or_else(|| legacy_evidence_heading(&task_id, name));
                let label = html_escape(&format!("EV-{:02} {name}", index + 1));
                let how = evidence_field(&content, &heading, EVIDENCE_HOW);
                let reach = match log_command(&evidence_dir.join(name)) {
                    Some(command) => format!("<code>{}</code>", html_escape(&command)),
                    None => inline_html(&how),
                };
                let values: HashMap<&str, String> = HashMap::from([
                    ("TASK_ID", html_escape(&task_id)),
                    ("TASK_TITLE", html_escape(&task_title)),
                    ("NAME", label.clone()),
                    ("KIND", html_escape(kind)),
                    (
                        "KIND_CLASS",
                        if kind == "manual" { "manual" } else { "auto" }.to_string(),
                    ),
                    ("BEFORE", self.before_html(&evidence_dir, name)),
                    ("MEDIA", MEDIA_MARKER.to_string()),
                    (
                        "HOW",
                        inline_html(&evidence_field(&content, &heading, EVIDENCE_HOW)),
                    ),
                    (
                        "READ",
                        inline_html(&evidence_field(&content, &heading, EVIDENCE_READ)),
                    ),
                    ("PROOF", proof_html(&content, &heading)),
                    (
                        "TITLE",
                        if note.title.is_empty() {
                            label.clone()
                        } else {
                            html_escape(&note.title)
                        },
                    ),
                    ("FILE", label.clone()),
                    ("SHOWS", shows_html(&note, &content, &heading)),
                    ("WHERE", where_html(&note)),
                    ("FAILURE", note_html("Falha", &note.failure)),
                    ("REACH", reach),
                ]);
                let template = read_template(&templates.join(EVIDENCE_ITEM_TEMPLATE), errors);
                items.extend(splice(
                    &render_lines(&template, &values),
                    MEDIA_MARKER,
                    &media,
                ));
            }
            let accepted = accepted_regressions(&evidence_dir.join("validation/regression"));
            for item in &accepted {
                let diff = fs::read(
                    evidence_dir
                        .join("validation/regression")
                        .join(accepted_diff_name(item.number)),
                )
                .unwrap_or_default();
                items.extend(format!(
                    "<figure class=\"piece\">\n  <div class=\"piece-top\"><span class=\"piece-id\">{}</span> — {}</div>\n  <div class=\"piece-meta\"><b>Diferença de regressão aceita: {}</b><span class=\"tag manual\">aceita</span></div>\n  <div class=\"piece-media\">\n<pre><code>",
                    html_escape(&task_id),
                    html_escape(&task_title),
                    html_escape(&item.command)
                ).into_bytes());
                items.extend(html_escape_bytes(&diff));
                items.extend(format!(
                    "</code></pre>\n  </div>\n  <figcaption class=\"piece-foot\">\n    <p><b>Decisão do usuário</b>{}</p>\n    <p><b>Aceita em</b>{}</p>\n  </figcaption>\n</figure>\n",
                    html_escape(&item.decision),
                    html_escape(&item.date)
                ).into_bytes());
            }
            if evidences.is_empty() && accepted.is_empty() {
                items.extend(format!(
                    "<div class=\"piece none\"><div class=\"piece-top\"><span class=\"piece-id\">{}</span> — {}</div><p class=\"na\">Não se aplica. Esta tarefa não gera evidência.</p></div>\n",
                    html_escape(&task_id),
                    html_escape(&task_title)
                ).into_bytes());
            }
            items.extend_from_slice(b"</section>\n");
        }
        let summary = format!(
            "<p><strong>{total} evidência(s)</strong> em <strong>{} tarefa(s)</strong>: {} embutida(s) no documento e {} na pasta <code>{ASSETS_DIR}/</code>, ao lado dele.</p>",
            self.tasks.len(),
            counts.embedded,
            counts.assets
        );
        let part = captured(&content_part(&content, "Evidências"));
        let written = section_html(&part, "Em resumo");
        let summary = if written.trim().is_empty() {
            summary
        } else {
            written
        };
        let coverage = section_html(&part, COVERAGE_SECTION);
        let (coverage, coverage_nav) = if coverage.trim().is_empty() {
            (String::new(), String::new())
        } else {
            (
                format!(
                    "<section class=\"panel\" id=\"p-cobertura\">\n<div class=\"sec-h\"><h2>{COVERAGE_SECTION}</h2></div>\n{coverage}\n</section>"
                ),
                "    <a href=\"#p-cobertura\">Cobertura</a>".to_string(),
            )
        };
        let mut values = self.header_values(&header, "Evidências", tokens, errors);
        values.extend([
            ("SUMMARY", summary),
            ("COVERAGE", coverage),
            ("COVERAGE_NAV", coverage_nav),
            ("NAV", nav),
            ("ITEMS", ITEMS_MARKER.to_string()),
        ]);
        let template = read_template(&templates.join(EVIDENCE_TEMPLATE), errors);
        let html = render_lines(&template, &values.into_iter().collect());
        Ok(splice(&html, ITEMS_MARKER, &items))
    }

    /// Saída da evidência coletada antes da tarefa, vazia se não houve baseline. As duas primeiras
    /// linhas do arquivo guardam o comando e o código de saída e não entram no documento.
    fn before_html(&self, evidence_dir: &Path, name: &str) -> String {
        let Ok(baseline_dir) = self.task_setting("evidence_baseline_dir") else {
            return String::new();
        };
        let Ok(recorded) = fs::read(evidence_dir.join("baseline").join(baseline_dir).join(name))
        else {
            return String::new();
        };
        let output: Vec<u8> = recorded
            .splitn(3, |&byte| byte == b'\n')
            .nth(2)
            .unwrap_or_default()
            .to_vec();
        let escaped = String::from_utf8_lossy(&html_escape_bytes(&output)).into_owned();
        format!(
            "<p class=\"piece-label\">Antes da tarefa</p><pre><code>{escaped}</code></pre><p class=\"piece-label\">Depois da tarefa</p>"
        )
    }

    /// HTML da mídia de uma evidência. Imagem e vídeo até o limite vão em base64 e texto, escapado num
    /// bloco de código; o resto é copiado para `<montagem>/assets/<TASK_ID>/` e citado por link.
    fn media_html(
        &self,
        source: &Path,
        task_id: &str,
        name: &str,
        stage: &Path,
        counts: &mut MediaCounts,
        errors: &mut String,
    ) -> Result<Vec<u8>> {
        let config = self.documents_config();
        let kind = media_type(&config, name, errors);
        let size = fs::metadata(source).map(|meta| meta.len()).unwrap_or(0);
        let limit_mb = config_get(&config, "embed_max_mb").unwrap_or_else(|error| {
            errors.push_str(&format!("{error}\n"));
            String::new()
        });
        let limit = (limit_mb.parse::<f64>().unwrap_or(0.0) * 1_048_576.0) as u64;
        let reason = if kind == MediaType::Other {
            Some("tipo sem visualização no documento".to_string())
        } else if size > limit {
            Some(format!("acima do limite de {limit_mb} MB para embutir"))
        } else {
            None
        };
        let Some(reason) = reason else {
            counts.embedded += 1;
            let bytes = fs::read(source)
                .with_context(|| format!("não foi possível ler {}", source.display()))?;
            return Ok(embedded_media(kind, name, &bytes));
        };
        counts.assets += 1;
        let target_dir = stage.join(ASSETS_DIR).join(task_id);
        fs::create_dir_all(&target_dir)?;
        fs::copy(source, target_dir.join(name))
            .with_context(|| format!("não foi possível copiar {}", source.display()))?;
        let link = format!("{ASSETS_DIR}/{task_id}/{name}");
        Ok(format!(
            "<p class=\"asset\"><a href=\"{}\">{}</a><small>{}, {reason}; abre ao lado do documento.</small></p>",
            html_escape(&url_path(&link)),
            html_escape(name),
            human_size(size)
        )
        .into_bytes())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MediaType {
    Text,
    Image,
    Video,
    Other,
}

/// Tipo de uma evidência pela extensão do nome, segundo as listas de `documents.yml`.
fn media_type(config: &Path, name: &str, errors: &mut String) -> MediaType {
    let Some((_, extension)) = name.rsplit_once('.') else {
        return MediaType::Other;
    };
    let extension = extension.to_ascii_lowercase();
    for (key, kind) in [
        ("text_extensions", MediaType::Text),
        ("image_extensions", MediaType::Image),
        ("video_extensions", MediaType::Video),
    ] {
        match config_list(config, key) {
            Ok(list) if list.contains(&extension) => return kind,
            Ok(_) => {}
            Err(error) => errors.push_str(&format!("{error}\n")),
        }
    }
    MediaType::Other
}

fn mime_type(name: &str) -> &'static str {
    let extension = name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match extension.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        _ => "application/octet-stream",
    }
}

fn embedded_media(kind: MediaType, name: &str, bytes: &[u8]) -> Vec<u8> {
    match kind {
        MediaType::Text => [
            b"<pre><code>".as_slice(),
            &html_escape_bytes(bytes),
            b"</code></pre>",
        ]
        .concat(),
        MediaType::Image => format!(
            "<img alt=\"{}\" src=\"data:{};base64,{}\">",
            html_escape(name),
            mime_type(name),
            base64(bytes)
        )
        .into_bytes(),
        MediaType::Video => format!(
            "<video controls preload=\"metadata\" src=\"data:{};base64,{}\"></video>",
            mime_type(name),
            base64(bytes)
        )
        .into_bytes(),
        MediaType::Other => Vec::new(),
    }
}

/// Tamanho máximo do ícone da marca embutido no topo dos documentos.
const BRAND_ICON_MAX_BYTES: usize = 256 * 1024;

fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let group = chunk.iter().enumerate().fold(0u32, |acc, (index, byte)| {
            acc | u32::from(*byte) << (16 - 8 * index)
        });
        for index in 0..4 {
            if index <= chunk.len() {
                out.push(char::from(
                    ALPHABET[(group >> (18 - 6 * index) & 63) as usize],
                ));
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// Tamanho legível: KB ou MB, com uma casa decimal.
fn human_size(bytes: u64) -> String {
    let bytes = bytes as f64;
    if bytes >= 1_048_576.0 {
        format!("{:.1} MB", bytes / 1_048_576.0)
    } else {
        format!("{:.1} KB", bytes / 1024.0)
    }
}

/// Caminho relativo seguro para um `href`: `%`, espaço, `#` e `?` codificados.
fn url_path(path: &str) -> String {
    path.replace('%', "%25")
        .replace(' ', "%20")
        .replace('#', "%23")
        .replace('?', "%3F")
}

/// Texto de um campo ("Como chegar" ou "Como ler") do bloco `## <TASK_ID> / <nome>` do conteúdo.
fn evidence_field(content: &str, heading: &str, label: &str) -> String {
    let prefix = format!("- **{label}:** ");
    let mut inside = false;
    for line in content.lines().map(strip_cr) {
        if line.starts_with("# ") || line.starts_with("## ") {
            inside = line == heading;
        } else if inside {
            if let Some(value) = line.strip_prefix(&prefix) {
                return value.to_string();
            }
        }
    }
    String::new()
}

// --- Arquivamento ---

impl Finalization {
    fn archive(&self) -> Result<()> {
        let problems = self.archive_problems()?;
        if !problems.is_empty() {
            bail!(problem_list(
                &format!("a spec {} não pode ser arquivada:", self.id),
                &problems
            ));
        }
        let archive = self.archive_dir()?;
        let docs = self.spec_docs_dir()?;
        if let Err(stderr) = self.copy_to_archive(&archive, &docs) {
            remove_path(&archive)?;
            remove_path(&docs)?;
            return Err(Reported(format!(
                "{stderr}falha ao copiar para o arquivo; nada foi removido\n"
            ))
            .into());
        }
        remove_path(&self.spec)?;
        remove_path(&self.tasks_dir)?;
        remove_path(&self.content_dir()?)?;
        println!(
            "Spec {} arquivada em {}, com status 'archived'.",
            self.id,
            self.rel(&archive)
        );
        println!("Executivo e técnico publicados em {}.", self.rel(&docs));
        Ok(())
    }

    fn archive_problems(&self) -> Result<Vec<String>> {
        let mut problems = Vec::new();
        for kind in ["executive", "technical", "evidence"] {
            let document = self.document_path(kind)?;
            if !document.is_file() {
                problems.push(format!(
                    "documento ausente: {}; gere com vostok finalize build",
                    self.rel(&document)
                ));
            }
        }
        let archive = self.archive_dir()?;
        if archive.exists() {
            problems.push(format!(
                "o destino do arquivo já existe: {}",
                self.rel(&archive)
            ));
        }
        let docs = self.spec_docs_dir()?;
        if docs.exists() {
            problems.push(format!(
                "o destino dos documentos já existe: {}",
                self.rel(&docs)
            ));
        }
        Ok(problems)
    }

    /// Monta `<arquivo>/<SPEC_ID>/` (spec, conteúdo, tarefas e documentos), publica o executivo e o
    /// técnico e grava o status `archived`. Em falha, devolve a saída de erro das ferramentas.
    fn copy_to_archive(&self, archive: &Path, docs: &Path) -> std::result::Result<(), String> {
        let created: Vec<String> = [
            archive.join("tasks"),
            archive.join("docs"),
            docs.to_path_buf(),
        ]
        .iter()
        .filter_map(|dir| mkdir_p(dir).err())
        .collect();
        if !created.is_empty() {
            return Err(created.concat());
        }
        let documents = self.archive_documents().map_err(|e| format!("{e}\n"))?;
        let content = self.content_path().map_err(|e| format!("{e}\n"))?;
        copy_file(&self.spec, &archive.join("spec.md"))?;
        copy_file(&content, &archive.join("content.md"))?;
        copy_dir(&self.tasks_dir, &archive.join("tasks"))?;
        for document in &documents {
            copy_file(document, &archive.join("docs").join(file_name(document)))?;
        }
        let assets = self
            .content_dir()
            .map_err(|e| format!("{e}\n"))?
            .join(ASSETS_DIR);
        if assets.is_dir() {
            copy_dir(&assets, &archive.join("docs").join(ASSETS_DIR))?;
        }
        for document in &documents[..2] {
            copy_file(document, &docs.join(file_name(document)))?;
        }
        let spec = archive.join("spec.md");
        let archived = self
            .spec_setting("status.archived")
            .map_err(|e| format!("{e}\n"))?;
        frontmatter_put(&spec, "status", &archived).map_err(|e| format!("{e}\n"))?;
        frontmatter_put(&spec, "updated_at", &utc_datetime()).map_err(|e| format!("{e}\n"))?;
        crate::commands::spec::record_status(&spec, &archived).map_err(|e| format!("{e}\n"))
    }

    fn archive_documents(&self) -> Result<Vec<PathBuf>> {
        ["executive", "technical", "evidence"]
            .iter()
            .map(|kind| self.document_path(kind))
            .collect()
    }
}

/// `mkdir -p`, com a mensagem do `mkdir` para o primeiro trecho do caminho que não pôde ser criado.
fn mkdir_p(path: &Path) -> std::result::Result<(), String> {
    let mut prefix = PathBuf::new();
    for component in path.components() {
        prefix.push(component);
        if prefix.is_dir() {
            continue;
        }
        let failure = if prefix.exists() {
            "Not a directory".to_string()
        } else {
            match fs::create_dir(&prefix) {
                Ok(()) => continue,
                Err(error) => strerror(&error),
            }
        };
        return Err(format!(
            "mkdir: cannot create directory ‘{}’: {failure}\n",
            prefix.display()
        ));
    }
    Ok(())
}

fn copy_file(source: &Path, target: &Path) -> std::result::Result<(), String> {
    fs::copy(source, target).map(|_| ()).map_err(|error| {
        format!(
            "cp: cannot copy '{}' to '{}': {}\n",
            source.display(),
            target.display(),
            strerror(&error)
        )
    })
}

/// Copia o conteúdo de uma pasta, inclusive arquivos ocultos e pastas vazias.
fn copy_dir(source: &Path, target: &Path) -> std::result::Result<(), String> {
    let error = |path: &Path, error: io::Error| {
        format!(
            "cp: cannot copy '{}': {}\n",
            path.display(),
            strerror(&error)
        )
    };
    fs::create_dir_all(target).map_err(|e| error(target, e))?;
    for entry in fs::read_dir(source).map_err(|e| error(source, e))? {
        let entry = entry.map_err(|e| error(source, e))?;
        let path = entry.path();
        let destination = target.join(entry.file_name());
        if path.is_dir() {
            copy_dir(&path, &destination)?;
        } else {
            copy_file(&path, &destination)?;
        }
    }
    Ok(())
}

/// Remove um arquivo ou uma pasta inteira; não falha se o caminho não existir.
fn remove_path(path: &Path) -> Result<()> {
    let result = match fs::symlink_metadata(path) {
        Ok(meta) if meta.is_dir() => fs::remove_dir_all(path),
        Ok(_) => fs::remove_file(path),
        Err(_) => return Ok(()),
    };
    result.with_context(|| format!("não foi possível remover {}", path.display()))
}

/// Troca o valor de uma chave do front-matter ou a acrescenta no fim dele, se não existir.
fn frontmatter_put(path: &Path, key: &str, value: &str) -> Result<()> {
    if frontmatter_set(path, key, value).is_ok() {
        return Ok(());
    }
    let content = fs::read_to_string(path)?;
    let mut lines: Vec<String> = content.lines().map(str::to_string).collect();
    let closing = lines
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, line)| line.trim_end_matches('\r') == "---")
        .map(|(index, _)| index)
        .filter(|_| {
            lines
                .first()
                .is_some_and(|first| first.trim_end_matches('\r') == "---")
        });
    let Some(closing) = closing else {
        bail!("Front-matter não encontrado em {}", path.display());
    };
    lines.insert(closing, format!("{key}: {value}"));
    let mut out = lines.join("\n");
    out.push('\n');
    fs::write(path, out)?;
    Ok(())
}

// --- Datas ---

fn unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or(0)
}

/// Data civil (ano, mês, dia) de um número de dias desde 1970-01-01.
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

/// Dias desde 1970-01-01 de uma data civil; inverso de `civil_from_days`.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = year - i64::from(month <= 2);
    let era = year.div_euclid(400);
    let year_of_era = year.rem_euclid(400);
    let month_index = if month > 2 { month - 3 } else { month + 9 };
    let day_of_year = (153 * month_index + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// Segundos desde 1970-01-01 de um `AAAA-MM-DDTHH:MM:SSZ`; `None` em qualquer outro formato.
fn seconds_of(text: &str) -> Option<i64> {
    let bytes = text.as_bytes();
    if bytes.len() != 20 {
        return None;
    }
    let separators = [
        (4, b'-'),
        (7, b'-'),
        (10, b'T'),
        (13, b':'),
        (16, b':'),
        (19, b'Z'),
    ];
    if separators.iter().any(|&(at, byte)| bytes[at] != byte) {
        return None;
    }
    let number = |from: usize, to: usize| -> Option<i64> {
        let part = &text[from..to];
        part.bytes()
            .all(|b| b.is_ascii_digit())
            .then(|| part.parse().ok())
            .flatten()
    };
    let (year, month, day) = (number(0, 4)?, number(5, 7)?, number(8, 10)?);
    let (hour, minute, second) = (number(11, 13)?, number(14, 16)?, number(17, 19)?);
    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 60
    {
        return None;
    }
    Some(days_from_civil(year, month, day) * 86_400 + hour * 3600 + minute * 60 + second)
}

/// Hora do dia em UTC, `HH:MM`.
fn hour_of(seconds: i64) -> String {
    let of_day = seconds.rem_euclid(86_400);
    format!("{:02}:{:02}", of_day / 3600, of_day / 60 % 60)
}

/// Duração em minutos ou em horas e minutos: `49 min`, `1 h 49 min`, `26 h 5 min`.
fn duration_text(seconds: i64) -> String {
    let minutes = seconds.max(0) / 60;
    if minutes < 60 {
        format!("{minutes} min")
    } else {
        format!("{} h {} min", minutes / 60, minutes % 60)
    }
}

fn date_of(seconds: i64) -> String {
    let (year, month, day) = civil_from_days(seconds.div_euclid(86_400));
    format!("{year:04}-{month:02}-{day:02}")
}

/// Data de hoje em UTC, `AAAA-MM-DD`.
fn utc_date() -> String {
    date_of(unix_seconds())
}

/// Data e hora atuais em UTC, `AAAA-MM-DDTHH:MM:SSZ`.
fn utc_datetime() -> String {
    let now = unix_seconds();
    let seconds = now.rem_euclid(86_400);
    format!(
        "{}T{:02}:{:02}:{:02}Z",
        date_of(now),
        seconds / 3600,
        seconds / 60 % 60,
        seconds % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_review_states_keep_the_last_decision_of_each_evidence() {
        let evidences = vec![
            ("saida.txt".to_string(), "automática".to_string()),
            ("tela.png".to_string(), "print".to_string()),
            ("outra.txt".to_string(), "automática".to_string()),
        ];
        let details = vec![
            "EV-01 saida.txt recusada pelo usuário: motivo".to_string(),
            "EV-01 saida.txt conferida pelo usuário (sha256 abc)".to_string(),
            "EV-02 tela.png recusada pelo usuário: cortada".to_string(),
        ];
        let states = legacy_review_states(&details, &evidences);
        assert_eq!(
            states.get("saida.txt").map(String::as_str),
            Some("conferida")
        );
        assert_eq!(states.get("tela.png").map(String::as_str), Some("recusada"));
        assert_eq!(states.get("outra.txt"), None);
    }

    #[test]
    fn civil_dates_follow_the_calendar() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(20_727), (2026, 10, 1));
        assert_eq!(civil_from_days(11_016), (2000, 2, 29));
    }

    #[test]
    fn base64_pads_the_last_group() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn read_fields_collapses_tabs_like_bash() {
        assert_eq!(read_fields("T\t", 3), ["T", "", ""]);
        assert_eq!(read_fields("F\tValor\t\tx\ty", 3), ["F", "Valor", "x\ty"]);
    }

    #[test]
    fn markdown_keeps_unpaired_backticks_and_escapes() {
        assert_eq!(
            inline_html("a `b` **c** <d>"),
            "a <code>b</code> <strong>c</strong> &lt;d&gt;"
        );
        assert_eq!(inline_html("só ` uma"), "só ` uma");
        assert_eq!(inline_html(""), "");
    }

    #[test]
    fn markdown_tables_lists_and_paragraphs() {
        let html =
            block_html("| A | B |\n|---|---|\n| 1 | 2 |\n\n- x\n- y\n1. z\nlinha\n  junta\n");
        assert_eq!(
            html,
            "<div class=\"tw\"><table>\n<thead>\n<tr><th>A</th><th>B</th></tr>\n</thead>\n<tbody>\n\
             <tr><td>1</td><td>2</td></tr>\n</tbody>\n</table></div>\n<ul>\n<li>x</li>\n<li>y</li>\n</ul>\n\
             <ol>\n<li>z</li>\n</ol>\n<p>linha junta</p>\n"
        );
    }

    #[test]
    fn diff_rows_pair_removals_and_additions() {
        let diff =
            "--- a/x\n+++ b/x\n@@ -1,2 +1,2 @@ fn\n a\n-b\n+c\n+d\n\\ No newline at end of file\n";
        assert_eq!(
            diff_rows(diff),
            "<tr class=\"hunk\"><td colspan=\"4\">@@ -1,2 +1,2 @@ fn</td></tr>\n\
             <tr><td class=\"ln\">1</td><td class=\"code\">a</td><td class=\"ln\">1</td><td class=\"code\">a</td></tr>\n\
             <tr><td class=\"ln\">2</td><td class=\"code del\">b</td><td class=\"ln\">2</td><td class=\"code add\">c</td></tr>\n\
             <tr><td class=\"ln\"></td><td class=\"code empty\"></td><td class=\"ln\">3</td><td class=\"code add\">d</td></tr>\n"
        );
    }

    #[test]
    fn delivery_errors_report_missing_fields_and_bad_impact() {
        let section = "### Uma\n- **Impacto:** enorme\n- **O que mudou:** x\n";
        assert_eq!(
            delivery_errors(section),
            [
                "impacto inválido em ### Uma: 'enorme' (use alto, médio ou baixo)",
                "entrega sem \"Valor\": ### Uma"
            ]
        );
        assert_eq!(
            delivery_errors("texto solto\n"),
            ["entregas sem nenhuma entrega: use um \"### \" por entrega"]
        );
    }

    #[test]
    fn duration_text_uses_hours_and_minutes() {
        assert_eq!(duration_text(49 * 60), "49 min");
        assert_eq!(duration_text(6565), "1 h 49 min");
        assert_eq!(duration_text(26 * 3600 + 300), "26 h 5 min");
        assert_eq!(duration_text(30), "0 min");
    }

    #[test]
    fn seconds_of_parses_utc_datetimes() {
        assert_eq!(seconds_of("1970-01-01T00:01:00Z"), Some(60));
        assert_eq!(seconds_of("2026-11-13"), None);
        assert_eq!(seconds_of("x"), None);
        assert_eq!(
            date_of(seconds_of("2026-11-14T00:26:44Z").unwrap()),
            "2026-11-14"
        );
    }
}
