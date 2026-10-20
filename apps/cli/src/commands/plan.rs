//! Comandos `plan`: planos de tarefas: criação, níveis, ajuste e aprovação.
//! Reproduzem `plan-tasks.sh`, `adjust-plan.sh`, `approve-plan.sh` e a lib `plan.sh`.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use clap::Subcommand;

use super::task::evidence_name_problem;
use crate::harness::Harness;
use crate::support::{
    config_get, config_list, evidence_subitem_errors, frontmatter_get, is_indented, markdown_cell,
    markdown_section, markdown_subsection, markdown_table_rows, render, sha256_hex,
    template_leftovers, yaml_escape, EvidenceFields,
};

#[derive(Debug, Subcommand)]
pub enum PlanCommand {
    /// Cria o plano de uma spec aprovada.
    Create { spec_id: String },
    /// Calcula e grava os níveis de esforço.
    Score { spec_id: String },
    /// Valida o plano.
    Validate { spec_id: String },
    /// Ajustes do plano.
    #[command(subcommand)]
    Adjust(AdjustCommand),
    /// Aprovação do plano e criação das tarefas.
    #[command(subcommand)]
    Approve(ApproveCommand),
}

#[derive(Debug, Subcommand)]
pub enum AdjustCommand {
    /// Registra edições feitas à mão.
    RecordManual { spec_id: String },
    /// Valida, sem alterar nada.
    Validate { spec_id: String },
    /// Registra o ajuste no histórico.
    Record {
        spec_id: String,
        #[arg(allow_hyphen_values = true)]
        summary: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum ApproveCommand {
    /// Cria os arquivos das tarefas.
    CreateTasks { spec_id: String },
    /// Valida as tarefas criadas.
    Validate { spec_id: String },
    /// Marca a spec como planejada e remove o plano.
    MarkPlanned { spec_id: String },
}

const PLAN_USAGE: &str = "Uso: vostok plan create|score|validate <SPEC_ID>";
const ADJUST_USAGE: &str =
    "Uso: vostok plan adjust record-manual <SPEC_ID> | validate <SPEC_ID> | record <SPEC_ID> \"resumo\"";
const APPROVE_USAGE: &str = "Uso: vostok plan approve create-tasks|validate|mark-planned <SPEC_ID>";

const TASKS_HEADING: &str = "Tarefas";
const FACTS_HEADING: &str = "Fatos de esforço";
const OVERVIEW_HEADING: &str = "Visão geral";
const COMPONENT_ACTIONS: &str = "criar alterar remover";

pub fn run(command: PlanCommand) -> Result<()> {
    let workspace = Workspace::new(Harness::from_current_dir()?);
    match command {
        PlanCommand::Create { spec_id } => workspace.create(require(&spec_id, PLAN_USAGE)?),
        PlanCommand::Score { spec_id } => workspace.score(require(&spec_id, PLAN_USAGE)?),
        PlanCommand::Validate { spec_id } => workspace.validate(require(&spec_id, PLAN_USAGE)?),
        PlanCommand::Adjust(command) => run_adjust(&workspace, command),
        PlanCommand::Approve(command) => run_approve(&workspace, command),
    }
}

fn run_adjust(workspace: &Workspace, command: AdjustCommand) -> Result<()> {
    match command {
        AdjustCommand::RecordManual { spec_id } => {
            workspace.record_manual(require(&spec_id, ADJUST_USAGE)?)
        }
        AdjustCommand::Validate { spec_id } => {
            workspace.adjust_validate(require(&spec_id, ADJUST_USAGE)?)
        }
        AdjustCommand::Record { spec_id, summary } => {
            let Some(summary) = summary else {
                bail!(ADJUST_USAGE);
            };
            workspace.record(require(&spec_id, ADJUST_USAGE)?, &summary)
        }
    }
}

fn run_approve(workspace: &Workspace, command: ApproveCommand) -> Result<()> {
    match command {
        ApproveCommand::CreateTasks { spec_id } => {
            workspace.create_tasks(require(&spec_id, APPROVE_USAGE)?)
        }
        ApproveCommand::Validate { spec_id } => {
            workspace.approve_validate(require(&spec_id, APPROVE_USAGE)?)
        }
        ApproveCommand::MarkPlanned { spec_id } => {
            workspace.mark_planned(require(&spec_id, APPROVE_USAGE)?)
        }
    }
}

fn require<'a>(spec_id: &'a str, usage: &str) -> Result<&'a str> {
    if spec_id.is_empty() {
        bail!("{usage}");
    }
    Ok(spec_id)
}

fn problems_message(header: String, problems: &[String]) -> anyhow::Error {
    anyhow::anyhow!("{header}\n{}", problems.join("\n"))
}

/// O harness do projeto e as leituras de configuração usadas pelos comandos `plan`.
struct Workspace {
    harness: Harness,
}

impl Workspace {
    fn new(harness: Harness) -> Self {
        Self { harness }
    }

    fn spec_setting(&self, key: &str) -> Result<String> {
        config_get(&self.harness.config_file("specs.yml"), key)
    }

    fn task_setting(&self, key: &str) -> Result<String> {
        config_get(&self.harness.config_file("tasks.yml"), key)
    }

    fn task_list(&self, key: &str) -> Result<Vec<String>> {
        config_list(&self.harness.config_file("tasks.yml"), key)
    }

    /// Subitens das evidências; sem `evidence_fields` na configuração, eles não são exigidos.
    fn evidence_fields(&self) -> Option<EvidenceFields> {
        Some(EvidenceFields {
            required: self.task_list("evidence_fields").ok()?,
            optional: self
                .task_list("evidence_optional_fields")
                .unwrap_or_default(),
        })
    }

    fn plan_path(&self, spec_id: &str) -> Result<PathBuf> {
        Ok(self.harness.path("plans")?.join(format!("{spec_id}.md")))
    }

    fn tasks_dir(&self, spec_id: &str) -> Result<PathBuf> {
        Ok(self.harness.path("tasks")?.join(spec_id))
    }

    fn template(&self, key: &str) -> Result<PathBuf> {
        let template = self
            .harness
            .path("templates")?
            .join(self.task_setting(key)?);
        if !template.is_file() {
            bail!("template não encontrado: {}", template.display());
        }
        Ok(template)
    }

    fn existing_plan(&self, spec_id: &str, missing: String) -> Result<PathBuf> {
        let plan = self.plan_path(spec_id)?;
        if !plan.is_file() {
            bail!(missing);
        }
        Ok(plan)
    }

    fn find_spec(&self, spec_id: &str) -> Result<PathBuf> {
        let dir = self
            .harness
            .path("specs")?
            .join(self.spec_setting("refined_dir")?);
        markdown_files(&dir)
            .into_iter()
            .find(|file| field(file, "id") == spec_id)
            .with_context(|| format!("spec refinada {spec_id} não encontrada"))
    }

    fn section_template(&self, section: &str) -> Result<PathBuf> {
        Ok(self
            .harness
            .path("templates")?
            .join(self.spec_setting("sections_dir")?)
            .join(format!("{section}.md")))
    }

    /// Título da seção, lido da primeira linha do template (`## Título`).
    fn section_heading(&self, section: &str) -> Result<String> {
        let content = fs::read_to_string(self.section_template(section)?).unwrap_or_default();
        let first = content.split('\n').next().unwrap_or("");
        Ok(first.strip_prefix("## ").unwrap_or(first).to_string())
    }

    fn history_heading(&self) -> Result<String> {
        self.section_heading(&self.spec_setting("adjustment_history_section")?)
    }

    // --- Hash do conteúdo e registro de ajustes ---

    /// Hash do conteúdo depois do front-matter; em tarefas, sem o "Registro de execução".
    fn content_hash(&self, content: &str) -> Result<String> {
        let record = self.task_setting("record_section")?;
        let body = frontmatter_body(content);
        if markdown_section(content, &record).is_none() {
            return Ok(sha256_hex(body.as_bytes()));
        }
        Ok(sha256_hex(without_section(&body, &record).as_bytes()))
    }

    fn with_stored_hash(&self, content: &str) -> Result<String> {
        let field = self.spec_setting("content_hash_field")?;
        frontmatter_put(content, &field, &self.content_hash(content)?)
    }

    fn store_hash(&self, path: &Path) -> Result<()> {
        let content = read(path)?;
        write(path, &self.with_stored_hash(&content)?)
    }

    fn hash_unchanged(&self, path: &Path) -> Result<bool> {
        let stored = field(path, &self.spec_setting("content_hash_field")?);
        Ok(stored == self.content_hash(&read(path)?)?)
    }

    /// Registra um ajuste: linha no histórico, contagem, datas e o hash do conteúdo.
    fn record_adjustment(&self, path: &Path, origin: &str, summary: &str) -> Result<u64> {
        let content = read(path)?;
        let count = field(path, "adjustments").parse::<u64>().unwrap_or(0) + 1;
        let now = utc_now();
        let date = now.split('T').next().unwrap_or(&now);
        let row = format!(
            "| {count} | {date} | {origin} | {} |",
            summary.replace('|', "\\|")
        );
        let heading = format!("## {}", self.history_heading()?);
        let Some(content) = append_table_row(&content, &heading, &row) else {
            bail!("seção ausente: '{heading}'; nenhum ajuste registrado");
        };
        let content = frontmatter_put(&content, "adjustments", &count.to_string())?;
        let content = frontmatter_put(&content, "adjusted_at", &now)?;
        let content = frontmatter_put(&content, "updated_at", &now)?;
        write(path, &self.with_stored_hash(&content)?)?;
        Ok(count)
    }

    /// Resumo de um ajuste manual, com as seções alteradas desde o último commit quando houver.
    fn manual_summary(&self, path: &Path) -> Result<String> {
        let summary = self.spec_setting("manual_adjustment_summary")?;
        let sections = changed_sections(path, &self.history_heading()?)?;
        if sections.is_empty() {
            return Ok(summary);
        }
        Ok(format!("{summary}: {}", sections.join(", ")))
    }

    // --- Análise do plano ---

    fn effort(&self) -> Result<Effort> {
        let levels = self
            .task_list("effort_levels")?
            .into_iter()
            .map(|level| {
                let limit = self
                    .task_setting(&format!("effort_limits.{level}"))
                    .ok()
                    .filter(|limit| !limit.is_empty() && limit != "-")
                    .map(|limit| limit.parse().unwrap_or(0));
                (level, limit)
            })
            .collect();
        let weight = |name: &str| -> Result<i64> {
            Ok(self
                .task_setting(&format!("effort_weights.{name}"))?
                .parse()
                .unwrap_or(0))
        };
        Ok(Effort {
            levels,
            types: self.task_list("task_types")?,
            id_prefix: self.task_setting("id.prefix")?,
            id_digits: self.task_setting("id.digits")?.parse().unwrap_or(0),
            weights: Weights {
                create: weight("files_create")?,
                change: weight("files_change")?,
                ids: weight("ids_covered")?,
                contract: weight("public_contract")?,
                inference: weight("inference")?,
                risk: weight("high_risk")?,
            },
        })
    }

    fn analyze(&self, plan: &str) -> Result<Vec<Finding>> {
        Ok(Analyzer::new(self.effort()?).run(plan))
    }

    /// O plano com os níveis calculados na coluna "Nível", ou os problemas que impedem o cálculo.
    fn scored(&self, plan: &str) -> Result<std::result::Result<String, Vec<String>>> {
        let findings = self.analyze(plan)?;
        let problems: Vec<String> = findings.iter().filter_map(Finding::problem).collect();
        if !problems.is_empty() {
            return Ok(Err(problems));
        }
        let levels = findings
            .into_iter()
            .filter_map(|finding| match finding {
                Finding::Level { id, computed, .. } => Some((id, computed)),
                Finding::Problem(_) => None,
            })
            .collect();
        Ok(Ok(rewrite_levels(plan, &levels)))
    }

    fn plan_is_outdated(&self, plan: &Path, spec: &Path) -> Result<bool> {
        let approved = field(spec, &self.spec_setting("approved_hash_field")?);
        Ok(approved.is_empty() || approved == "null" || field(plan, "spec_hash") != approved)
    }

    /// IDs da spec que o plano precisa cobrir: primeira coluna das seções `coverage_<tipo>`.
    fn spec_ids(&self, spec: &Path) -> Result<Vec<String>> {
        let content = read(spec)?;
        let sections = self
            .task_list(&format!("coverage_{}", field(spec, "type")))
            .unwrap_or_default();
        let mut ids = Vec::new();
        for section in sections {
            let heading = self.section_heading(&section)?;
            let text = markdown_section(&content, &heading).unwrap_or_default();
            ids.extend(
                markdown_table_rows(&text)
                    .iter()
                    .map(|row| markdown_cell(row, 1))
                    .filter(|id| id != "-"),
            );
        }
        Ok(ids)
    }

    fn plan_errors(&self, plan: &str, spec: &Path) -> Result<Vec<String>> {
        let mut errors: Vec<String> = template_leftovers(plan)
            .into_iter()
            .map(|(line, text)| format!("variável sem preencher: {line}:{text}"))
            .collect();
        let overview = markdown_section(plan, OVERVIEW_HEADING).unwrap_or_default();
        if overview.trim().is_empty() {
            errors.push(format!("a seção '{OVERVIEW_HEADING}' está vazia"));
        }
        errors.extend(
            self.analyze(plan)?
                .into_iter()
                .filter_map(|finding| finding.into_error()),
        );
        errors.extend(self.coverage_errors(plan, spec)?);
        Ok(errors)
    }

    fn coverage_errors(&self, plan: &str, spec: &Path) -> Result<Vec<String>> {
        let tasks = markdown_section(plan, TASKS_HEADING).unwrap_or_default();
        let covered: Vec<String> = markdown_table_rows(&tasks)
            .iter()
            .flat_map(|row| {
                markdown_cell(row, 7)
                    .split(',')
                    .map(|id| id.trim().to_string())
                    .collect::<Vec<_>>()
            })
            .filter(|id| !id.is_empty() && id != "-")
            .collect();
        let ids = self.spec_ids(spec)?;
        let mut errors: Vec<String> = ids
            .iter()
            .filter(|id| !id.is_empty() && !covered.contains(id))
            .map(|id| format!("{id} não é coberto por nenhuma tarefa"))
            .collect();
        let unique: BTreeSet<&String> = covered.iter().collect();
        errors.extend(
            unique
                .into_iter()
                .filter(|id| !ids.contains(id))
                .map(|id| format!("a tarefa cobre {id}, que não existe na spec")),
        );
        Ok(errors)
    }

    fn outdated_message(spec_id: &str) -> String {
        format!(
            "o plano foi feito para outra versão da spec {spec_id}; recrie-o com: vostok plan create {spec_id}"
        )
    }

    /// Problemas do plano, com o aviso de plano desatualizado primeiro.
    fn all_errors(
        &self,
        spec_id: &str,
        plan: &Path,
        content: &str,
        spec: &Path,
    ) -> Result<Vec<String>> {
        let mut errors = Vec::new();
        if self.plan_is_outdated(plan, spec)? {
            errors.push(Self::outdated_message(spec_id));
        }
        errors.extend(self.plan_errors(content, spec)?);
        Ok(errors)
    }

    // --- plan create | score | validate ---

    fn create(&self, spec_id: &str) -> Result<()> {
        let spec = self.find_spec(spec_id)?;
        let approved_status = self.spec_setting("status.approved")?;
        let planned_status = self.spec_setting("status.planned")?;
        let status = field(&spec, "status");
        if status == planned_status {
            bail!("a spec {spec_id} já está com status '{planned_status}': as tarefas já foram criadas");
        }
        if status != approved_status {
            bail!("a spec {spec_id} está com status '{status}'; só specs com status '{approved_status}' são planejadas");
        }
        let approved = field(&spec, &self.spec_setting("approved_hash_field")?);
        let stored = field(&spec, &self.spec_setting("content_hash_field")?);
        let current = sha256_hex(frontmatter_body(&read(&spec)?).as_bytes());
        if approved != stored || approved != current {
            bail!("a spec {spec_id} mudou depois da aprovação; aprove-a de novo antes de planejar");
        }

        let plan = self.plan_path(spec_id)?;
        if plan.is_file() {
            if !self.plan_is_outdated(&plan, &spec)? {
                bail!(
                    "a spec {spec_id} já tem um plano atualizado: {}",
                    plan.display()
                );
            }
            println!(
                "Aviso: o plano existente foi feito para outra versão da spec e será recriado."
            );
        }

        let template = self.template("plan_template")?;
        let history =
            read(&self.section_template(&self.spec_setting("adjustment_history_section")?)?)?;
        let now = utc_now();
        let values = HashMap::from([
            ("SPEC_ID", spec_id.to_string()),
            ("SPEC_TITLE", field(&spec, "title")),
            ("SPEC_HASH", approved),
            ("DATETIME", now),
            ("EFFORT_LEVELS", self.effort_level_rows()?),
            (
                "ADJUSTMENT_HISTORY",
                history.trim_end_matches('\n').to_string(),
            ),
        ]);
        if let Some(dir) = plan.parent() {
            fs::create_dir_all(dir)
                .with_context(|| format!("não foi possível criar {}", dir.display()))?;
        }
        let content = render_lines(&read(&template)?, &values);
        write(&plan, &self.with_stored_hash(&content)?)?;
        println!("Plano de {spec_id} criado em {}", plan.display());
        Ok(())
    }

    fn effort_level_rows(&self) -> Result<String> {
        let rows = self
            .task_list("effort_levels")?
            .iter()
            .map(|level| {
                let description = self.task_setting(&format!("effort_descriptions.{level}"))?;
                Ok(format!("| {level} | {description} |"))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(rows.join("\n"))
    }

    fn missing_plan_for_planning(spec_id: &str) -> String {
        format!(
            "a spec {spec_id} não tem plano de tarefas. Crie-o com: vostok plan create {spec_id}"
        )
    }

    fn score(&self, spec_id: &str) -> Result<()> {
        let plan = self.existing_plan(spec_id, Self::missing_plan_for_planning(spec_id))?;
        let content = match self.scored(&read(&plan)?)? {
            Ok(content) => content,
            Err(problems) => {
                return Err(problems_message(
                    format!("não foi possível calcular os níveis de {spec_id}; nada foi alterado:"),
                    &problems,
                ))
            }
        };
        let content = frontmatter_put(&content, "updated_at", &utc_now())?;
        write(&plan, &self.with_stored_hash(&content)?)?;
        println!(
            "Níveis de esforço gravados no plano de {spec_id}: {}",
            plan.display()
        );
        Ok(())
    }

    fn validate(&self, spec_id: &str) -> Result<()> {
        let plan = self.existing_plan(spec_id, Self::missing_plan_for_planning(spec_id))?;
        let spec = self.find_spec(spec_id)?;
        let errors = self.plan_errors(&read(&plan)?, &spec)?;
        if !errors.is_empty() {
            return Err(problems_message(
                format!("o plano de {spec_id} tem problemas:"),
                &errors,
            ));
        }
        println!("Plano de {spec_id} válido: {}", plan.display());
        Ok(())
    }

    // --- plan adjust ---

    fn plan_to_adjust(&self, spec_id: &str) -> Result<PathBuf> {
        self.existing_plan(
            spec_id,
            format!("a spec {spec_id} não tem plano de tarefas para ajustar"),
        )
    }

    fn record_manual(&self, spec_id: &str) -> Result<()> {
        let plan = self.plan_to_adjust(spec_id)?;
        if self.hash_unchanged(&plan)? {
            println!("Nenhuma edição manual.");
            return Ok(());
        }
        let summary = self.manual_summary(&plan)?;
        let origin = self.spec_setting("adjustment_origins.manual")?;
        let count = self.record_adjustment(&plan, &origin, &summary)?;
        println!("Ajuste manual {count} registrado no plano de {spec_id}: {summary}");
        Ok(())
    }

    fn adjust_validate(&self, spec_id: &str) -> Result<()> {
        let plan = self.plan_to_adjust(spec_id)?;
        let spec = self.find_spec(spec_id)?;
        let errors = self.all_errors(spec_id, &plan, &read(&plan)?, &spec)?;
        if !errors.is_empty() {
            return Err(problems_message(
                format!("o plano de {spec_id} tem problemas:"),
                &errors,
            ));
        }
        println!("Plano de {spec_id} válido: {}", plan.display());
        Ok(())
    }

    fn record(&self, spec_id: &str, summary: &str) -> Result<()> {
        let summary = clean_summary(summary)?;
        let plan = self.plan_to_adjust(spec_id)?;
        let spec = self.find_spec(spec_id)?;
        let header = format!("o plano de {spec_id} tem problemas; nenhum ajuste registrado:");
        let content = match self.scored(&read(&plan)?)? {
            Ok(content) => content,
            Err(problems) => return Err(problems_message(header, &problems)),
        };
        let errors = self.all_errors(spec_id, &plan, &content, &spec)?;
        if !errors.is_empty() {
            return Err(problems_message(header, &errors));
        }
        write(&plan, &content)?;
        let origin = self.spec_setting("adjustment_origins.agent")?;
        let count = self.record_adjustment(&plan, &origin, &summary)?;
        println!(
            "Ajuste {count} registrado no plano de {spec_id}: {}",
            plan.display()
        );
        Ok(())
    }

    // --- plan approve ---

    fn plan_to_approve(&self, spec_id: &str) -> Result<PathBuf> {
        self.existing_plan(
            spec_id,
            format!("a spec {spec_id} não tem plano de tarefas. Crie-o com a skill plan-tasks"),
        )
    }

    /// Impedimentos do plano: status da spec, spec ou plano alterados e problemas de validação.
    fn plan_blockers(&self, spec_id: &str, plan: &Path, spec: &Path) -> Result<Vec<String>> {
        let approved_status = self.spec_setting("status.approved")?;
        let status = field(spec, "status");
        if status != approved_status {
            return Ok(vec![format!(
                "a spec {spec_id} está com status '{status}'; só planos de specs com status '{approved_status}' são aprovados"
            )]);
        }
        let mut blockers = Vec::new();
        if !self.hash_unchanged(spec)? {
            blockers.push(format!(
                "a spec {spec_id} mudou depois da aprovação; aprove-a de novo e recrie o plano"
            ));
        }
        if self.plan_is_outdated(plan, spec)? {
            blockers.push(Self::outdated_message(spec_id));
        }
        if !self.hash_unchanged(plan)? {
            blockers.push(format!(
                "edição manual no plano sem registro; registre-a com: vostok plan adjust record-manual {spec_id}"
            ));
        }
        blockers.extend(self.plan_errors(&read(plan)?, spec)?);
        Ok(blockers)
    }

    fn create_tasks(&self, spec_id: &str) -> Result<()> {
        let plan = self.plan_to_approve(spec_id)?;
        let spec = self.find_spec(spec_id)?;
        let blockers = self.plan_blockers(spec_id, &plan, &spec)?;
        if !blockers.is_empty() {
            return Err(problems_message(
                format!("o plano de {spec_id} não pode ser aprovado; nada foi alterado:"),
                &blockers,
            ));
        }
        let target = self.tasks_dir(spec_id)?;
        if target.exists() {
            bail!(
                "a pasta de tarefas de {spec_id} já existe: {}",
                target.display()
            );
        }
        let template = read(&self.template("task_template")?)?;
        let pending = self.task_setting("status.pending")?;

        // As tarefas são montadas em memória e gravadas juntas: ou todas existem, ou nenhuma.
        let mut files = Vec::new();
        for task in plan_tasks(&read(&plan)?) {
            let depends = as_list(&task.depends);
            let blocked_by = if depends.is_empty() {
                "-".to_string()
            } else {
                depends.clone()
            };
            let values = HashMap::from([
                ("TASK_ID", task.id.clone()),
                ("SPEC_ID", spec_id.to_string()),
                ("TASK_TITLE", task.title.clone()),
                ("TASK_TITLE_YAML", yaml_escape(&task.title)),
                ("TASK_EFFORT", task.effort.clone()),
                ("TASK_DEPENDS", depends),
                ("TASK_COVERS", as_list(&task.covers)),
                ("TASK_TYPE", task.kind.clone()),
                ("TASK_STATUS", pending.clone()),
                ("TASK_BLOCKED_BY", blocked_by),
            ]);
            let content = self.with_stored_hash(&render_lines(&template, &values))?;
            files.push((format!("{}.md", task.id), content));
        }
        fs::create_dir_all(&target)
            .with_context(|| format!("não foi possível criar {}", target.display()))?;
        for (name, content) in &files {
            write(&target.join(name), content)?;
        }
        println!("Tarefas de {spec_id} criadas em {}:", target.display());
        let names: BTreeSet<&String> = files.iter().map(|(name, _)| name).collect();
        for name in names {
            println!("- {name}");
        }
        Ok(())
    }

    fn approve_validate(&self, spec_id: &str) -> Result<()> {
        let plan = self.plan_to_approve(spec_id)?;
        let errors = self.tasks_errors(spec_id, &read(&plan)?)?;
        if !errors.is_empty() {
            return Err(problems_message(
                format!("as tarefas de {spec_id} têm problemas:"),
                &errors,
            ));
        }
        println!(
            "Tarefas de {spec_id} válidas: {}",
            self.tasks_dir(spec_id)?.display()
        );
        Ok(())
    }

    fn mark_planned(&self, spec_id: &str) -> Result<()> {
        let plan = self.plan_to_approve(spec_id)?;
        let spec = self.find_spec(spec_id)?;
        let mut errors = self.plan_blockers(spec_id, &plan, &spec)?;
        errors.extend(self.tasks_errors(spec_id, &read(&plan)?)?);
        if !errors.is_empty() {
            return Err(problems_message(
                format!("o plano de {spec_id} não pode ser concluído; nada foi alterado:"),
                &errors,
            ));
        }

        // As tarefas já estão detalhadas e validadas: o hash passa a valer para a aprovação delas.
        for file in markdown_files(&self.tasks_dir(spec_id)?) {
            self.store_hash(&file)?;
        }
        let planned = self.spec_setting("status.planned")?;
        let content = read(&spec)?;
        let Some(content) = frontmatter_set(&content, "status", &planned) else {
            bail!(
                "Chave 'status' não encontrada no front-matter de {}",
                spec.display()
            );
        };
        write(&spec, &frontmatter_put(&content, "updated_at", &utc_now())?)?;
        fs::remove_file(&plan)
            .with_context(|| format!("não foi possível remover {}", plan.display()))?;
        println!(
            "Spec {spec_id} marcada como '{planned}': {}",
            spec.display()
        );
        println!("Plano removido: {}", plan.display());
        Ok(())
    }

    // --- Validação das tarefas ---

    /// Problemas de todas as tarefas da spec, um por item.
    fn tasks_errors(&self, spec_id: &str, plan: &str) -> Result<Vec<String>> {
        let dir = self.tasks_dir(spec_id)?;
        if !dir.is_dir() {
            return Ok(vec![format!(
                "as tarefas de {spec_id} ainda não foram criadas; execute: vostok plan approve create-tasks {spec_id}"
            )]);
        }
        let mut tasks = plan_tasks(plan);
        let ids: Vec<String> = tasks.iter().map(|task| task.id.clone()).collect();
        let upstream = self.upstream_paths(&dir, &tasks)?;
        if tasks.is_empty() {
            tasks.push(PlanTask::default());
        }
        let mut errors = Vec::new();
        let none = HashSet::new();
        for task in &tasks {
            let file = dir.join(format!("{}.md", task.id));
            if !file.is_file() {
                errors.push(format!(
                    "{}: arquivo da tarefa ausente em {}",
                    task.id,
                    dir.display()
                ));
                continue;
            }
            let created = upstream.get(&task.id).unwrap_or(&none);
            errors.extend(self.task_errors(&file, &task.covers, created)?);
        }
        for file in markdown_files(&dir) {
            let name = file_stem(&file);
            if !ids.contains(&name) {
                errors.push(format!("{name}: tarefa que não existe no plano"));
            }
        }
        Ok(errors)
    }

    /// Caminhos criados pelas tarefas de que cada tarefa depende, direta ou indiretamente.
    fn upstream_paths(
        &self,
        dir: &Path,
        tasks: &[PlanTask],
    ) -> Result<HashMap<String, HashSet<String>>> {
        let mut graph = CreatedPaths::default();
        for task in tasks {
            let depends = as_list(&task.depends);
            for dependency in depends.split(", ").filter(|dep| !dep.is_empty()) {
                graph.add_dependency(&task.id, dependency);
            }
            let file = dir.join(format!("{}.md", task.id));
            if file.is_file() {
                for (action, path) in self.change_items(&read(&file)?)? {
                    if action == "criar" {
                        graph.add_created(&task.id, &path);
                    }
                }
            }
        }
        Ok(graph.resolve())
    }

    fn execution_subsection(&self, content: &str, key: &str) -> Result<Option<String>> {
        Ok(markdown_subsection(
            content,
            &self.task_setting("execution_section")?,
            &self.task_setting(key)?,
        ))
    }

    /// Itens de "Mudar" no formato "- `caminho` (ação): motivo", como `(ação, caminho)`.
    fn change_items(&self, content: &str) -> Result<Vec<(String, String)>> {
        let text = self
            .execution_subsection(content, "change_subsection")?
            .unwrap_or_default();
        Ok(text.split('\n').filter_map(change_item).collect())
    }

    /// Problemas de uma tarefa, um por item, prefixados pelo ID.
    fn task_errors(
        &self,
        file: &Path,
        plan_covers: &str,
        upstream: &HashSet<String>,
    ) -> Result<Vec<String>> {
        let content = read(file)?;
        let id = file_stem(file);
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
        let covers = field(file, "covers").replace(['[', ']'], "");
        if sorted_list(&covers) != sorted_list(plan_covers) {
            errors.push(format!(
                "cobertura '{covers}' diferente da do plano '{plan_covers}'"
            ));
        }
        Ok(errors
            .into_iter()
            .map(|error| format!("{id}: {error}"))
            .collect())
    }

    /// Seções escritas só pelos scripts: precisam existir com a tabela, para receber as linhas.
    fn fixed_section_errors(&self, content: &str) -> Result<Vec<String>> {
        let mut errors = Vec::new();
        for section in [
            self.task_setting("record_section")?,
            self.history_heading()?,
        ] {
            match markdown_section(content, &section) {
                None => errors.push(format!("seção ausente: '## {section}'")),
                Some(text) if !text.split('\n').any(|line| line.starts_with('|')) => {
                    errors.push(format!("a seção '## {section}' está sem a tabela"))
                }
                Some(_) => {}
            }
        }
        Ok(errors)
    }

    fn subsection_errors(&self, content: &str) -> Result<Vec<String>> {
        let mut errors = Vec::new();
        for section in ["execution", "description"] {
            let title = self.task_setting(&format!("{section}_section"))?;
            for sub in self.task_list(&format!("{section}_subsections"))? {
                match markdown_subsection(content, &title, &sub) {
                    None => errors.push(format!("subseção ausente: '### {sub}'")),
                    Some(text) if text.trim().is_empty() => {
                        errors.push(format!("a subseção '{sub}' está vazia"))
                    }
                    Some(_) => {}
                }
            }
        }
        Ok(errors)
    }

    /// Resumo executivo dentro do limite de linhas e campos obrigatórios do card.
    fn description_errors(&self, content: &str) -> Result<Vec<String>> {
        let mut errors = Vec::new();
        let section = self.task_setting("description_section")?;
        let max: usize = self
            .task_setting("executive_summary_max_lines")?
            .parse()
            .unwrap_or(0);
        let summary = self.task_setting("executive_summary_subsection")?;
        let lines = markdown_subsection(content, &section, &summary)
            .unwrap_or_default()
            .split('\n')
            .filter(|line| !line.trim().is_empty())
            .count();
        if lines > max {
            errors.push(format!(
                "o resumo executivo tem {lines} linhas (máximo: {max})"
            ));
        }
        let card = markdown_subsection(content, &section, &self.task_setting("card_subsection")?)
            .unwrap_or_default();
        for name in self.task_list("card_fields")? {
            if !card.contains(&format!("- **{name}:**")) {
                errors.push(format!("campo do card ausente: '{name}'"));
            }
        }
        Ok(errors)
    }

    /// Arquivos a mudar, com a ação coerente com a existência do caminho. Um caminho criado por
    /// uma tarefa de que esta depende conta como existente.
    fn change_errors(&self, content: &str, upstream: &HashSet<String>) -> Result<Vec<String>> {
        let root = self.harness.path("root")?;
        let items = self.change_items(content)?;
        let mut errors = Vec::new();
        for (action, path) in &items {
            if !COMPONENT_ACTIONS.split(' ').any(|known| known == action) {
                errors.push(format!(
                    "ação inválida para '{path}': '{action}' (use: {COMPONENT_ACTIONS})"
                ));
                continue;
            }
            let exists = Path::new(&format!("{}/{path}", root.display())).exists();
            if action == "criar" && exists {
                errors.push(format!("'{path}' já existe; use a ação 'alterar'"));
            } else if action != "criar" && !exists && !upstream.contains(path) {
                errors.push(format!(
                    "'{path}' não existe nem é criado por uma tarefa de que ela depende; use a ação 'criar' ou corrija o caminho"
                ));
            }
        }
        if items.is_empty() {
            errors.push(format!(
                "'{}' sem nenhum item no formato: - `caminho` (ação): motivo",
                self.task_setting("change_subsection")?
            ));
        }
        Ok(errors)
    }

    /// Critérios de conclusão: um checkbox por critério, seguido da forma de comprová-lo numa
    /// citação recuada ("  > Como comprovar: ...").
    fn done_errors(&self, content: &str) -> Result<Vec<String>> {
        let sub = self.task_setting("done_subsection")?;
        let label = self.task_setting("done_proof_label")?;
        let text = command_output(
            &self
                .execution_subsection(content, "done_subsection")?
                .unwrap_or_default(),
        );
        let mut errors = Vec::new();
        let mut criterion = String::new();
        let mut proven = true;
        let mut count = 0;
        for item in text.split('\n').filter(|item| !item.trim().is_empty()) {
            if is_criterion(item) {
                if !proven {
                    errors.push(format!("'{sub}' com critério sem '{label}': {criterion}"));
                }
                criterion = item.to_string();
                proven = false;
                count += 1;
            } else if let Some(proof) = proof_text(item, &label).filter(|_| !criterion.is_empty()) {
                proven |= !proof.trim().is_empty();
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
        let sub = self.task_setting("baseline_subsection")?;
        let text = command_output(
            &self
                .execution_subsection(content, "baseline_subsection")?
                .unwrap_or_default(),
        );
        if text.is_empty() {
            return Ok(Vec::new());
        }
        let mut errors = Vec::new();
        let mut count = 0;
        for item in text.split('\n').filter(|item| !item.trim().is_empty()) {
            let command = item
                .strip_prefix("- ")
                .and_then(backticked)
                .filter(|(_, rest)| rest.is_empty());
            if command.is_some() {
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

    /// Itens de "Evidências" automáticos ou manuais, ou o texto único de `evidence_none`.
    fn evidence_errors(&self, content: &str) -> Result<Vec<String>> {
        let sub = self.task_setting("evidence_subsection")?;
        let automatic = self.task_setting("evidence_automatic")?;
        let manual = self.task_setting("evidence_manual")?;
        let none = self.task_setting("evidence_none")?;
        let text = command_output(
            &self
                .execution_subsection(content, "evidence_subsection")?
                .unwrap_or_default(),
        );
        if text.is_empty() {
            return Ok(Vec::new());
        }
        let meaningful: Vec<&str> = text
            .split('\n')
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect();
        if meaningful.join("\n") == none {
            return Ok(Vec::new());
        }
        let mut errors = Vec::new();
        let mut count = 0;
        for item in text
            .split('\n')
            .filter(|item| !item.trim().is_empty() && !is_indented(item))
        {
            if is_automatic_evidence(item, &automatic) || is_manual_evidence(item, &manual) {
                count += 1;
                errors.extend(
                    evidence_name_problem(item).map(|problem| format!("'{sub}' com {problem}")),
                );
            } else {
                errors.push(format!(
                    "'{sub}' com item fora do formato: {item} (use: - `arquivo` ({automatic}): `comando` ou - `arquivo` ({manual}): descrição)"
                ));
            }
        }
        if count == 0 {
            errors.push(format!(
                "'{sub}' sem nenhum item no formato e sem o texto '{none}'"
            ));
        }
        let fields = self.evidence_fields();
        errors.extend(evidence_subitem_errors(&text, &sub, fields.as_ref(), true));
        Ok(errors)
    }
}

// --- Análise do plano: níveis de esforço, tabelas e dependências ---

struct Weights {
    create: i64,
    change: i64,
    ids: i64,
    contract: i64,
    inference: i64,
    risk: i64,
}

/// Níveis com o limite de pontuação (o último não tem), tipos aceitos, formato do ID e pesos.
struct Effort {
    levels: Vec<(String, Option<i64>)>,
    types: Vec<String>,
    id_prefix: String,
    id_digits: usize,
    weights: Weights,
}

impl Effort {
    fn level_for(&self, total: i64) -> String {
        self.levels
            .iter()
            .find(|(_, limit)| limit.is_some_and(|limit| total <= limit))
            .or(self.levels.last())
            .map(|(level, _)| level.clone())
            .unwrap_or_default()
    }

    fn is_task_id(&self, id: &str) -> bool {
        id.strip_prefix(&format!("{}-", self.id_prefix))
            .is_some_and(|digits| {
                digits.len() == self.id_digits && digits.chars().all(|c| c.is_ascii_digit())
            })
    }

    fn types_text(&self) -> String {
        self.types.iter().map(|kind| format!("{kind} ")).collect()
    }
}

/// Resultado da análise, na ordem em que os problemas aparecem.
enum Finding {
    Problem(String),
    Level {
        id: String,
        computed: String,
        stored: String,
    },
}

impl Finding {
    fn problem(&self) -> Option<String> {
        match self {
            Finding::Problem(problem) => Some(problem.clone()),
            Finding::Level { .. } => None,
        }
    }

    /// Problema para a validação: inclui o nível gravado diferente do calculado.
    fn into_error(self) -> Option<String> {
        match self {
            Finding::Problem(problem) => Some(problem),
            Finding::Level {
                id,
                computed,
                stored,
            } => (computed != stored).then(|| {
                format!(
                    "{id}: nível \"{stored}\" diferente do calculado \"{computed}\" (execute vostok plan score)"
                )
            }),
        }
    }
}

/// Analisa as tabelas de tarefas e de fatos de esforço do plano.
struct Analyzer {
    effort: Effort,
    findings: Vec<Finding>,
    order: Vec<String>,
    tasks: HashSet<String>,
    stored: HashMap<String, String>,
    depends: HashMap<String, Vec<String>>,
    covered: HashMap<String, i64>,
    facts: Vec<String>,
    scores: HashMap<String, i64>,
    invalid: HashSet<String>,
}

impl Analyzer {
    fn new(effort: Effort) -> Self {
        Self {
            effort,
            findings: Vec::new(),
            order: Vec::new(),
            tasks: HashSet::new(),
            stored: HashMap::new(),
            depends: HashMap::new(),
            covered: HashMap::new(),
            facts: Vec::new(),
            scores: HashMap::new(),
            invalid: HashSet::new(),
        }
    }

    fn problem(&mut self, message: String) {
        self.findings.push(Finding::Problem(message));
    }

    fn run(mut self, plan: &str) -> Vec<Finding> {
        for row in table_rows(plan, TASKS_HEADING) {
            self.task_row(&row);
        }
        for row in table_rows(plan, FACTS_HEADING) {
            self.fact_row(&row);
        }
        self.finish();
        self.findings
    }

    fn task_row(&mut self, line: &str) {
        let cells: Vec<&str> = line.split('|').collect();
        let id = blank_trim(cells.get(1).copied().unwrap_or("")).to_string();
        if id == "-" {
            return;
        }
        if cells.len() != 9 {
            let columns = cells.len() as i64 - 2;
            self.problem(format!(
                "linha da tabela de tarefas com {columns} colunas (esperado 7): {line}"
            ));
            return;
        }
        if !self.effort.is_task_id(&id) {
            self.problem(format!("ID de tarefa inválido: \"{id}\""));
        }
        if !self.tasks.insert(id.clone()) {
            self.problem(format!("ID de tarefa repetido: {id}"));
        }
        self.order.push(id.clone());
        let kind = blank_trim(cells[4]);
        if !self.effort.types.iter().any(|known| known == kind) {
            let types = self.effort.types_text();
            self.problem(format!("{id}: tipo inválido \"{kind}\" (use: {types})"));
        }
        self.stored
            .insert(id.clone(), blank_trim(cells[5]).to_string());
        self.depends.insert(id.clone(), id_list(cells[6]));
        let covered = id_list(cells[7]).len() as i64;
        self.covered.insert(id, covered);
    }

    fn fact_row(&mut self, line: &str) {
        let cells: Vec<&str> = line.split('|').collect();
        let id = blank_trim(cells.get(1).copied().unwrap_or("")).to_string();
        if id == "-" {
            return;
        }
        if cells.len() != 8 {
            let columns = cells.len() as i64 - 2;
            self.problem(format!(
                "linha da tabela de fatos com {columns} colunas (esperado 6): {line}"
            ));
            return;
        }
        if self.facts.contains(&id) {
            self.problem(format!("fatos repetidos para {id}"));
        } else {
            self.facts.push(id.clone());
        }
        let weights = &self.effort.weights;
        let (create, change, contract, inference, risk) = (
            weights.create,
            weights.change,
            weights.contract,
            weights.inference,
            weights.risk,
        );
        let score = self.number(cells[2], &id, "Arquivos a criar") * create
            + self.number(cells[3], &id, "Arquivos a alterar") * change
            + self.yes_no(cells[4], &id, "Contrato público") * contract
            + self.yes_no(cells[5], &id, "Inferência") * inference
            + self.yes_no(cells[6], &id, "Risco alto") * risk;
        self.scores.insert(id, score);
    }

    fn number(&mut self, cell: &str, id: &str, name: &str) -> i64 {
        let value = blank_trim(cell);
        if !value.is_empty() && value.chars().all(|c| c.is_ascii_digit()) {
            return value.parse().unwrap_or(i64::MAX);
        }
        self.problem(format!(
            "{id}: fato \"{name}\" inválido: \"{value}\" (use um número inteiro)"
        ));
        self.invalid.insert(id.to_string());
        0
    }

    fn yes_no(&mut self, cell: &str, id: &str, name: &str) -> i64 {
        let value = blank_trim(cell).to_lowercase();
        match value.as_str() {
            "sim" => 1,
            "não" | "nao" => 0,
            _ => {
                self.problem(format!(
                    "{id}: fato \"{name}\" inválido: \"{value}\" (use sim ou não)"
                ));
                self.invalid.insert(id.to_string());
                0
            }
        }
    }

    fn finish(&mut self) {
        if self.order.is_empty() {
            self.problem("a tabela de tarefas está vazia".to_string());
        }
        for id in self.order.clone() {
            self.check_dependencies(&id);
            if !self.facts.contains(&id) {
                self.problem(format!("{id}: sem fatos de esforço"));
                continue;
            }
            if self.invalid.contains(&id) {
                continue;
            }
            let total = self.scores[&id] + self.covered[&id] * self.effort.weights.ids;
            self.findings.push(Finding::Level {
                computed: self.effort.level_for(total),
                stored: self.stored[&id].clone(),
                id,
            });
        }
        for id in self.facts.clone() {
            if !self.tasks.contains(&id) {
                self.problem(format!("fatos de esforço de uma tarefa inexistente: {id}"));
            }
        }
        self.check_cycles();
    }

    fn check_dependencies(&mut self, id: &str) {
        for dependency in self.depends[id].clone() {
            if dependency == id {
                self.problem(format!("{id}: depende de si mesma"));
            } else if !self.tasks.contains(&dependency) {
                self.problem(format!("{id}: dependência inexistente {dependency}"));
            }
        }
    }

    /// Busca em profundidade: um vértice em andamento revisitado fecha um ciclo.
    fn check_cycles(&mut self) {
        let mut visits = Visits::default();
        for id in self.order.clone() {
            if !visits.state.contains_key(&id) {
                self.visit(&id, &mut visits);
            }
        }
    }

    fn visit(&mut self, id: &str, visits: &mut Visits) {
        visits.state.insert(id.to_string(), Visit::Running);
        for dependency in self.depends.get(id).cloned().unwrap_or_default() {
            if !self.tasks.contains(&dependency) {
                continue;
            }
            match visits.state.get(&dependency) {
                Some(Visit::Running) => {
                    if visits.cycled.insert(dependency.clone()) {
                        self.problem(format!("dependência em ciclo envolvendo {dependency}"));
                    }
                }
                Some(Visit::Done) => {}
                None => self.visit(&dependency, visits),
            }
        }
        visits.state.insert(id.to_string(), Visit::Done);
    }
}

enum Visit {
    Running,
    Done,
}

#[derive(Default)]
struct Visits {
    state: HashMap<String, Visit>,
    cycled: HashSet<String>,
}

/// Grafo das dependências e dos caminhos criados por cada tarefa.
#[derive(Default)]
struct CreatedPaths {
    depends: HashMap<String, Vec<String>>,
    created: HashMap<String, Vec<String>>,
}

impl CreatedPaths {
    fn add_dependency(&mut self, task: &str, dependency: &str) {
        self.depends
            .entry(task.to_string())
            .or_default()
            .extend(dependency.split_whitespace().map(str::to_string));
    }

    fn add_created(&mut self, task: &str, path: &str) {
        self.created
            .entry(task.to_string())
            .or_default()
            .push(path.to_string());
    }

    fn resolve(&self) -> HashMap<String, HashSet<String>> {
        let mut result: HashMap<String, HashSet<String>> = HashMap::new();
        for (task, direct) in &self.depends {
            let mut seen = HashSet::new();
            let paths = result.entry(task.clone()).or_default();
            for dependency in direct {
                self.collect(dependency, &mut seen, paths);
            }
        }
        result
    }

    fn collect(&self, dependency: &str, seen: &mut HashSet<String>, paths: &mut HashSet<String>) {
        if !seen.insert(dependency.to_string()) {
            return;
        }
        paths.extend(self.created.get(dependency).into_iter().flatten().cloned());
        for next in self.depends.get(dependency).into_iter().flatten() {
            self.collect(next, seen, paths);
        }
    }
}

/// Linha da tabela de tarefas do plano.
#[derive(Debug, Clone, Default)]
struct PlanTask {
    id: String,
    title: String,
    kind: String,
    effort: String,
    depends: String,
    covers: String,
}

/// Tarefas do plano. Como no `read` do bash com tabulação, colunas vazias não ocupam posição.
fn plan_tasks(plan: &str) -> Vec<PlanTask> {
    table_rows(plan, TASKS_HEADING)
        .iter()
        .filter_map(|row| {
            let cells: Vec<&str> = row.split('|').collect();
            if cells.len() != 9 || blank_trim(cells[1]) == "-" {
                return None;
            }
            let mut fields = cells[1..8]
                .iter()
                .map(|cell| blank_trim(cell))
                .filter(|cell| !cell.is_empty())
                .map(str::to_string);
            let mut next = || fields.next().unwrap_or_default();
            let id = next();
            let title = next();
            let _description = next();
            Some(PlanTask {
                id,
                title,
                kind: next(),
                effort: next(),
                depends: next(),
                covers: next(),
            })
        })
        .collect()
}

fn table_rows(content: &str, heading: &str) -> Vec<String> {
    markdown_table_rows(&markdown_section(content, heading).unwrap_or_default())
}

/// IDs de uma célula separados por vírgula ou espaço; `-` é uma lista vazia.
fn id_list(cell: &str) -> Vec<String> {
    let text = blank_trim(cell).replace(',', " ");
    if text == "-" {
        return Vec::new();
    }
    text.split_whitespace().map(str::to_string).collect()
}

/// Grava o nível calculado na coluna "Nível" de cada tarefa.
fn rewrite_levels(plan: &str, levels: &HashMap<String, String>) -> String {
    let heading = format!("## {TASKS_HEADING}");
    let mut in_code = false;
    let mut inside = false;
    let mut header = false;
    let mut out = Vec::new();
    for raw in file_lines(plan) {
        let plain = raw.strip_suffix('\r').unwrap_or(raw);
        if is_fence(plain) {
            in_code = !in_code;
        }
        if !in_code && plain == heading {
            inside = true;
            out.push(raw.to_string());
            continue;
        }
        if !in_code && inside && plain.starts_with("## ") {
            inside = false;
        }
        if inside && plain.starts_with('|') {
            if !header || is_separator_row(plain) {
                header = true;
                out.push(raw.to_string());
                continue;
            }
            let mut cells: Vec<String> = plain.split('|').map(str::to_string).collect();
            if let Some(level) = levels.get(blank_trim(cells.get(1).map_or("", |c| c))) {
                if cells.len() == 9 {
                    cells[5] = format!(" {level} ");
                    out.push(cells.join("|"));
                    continue;
                }
            }
        }
        out.push(raw.to_string());
    }
    join_lines(&out)
}

fn is_separator_row(line: &str) -> bool {
    let trimmed = line.trim_end_matches([' ', '\t']);
    trimmed.len() >= 3
        && trimmed.starts_with('|')
        && trimmed.ends_with('|')
        && trimmed[1..trimmed.len() - 1]
            .chars()
            .all(|c| matches!(c, ' ' | '\t' | ':' | '|' | '-'))
        && line.contains("---")
}

// --- Itens das subseções da tarefa ---

/// `(ação, caminho)` de um item "- `caminho` (ação)".
fn change_item(line: &str) -> Option<(String, String)> {
    let (path, rest) = backticked(line.strip_prefix("- ")?)?;
    let rest = rest.strip_prefix(" (")?;
    let action: String = rest.chars().take_while(char::is_ascii_lowercase).collect();
    if action.is_empty() || !rest[action.len()..].starts_with(')') {
        return None;
    }
    Some((action, path.to_string()))
}

/// Texto entre crases no início, não vazio, e o resto da linha.
fn backticked(text: &str) -> Option<(&str, &str)> {
    let inner = text.strip_prefix('`')?;
    let end = inner.find('`')?;
    (end > 0).then(|| (&inner[..end], &inner[end + 1..]))
}

fn is_criterion(line: &str) -> bool {
    let Some(rest) = line
        .strip_prefix("- [ ] ")
        .or_else(|| line.strip_prefix("- [x] "))
    else {
        return false;
    };
    rest.chars().next().is_some_and(|c| !c.is_whitespace())
}

/// Texto depois de `<rótulo>:` numa citação recuada ("  > Como comprovar: ...").
fn proof_text<'a>(line: &'a str, label: &str) -> Option<&'a str> {
    let quote = line.trim_start();
    if quote.len() == line.len() {
        return None;
    }
    quote
        .strip_prefix("> ")?
        .strip_prefix(label)?
        .strip_prefix(':')
}

fn is_automatic_evidence(line: &str, label: &str) -> bool {
    line.strip_prefix("- ")
        .and_then(backticked)
        .and_then(|(_, rest)| rest.strip_prefix(&format!(" ({label}): ")))
        .and_then(backticked)
        .is_some_and(|(_, rest)| rest.is_empty())
}

fn is_manual_evidence(line: &str, label: &str) -> bool {
    line.strip_prefix("- ")
        .and_then(backticked)
        .and_then(|(_, rest)| rest.strip_prefix(&format!(" ({label}): ")))
        .and_then(|description| description.chars().next())
        .is_some_and(|c| !c.is_whitespace())
}

/// Converte "TASK-01, TASK-02" ou "-" numa lista separada por ", " (vazia para "-").
fn as_list(value: &str) -> String {
    if value == "-" {
        return String::new();
    }
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>()
        .join(", ")
}

fn sorted_list(value: &str) -> Vec<String> {
    let mut items: Vec<String> = as_list(value)
        .replace(' ', "")
        .split(',')
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect();
    items.sort();
    items
}

/// Limpa o resumo de um ajuste do agente: sem espaços nas pontas das linhas, não vazio e numa linha.
fn clean_summary(summary: &str) -> Result<String> {
    let lines: Vec<&str> = summary.split('\n').map(str::trim).collect();
    let summary = lines.join("\n").trim_end_matches('\n').to_string();
    if summary.is_empty() {
        bail!("o resumo do ajuste está vazio");
    }
    if summary.contains('\n') {
        bail!("o resumo não pode ter quebra de linha; separe os itens com '; '");
    }
    Ok(summary)
}

// --- Texto, front-matter e arquivos ---

fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("não foi possível ler {}", path.display()))
}

fn write(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content).with_context(|| format!("não foi possível gravar {}", path.display()))
}

/// Valor de uma chave do front-matter, ou vazio se não existir.
fn field(path: &Path, key: &str) -> String {
    frontmatter_get(path, key).unwrap_or_default()
}

fn file_stem(path: &Path) -> String {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Arquivos `.md` de uma pasta, em ordem alfabética.
fn markdown_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension().is_some_and(|ext| ext == "md"))
        .collect();
    files.sort();
    files
}

/// Linhas como o awk as lê: separadas por `\n`, sem a linha vazia depois do último `\n`.
fn file_lines(content: &str) -> Vec<&str> {
    if content.is_empty() {
        return Vec::new();
    }
    content
        .strip_suffix('\n')
        .unwrap_or(content)
        .split('\n')
        .collect()
}

/// Junta as linhas com `\n` depois de cada uma, como a saída do awk.
fn join_lines<S: AsRef<str>>(lines: &[S]) -> String {
    lines
        .iter()
        .map(|line| format!("{}\n", line.as_ref()))
        .collect()
}

/// O texto como sai de `$(...)` no bash: sem as quebras de linha do fim.
fn command_output(text: &str) -> String {
    text.trim_end_matches('\n').to_string()
}

fn blank_trim(text: &str) -> &str {
    text.trim_matches([' ', '\t'])
}

fn is_fence(line: &str) -> bool {
    line.trim_start_matches([' ', '\t']).starts_with("```")
}

/// Template preenchido linha a linha, com `\n` depois de cada linha.
fn render_lines(template: &str, values: &HashMap<&str, String>) -> String {
    join_lines(&file_lines(&render(template, values)))
}

/// Conteúdo depois do front-matter, sem `\r` no fim das linhas.
fn frontmatter_body(content: &str) -> String {
    let mut inside = false;
    let mut out = Vec::new();
    for (index, raw) in file_lines(content).into_iter().enumerate() {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        if index == 0 && line == "---" {
            inside = true;
        } else if inside && line == "---" {
            inside = false;
        } else if !inside {
            out.push(line);
        }
    }
    join_lines(&out)
}

/// O texto sem a seção `## <título>`, ignorando títulos em blocos de código.
fn without_section(body: &str, title: &str) -> String {
    let heading = format!("## {title}");
    let mut in_code = false;
    let mut skipping = false;
    let mut out = Vec::new();
    for line in file_lines(body) {
        if is_fence(line) {
            in_code = !in_code;
        }
        if !in_code && line.starts_with("## ") {
            skipping = line == heading;
        }
        if !skipping {
            out.push(line);
        }
    }
    join_lines(&out)
}

/// Troca o valor de todas as linhas da chave no front-matter. `None` se a chave não existir.
fn frontmatter_set(content: &str, key: &str, value: &str) -> Option<String> {
    let prefix = format!("{key}:");
    let mut inside = false;
    let mut found = false;
    let mut out = Vec::new();
    for (index, raw) in file_lines(content).into_iter().enumerate() {
        let plain = raw.strip_suffix('\r').unwrap_or(raw);
        if index == 0 && plain == "---" {
            inside = true;
        } else if inside && plain == "---" {
            inside = false;
        } else if inside && raw.starts_with(&prefix) {
            out.push(format!("{key}: {value}"));
            found = true;
            continue;
        }
        out.push(raw.to_string());
    }
    found.then(|| join_lines(&out))
}

/// Como `frontmatter_set`, mas acrescenta a chave no fim do front-matter se ela não existir.
fn frontmatter_put(content: &str, key: &str, value: &str) -> Result<String> {
    if let Some(updated) = frontmatter_set(content, key, value) {
        return Ok(updated);
    }
    let mut inside = false;
    let mut found = false;
    let mut out = Vec::new();
    for (index, raw) in file_lines(content).into_iter().enumerate() {
        let plain = raw.strip_suffix('\r').unwrap_or(raw);
        if index == 0 && plain == "---" {
            inside = true;
        } else if inside && plain == "---" {
            out.push(format!("{key}: {value}"));
            inside = false;
            found = true;
        }
        out.push(raw.to_string());
    }
    if !found {
        bail!("Front-matter não encontrado");
    }
    Ok(join_lines(&out))
}

/// Acrescenta uma linha depois da última linha de tabela da seção. `None` se a seção não existir.
fn append_table_row(content: &str, heading: &str, row: &str) -> Option<String> {
    let lines = file_lines(content);
    let mut in_code = false;
    let mut inside = false;
    let mut at = None;
    for (index, raw) in lines.iter().enumerate() {
        let plain = raw.strip_suffix('\r').unwrap_or(raw);
        if is_fence(plain) {
            in_code = !in_code;
            continue;
        }
        if in_code {
            continue;
        }
        if plain == heading {
            inside = true;
            at = Some(index);
            continue;
        }
        if inside && plain.starts_with("## ") {
            inside = false;
        }
        if inside && plain.starts_with('|') {
            at = Some(index);
        }
    }
    let at = at?;
    let mut out: Vec<&str> = Vec::with_capacity(lines.len() + 1);
    for (index, line) in lines.iter().enumerate() {
        out.push(line);
        if index == at {
            out.push(row);
        }
    }
    Some(join_lines(&out))
}

/// Seções (títulos `## `) com linhas alteradas desde o último commit, quando o arquivo está no git.
fn changed_sections(path: &Path, skip_heading: &str) -> Result<Vec<String>> {
    let (Some(dir), Some(name)) = (path.parent(), path.file_name()) else {
        return Ok(Vec::new());
    };
    let git = |args: &[&str]| {
        Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .arg("--")
            .arg(name)
            .output()
            .ok()
    };
    let tracked = git(&["ls-files", "--error-unmatch"]).is_some_and(|out| out.status.success());
    if !tracked {
        return Ok(Vec::new());
    }
    let diff = git(&["diff", "-U0", "HEAD"])
        .map(|out| String::from_utf8_lossy(&out.stdout).into_owned())
        .unwrap_or_default();
    let changed: HashSet<usize> = diff.lines().filter_map(hunk_start).collect();
    if changed.is_empty() {
        return Ok(Vec::new());
    }
    Ok(sections_at(&read(path)?, &changed, skip_heading))
}

/// Linha inicial do lado novo de um cabeçalho de hunk (`@@ -a,b +N,c @@`).
fn hunk_start(line: &str) -> Option<usize> {
    let rest = line.strip_prefix("@@ -")?;
    let old_len = rest
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == ',')
        .count();
    if old_len == 0 {
        return None;
    }
    let digits: String = rest[old_len..]
        .strip_prefix(" +")?
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().ok()
}

fn sections_at(content: &str, changed: &HashSet<usize>, skip_heading: &str) -> Vec<String> {
    let mut in_frontmatter = false;
    let mut in_code = false;
    let mut heading = String::new();
    let mut sections: Vec<String> = Vec::new();
    for (index, raw) in file_lines(content).into_iter().enumerate() {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        if index == 0 && line == "---" {
            in_frontmatter = true;
            continue;
        }
        if in_frontmatter {
            in_frontmatter = line != "---";
            continue;
        }
        if is_fence(line) {
            in_code = !in_code;
        }
        if !in_code {
            if let Some(title) = line.strip_prefix("## ") {
                heading = title.to_string();
            }
        }
        if changed.contains(&(index + 1))
            && !heading.is_empty()
            && heading != skip_heading
            && !sections.contains(&heading)
        {
            sections.push(heading.clone());
        }
    }
    sections
}

/// Data e hora atuais em UTC, no formato `AAAA-MM-DDTHH:MM:SSZ`.
fn utc_now() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    let (year, month, day) = civil_from_days(seconds.div_euclid(86_400));
    let time = seconds.rem_euclid(86_400);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        time / 3600,
        time % 3600 / 60,
        time % 60
    )
}

/// Data civil (ano, mês, dia) de um número de dias desde 1970-01-01 (algoritmo de Howard Hinnant).
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted - era * 146_097;
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
