//! Comandos `task`: tarefas: aprovação, ajuste, execução e conclusão.
//! Reproduzem `load-task.sh`, `check-done.sh`, `approve-task.sh`, `adjust-task.sh`, `execute-task.sh`
//! e `finish-task.sh`, com as mesmas mensagens, recusas e códigos de saída (SPEC-005, RN-03).

use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use clap::Subcommand;

use crate::harness::Harness;
use crate::support::{
    config_get, config_list, evidence_subitem_errors, frontmatter_get, frontmatter_lines,
    frontmatter_set, is_indented, markdown_section, markdown_subsection, sha256_hex,
    template_leftovers, EvidenceFields,
};

#[derive(Debug, Subcommand)]
pub enum TaskCommand {
    /// Imprime a seção de execução da tarefa.
    Load {
        spec_id: Option<String>,
        task_id: Option<String>,
    },
    /// Confere os critérios de conclusão.
    CheckDone {
        spec_id: Option<String>,
        task_id: Option<String>,
    },
    /// Aprovação de tarefas.
    #[command(subcommand)]
    Approve(ApproveCommand),
    /// Ajustes de uma tarefa.
    #[command(subcommand)]
    Adjust(AdjustCommand),
    /// Inicia a execução.
    Start {
        spec_id: Option<String>,
        task_id: Option<String>,
    },
    /// Coleta o baseline.
    Baseline {
        spec_id: Option<String>,
        task_id: Option<String>,
    },
    /// Executa a verificação.
    Tests {
        spec_id: Option<String>,
        task_id: Option<String>,
    },
    /// Compara com o baseline.
    Regression {
        spec_id: Option<String>,
        task_id: Option<String>,
    },
    /// Grava as evidências.
    Evidence {
        spec_id: Option<String>,
        task_id: Option<String>,
    },
    /// Conclui a tarefa.
    Finish {
        spec_id: Option<String>,
        task_id: Option<String>,
    },
    /// Retoma uma tarefa bloqueada com a decisão do usuário.
    Resume {
        spec_id: Option<String>,
        task_id: Option<String>,
        #[arg(allow_hyphen_values = true)]
        decision: Option<String>,
        /// Números dos comandos da regressão cuja diferença o usuário aceitou (ex.: 1,2).
        #[arg(long, value_name = "N[,N...]")]
        accept_regression: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum AdjustCommand {
    /// Registra edições feitas à mão.
    RecordManual {
        spec_id: Option<String>,
        task_id: Option<String>,
    },
    /// Valida, sem alterar nada.
    Validate {
        spec_id: Option<String>,
        task_id: Option<String>,
    },
    /// Registra o ajuste no histórico.
    Record {
        spec_id: Option<String>,
        task_id: Option<String>,
        #[arg(allow_hyphen_values = true)]
        summary: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum ApproveCommand {
    /// Lista as tarefas que podem ser aprovadas.
    Validate {
        spec_id: Option<String>,
        task_ids: Vec<String>,
        #[arg(long)]
        all: bool,
    },
    /// Aprova as tarefas, todas ou nenhuma.
    Approve {
        spec_id: Option<String>,
        task_ids: Vec<String>,
        #[arg(long)]
        all: bool,
    },
}

const LOAD_USAGE: &str = "Uso: vostok task load <SPEC_ID> <TASK_ID>";
const CHECK_DONE_USAGE: &str = "Uso: vostok task check-done <SPEC_ID> <TASK_ID>";
const APPROVE_USAGE: &str =
    "Uso: vostok task approve validate|approve <SPEC_ID> <TASK_ID ...>|--all";
const ADJUST_USAGE: &str = "Uso: vostok task adjust record-manual <SPEC_ID> <TASK_ID> | validate <SPEC_ID> <TASK_ID> | record <SPEC_ID> <TASK_ID> \"resumo\"";
const EXECUTE_USAGE: &str = "Uso: vostok task start|baseline <SPEC_ID> <TASK_ID>";
const FINISH_USAGE: &str = "Uso: vostok task tests|regression|evidence|finish <SPEC_ID> <TASK_ID> | vostok task resume <SPEC_ID> <TASK_ID> [--accept-regression <N[,N...]>] \"decisão\"";

/// Aceites de diferença de regressão, em `validation/regression/`: uma linha por aceite, com número,
/// SHA-256 da saída aceita, data, comando e decisão, separados por tabulação.
const ACCEPTED_FILE: &str = "accepted";

/// Ações aceitas em "Mudar", as mesmas dos componentes afetados da spec.
const COMPONENT_ACTIONS: [&str; 3] = ["criar", "alterar", "remover"];
const CREATE_ACTION: &str = "criar";

/// Trecho executado antes da "Verificação": o trap registra o comando que falhou, já que muitos
/// comandos de teste falham sem imprimir nada.
const VERIFY_PRELUDE: &str =
    "set -eo pipefail; trap 'echo \"comando com falha (código $?): $BASH_COMMAND\"' ERR";

pub fn run(command: TaskCommand) -> Result<()> {
    let tasks = Tasks::new(Harness::from_current_dir()?)?;
    match command {
        TaskCommand::Load { spec_id, task_id } => {
            let (spec, task) = filled(spec_id, task_id, LOAD_USAGE)?;
            tasks.load(&spec, &task)
        }
        TaskCommand::CheckDone { spec_id, task_id } => {
            let (spec, task) = filled(spec_id, task_id, CHECK_DONE_USAGE)?;
            println!("{}", tasks.check_done(&spec, &task)?);
            Ok(())
        }
        TaskCommand::Approve(command) => tasks.run_approve(command),
        TaskCommand::Adjust(command) => tasks.run_adjust(command),
        TaskCommand::Start { spec_id, task_id } => {
            let (spec, task) = given(spec_id, task_id, EXECUTE_USAGE)?;
            tasks.start(&spec, &task)
        }
        TaskCommand::Baseline { spec_id, task_id } => {
            let (spec, task) = given(spec_id, task_id, EXECUTE_USAGE)?;
            tasks.baseline(&spec, &task)
        }
        TaskCommand::Tests { spec_id, task_id } => {
            let (spec, task) = given(spec_id, task_id, FINISH_USAGE)?;
            tasks.tests(&spec, &task)
        }
        TaskCommand::Regression { spec_id, task_id } => {
            let (spec, task) = given(spec_id, task_id, FINISH_USAGE)?;
            tasks.regression(&spec, &task)
        }
        TaskCommand::Evidence { spec_id, task_id } => {
            let (spec, task) = given(spec_id, task_id, FINISH_USAGE)?;
            tasks.evidence(&spec, &task)
        }
        TaskCommand::Finish { spec_id, task_id } => {
            let (spec, task) = given(spec_id, task_id, FINISH_USAGE)?;
            tasks.finish(&spec, &task)
        }
        TaskCommand::Resume {
            spec_id,
            task_id,
            decision,
            accept_regression,
        } => {
            let (spec, task) = given(spec_id, task_id, FINISH_USAGE)?;
            let decision = decision.ok_or_else(|| usage(FINISH_USAGE))?;
            tasks.resume(&spec, &task, &decision, accept_regression.as_deref())
        }
    }
}

fn usage(text: &'static str) -> anyhow::Error {
    anyhow::Error::msg(text)
}

/// IDs presentes, mesmo vazios: os scripts contam os argumentos.
fn given(
    spec: Option<String>,
    task: Option<String>,
    text: &'static str,
) -> Result<(String, String)> {
    spec.zip(task).ok_or_else(|| usage(text))
}

/// IDs presentes e não vazios.
fn filled(
    spec: Option<String>,
    task: Option<String>,
    text: &'static str,
) -> Result<(String, String)> {
    let non_empty = |value: Option<String>| value.filter(|value| !value.is_empty());
    given(non_empty(spec), non_empty(task), text)
}

/// Status configurados em `config/tasks.yml`.
struct Statuses {
    pending: String,
    approved: String,
    in_progress: String,
    validating: String,
    blocked: String,
    done: String,
}

/// Tarefas do harness: configuração e caminhos.
struct Tasks {
    harness: Harness,
    tasks_root: PathBuf,
    status: Statuses,
}

impl Tasks {
    fn new(harness: Harness) -> Result<Self> {
        let tasks_root = harness.path("tasks")?;
        let config = harness.config_file("tasks.yml");
        let status = |name: &str| config_get(&config, &format!("status.{name}"));
        let status = Statuses {
            pending: status("pending")?,
            approved: status("approved")?,
            in_progress: status("in_progress")?,
            validating: status("validating")?,
            blocked: status("blocked")?,
            done: status("done")?,
        };
        Ok(Self {
            harness,
            tasks_root,
            status,
        })
    }

    fn cfg(&self, key: &str) -> Result<String> {
        config_get(&self.harness.config_file("tasks.yml"), key)
    }

    fn cfg_list(&self, key: &str) -> Result<Vec<String>> {
        config_list(&self.harness.config_file("tasks.yml"), key)
    }

    /// Subitens das evidências; sem `evidence_fields` na configuração, só o formato é conferido.
    fn evidence_fields(&self) -> Option<EvidenceFields> {
        Some(EvidenceFields {
            required: self.cfg_list("evidence_fields").ok()?,
            optional: self
                .cfg_list("evidence_optional_fields")
                .unwrap_or_default(),
        })
    }

    fn spec_cfg(&self, key: &str) -> Result<String> {
        config_get(&self.harness.config_file("specs.yml"), key)
    }

    /// Raiz do projeto (`paths.root`), de onde os comandos da tarefa são executados.
    fn project_root(&self) -> Result<PathBuf> {
        let root = self.harness.path("root")?;
        crate::support::existing_dir(&root)
    }

    fn task_file(&self, spec: &str, task: &str) -> PathBuf {
        self.tasks_root.join(spec).join(format!("{task}.md"))
    }

    /// Caminho da tarefa, sem validar os IDs (`load-task.sh` e `check-done.sh`).
    fn existing_task(&self, spec: &str, task: &str) -> Result<PathBuf> {
        let file = self.task_file(spec, task);
        if !file.is_file() {
            bail!(
                "tarefa {task} da spec {spec} não encontrada: {}",
                file.display()
            );
        }
        Ok(file)
    }

    /// Caminho da tarefa. Falha se os IDs forem inválidos ou a tarefa não existir.
    fn find_task(&self, spec: &str, task: &str) -> Result<PathBuf> {
        if !is_spec_id(spec) {
            bail!("ID de spec inválido: '{spec}'");
        }
        self.check_task_id(task)?;
        let file = self.task_file(spec, task);
        if !file.is_file() {
            bail!("tarefa {task} não encontrada em {spec}");
        }
        Ok(file)
    }

    fn check_task_id(&self, task: &str) -> Result<()> {
        let prefix = self.cfg("id.prefix")?;
        let valid = task
            .strip_prefix(&prefix)
            .and_then(|rest| rest.strip_prefix('-'))
            .is_some_and(is_number);
        if !valid {
            bail!("ID de tarefa inválido: '{task}'");
        }
        Ok(())
    }

    /// Texto de uma subseção de "Execução", sem as quebras de linha do fim. Vazio se ela não existir.
    fn subsection(&self, content: &str, key: &str) -> Result<String> {
        let text = markdown_subsection(content, &self.cfg("execution_section")?, &self.cfg(key)?);
        Ok(trim_newlines(&text.unwrap_or_default()).to_string())
    }

    fn evidence_dir(&self, file: &Path) -> Result<PathBuf> {
        Ok(file
            .parent()
            .unwrap_or(Path::new(""))
            .join(self.cfg("evidence_dir")?)
            .join(task_id_of(file)))
    }

    /// Título da seção de histórico de ajustes, lido da primeira linha do template da seção.
    fn history_heading(&self) -> Result<String> {
        let template = self
            .harness
            .path("templates")?
            .join(self.spec_cfg("sections_dir")?)
            .join(format!(
                "{}.md",
                self.spec_cfg("adjustment_history_section")?
            ));
        let content = read(&template)?;
        let first = content.lines().next().unwrap_or("");
        Ok(first.strip_prefix("## ").unwrap_or(first).to_string())
    }

    // --- load e check-done ---

    fn load(&self, spec: &str, task: &str) -> Result<()> {
        let file = self.existing_task(spec, task)?;
        let section = self.cfg("execution_section")?;
        let content = read(&file)?;
        let front_matter = frontmatter_lines(&content)
            .ok_or_else(|| anyhow!("a tarefa {task} não tem front-matter: {}", file.display()))?;
        let execution = markdown_section(&content, &section).ok_or_else(|| {
            anyhow!(
                "a tarefa {task} não tem a seção '## {section}': {}",
                file.display()
            )
        })?;
        print!(
            "---\n{}\n---\n\n## {section}\n{}\n",
            trim_newlines(&front_matter.join("\n")),
            trim_newlines(&execution)
        );
        Ok(())
    }

    fn check_done(&self, spec: &str, task: &str) -> Result<String> {
        let file = self.existing_task(spec, task)?;
        let sub = self.cfg("done_subsection")?;
        let content = read(&file)?;
        let text = markdown_subsection(&content, &self.cfg("execution_section")?, &sub)
            .ok_or_else(|| {
                anyhow!(
                    "a tarefa {task} não tem a subseção '### {sub}': {}",
                    file.display()
                )
            })?;
        let total = text
            .lines()
            .filter(|line| line.starts_with("- [ ] ") || line.starts_with("- [x] "))
            .count();
        if total == 0 {
            bail!("a tarefa {task} não tem critérios de conclusão em '### {sub}'");
        }
        let pending: Vec<&str> = text
            .lines()
            .filter(|line| line.starts_with("- [ ] "))
            .collect();
        if !pending.is_empty() {
            bail!(
                "a tarefa {task} tem critérios de conclusão sem marcar; confira a comprovação de cada um e marque [x]:\n{}",
                pending.join("\n")
            );
        }
        Ok(format!(
            "Todos os {total} critérios de conclusão de {task} estão marcados."
        ))
    }

    // --- approve ---

    fn run_approve(&self, command: ApproveCommand) -> Result<()> {
        let (spec, ids, all, apply) = match command {
            ApproveCommand::Validate {
                spec_id,
                task_ids,
                all,
            } => (spec_id, task_ids, all, false),
            ApproveCommand::Approve {
                spec_id,
                task_ids,
                all,
            } => (spec_id, task_ids, all, true),
        };
        let spec = spec
            .filter(|spec| !spec.is_empty())
            .ok_or_else(|| usage(APPROVE_USAGE))?;
        let files = self.selected_tasks(&spec, &ids, all)?;
        let problems = self.approval_problems(&spec, &files)?;
        if !problems.is_empty() {
            let unchanged = if apply { "; nenhuma foi alterada" } else { "" };
            bail!(
                "as tarefas de {spec} não podem ser aprovadas{unchanged}:\n{}",
                problems.join("\n")
            );
        }
        if apply {
            self.approve(&files)?;
            println!("Tarefas de {spec} aprovadas:");
        } else {
            println!("Tarefas de {spec} que podem ser aprovadas:");
        }
        for file in &files {
            println!("- {}", task_id_of(file));
        }
        Ok(())
    }

    fn selected_tasks(&self, spec: &str, ids: &[String], all: bool) -> Result<Vec<PathBuf>> {
        if !is_spec_id(spec) {
            bail!("ID de spec inválido: '{spec}'");
        }
        let dir = self.tasks_root.join(spec);
        if !dir.is_dir() {
            bail!("a spec {spec} não tem tarefas em {}", dir.display());
        }
        if all {
            if !ids.is_empty() {
                bail!("use --all ou TASK_IDs, não os dois");
            }
            let pending: Vec<PathBuf> = markdown_files(&dir)?
                .into_iter()
                .filter(|file| status_of(file) == self.status.pending)
                .collect();
            if pending.is_empty() {
                bail!(
                    "a spec {spec} não tem tarefas com status '{}'",
                    self.status.pending
                );
            }
            return Ok(pending);
        }
        if ids.is_empty() {
            bail!("informe as tarefas a aprovar ou --all (só com pedido explícito de aprovar todas)\n{APPROVE_USAGE}");
        }
        let mut files: Vec<PathBuf> = Vec::new();
        for id in ids {
            self.check_task_id(id)?;
            let file = self.task_file(spec, id);
            if !file.is_file() {
                bail!("tarefa {id} não encontrada em {spec}");
            }
            if !files.contains(&file) {
                files.push(file);
            }
        }
        Ok(files)
    }

    fn approval_problems(&self, spec: &str, files: &[PathBuf]) -> Result<Vec<String>> {
        let mut problems = Vec::new();
        for file in files {
            let id = task_id_of(file);
            let status = status_of(file);
            if status != self.status.pending {
                problems.push(format!(
                    "{id}: status '{status}'; só tarefas com status '{}' são aprovadas",
                    self.status.pending
                ));
                continue;
            }
            if !self.hash_is_current(file)? {
                problems.push(format!(
                    "{id}: edição manual sem registro; registre-a com: vostok task adjust record-manual {spec} {id}"
                ));
            }
            problems.extend(self.errors_of(spec, file)?);
        }
        Ok(problems)
    }

    fn approve(&self, files: &[PathBuf]) -> Result<()> {
        let now = utc_now();
        let hash_field = self.spec_cfg("content_hash_field")?;
        let approved_at = self.spec_cfg("approved_at_field")?;
        let approved_hash = self.spec_cfg("approved_hash_field")?;
        for file in files {
            frontmatter_set(file, "status", &self.status.approved)?;
            frontmatter_put(file, &approved_at, &now)?;
            frontmatter_put(file, &approved_hash, &frontmatter_get(file, &hash_field)?)?;
        }
        Ok(())
    }

    // --- adjust ---

    fn run_adjust(&self, command: AdjustCommand) -> Result<()> {
        match command {
            AdjustCommand::RecordManual { spec_id, task_id } => {
                let (spec, task) = given(spec_id, task_id, ADJUST_USAGE)?;
                self.record_manual(&spec, &task)
            }
            AdjustCommand::Validate { spec_id, task_id } => {
                let (spec, task) = given(spec_id, task_id, ADJUST_USAGE)?;
                self.validate(&spec, &task)
            }
            AdjustCommand::Record {
                spec_id,
                task_id,
                summary,
            } => {
                let (spec, task) = given(spec_id, task_id, ADJUST_USAGE)?;
                let summary = summary.ok_or_else(|| usage(ADJUST_USAGE))?;
                self.record(&spec, &task, &summary)
            }
        }
    }

    /// Caminho de uma tarefa que pode ser ajustada: qualquer status menos `done`.
    fn adjustable_task(&self, spec: &str, task: &str) -> Result<PathBuf> {
        let file = self.find_task(spec, task)?;
        let status = status_of(&file);
        let adjustable = [
            &self.status.pending,
            &self.status.approved,
            &self.status.in_progress,
            &self.status.validating,
            &self.status.blocked,
        ];
        if !adjustable.contains(&&status) {
            bail!(
                "a tarefa {task} está com status '{status}'; tarefas com status '{}' não são ajustadas",
                self.status.done
            );
        }
        Ok(file)
    }

    fn record_manual(&self, spec: &str, task: &str) -> Result<()> {
        let file = self.adjustable_task(spec, task)?;
        if self.hash_is_current(&file)? {
            println!("Nenhuma edição manual.");
            return Ok(());
        }
        let summary = self.manual_summary(&file)?;
        let origin = self.spec_cfg("adjustment_origins.manual")?;
        let count = self.record_adjustment(&file, &origin, &summary)?;
        println!("Ajuste manual {count} registrado em {task}: {summary}");
        self.drop_approval(&file, task)?;
        self.restart_execution(&file, task, &summary)?;
        let errors = self.errors_of(spec, &file)?;
        if !errors.is_empty() {
            println!("Problemas de validação:\n{}", errors.join("\n"));
        }
        Ok(())
    }

    fn validate(&self, spec: &str, task: &str) -> Result<()> {
        let file = self.adjustable_task(spec, task)?;
        let errors = self.errors_of(spec, &file)?;
        if !errors.is_empty() {
            bail!("a tarefa {task} tem problemas:\n{}", errors.join("\n"));
        }
        println!("Tarefa {task} válida: {}", file.display());
        Ok(())
    }

    fn record(&self, spec: &str, task: &str, summary: &str) -> Result<()> {
        let summary = clean_summary(summary)?;
        let file = self.adjustable_task(spec, task)?;
        let errors = self.errors_of(spec, &file)?;
        if !errors.is_empty() {
            bail!(
                "a tarefa {task} tem problemas; nenhum ajuste registrado:\n{}",
                errors.join("\n")
            );
        }
        let origin = self.spec_cfg("adjustment_origins.agent")?;
        let count = self.record_adjustment(&file, &origin, &summary)?;
        println!("Ajuste {count} registrado em {task}: {}", file.display());
        self.drop_approval(&file, task)?;
        self.restart_execution(&file, task, &summary)
    }

    /// Devolve uma tarefa aprovada para pending e avisa. Não faz nada nas demais.
    fn drop_approval(&self, file: &Path, task: &str) -> Result<()> {
        if status_of(file) != self.status.approved {
            return Ok(());
        }
        frontmatter_set(file, "status", &self.status.pending)?;
        frontmatter_put(file, &self.spec_cfg("approved_at_field")?, "null")?;
        frontmatter_put(file, &self.spec_cfg("approved_hash_field")?, "null")?;
        println!(
            "Aprovação de {task} derrubada: o status voltou para '{}'; aprove-a de novo antes de executar.",
            self.status.pending
        );
        Ok(())
    }

    /// Devolve uma tarefa em execução para in_progress, com tentativas e resultados zerados, mantendo o
    /// baseline e a aprovação. Não faz nada nas demais.
    fn restart_execution(&self, file: &Path, task: &str, summary: &str) -> Result<()> {
        let status = status_of(file);
        let running = [
            &self.status.in_progress,
            &self.status.validating,
            &self.status.blocked,
        ];
        if !running.contains(&&status) {
            return Ok(());
        }
        let in_progress = &self.status.in_progress;
        frontmatter_set(file, "status", in_progress)?;
        reset_results(file)?;
        self.record_step(file, "ajuste", in_progress, summary)?;
        println!(
            "Ajuste em execução: {task} voltou para '{in_progress}', com tentativas e resultados zerados e o baseline mantido."
        );
        Ok(())
    }

    // --- start e baseline ---

    fn start(&self, spec: &str, task: &str) -> Result<()> {
        let file = self.find_task(spec, task)?;
        let blockers = self.start_blockers(spec, task, &file)?;
        if !blockers.is_empty() {
            bail!(
                "a tarefa {task} não pode ser iniciada; nada foi alterado:\n{}",
                blockers.join("\n")
            );
        }
        let in_progress = &self.status.in_progress;
        frontmatter_set(&file, "status", in_progress)?;
        let approved_at = frontmatter_get(&file, "approved_at").unwrap_or_else(|_| "?".into());
        self.record_step(
            &file,
            "início",
            in_progress,
            &format!("aprovada em {approved_at}"),
        )?;
        println!(
            "Tarefa {task} iniciada: status '{in_progress}'. Colete o baseline antes de mudar qualquer arquivo: vostok task baseline {spec} {task}"
        );
        Ok(())
    }

    fn start_blockers(&self, spec: &str, task: &str, file: &Path) -> Result<Vec<String>> {
        let status = status_of(file);
        let approved = &self.status.approved;
        if status != *approved {
            return Ok(vec![format!(
                "a tarefa {task} está com status '{status}'; só tarefas com status '{approved}' são executadas. Peça a aprovação ao usuário (skill approve-task)"
            )]);
        }
        let mut blockers = Vec::new();
        let approved_hash =
            frontmatter_get(file, &self.spec_cfg("approved_hash_field")?).unwrap_or_default();
        if approved_hash != self.content_hash(&read(file)?)? {
            blockers.push(format!(
                "a tarefa {task} mudou depois da aprovação; registre o ajuste (skill adjust-task) e peça a aprovação de novo"
            ));
        }
        for dependency in dependencies_of(file) {
            let dependency_file = self.task_file(spec, &dependency);
            if !dependency_file.is_file() {
                blockers.push(format!("a dependência {dependency} não existe em {spec}"));
                continue;
            }
            let dependency_status = status_of(&dependency_file);
            if dependency_status != self.status.done {
                blockers.push(format!(
                    "a dependência {dependency} está com status '{dependency_status}'; ela precisa estar '{}' antes",
                    self.status.done
                ));
            }
        }
        Ok(blockers)
    }

    fn baseline(&self, spec: &str, task: &str) -> Result<()> {
        let file = self.find_task(spec, task)?;
        let status = status_of(&file);
        let in_progress = &self.status.in_progress;
        if status != *in_progress {
            bail!("a tarefa {task} está com status '{status}'; o baseline só é coletado com status '{in_progress}' (vostok task start)");
        }
        let dir = self.evidence_dir(&file)?.join("baseline");
        if dir.exists() {
            bail!(
                "o baseline de {task} já foi coletado em {}; ele não é sobrescrito",
                dir.display()
            );
        }
        let commands = self.baseline_commands(&read(&file)?)?;
        if commands.is_empty() {
            bail!("a tarefa {task} não tem comandos em 'Baseline'");
        }
        let root = self.project_root()?;
        // Os resultados são gravados juntos no fim: ou o baseline existe inteiro, ou não existe.
        let mut results = Vec::new();
        for command in &commands {
            results.push(recorded_run(command, &root)?);
        }
        fs::create_dir_all(&dir)?;
        for (index, (_, output)) in results.iter().enumerate() {
            fs::write(dir.join(output_name(index + 1)), output)?;
        }
        let codes = results
            .iter()
            .map(|(code, _)| code.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let detail = format!("{} comando(s); códigos de saída: {codes}", commands.len());
        self.record_step(&file, "baseline", "coletado", &detail)?;
        println!("Baseline de {task} coletado em {}: {detail}", dir.display());
        Ok(())
    }

    /// Comandos de "Baseline", sem os acentos graves.
    fn baseline_commands(&self, content: &str) -> Result<Vec<String>> {
        Ok(self
            .subsection(content, "baseline_subsection")?
            .lines()
            .filter_map(baseline_command)
            .map(str::to_string)
            .collect())
    }

    // --- tests, regression e evidence ---

    /// Recusa tarefa fora de in_progress e validating; a bloqueada só volta pelo resume.
    fn require_running(&self, file: &Path, task: &str) -> Result<()> {
        let status = status_of(file);
        if status == self.status.blocked {
            bail!("a tarefa {task} está bloqueada; mostre as falhas ao usuário e, com a decisão dele, execute: vostok task resume");
        }
        if status != self.status.in_progress && status != self.status.validating {
            bail!(
                "a tarefa {task} está com status '{status}'; só tarefas com status '{}' ou '{}' são validadas",
                self.status.in_progress,
                self.status.validating
            );
        }
        Ok(())
    }

    fn tests(&self, spec: &str, task: &str) -> Result<()> {
        let file = self.find_task(spec, task)?;
        self.require_running(&file, task)?;
        let block = verify_block(&self.subsection(&read(&file)?, "verify_subsection")?);
        if !has_text(&block) {
            bail!(
                "a tarefa {task} não tem um bloco `bash` em '{}'",
                self.cfg("verify_subsection")?
            );
        }
        let dir = self.evidence_dir(&file)?.join("validation");
        fs::create_dir_all(&dir)?;
        let output_path = dir.join("tests.out");
        let output = File::create(&output_path)?;
        let status = Command::new("bash")
            .arg("-c")
            .arg(format!("{VERIFY_PRELUDE}\n{block}"))
            .current_dir(self.project_root()?)
            .stdin(Stdio::null())
            .stdout(output.try_clone()?)
            .stderr(output)
            .status()
            .context("não foi possível executar o bash")?;
        let code = exit_code(status);
        let validating = &self.status.validating;

        if code == 0 {
            frontmatter_set(&file, "status", validating)?;
            frontmatter_set(&file, "tests_result", "passed")?;
            self.record_step(
                &file,
                "testes",
                "passed",
                "Verificação sem falhas; saída em validation/tests.out",
            )?;
            println!(
                "Testes de {task} passaram. Saída: {}",
                output_path.display()
            );
            return Ok(());
        }

        let attempts = number_or_zero(&frontmatter_get(&file, "attempts").unwrap_or_default()) + 1;
        let max_attempts = number_or_zero(&self.cfg("max_attempts")?);
        frontmatter_set(&file, "tests_result", "failed")?;
        frontmatter_set(&file, "regression_result", "null")?;
        frontmatter_set(&file, "evidence_result", "null")?;
        frontmatter_set(&file, "attempts", &attempts.to_string())?;
        println!(
            "Testes de {task} falharam (código de saída {code}), tentativa {attempts} de {max_attempts}. Últimas linhas da saída ({}):",
            output_path.display()
        );
        print!("{}", indented_tail(&fs::read(&output_path)?, 30));
        if attempts >= max_attempts {
            frontmatter_set(&file, "status", &self.status.blocked)?;
            let detail = format!("tentativa {attempts} de {max_attempts}; bloqueada");
            self.record_step(&file, "testes", "failed", &detail)?;
            bail!("a tarefa {task} ficou bloqueada depois de {attempts} tentativas. PARE: responda com o template `blocker`, mostrando as falhas, e aguarde a decisão do usuário");
        }
        frontmatter_set(&file, "status", validating)?;
        let detail = format!("tentativa {attempts} de {max_attempts}");
        self.record_step(&file, "testes", "failed", &detail)?;
        bail!("os testes de {task} falharam; corrija e execute vostok task tests de novo")
    }

    fn regression(&self, spec: &str, task: &str) -> Result<()> {
        let file = self.find_task(spec, task)?;
        self.require_running(&file, task)?;
        let evidence = self.evidence_dir(&file)?;
        let base = evidence.join("baseline");
        if !base.is_dir() {
            bail!("o baseline de {task} não foi coletado; execute: vostok task baseline {spec} {task}");
        }
        let commands = self.baseline_commands(&read(&file)?)?;
        let dir = evidence.join("validation/regression");
        clear_regression_outputs(&dir)?;
        let accepted = accepted_hashes(&dir)?;

        let root = self.project_root()?;
        let mut changed = String::new();
        let mut equal = 0;
        let mut accepted_count = 0;
        for (index, command) in commands.iter().enumerate() {
            let number = index + 1;
            let name = output_name(number);
            let (_, output) = recorded_run(command, &root)?;
            let current = dir.join(&name);
            fs::write(&current, &output)?;
            let previous = base.join(&name);
            match fs::read(&previous) {
                Err(_) => changed.push_str(&format!("- {command}: sem resultado no baseline\n")),
                Ok(expected) if expected == output => equal += 1,
                Ok(_) if accepted.get(&number) == Some(&sha256_hex(&output)) => accepted_count += 1,
                Ok(_) => {
                    fs::write(
                        dir.join(diff_name(number)),
                        diff_output(&previous, &current),
                    )?;
                    changed.push_str(&format!(
                        "- [{number}] {command}\n{}\n",
                        diff_excerpt(&previous, &current)
                    ));
                }
            }
        }
        for previous in files_with_extension(&base, "out")? {
            if !dir.join(previous.file_name().unwrap_or_default()).exists() {
                let content = fs::read_to_string(&previous).unwrap_or_default();
                let first = content.lines().next().unwrap_or("");
                changed.push_str(&format!(
                    "- {}: comando do baseline que não existe mais\n",
                    first.strip_prefix("$ ").unwrap_or(first)
                ));
            }
        }

        if changed.is_empty() {
            frontmatter_set(&file, "status", &self.status.validating)?;
            frontmatter_set(&file, "regression_result", "passed")?;
            let detail = if accepted_count == 0 {
                format!("{equal} comando(s) iguais ao baseline")
            } else {
                format!("{equal} igual(is) ao baseline, {accepted_count} aceito(s) pelo usuário")
            };
            self.record_step(&file, "regressão", "passed", &detail)?;
            println!("Sem regressão em {task}: {detail}.");
            return Ok(());
        }
        frontmatter_set(&file, "regression_result", "failed")?;
        frontmatter_set(&file, "status", &self.status.blocked)?;
        self.record_step(
            &file,
            "regressão",
            "failed",
            "comandos diferentes do baseline; bloqueada",
        )?;
        print!(
            "Comandos com resultado diferente do baseline (saídas em {}):\n{changed}Para aceitar uma diferença como esperada, com a decisão do usuário: vostok task resume {spec} {task} --accept-regression <N> \"decisão\"\n",
            dir.display()
        );
        bail!("a tarefa {task} ficou bloqueada por regressão. PARE: responda com o template `blocker`, mostrando os comandos, e aguarde a decisão do usuário")
    }

    fn evidence(&self, spec: &str, task: &str) -> Result<()> {
        let file = self.find_task(spec, task)?;
        self.require_running(&file, task)?;
        let dir = self.evidence_dir(&file)?;
        let text = self.subsection(&read(&file)?, "evidence_subsection")?;
        let items: Vec<&str> = text
            .lines()
            .filter(|line| has_text(line) && !is_indented(line))
            .collect();
        let none = self.cfg("evidence_none")?;
        let automatic = self.cfg("evidence_automatic")?;
        let manual = self.cfg("evidence_manual")?;

        let trimmed: Vec<&str> = items.iter().map(|line| line.trim()).collect();
        if trimmed.join("\n") == none {
            frontmatter_set(&file, "evidence_result", "passed")?;
            self.record_step(&file, "evidências", "passed", &none)?;
            println!("Evidências de {task}: {none} Resultado: passed.");
            return Ok(());
        }

        fs::create_dir_all(&dir)?;
        let root = self.project_root()?;
        let mut missing = String::new();
        let mut failed = String::new();
        for item in &items {
            let (name, kind, rest) = evidence_item(item)
                .ok_or_else(|| anyhow!("item de evidência fora do formato: {item}"))?;
            if name.contains('/')
                || name.starts_with('.')
                || name == "baseline"
                || name == "validation"
            {
                bail!("nome de evidência inválido: '{name}'");
            }
            let target = dir.join(name);
            if kind == automatic {
                let command = rest
                    .strip_prefix('`')
                    .and_then(|rest| rest.strip_suffix('`'))
                    .filter(|command| !command.is_empty())
                    .ok_or_else(|| anyhow!("evidência automática sem comando: {item}"))?;
                // A saída só vira evidência se o comando terminar sem erro; com erro, ela vai para a mensagem.
                let (code, output) = shell_output(command, &root)?;
                if code == 0 {
                    fs::write(&target, output)?;
                } else {
                    failed.push_str(&format!(
                        "- {name}: `{command}` terminou com erro\n{}\n",
                        trim_newlines(&indented_tail(&output, 5))
                    ));
                    if target.exists() {
                        fs::remove_file(&target)?;
                    }
                }
            } else if kind == manual && !fs::metadata(&target).is_ok_and(|meta| meta.len() > 0) {
                missing.push_str(&format!("- {}: {rest}\n", target.display()));
            }
        }

        if !failed.is_empty() {
            self.record_step(&file, "evidências", "erro", "evidência automática com erro")?;
            bail!("evidências automáticas de {task} com erro; corrija e execute vostok task evidence de novo:\n{failed}");
        }
        if !missing.is_empty() {
            frontmatter_set(&file, "evidence_result", "pending")?;
            self.record_step(
                &file,
                "evidências",
                "pending",
                "evidência manual sem arquivo",
            )?;
            print!("Evidências manuais de {task} sem arquivo; peça ao usuário para anexá-las e execute vostok task evidence de novo:\n{missing}");
            return Ok(());
        }
        frontmatter_set(&file, "evidence_result", "passed")?;
        let folder = dir
            .parent()
            .and_then(Path::file_name)
            .unwrap_or_default()
            .to_string_lossy();
        let detail = format!("{} evidência(s) em {folder}/{task}/", items.len());
        self.record_step(&file, "evidências", "passed", &detail)?;
        println!(
            "Evidências de {task} gravadas em {}: {} item(ns). Resultado: passed.",
            dir.display(),
            items.len()
        );
        Ok(())
    }

    // --- finish e resume ---

    fn finish(&self, spec: &str, task: &str) -> Result<()> {
        let file = self.find_task(spec, task)?;
        self.require_running(&file, task)?;
        let mut problems = String::new();
        for key in ["tests_result", "regression_result", "evidence_result"] {
            let value = frontmatter_get(&file, key).unwrap_or_default();
            if value != "passed" {
                problems.push_str(&format!("- {key}: '{value}' (precisa ser 'passed')\n"));
            }
        }
        if let Err(error) = self.check_done(spec, task) {
            problems.push_str(&format!("- {error}\n"));
        }
        if !problems.is_empty() {
            bail!("a tarefa {task} não pode ser concluída; nada foi alterado:\n{problems}");
        }

        let done = &self.status.done;
        frontmatter_set(&file, "status", done)?;
        self.record_step(
            &file,
            "conclusão",
            done,
            "testes, regressão e evidências passed; critérios de conclusão marcados",
        )?;
        println!("Tarefa {task} concluída: status '{done}'.");

        let unfinished: Vec<String> = markdown_files(file.parent().unwrap_or(Path::new("")))?
            .iter()
            .filter(|other| status_of(other) != *done)
            .map(|other| task_id_of(other))
            .collect();
        if !unfinished.is_empty() {
            println!(
                "Tarefas de {spec} ainda não concluídas: {}",
                unfinished.join(" ")
            );
            return Ok(());
        }
        let implemented = self.spec_cfg("status.implemented")?;
        let spec_file = self.refined_spec(spec)?.ok_or_else(|| {
            anyhow!("a tarefa foi concluída, mas a spec {spec} não foi encontrada para marcar como '{implemented}'")
        })?;
        frontmatter_set(&spec_file, "status", &implemented)?;
        frontmatter_put(&spec_file, "updated_at", &utc_now())?;
        println!("Todas as tarefas de {spec} concluídas: spec marcada como '{implemented}'.");
        Ok(())
    }

    /// Spec refinada com o `id` informado no front-matter.
    fn refined_spec(&self, spec: &str) -> Result<Option<PathBuf>> {
        let dir = self
            .harness
            .path("specs")?
            .join(self.spec_cfg("refined_dir")?);
        Ok(markdown_files(&dir)
            .unwrap_or_default()
            .into_iter()
            .find(|file| frontmatter_get(file, "id").is_ok_and(|id| id == spec)))
    }

    fn resume(&self, spec: &str, task: &str, decision: &str, accept: Option<&str>) -> Result<()> {
        let file = self.find_task(spec, task)?;
        let status = status_of(&file);
        let blocked = &self.status.blocked;
        if status != *blocked {
            bail!("a tarefa {task} está com status '{status}'; só tarefas com status '{blocked}' são retomadas");
        }
        let decision = trim_lines(decision);
        if decision.is_empty() {
            bail!("informe a decisão do usuário");
        }
        if decision.contains('\n') {
            bail!("a decisão não pode ter quebra de linha");
        }
        let dir = self.evidence_dir(&file)?.join("validation/regression");
        let acceptances = match accept {
            Some(list) => regression_acceptances(&dir, task, list, &decision)?,
            None => Vec::new(),
        };

        let mut detail = format!("decisão do usuário: {decision}");
        if !acceptances.is_empty() {
            let mut accepted = fs::read_to_string(dir.join(ACCEPTED_FILE)).unwrap_or_default();
            for item in &acceptances {
                fs::copy(
                    dir.join(diff_name(item.number)),
                    dir.join(accepted_diff_name(item.number)),
                )?;
                accepted.push_str(&item.line);
            }
            fs::write(dir.join(ACCEPTED_FILE), accepted)?;
            let numbers: Vec<String> = acceptances
                .iter()
                .map(|item| item.number.to_string())
                .collect();
            detail.push_str(&format!("; aceito(s): {}", numbers.join(", ")));
        }
        let validating = &self.status.validating;
        frontmatter_set(&file, "status", validating)?;
        reset_results(&file)?;
        self.record_step(&file, "retomada", validating, &detail)?;
        println!("Tarefa {task} retomada: status '{validating}', tentativas e resultados zerados. Aplique a decisão e valide de novo, começando por vostok task tests.");
        Ok(())
    }

    // --- Hash, registro de execução e histórico de ajustes ---

    /// Hash do conteúdo depois do front-matter. Em tarefas, sem o "Registro de execução", escrito só
    /// pelos scripts: gravar uma linha nele não muda o hash.
    fn content_hash(&self, content: &str) -> Result<String> {
        let record = self.cfg("record_section")?;
        let has_record = markdown_section(content, &record).is_some();
        let record_heading = format!("## {record}");
        let mut in_code = false;
        let mut skipping = false;
        let mut body = String::new();
        for line in body_lines(content) {
            if has_record {
                if is_fence(line) {
                    in_code = !in_code;
                }
                if !in_code && line.starts_with("## ") {
                    skipping = line == record_heading;
                }
                if skipping {
                    continue;
                }
            }
            body.push_str(line);
            body.push('\n');
        }
        Ok(sha256_hex(body.as_bytes()))
    }

    fn hash_is_current(&self, file: &Path) -> Result<bool> {
        let stored =
            frontmatter_get(file, &self.spec_cfg("content_hash_field")?).unwrap_or_default();
        Ok(stored == self.content_hash(&read(file)?)?)
    }

    fn store_hash(&self, file: &Path) -> Result<()> {
        let hash = self.content_hash(&read(file)?)?;
        frontmatter_put(file, &self.spec_cfg("content_hash_field")?, &hash)
    }

    /// Acrescenta uma linha ao "Registro de execução" e grava o hash do conteúdo.
    fn record_step(&self, file: &Path, step: &str, result: &str, detail: &str) -> Result<()> {
        let section = self.cfg("record_section")?;
        let content = read(file)?;
        let count = markdown_section(&content, &section)
            .map(|text| text.lines().filter(|line| is_numbered_row(line)).count())
            .unwrap_or(0);
        let row = format!(
            "| {} | {} | {} | {} | {} |",
            count + 1,
            &utc_now()[..10],
            escape_pipes(step),
            escape_pipes(result),
            escape_pipes(detail)
        );
        let updated = insert_table_row(&content, &format!("## {section}"), &row, false)
            .ok_or_else(|| anyhow!("seção '## {section}' sem a tabela; nada registrado"))?;
        write(file, &updated)?;
        self.store_hash(file)
    }

    /// Registra um ajuste: linha no histórico, contagem, datas e, por último, o hash. Devolve o número.
    fn record_adjustment(&self, file: &Path, origin: &str, summary: &str) -> Result<u64> {
        let count = number_or_zero(&frontmatter_get(file, "adjustments").unwrap_or_default()) + 1;
        let now = utc_now();
        let heading = self.history_heading()?;
        let row = format!(
            "| {count} | {} | {origin} | {} |",
            &now[..10],
            escape_pipes(summary)
        );
        let updated = insert_table_row(&read(file)?, &format!("## {heading}"), &row, true)
            .ok_or_else(|| anyhow!("seção ausente: '## {heading}'; nenhum ajuste registrado"))?;
        write(file, &updated)?;
        frontmatter_put(file, "adjustments", &count.to_string())?;
        frontmatter_put(file, "adjusted_at", &now)?;
        frontmatter_put(file, "updated_at", &now)?;
        self.store_hash(file)?;
        Ok(count)
    }

    /// Resumo de um ajuste manual, com as seções alteradas desde o último commit quando houver.
    fn manual_summary(&self, file: &Path) -> Result<String> {
        let summary = self.spec_cfg("manual_adjustment_summary")?;
        let sections = changed_sections(file, &self.history_heading()?)?;
        Ok(if sections.is_empty() {
            summary
        } else {
            format!("{summary}: {sections}")
        })
    }

    // --- Validação da tarefa ---

    /// Problemas da tarefa, prefixados pelo ID. A cobertura é a da própria tarefa, que não muda aqui.
    fn errors_of(&self, spec: &str, file: &Path) -> Result<Vec<String>> {
        let upstream = upstream_paths(&self.tasks_root.join(spec), &task_id_of(file), |file| {
            self.created_paths(file)
        })?;
        self.task_errors(file, &upstream)
    }

    fn task_errors(&self, file: &Path, upstream: &HashSet<String>) -> Result<Vec<String>> {
        let id = task_id_of(file);
        let content = read(file)?;
        let mut errors: Vec<String> = template_leftovers(&content)
            .into_iter()
            .map(|(line, text)| format!("variável sem preencher: {line}:{text}"))
            .collect();
        errors.extend(self.fixed_section_errors(&content)?);
        errors.extend(self.subsection_errors(&content)?);
        errors.extend(self.description_errors(&content)?);
        errors.extend(self.change_errors(&content, upstream)?);
        errors.extend(self.done_errors(&content)?);
        errors.extend(self.baseline_errors(&content)?);
        errors.extend(self.evidence_errors(&content)?);
        Ok(errors
            .into_iter()
            .map(|error| format!("{id}: {error}"))
            .collect())
    }

    /// Seções escritas só pelos scripts: precisam existir com a tabela, para receber as linhas.
    fn fixed_section_errors(&self, content: &str) -> Result<Vec<String>> {
        let mut errors = Vec::new();
        for section in [self.cfg("record_section")?, self.history_heading()?] {
            match markdown_section(content, &section) {
                None => errors.push(format!("seção ausente: '## {section}'")),
                Some(text) if !text.lines().any(|line| line.starts_with('|')) => {
                    errors.push(format!("a seção '## {section}' está sem a tabela"))
                }
                Some(_) => {}
            }
        }
        Ok(errors)
    }

    fn subsection_errors(&self, content: &str) -> Result<Vec<String>> {
        let mut errors = Vec::new();
        for kind in ["execution", "description"] {
            let section = self.cfg(&format!("{kind}_section"))?;
            for sub in self.cfg_list(&format!("{kind}_subsections"))? {
                match markdown_subsection(content, &section, &sub) {
                    None => errors.push(format!("subseção ausente: '### {sub}'")),
                    Some(text) if !has_text(&text) => {
                        errors.push(format!("a subseção '{sub}' está vazia"))
                    }
                    Some(_) => {}
                }
            }
        }
        Ok(errors)
    }

    fn description_errors(&self, content: &str) -> Result<Vec<String>> {
        let mut errors = Vec::new();
        let section = self.cfg("description_section")?;
        let max = number_or_zero(&self.cfg("executive_summary_max_lines")?);
        let lines = markdown_subsection(
            content,
            &section,
            &self.cfg("executive_summary_subsection")?,
        )
        .map_or(0, |text| text.lines().filter(|line| has_text(line)).count());
        if lines as u64 > max {
            errors.push(format!(
                "o resumo executivo tem {lines} linhas (máximo: {max})"
            ));
        }
        let card = markdown_subsection(content, &section, &self.cfg("card_subsection")?)
            .unwrap_or_default();
        for field in self.cfg_list("card_fields")? {
            if !card.contains(&format!("- **{field}:**")) {
                errors.push(format!("campo do card ausente: '{field}'"));
            }
        }
        Ok(errors)
    }

    /// Arquivos a mudar, com a ação coerente com a existência do caminho. Um caminho criado por uma
    /// tarefa de que esta depende conta como existente.
    fn change_errors(&self, content: &str, upstream: &HashSet<String>) -> Result<Vec<String>> {
        let root = self.project_root()?;
        let items = change_items(&self.subsection(content, "change_subsection")?);
        let mut errors = Vec::new();
        for (action, path) in &items {
            if !COMPONENT_ACTIONS.contains(&action.as_str()) {
                errors.push(format!(
                    "ação inválida para '{path}': '{action}' (use: {})",
                    COMPONENT_ACTIONS.join(" ")
                ));
                continue;
            }
            let exists = Path::new(&format!("{}/{path}", root.display())).exists();
            if action == CREATE_ACTION && exists {
                errors.push(format!("'{path}' já existe; use a ação 'alterar'"));
            } else if action != CREATE_ACTION && !exists && !upstream.contains(path) {
                errors.push(format!("'{path}' não existe nem é criado por uma tarefa de que ela depende; use a ação 'criar' ou corrija o caminho"));
            }
        }
        if items.is_empty() {
            errors.push(format!(
                "'{}' sem nenhum item no formato: - `caminho` (ação): motivo",
                self.cfg("change_subsection")?
            ));
        }
        Ok(errors)
    }

    /// Critérios de conclusão: um checkbox por critério, seguido da forma de comprová-lo numa citação
    /// recuada ("  > Como comprovar: ...").
    fn done_errors(&self, content: &str) -> Result<Vec<String>> {
        let sub = self.cfg("done_subsection")?;
        let label = format!("{}:", self.cfg("done_proof_label")?);
        let text = self.subsection(content, "done_subsection")?;
        let mut errors = Vec::new();
        let mut criterion = "";
        let mut proven = true;
        let mut count = 0;
        for item in text.lines().filter(|line| has_text(line)) {
            if is_checkbox(item) {
                if !proven {
                    errors.push(format!("'{sub}' com critério sem '{label}': {criterion}"));
                }
                criterion = item;
                proven = false;
                count += 1;
            } else if let Some(proof) = quoted(item)
                .and_then(|quote| quote.strip_prefix(label.as_str()))
                .filter(|_| !criterion.is_empty())
            {
                proven |= has_text(proof);
            } else {
                errors.push(format!("'{sub}' com linha fora do formato: {item}"));
            }
        }
        if !proven {
            errors.push(format!("'{sub}' com critério sem '{label}': {criterion}"));
        }
        if count == 0 && !text.is_empty() {
            errors.push(format!(
                "'{sub}' sem nenhum critério no formato de checkbox: - [ ] critério"
            ));
        }
        Ok(errors)
    }

    /// Cada item de "Baseline" é um comando no formato "- `comando`", e há ao menos um.
    fn baseline_errors(&self, content: &str) -> Result<Vec<String>> {
        let text = self.subsection(content, "baseline_subsection")?;
        if text.is_empty() {
            return Ok(Vec::new());
        }
        let sub = self.cfg("baseline_subsection")?;
        let mut errors = Vec::new();
        let mut count = 0;
        for item in text.lines().filter(|line| has_text(line)) {
            if is_baseline_item(item) {
                count += 1;
            } else {
                errors.push(format!(
                    "'{sub}' com item fora do formato: {item} (use: - `comando`)"
                ));
            }
        }
        if count == 0 {
            errors.push(format!("'{sub}' sem nenhum item no formato: - `comando`"));
        }
        Ok(errors)
    }

    /// Itens de "Evidências": "- `arquivo` (automática): `comando`", "- `arquivo` (manual): descrição"
    /// ou o texto único de `evidence_none`.
    fn evidence_errors(&self, content: &str) -> Result<Vec<String>> {
        let text = self.subsection(content, "evidence_subsection")?;
        if text.is_empty() {
            return Ok(Vec::new());
        }
        let sub = self.cfg("evidence_subsection")?;
        let automatic = self.cfg("evidence_automatic")?;
        let manual = self.cfg("evidence_manual")?;
        let none = self.cfg("evidence_none")?;
        let filled_lines: Vec<&str> = text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect();
        if filled_lines.join("\n") == none {
            return Ok(Vec::new());
        }
        let mut errors = Vec::new();
        let mut count = 0;
        for item in text
            .lines()
            .filter(|line| has_text(line) && !is_indented(line))
        {
            if is_automatic_evidence(item, &automatic) || is_manual_evidence(item, &manual) {
                count += 1;
            } else {
                errors.push(format!("'{sub}' com item fora do formato: {item} (use: - `arquivo` ({automatic}): `comando` ou - `arquivo` ({manual}): descrição)"));
            }
        }
        if count == 0 {
            errors.push(format!(
                "'{sub}' sem nenhum item no formato e sem o texto '{none}'"
            ));
        }
        let fields = self.evidence_fields();
        errors.extend(evidence_subitem_errors(&text, &sub, fields.as_ref(), false));
        Ok(errors)
    }

    /// Caminhos que a tarefa cria ("Mudar" com a ação `criar`).
    fn created_paths(&self, file: &Path) -> Result<Vec<String>> {
        let content = read(file)?;
        Ok(
            change_items(&self.subsection(&content, "change_subsection")?)
                .into_iter()
                .filter(|(action, _)| action == CREATE_ACTION)
                .map(|(_, path)| path)
                .collect(),
        )
    }
}

// --- Leitura da tarefa ---

fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("não foi possível ler {}", path.display()))
}

fn write(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content).with_context(|| format!("não foi possível gravar {}", path.display()))
}

fn task_id_of(file: &Path) -> String {
    file.file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

fn status_of(file: &Path) -> String {
    frontmatter_get(file, "status").unwrap_or_default()
}

/// Dependências do front-matter (`depends_on: [TASK-01, TASK-02]`).
fn dependencies_of(file: &Path) -> Vec<String> {
    frontmatter_get(file, "depends_on")
        .unwrap_or_default()
        .replace(['[', ']'], "")
        .split(',')
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(str::to_string)
        .collect()
}

fn reset_results(file: &Path) -> Result<()> {
    frontmatter_set(file, "attempts", "0")?;
    for key in ["tests_result", "regression_result", "evidence_result"] {
        frontmatter_set(file, key, "null")?;
    }
    Ok(())
}

/// Arquivos `.md` visíveis de uma pasta, em ordem alfabética.
fn markdown_files(dir: &Path) -> Result<Vec<PathBuf>> {
    files_with_extension(dir, "md")
}

fn files_with_extension(dir: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .with_context(|| format!("não foi possível ler {}", dir.display()))?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path.extension().is_some_and(|ext| ext == extension)
                && !path
                    .file_name()
                    .is_some_and(|name| name.to_string_lossy().starts_with('.'))
        })
        .collect();
    files.sort();
    Ok(files)
}

/// Caminhos criados pelas tarefas de que `id` depende, direta ou indiretamente.
fn upstream_paths(
    dir: &Path,
    id: &str,
    created_paths: impl Fn(&Path) -> Result<Vec<String>>,
) -> Result<HashSet<String>> {
    let mut dependencies: HashMap<String, Vec<String>> = HashMap::new();
    let mut created: HashMap<String, Vec<String>> = HashMap::new();
    for file in markdown_files(dir)? {
        let task = task_id_of(&file);
        dependencies.insert(task.clone(), dependencies_of(&file));
        created.insert(task, created_paths(&file)?);
    }
    let mut paths = HashSet::new();
    let mut seen = HashSet::new();
    let mut pending = dependencies.get(id).cloned().unwrap_or_default();
    while let Some(task) = pending.pop() {
        if !seen.insert(task.clone()) {
            continue;
        }
        paths.extend(created.get(&task).into_iter().flatten().cloned());
        pending.extend(dependencies.get(&task).into_iter().flatten().cloned());
    }
    Ok(paths)
}

/// Linhas do conteúdo depois do front-matter, sem `\r`. Sem front-matter, todas as linhas.
fn body_lines(content: &str) -> Vec<&str> {
    let mut inside = false;
    let mut lines = Vec::new();
    for (index, line) in content.lines().enumerate() {
        let line = line.trim_end_matches('\r');
        if index == 0 && line == "---" {
            inside = true;
        } else if inside && line == "---" {
            inside = false;
        } else if !inside {
            lines.push(line);
        }
    }
    lines
}

fn is_fence(line: &str) -> bool {
    line.trim_start_matches([' ', '\t']).starts_with("```")
}

/// Acrescenta `row` depois da última linha de tabela da seção `heading`; com `after_heading`, uma seção
/// sem tabela recebe a linha logo depois do título. Devolve `None` se não houver onde inserir.
fn insert_table_row(
    content: &str,
    heading: &str,
    row: &str,
    after_heading: bool,
) -> Option<String> {
    let raw_lines = raw_lines(content);
    let mut in_code = false;
    let mut inside = false;
    let mut anchor = None;
    for (index, raw) in raw_lines.iter().enumerate() {
        let line = raw.trim_end_matches('\r');
        if is_fence(line) {
            in_code = !in_code;
            continue;
        }
        if in_code {
            continue;
        }
        if line == heading {
            inside = true;
            if after_heading {
                anchor = Some(index);
            }
            continue;
        }
        if inside && line.starts_with("## ") {
            inside = false;
        }
        if inside && line.starts_with('|') {
            anchor = Some(index);
        }
    }
    let anchor = anchor?;
    let mut out = String::new();
    for (index, raw) in raw_lines.iter().enumerate() {
        out.push_str(raw);
        out.push('\n');
        if index == anchor {
            out.push_str(row);
            out.push('\n');
        }
    }
    Some(out)
}

/// Linhas como o awk as lê: separadas por `\n`, mantendo o `\r`.
fn raw_lines(content: &str) -> Vec<&str> {
    let content = content.strip_suffix('\n').unwrap_or(content);
    if content.is_empty() {
        Vec::new()
    } else {
        content.split('\n').collect()
    }
}

/// Como `frontmatter_set`, mas acrescenta a chave no fim do front-matter se ela não existir.
fn frontmatter_put(file: &Path, key: &str, value: &str) -> Result<()> {
    let content = read(file)?;
    let prefix = format!("{key}:");
    let lines = frontmatter_lines(&content)
        .ok_or_else(|| anyhow!("Front-matter não encontrado em {}", file.display()))?;
    if lines.iter().any(|line| line.starts_with(&prefix)) {
        return frontmatter_set(file, key, value);
    }
    let mut out = String::new();
    let mut inside = false;
    for (index, raw) in raw_lines(&content).into_iter().enumerate() {
        let line = raw.trim_end_matches('\r');
        if index == 0 && line == "---" {
            inside = true;
        } else if inside && line == "---" {
            out.push_str(&format!("{key}: {value}\n"));
            inside = false;
        }
        out.push_str(raw);
        out.push('\n');
    }
    write(file, &out)
}

/// Seções (`## `) com linhas alteradas desde o último commit, separadas por vírgula. Vazio se o arquivo
/// não estiver versionado no git ou não tiver alterações.
fn changed_sections(file: &Path, skip_heading: &str) -> Result<String> {
    let dir = file.parent().unwrap_or(Path::new("."));
    let base = file.file_name().unwrap_or_default();
    let tracked = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["ls-files", "--error-unmatch", "--"])
        .arg(base)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success());
    if !tracked {
        return Ok(String::new());
    }
    let diff = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["diff", "-U0", "HEAD", "--"])
        .arg(base)
        .stderr(Stdio::null())
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
        .unwrap_or_default();
    let changed: HashSet<usize> = diff.lines().filter_map(hunk_start).collect();
    if changed.is_empty() {
        return Ok(String::new());
    }

    let mut sections: Vec<&str> = Vec::new();
    let mut in_front_matter = false;
    let mut in_code = false;
    let mut heading = "";
    let content = read(file)?;
    for (index, line) in content.lines().enumerate() {
        let line = line.trim_end_matches('\r');
        if index == 0 && line == "---" {
            in_front_matter = true;
            continue;
        }
        if in_front_matter {
            in_front_matter = line != "---";
            continue;
        }
        if is_fence(line) {
            in_code = !in_code;
        }
        if !in_code {
            if let Some(title) = line.strip_prefix("## ") {
                heading = title;
            }
        }
        let counts = changed.contains(&(index + 1))
            && !heading.is_empty()
            && heading != skip_heading
            && !sections.contains(&heading);
        if counts {
            sections.push(heading);
        }
    }
    Ok(sections.join(", "))
}

/// Linha inicial do lado novo de um cabeçalho de hunk (`@@ -a,b +N,M @@`).
fn hunk_start(line: &str) -> Option<usize> {
    let rest = line.strip_prefix("@@ -")?;
    let (old, rest) = rest.split_once(' ')?;
    if old.is_empty() || !old.chars().all(|c| c.is_ascii_digit() || c == ',') {
        return None;
    }
    let digits: String = rest
        .strip_prefix('+')?
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().ok()
}

// --- Formatos dos itens ---

fn is_number(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|c| c.is_ascii_digit())
}

fn number_or_zero(text: &str) -> u64 {
    if is_number(text) {
        text.parse().unwrap_or(0)
    } else {
        0
    }
}

fn is_spec_id(id: &str) -> bool {
    id.split_once('-').is_some_and(|(prefix, number)| {
        !prefix.is_empty() && prefix.chars().all(|c| c.is_ascii_uppercase()) && is_number(number)
    })
}

fn has_text(text: &str) -> bool {
    text.chars().any(|c| !c.is_whitespace())
}

fn trim_newlines(text: &str) -> &str {
    text.trim_end_matches('\n')
}

/// Remove os espaços das pontas de cada linha, como `sed` nos scripts, e as quebras de linha do fim.
fn trim_lines(text: &str) -> String {
    let lines: Vec<&str> = text.split('\n').map(str::trim).collect();
    trim_newlines(&lines.join("\n")).to_string()
}

fn clean_summary(summary: &str) -> Result<String> {
    let summary = trim_lines(summary);
    if summary.is_empty() {
        bail!("o resumo do ajuste está vazio");
    }
    if summary.contains('\n') {
        bail!("o resumo não pode ter quebra de linha; separe os itens com '; '");
    }
    Ok(summary)
}

fn escape_pipes(text: &str) -> String {
    text.replace('|', "\\|")
}

fn is_numbered_row(line: &str) -> bool {
    line.strip_prefix("| ")
        .is_some_and(|rest| rest.starts_with(|c: char| c.is_ascii_digit()))
}

/// Trecho entre crases no começo do texto (sem crases dentro) e o resto depois da crase de fechamento.
fn quoted_name(text: &str) -> Option<(&str, &str)> {
    let rest = text.strip_prefix('`')?;
    let end = rest.find('`')?;
    (end > 0).then(|| (&rest[..end], &rest[end + 1..]))
}

/// Item de "Mudar" no formato "- `caminho` (ação): motivo", como (ação, caminho).
fn change_items(text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter_map(|line| {
            let (path, rest) = quoted_name(line.strip_prefix("- ")?)?;
            let rest = rest.strip_prefix(" (")?;
            let end = rest.find(')')?;
            let action = &rest[..end];
            let valid = !action.is_empty() && action.chars().all(|c| c.is_ascii_lowercase());
            valid.then(|| (action.to_string(), path.to_string()))
        })
        .collect()
}

fn is_baseline_item(line: &str) -> bool {
    line.strip_prefix("- ")
        .and_then(quoted_name)
        .is_some_and(|(_, rest)| rest.is_empty())
}

/// Comando de um item de "Baseline" ("- `comando`"); o comando pode ter crases.
fn baseline_command(line: &str) -> Option<&str> {
    line.strip_prefix("- `")?
        .strip_suffix('`')
        .filter(|command| !command.is_empty())
}

fn evidence_kind_and_rest<'a>(line: &'a str, kind: &str) -> Option<&'a str> {
    let (_, rest) = quoted_name(line.strip_prefix("- ")?)?;
    rest.strip_prefix(" (")?
        .strip_prefix(kind)?
        .strip_prefix("): ")
}

fn is_automatic_evidence(line: &str, automatic: &str) -> bool {
    evidence_kind_and_rest(line, automatic)
        .and_then(quoted_name)
        .is_some_and(|(_, rest)| rest.is_empty())
}

fn is_manual_evidence(line: &str, manual: &str) -> bool {
    evidence_kind_and_rest(line, manual)
        .is_some_and(|rest| rest.starts_with(|c: char| !c.is_whitespace()))
}

/// Item de "Evidências" no formato "- `arquivo` (tipo): resto", como (arquivo, tipo, resto).
fn evidence_item(line: &str) -> Option<(&str, &str, &str)> {
    let (name, rest) = quoted_name(line.strip_prefix("- ")?)?;
    let rest = rest.strip_prefix(" (")?;
    let end = rest.find(')')?;
    let kind = &rest[..end];
    let rest = rest[end + 1..].strip_prefix(": ")?;
    (!kind.is_empty() && !rest.is_empty()).then_some((name, kind, rest))
}

fn is_checkbox(line: &str) -> bool {
    ["- [ ] ", "- [x] "].iter().any(|prefix| {
        line.strip_prefix(prefix)
            .is_some_and(|rest| rest.starts_with(|c: char| !c.is_whitespace()))
    })
}

/// Texto de uma citação recuada ("  > texto").
fn quoted(line: &str) -> Option<&str> {
    let rest = line.trim_start();
    (rest.len() < line.len())
        .then_some(rest)?
        .strip_prefix("> ")
}

/// Comandos do primeiro bloco `bash` do texto.
fn verify_block(text: &str) -> String {
    let mut inside = false;
    let mut lines = Vec::new();
    for line in text.lines() {
        if !inside
            && line
                .strip_prefix("```bash")
                .is_some_and(|rest| rest.trim().is_empty())
        {
            inside = true;
        } else if inside
            && line
                .strip_prefix("```")
                .is_some_and(|rest| rest.trim().is_empty())
        {
            break;
        } else if inside {
            lines.push(line);
        }
    }
    trim_newlines(&lines.join("\n")).to_string()
}

// --- Execução de comandos ---

fn output_name(number: usize) -> String {
    format!("{number:02}.out")
}

fn diff_name(number: usize) -> String {
    format!("{number:02}.diff")
}

/// Cópia do diff aceito, preservada entre execuções da regressão para o documento de evidências.
pub fn accepted_diff_name(number: usize) -> String {
    format!("accepted-{number:02}.diff")
}

/// Remove só as saídas e os diffs da última regressão (`NN.out`, `NN.diff`), preservando os aceites.
fn clear_regression_outputs(dir: &Path) -> Result<()> {
    if !dir.is_dir() {
        fs::create_dir_all(dir)?;
        return Ok(());
    }
    for entry in fs::read_dir(dir)?.flatten() {
        let path = entry.path();
        let numbered = path.file_stem().is_some_and(|stem| {
            stem.to_string_lossy()
                .bytes()
                .all(|byte| byte.is_ascii_digit())
        });
        let output = path
            .extension()
            .is_some_and(|ext| ext == "out" || ext == "diff");
        if path.is_file() && numbered && output {
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}

/// Um aceite de diferença de regressão, lido do arquivo `accepted`.
pub struct AcceptedRegression {
    pub number: usize,
    pub hash: String,
    pub date: String,
    pub command: String,
    pub decision: String,
}

/// Aceites gravados em `dir/accepted`, na ordem do arquivo.
pub fn accepted_regressions(dir: &Path) -> Vec<AcceptedRegression> {
    fs::read_to_string(dir.join(ACCEPTED_FILE))
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            let mut fields = line.splitn(5, '\t');
            Some(AcceptedRegression {
                number: fields.next()?.parse().ok()?,
                hash: fields.next()?.to_string(),
                date: fields.next()?.to_string(),
                command: fields.next()?.to_string(),
                decision: fields.next()?.to_string(),
            })
        })
        .collect()
}

/// SHA-256 aceito por comando; o aceite mais recente de um número vale.
fn accepted_hashes(dir: &Path) -> Result<HashMap<usize, String>> {
    Ok(accepted_regressions(dir)
        .into_iter()
        .map(|item| (item.number, item.hash))
        .collect())
}

struct Acceptance {
    number: usize,
    line: String,
}

/// Confere os números de `--accept-regression` contra a última regressão e monta as linhas do
/// arquivo `accepted`, sem gravar nada.
fn regression_acceptances(
    dir: &Path,
    task: &str,
    list: &str,
    decision: &str,
) -> Result<Vec<Acceptance>> {
    let mut acceptances: Vec<Acceptance> = Vec::new();
    for item in list.split(',').map(str::trim) {
        let number: usize = match item.parse() {
            Ok(number) if number > 0 => number,
            _ => bail!("número inválido em --accept-regression: '{item}'"),
        };
        if acceptances.iter().any(|accepted| accepted.number == number) {
            continue;
        }
        if !dir.join(diff_name(number)).is_file() {
            bail!("o comando {number} não divergiu na última regressão de {task}; não há diferença a aceitar. Nada foi alterado");
        }
        let output = fs::read(dir.join(output_name(number))).map_err(|_| {
            anyhow!("não há saída gravada do comando {number} na última regressão de {task}. Nada foi alterado")
        })?;
        let first = String::from_utf8_lossy(&output)
            .lines()
            .next()
            .unwrap_or("")
            .to_string();
        let command = first.strip_prefix("$ ").unwrap_or(&first).to_string();
        acceptances.push(Acceptance {
            number,
            line: format!(
                "{number}\t{}\t{}\t{}\t{decision}\n",
                sha256_hex(&output),
                utc_now(),
                command.replace('\t', " ")
            ),
        });
    }
    Ok(acceptances)
}

/// Executa um comando com `bash -c` a partir de `dir` e devolve o código de saída e a saída, com
/// stdout e stderr juntos na ordem em que foram escritos.
fn shell_output(command: &str, dir: &Path) -> Result<(i32, Vec<u8>)> {
    let (mut reader, writer) = std::io::pipe()?;
    let mut child = Command::new("bash")
        .arg("-c")
        .arg(command)
        .current_dir(dir)
        .stdin(Stdio::null())
        .stdout(writer.try_clone()?)
        .stderr(writer)
        .spawn()
        .context("não foi possível executar o bash")?;
    let mut output = Vec::new();
    reader.read_to_end(&mut output)?;
    Ok((exit_code(child.wait()?), output))
}

/// Executa um comando e devolve o código de saída e o registro no formato do baseline: o comando, o
/// código de saída e a saída.
fn recorded_run(command: &str, dir: &Path) -> Result<(i32, Vec<u8>)> {
    let (code, output) = shell_output(command, dir)?;
    let mut record = format!("$ {command}\nexit: {code}\n").into_bytes();
    record.extend(output);
    Ok((code, record))
}

#[cfg(unix)]
fn exit_code(status: ExitStatus) -> i32 {
    use std::os::unix::process::ExitStatusExt;
    status
        .code()
        .or_else(|| status.signal().map(|signal| 128 + signal))
        .unwrap_or(1)
}

#[cfg(not(unix))]
fn exit_code(status: ExitStatus) -> i32 {
    status.code().unwrap_or(1)
}

/// Últimas `count` linhas da saída, recuadas em quatro espaços.
fn indented_tail(output: &[u8], count: usize) -> String {
    let text = String::from_utf8_lossy(output);
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    lines[lines.len().saturating_sub(count)..]
        .iter()
        .map(|line| format!("    {line}"))
        .collect()
}

/// Saída completa do `diff` entre o baseline e a nova saída.
fn diff_output(previous: &Path, current: &Path) -> Vec<u8> {
    Command::new("diff")
        .arg(previous)
        .arg(current)
        .output()
        .map(|output| output.stdout)
        .unwrap_or_default()
}

/// Linhas 2 a 12 do `diff` entre o baseline e a nova saída, recuadas em quatro espaços.
fn diff_excerpt(previous: &Path, current: &Path) -> String {
    let output = Command::new("diff")
        .arg(previous)
        .arg(current)
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
        .unwrap_or_default();
    output
        .lines()
        .skip(1)
        .take(11)
        .map(|line| format!("    {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

// --- Data ---

/// Data e hora UTC atuais no formato `AAAA-MM-DDTHH:MM:SSZ`.
fn utc_now() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());
    let (year, month, day) = civil_from_days((seconds / 86_400) as i64);
    let time = seconds % 86_400;
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        time / 3_600,
        time % 3_600 / 60,
        time % 60
    )
}

/// Data civil de um número de dias desde 1970-01-01 (algoritmo de Howard Hinnant).
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * month_index + 2) / 5 + 1) as u32;
    let month = (if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    }) as u32;
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_dates_match_known_days() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(20_727), (2026, 10, 1));
        assert_eq!(civil_from_days(11_016), (2000, 2, 29));
    }

    #[test]
    fn spec_ids_need_uppercase_prefix_and_number() {
        assert!(is_spec_id("SPEC-005"));
        assert!(!is_spec_id("../SPEC-005"));
        assert!(!is_spec_id("spec-005"));
        assert!(!is_spec_id("SPEC-"));
        assert!(!is_spec_id("SPEC-1-2"));
    }

    #[test]
    fn change_items_require_path_and_lowercase_action() {
        let text = "- `a.rs` (criar): novo\n- `b.rs` (Alterar): x\n- b.rs (alterar): x\n- `c.rs` (remover)";
        assert_eq!(
            change_items(text),
            vec![
                ("criar".to_string(), "a.rs".to_string()),
                ("remover".to_string(), "c.rs".to_string())
            ]
        );
    }

    #[test]
    fn baseline_items_and_commands() {
        assert!(is_baseline_item("- `true`"));
        assert!(!is_baseline_item("- `a` b"));
        assert!(!is_baseline_item("- ``"));
        assert_eq!(baseline_command("- `echo `x``"), Some("echo `x`"));
        assert_eq!(baseline_command("- ``"), None);
    }

    #[test]
    fn evidence_items_follow_the_formats() {
        assert!(is_automatic_evidence(
            "- `a.txt` (automática): `ls`",
            "automática"
        ));
        assert!(!is_automatic_evidence(
            "- `a.txt` (automática): ls",
            "automática"
        ));
        assert!(is_manual_evidence("- `a.png` (manual): print", "manual"));
        assert!(!is_manual_evidence("- `a.png` (manual):  print", "manual"));
        assert_eq!(evidence_item("- `a` (x): resto"), Some(("a", "x", "resto")));
        assert_eq!(evidence_item("- `a` (): resto"), None);
    }

    #[test]
    fn checkboxes_and_quotes() {
        assert!(is_checkbox("- [ ] critério"));
        assert!(is_checkbox("- [x] critério"));
        assert!(!is_checkbox("- [ ]  critério"));
        assert_eq!(quoted("  > Como comprovar: x"), Some("Como comprovar: x"));
        assert_eq!(quoted("> sem recuo"), None);
    }

    #[test]
    fn rows_are_inserted_after_the_section_table() {
        let content = "## A\n\n| # |\n|---|\n| 1 |\n\n## B\n| x |\n";
        assert_eq!(
            insert_table_row(content, "## A", "| 2 |", false).unwrap(),
            "## A\n\n| # |\n|---|\n| 1 |\n| 2 |\n\n## B\n| x |\n"
        );
        assert_eq!(
            insert_table_row("## A\n\ntexto\n", "## A", "| 1 |", false),
            None
        );
        assert_eq!(
            insert_table_row("## A\n", "## A", "| 1 |", true).unwrap(),
            "## A\n| 1 |\n"
        );
    }

    #[test]
    fn summaries_are_trimmed_and_single_line() {
        assert_eq!(clean_summary("  ajuste  ").unwrap(), "ajuste");
        assert!(clean_summary(" \n ").is_err());
        assert!(clean_summary("a\nb").is_err());
    }

    #[test]
    fn hunk_headers_give_the_new_start_line() {
        assert_eq!(hunk_start("@@ -3,2 +5,4 @@ x"), Some(5));
        assert_eq!(hunk_start("@@ -3 +7 @@"), Some(7));
        assert_eq!(hunk_start("+linha"), None);
    }

    #[test]
    fn tails_keep_the_last_lines_indented() {
        assert_eq!(indented_tail(b"a\nb\nc\n", 2), "    b\n    c\n");
        assert_eq!(indented_tail(b"", 5), "");
    }

    #[test]
    fn verify_block_takes_the_first_bash_block() {
        let text = "texto\n```bash\ntrue\nfalse\n```\n```bash\nnao\n```";
        assert_eq!(verify_block(text), "true\nfalse");
        assert_eq!(verify_block("sem bloco"), "");
    }
}
