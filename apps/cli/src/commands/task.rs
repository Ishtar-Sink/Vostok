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
use crate::models;
use crate::support::{
    config_get, config_list, evidence_subitem, evidence_subitem_errors, frontmatter_get,
    frontmatter_lines, frontmatter_set, is_indented, markdown_section, markdown_subsection, render,
    sha256_hex, template_leftovers, yaml_escape, EvidenceFields,
};

#[derive(Debug, Subcommand)]
pub enum TaskCommand {
    /// Imprime a seção de execução da tarefa.
    Load {
        spec_id: Option<String>,
        task_id: Option<String>,
        /// Imprime só um arquivo da tarefa: execucao, verificacao, criterios, executivo ou tecnico.
        #[arg(long)]
        part: Option<String>,
    },
    /// Imprime as tabelas das tarefas e das ondas de uma spec.
    Status { spec_id: Option<String> },
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
    /// Muda as tarefas que precisam estar concluídas antes de esta fechar.
    #[command(subcommand, name = "closes-after")]
    ClosesAfter(ClosesAfterCommand),
    /// Muda as dependências de uma tarefa em execução, a pedido do usuário.
    #[command(subcommand)]
    Depends(DependsCommand),
    /// Inicia a execução.
    Start {
        spec_id: Option<String>,
        task_id: Option<String>,
        /// Modelo escolhido pelo Coordenador, gravado no registro de execução.
        #[arg(long, value_name = "MODELO")]
        model: Option<String>,
    },
    /// Coleta o baseline.
    Baseline {
        spec_id: Option<String>,
        task_id: Option<String>,
        /// Recoleta o baseline de uma tarefa em execução, com a decisão do usuário em --motivo.
        #[arg(long)]
        recoletar: bool,
        /// Motivo da recoleta, nas palavras do usuário.
        #[arg(
            long,
            value_name = "MOTIVO",
            allow_hyphen_values = true,
            requires = "recoletar"
        )]
        motivo: Option<String>,
    },
    /// Imprime o briefing do subagente.
    Brief {
        spec_id: Option<String>,
        task_id: Option<String>,
    },
    /// Converte as tarefas da spec para o formato de pasta.
    Migrate { spec_id: Option<String> },
    /// Imprime o material para escrever o executivo e o técnico da tarefa.
    Describe {
        spec_id: Option<String>,
        task_id: Option<String>,
    },
    /// Executa a verificação sem registrar.
    Check {
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
        /// Evidência conferida pelo usuário, de qualquer tipo, pelo ID (EV-NN) ou pelo arquivo relativo à pasta de evidências da tarefa.
        #[arg(long, value_name = "EV-NN|ARQUIVO")]
        reviewed: Vec<String>,
        /// Evidência rejeitada pelo usuário, pelo ID (EV-NN) ou pelo arquivo; exige --motivo e devolve a tarefa para in_progress, sem bloquear.
        #[arg(long, value_name = "EV-NN|ARQUIVO")]
        rejected: Option<String>,
        /// Motivo da rejeição, nas palavras do usuário.
        #[arg(long, value_name = "MOTIVO", allow_hyphen_values = true)]
        motivo: Option<String>,
        /// Gera as evidências no fim da execução, sem mudar status nem resultados.
        #[arg(long, conflicts_with_all = ["reviewed", "rejected"])]
        antecipada: bool,
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
    /// Persiste as evidências no projeto, sanitizadas, com o guia de interpretação.
    Persist {
        spec_id: Option<String>,
        task_id: Option<String>,
        /// Arquivo com dado sensível a redigir na cópia, relativo à pasta de evidências da tarefa.
        #[arg(long, value_name = "ARQUIVO")]
        redact: Vec<String>,
        /// Arquivo que não é copiado, por decisão do usuário.
        #[arg(long, value_name = "ARQUIVO")]
        exclude: Vec<String>,
        /// Arquivo com dado sensível copiado como está, por decisão do usuário.
        #[arg(long, value_name = "ARQUIVO")]
        accept: Vec<String>,
        /// Binário revisado, com quem revisou: agente ou usuário.
        #[arg(long, value_name = "ARQUIVO=QUEM")]
        reviewed: Vec<String>,
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
pub enum ClosesAfterCommand {
    /// Acrescenta uma tarefa que precisa estar concluída antes.
    Add {
        spec_id: Option<String>,
        task_id: Option<String>,
        target: Option<String>,
        #[arg(long = "motivo", allow_hyphen_values = true)]
        reason: Option<String>,
    },
    /// Tira uma tarefa da espera.
    Remove {
        spec_id: Option<String>,
        task_id: Option<String>,
        target: Option<String>,
        #[arg(long = "motivo", allow_hyphen_values = true)]
        reason: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum DependsCommand {
    /// Acrescenta uma dependência.
    Add {
        spec_id: Option<String>,
        task_id: Option<String>,
        dependency: Option<String>,
        #[arg(long = "motivo", allow_hyphen_values = true)]
        reason: Option<String>,
    },
    /// Tira uma dependência.
    Remove {
        spec_id: Option<String>,
        task_id: Option<String>,
        dependency: Option<String>,
        #[arg(long = "motivo", allow_hyphen_values = true)]
        reason: Option<String>,
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

const BRIEF_USAGE: &str = "Uso: vostok task brief|check <SPEC_ID> <TASK_ID>";
const DESCRIBE_USAGE: &str = "Uso: vostok task describe <SPEC_ID> <TASK_ID>";
const MIGRATE_USAGE: &str = "Uso: vostok task migrate <SPEC_ID>";
const LOAD_USAGE: &str = "Uso: vostok task load <SPEC_ID> <TASK_ID>";
const CHECK_DONE_USAGE: &str = "Uso: vostok task check-done <SPEC_ID> <TASK_ID>";
const APPROVE_USAGE: &str =
    "Uso: vostok task approve validate|approve <SPEC_ID> <TASK_ID ...>|--all";
const ADJUST_USAGE: &str = "Uso: vostok task adjust record-manual <SPEC_ID> <TASK_ID> | validate <SPEC_ID> <TASK_ID> | record <SPEC_ID> <TASK_ID> \"resumo\"";
const BLOCKED_BY_PREFIX: &str = "- **Bloqueada por:** ";
const WAITING_ON_FIELD: &str = "waiting_on";
const WAITING_REASON_FIELD: &str = "waiting_reason";
const DEPENDS_USAGE: &str =
    "Uso: vostok task depends add|remove <SPEC_ID> <TASK_ID> <DEPENDENCIA> --motivo \"texto\"";
const CLOSES_AFTER_USAGE: &str =
    "Uso: vostok task closes-after add|remove <SPEC_ID> <TASK_ID> <TAREFA> --motivo \"texto\"";
const STATUS_USAGE: &str = "Uso: vostok task status <SPEC_ID>";
const EXECUTE_USAGE: &str = "Uso: vostok task start|baseline <SPEC_ID> <TASK_ID>";
const FINISH_USAGE: &str = "Uso: vostok task tests|regression|evidence|finish <SPEC_ID> <TASK_ID> | vostok task evidence <SPEC_ID> <TASK_ID> [--reviewed <ARQUIVO>]... [--rejected <ARQUIVO> --motivo \"motivo\"] | vostok task resume <SPEC_ID> <TASK_ID> [--accept-regression <N[,N...]>] \"decisão\"";

/// Aceites de diferença de regressão, em `validation/regression/`: uma linha por aceite, com número,
/// SHA-256 da saída aceita, data, comando, decisão e código de saída aceito, separados por tabulação.
/// Linhas antigas, sem o código de saída, valem pelo hash.
const ACCEPTED_FILE: &str = "accepted";

const PERSIST_USAGE: &str = "Uso: vostok task persist <SPEC_ID> <TASK_ID> [--redact|--exclude|--accept <ARQUIVO>]... [--reviewed <ARQUIVO>=<agente|usuário>]...";
const PERSIST_TEMPLATE: &str = "task/persist-readme.md";
const PERSIST_ENTRY_MARKER: &str = "<!-- entrada -->\n";
const PERSIST_TRECHO_LABEL: &str = "- **Trecho que prova:** ";
const PERSIST_REVIEW_LABEL: &str = "- **Revisão:** ";
const REVIEWERS: [&str; 2] = ["agente", "usuário"];

/// Ações aceitas em "Mudar", as mesmas dos componentes afetados da spec.
const COMPONENT_ACTIONS: [&str; 3] = ["criar", "alterar", "remover"];
const CREATE_ACTION: &str = "criar";
const ALTER_ACTION: &str = "alterar";

/// Trap da "Verificação": registra o comando que falhou, já que muitos comandos de teste falham sem
/// imprimir nada.
const VERIFY_TRAP: &str = "trap 'echo \"comando com falha (código $?): $BASH_COMMAND\"' ERR";

/// Rótulo e tipo de evidência da verificação para projetos sem as chaves em `config/tasks.yml`.
const DEFAULT_VERIFY_LABEL: &str = "# evidência:";
const DEFAULT_EVIDENCE_FROM_VERIFY: &str = "automática, da verificação";
const DEFAULT_EVIDENCE_DIFF: &str = "diff";
const DEFAULT_EVIDENCE_EXPECT: &str = "Espera";
pub(super) const EVIDENCE_REQUIRES: &str = "Requer";
pub(super) const EVIDENCE_TIMEOUT: &str = "Tempo-limite";
const DEFAULT_EVIDENCE_BASELINE_TIMEOUT: u64 = 120;
/// Código gravado para uma evidência interrompida pelo tempo-limite, o mesmo do `timeout`.
const TIMEOUT_EXIT: i32 = 124;
const NOT_EXECUTED: &str = "não executada:";
const DEFAULT_EVIDENCE_PRINT: &str = "print";
const EXPECTED_CHANGE_MARK: &str = "(muda)";
const DEFAULT_REGRESSION_ALERT: [&str; 4] = ["FAILED", "error", "warning", "panicked"];
/// Extensões de "Mudar" que fazem da tarefa uma tarefa com interface, como na `approve-plan`.
const INTERFACE_EXTENSIONS: [&str; 7] =
    [".html", ".css", ".scss", ".tsx", ".jsx", ".vue", ".svelte"];
/// Extensões de imagem aceitas como print manual.
const IMAGE_EXTENSIONS: [&str; 5] = [".png", ".jpg", ".jpeg", ".webp", ".gif"];
/// Primeiras palavras de um comando que só lê arquivos: como evidência, não comprovam comportamento.
const READING_COMMANDS: [&str; 6] = ["grep", "sed", "cat", "head", "tail", "awk"];

/// Roteiro fixo do briefing; `<SPEC_ID>` e `<TASK_ID>` são trocados pelos IDs da tarefa.
const BRIEF_ROUTE: &str = "1. Confira no registro abaixo a linha `baseline` e não mude nada sem ela.
2. Leia os arquivos de \"Ler\" por busca e trecho (`grep -n`, `sed -n`), nunca inteiro um arquivo com mais de 400 linhas.
3. Implemente primeiro os testes pedidos e confirme que falham.
4. Siga os \"Passos\", mudando só os arquivos de \"Mudar\".
5. Verifique com `vostok task check <SPEC_ID> <TASK_ID>` até passar.
6. Sem commit, sem stage, sem `vostok task tests|regression|evidence|finish` (nem `vostok task evidence <SPEC_ID> <TASK_ID> --antecipada`, que é do Coordenador depois da onda), sem ler segredos.
7. Se um passo exigir algo fora da tarefa, pare e devolva a pergunta.
8. Devolva um bloco de até 5 linhas: `Resultado:`, `Arquivos:`, `Verificação:` (a última linha do `vostok task check`), `Em aberto:`.
";

/// Itens fixos de `# Regras` do briefing, antes dos `rules/...` da entrada `execute-task`.
const BRIEF_RULES: [&str; 3] = [
    "rules/agent/boundaries.md",
    "rules/project/security.md",
    "rules/project/language.md",
];

pub fn run(command: TaskCommand) -> Result<()> {
    let tasks = Tasks::new(Harness::from_current_dir()?)?;
    match command {
        TaskCommand::ClosesAfter(command) => tasks.run_closes_after(command),
        TaskCommand::Depends(command) => tasks.run_depends(command),
        TaskCommand::Load {
            spec_id,
            task_id,
            part,
        } => {
            let (spec, task) = filled(spec_id, task_id, LOAD_USAGE)?;
            match part {
                Some(part) => tasks.load_part(&spec, &task, &part),
                None => tasks.load(&spec, &task),
            }
        }
        TaskCommand::Status { spec_id } => {
            let spec = spec_id
                .filter(|spec| !spec.is_empty())
                .ok_or_else(|| usage(STATUS_USAGE))?;
            print!("{}", tasks.status_report(&spec)?);
            Ok(())
        }
        TaskCommand::CheckDone { spec_id, task_id } => {
            let (spec, task) = filled(spec_id, task_id, CHECK_DONE_USAGE)?;
            println!("{}", tasks.check_done(&spec, &task)?);
            Ok(())
        }
        TaskCommand::Approve(command) => tasks.run_approve(command),
        TaskCommand::Adjust(command) => tasks.run_adjust(command),
        TaskCommand::Start {
            spec_id,
            task_id,
            model,
        } => {
            let (spec, task) = given(spec_id, task_id, EXECUTE_USAGE)?;
            tasks.start(&spec, &task, model.as_deref())
        }
        TaskCommand::Baseline {
            spec_id,
            task_id,
            recoletar,
            motivo,
        } => {
            let (spec, task) = given(spec_id, task_id, EXECUTE_USAGE)?;
            if recoletar {
                tasks.recollect_baseline(&spec, &task, motivo.as_deref())
            } else {
                tasks.baseline(&spec, &task)
            }
        }
        TaskCommand::Brief { spec_id, task_id } => {
            let (spec, task) = given(spec_id, task_id, BRIEF_USAGE)?;
            tasks.brief(&spec, &task)
        }
        TaskCommand::Migrate { spec_id } => {
            let spec = spec_id
                .filter(|spec| !spec.is_empty())
                .ok_or_else(|| usage(MIGRATE_USAGE))?;
            tasks.migrate(&spec)
        }
        TaskCommand::Describe { spec_id, task_id } => {
            let (spec, task) = given(spec_id, task_id, DESCRIBE_USAGE)?;
            tasks.describe(&spec, &task)
        }
        TaskCommand::Check { spec_id, task_id } => {
            let (spec, task) = given(spec_id, task_id, BRIEF_USAGE)?;
            tasks.check(&spec, &task)
        }
        TaskCommand::Tests { spec_id, task_id } => {
            let (spec, task) = given(spec_id, task_id, FINISH_USAGE)?;
            tasks.tests(&spec, &task)
        }
        TaskCommand::Regression { spec_id, task_id } => {
            let (spec, task) = given(spec_id, task_id, FINISH_USAGE)?;
            tasks.regression(&spec, &task)
        }
        TaskCommand::Evidence {
            spec_id,
            task_id,
            reviewed,
            rejected,
            motivo,
            antecipada,
        } => {
            let (spec, task) = given(spec_id, task_id, FINISH_USAGE)?;
            if antecipada {
                return tasks.evidence_ahead(&spec, &task);
            }
            tasks.evidence(
                &spec,
                &task,
                &reviewed,
                rejected.as_deref(),
                motivo.as_deref(),
            )
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
        TaskCommand::Persist {
            spec_id,
            task_id,
            redact,
            exclude,
            accept,
            reviewed,
        } => {
            let (spec, task) = given(spec_id, task_id, PERSIST_USAGE)?;
            let options = PersistOptions {
                redact,
                exclude,
                accept,
                reviewed,
            };
            tasks.persist(&spec, &task, &options)
        }
    }
}

/// Status de uma onda a partir dos status das suas tarefas.
fn wave_status(statuses: &[String], status: &Statuses) -> String {
    let any = |name: &String| statuses.contains(name);
    if statuses.iter().all(|s| *s == status.done) {
        status.done.clone()
    } else if any(&status.blocked) {
        status.blocked.clone()
    } else if any(&status.in_progress)
        || any(&status.executed)
        || any(&status.validating)
        || any(&status.done)
    {
        status.in_progress.clone()
    } else if statuses.iter().all(|s| *s == status.approved) {
        status.approved.clone()
    } else {
        status.pending.clone()
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

/// Como a validação trata o nome de evidência com caminho.
#[derive(Clone, Copy, PartialEq)]
enum EvidenceNames {
    /// Recusa: a tarefa aprovada não é reescrita fora do ajuste (aprovação).
    Strict,
    /// Aceita o nome normalizável, que o `vostok task adjust record` troca pelo nome do arquivo.
    Normalizable,
}

/// Status configurados em `config/tasks.yml`.
struct Statuses {
    pending: String,
    approved: String,
    in_progress: String,
    executed: String,
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
            // Harness instalado antes do status `executed` não tem a chave (SPEC-047).
            executed: status("executed").unwrap_or_else(|_| "executed".to_string()),
            validating: status("validating")?,
            blocked: status("blocked")?,
            done: status("done")?,
        };
        init_layout(&harness);
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

    /// Subagentes ao mesmo tempo: `max_subagents` de 1 a 6; ausente vale 3, inválido vale 6 com aviso.
    fn max_subagents(&self) -> u32 {
        let Ok(value) = self.cfg("max_subagents") else {
            return 3;
        };
        match value.parse::<u32>() {
            Ok(n) if (1..=6).contains(&n) => n,
            _ => {
                eprintln!("Aviso: max_subagents '{value}' inválido em config/tasks.yml; use de 1 a 6. Vale 6.");
                6
            }
        }
    }

    /// Trechos de `regression_ignore`; sem a chave (projetos antigos), lista vazia.
    /// Hashes da conferência de uma evidência, com os tempos de `regression_ignore` fora do log.
    fn review_hashes(&self, bytes: &[u8]) -> ReviewHashes {
        ReviewHashes::of(bytes, &self.regression_ignore())
    }

    fn regression_ignore(&self) -> Vec<String> {
        self.cfg_list("regression_ignore")
            .unwrap_or_default()
            .into_iter()
            .filter(|item| !item.is_empty())
            .collect()
    }

    /// Trechos de `regression_alert`: uma linha nova com algum deles bloqueia o acréscimo. Sem a
    /// chave (projetos antigos), os textos padrão.
    fn regression_alert(&self) -> Vec<String> {
        match self.cfg_list("regression_alert") {
            Ok(items) => items.into_iter().filter(|item| !item.is_empty()).collect(),
            Err(_) => DEFAULT_REGRESSION_ALERT
                .iter()
                .map(|item| item.to_string())
                .collect(),
        }
    }

    /// Tempo-limite, em segundos, de cada evidência no baseline; sem a chave (projetos antigos) ou
    /// com valor inválido, o padrão.
    fn evidence_baseline_timeout(&self) -> u64 {
        self.cfg("evidence_baseline_timeout")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(DEFAULT_EVIDENCE_BASELINE_TIMEOUT)
    }

    /// Subpasta de `baseline/` com a saída das evidências automáticas; sem a chave (projetos
    /// antigos) ou vazia, nada é gravado nem comparado.
    fn evidence_baseline_dir(&self) -> Option<String> {
        self.cfg("evidence_baseline_dir")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }

    /// Início da linha que rotula um comando da "Verificação" (`verify_label`).
    fn verify_label(&self) -> String {
        self.cfg("verify_label")
            .unwrap_or_else(|_| DEFAULT_VERIFY_LABEL.to_string())
    }

    /// Tipo da evidência copiada da saída de um rótulo (`evidence_from_verify`).
    fn evidence_from_verify(&self) -> String {
        self.cfg("evidence_from_verify")
            .unwrap_or_else(|_| DEFAULT_EVIDENCE_FROM_VERIFY.to_string())
    }

    /// Tipo da evidência que grava o diff de um caminho (`evidence_diff`).
    fn evidence_diff(&self) -> String {
        self.cfg("evidence_diff")
            .unwrap_or_else(|_| DEFAULT_EVIDENCE_DIFF.to_string())
    }

    /// Tipo da evidência gerada por um comando de print (`evidence_print`).
    fn evidence_print(&self) -> String {
        self.cfg("evidence_print")
            .unwrap_or_else(|_| DEFAULT_EVIDENCE_PRINT.to_string())
    }

    /// Texto esperado na saída de cada evidência, pelo subitem `evidence_expect_field`.
    fn evidence_expectations(&self, text: &str) -> HashMap<String, String> {
        let field = self
            .cfg("evidence_expect_field")
            .unwrap_or_else(|_| DEFAULT_EVIDENCE_EXPECT.to_string());
        evidence_subitem_values(text, &field, &self.evidence_from_verify())
            .into_iter()
            .map(|(name, value)| (name, value.trim_matches('`').to_string()))
            .filter(|(_, value)| !value.is_empty())
            .collect()
    }

    /// Tipos de evidência e o rótulo da "Verificação", com os valores padrão.
    fn evidence_kinds(&self) -> EvidenceKinds {
        EvidenceKinds::read(|key| self.cfg(key).ok())
    }

    /// Tipos de evidência com aval do usuário: `evidence_review_kinds`, em bloco ou em lista na
    /// linha; ausente vale todos os tipos e `[]` vale nenhum.
    fn review_kinds(&self) -> Vec<String> {
        if let Some(inline) = self
            .cfg("evidence_review_kinds")
            .ok()
            .filter(|value| value.starts_with('[') && value.ends_with(']'))
        {
            return inline_list(&inline);
        }
        self.cfg_list("evidence_review_kinds")
            .unwrap_or_else(|_| self.evidence_kinds().all())
    }

    /// Itens de `evidence_review_kinds` que não são um tipo de evidência conhecido.
    fn unknown_review_kinds(&self) -> Vec<String> {
        let known = self.evidence_kinds().all();
        self.review_kinds()
            .into_iter()
            .filter(|kind| !known.contains(kind))
            .collect()
    }

    /// `(nome, comando)` das evidências automáticas de "Evidências", pela leitura do `evidence`.
    fn automatic_evidences(&self, content: &str) -> Result<Vec<(String, String)>> {
        let automatic = self.cfg("evidence_automatic")?;
        let text = self.subsection(content, "evidence_subsection")?;
        Ok(text
            .lines()
            .filter(|line| has_text(line) && !is_indented(line))
            .filter_map(evidence_item)
            .filter(|(name, kind, _)| *kind == automatic && is_valid_evidence_name(name))
            .filter_map(|(name, _, rest)| {
                let command = rest.strip_prefix('`')?.strip_suffix('`')?;
                (!command.is_empty()).then(|| (name.to_string(), command.to_string()))
            })
            .collect())
    }

    /// Subitens das evidências; sem `evidence_fields` na configuração, só o formato é conferido.
    /// O subitem do texto esperado vale mesmo sem estar em `evidence_optional_fields`, que o
    /// `vostok update` não completa em projetos antigos.
    /// Campo e limite do título das evidências; sem as chaves (projeto antigo) não há limite.
    fn evidence_title(&self) -> Option<(String, usize)> {
        let field = self.cfg("evidence_title_field").ok()?;
        let max = self.cfg("evidence_title_max").ok()?.trim().parse().ok()?;
        Some((field, max))
    }

    fn evidence_fields(&self) -> Option<EvidenceFields> {
        let mut optional = self
            .cfg_list("evidence_optional_fields")
            .unwrap_or_default();
        let expect = self
            .cfg("evidence_expect_field")
            .unwrap_or_else(|_| DEFAULT_EVIDENCE_EXPECT.to_string());
        for field in [
            expect,
            EVIDENCE_REQUIRES.to_string(),
            EVIDENCE_TIMEOUT.to_string(),
        ] {
            if !optional.contains(&field) {
                optional.push(field);
            }
        }
        Some(EvidenceFields {
            required: self.cfg_list("evidence_fields").ok()?,
            optional,
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

    /// Caminho da tarefa: a pasta `TASK-NN/` com o estado, ou o arquivo único `TASK-NN.md`.
    fn task_file(&self, spec: &str, task: &str) -> PathBuf {
        let folder = self.tasks_root.join(spec).join(task);
        if is_task_folder(&folder) {
            folder
        } else {
            self.tasks_root.join(spec).join(format!("{task}.md"))
        }
    }

    /// Caminho da tarefa, sem validar os IDs (`load-task.sh` e `check-done.sh`).
    fn existing_task(&self, spec: &str, task: &str) -> Result<PathBuf> {
        let file = self.task_file(spec, task);
        if !file.exists() {
            bail!(
                "tarefa {task} da spec {spec} não encontrada: {}",
                file.display()
            );
        }
        task_entries(file.parent().unwrap_or(Path::new("")))?;
        TaskDoc::at(&file).check()?;
        Ok(file)
    }

    /// Caminho da tarefa. Falha se os IDs forem inválidos ou a tarefa não existir.
    fn find_task(&self, spec: &str, task: &str) -> Result<PathBuf> {
        if !is_spec_id(spec) {
            bail!("ID de spec inválido: '{spec}'");
        }
        self.check_task_id(task)?;
        let file = self.task_file(spec, task);
        if !file.exists() {
            bail!("tarefa {task} não encontrada em {spec}");
        }
        task_entries(file.parent().unwrap_or(Path::new("")))?;
        TaskDoc::at(&file).check()?;
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
        if is_task_folder(file) {
            return Ok(file.join(&layout().evidence_dir));
        }
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
        print!("{}", self.load_text(spec, task)?);
        Ok(())
    }

    /// Imprime um arquivo da tarefa; no arquivo único, a subseção equivalente.
    fn load_part(&self, spec: &str, task: &str, part: &str) -> Result<()> {
        let file = self.existing_task(spec, task)?;
        let doc = TaskDoc::at(&file);
        let name = doc
            .part_file(part)
            .ok_or_else(|| anyhow!("parte inválida: '{part}'; use {}", TASK_PARTS.join(", ")))?;
        let text = part_text(&doc, name)?.ok_or_else(|| {
            anyhow!(
                "a tarefa {task} não tem a parte '{part}': {}",
                file.display()
            )
        })?;
        println!("{}", text.trim_matches('\n'));
        Ok(())
    }

    fn load_text(&self, spec: &str, task: &str) -> Result<String> {
        let file = self.existing_task(spec, task)?;
        let section = self.cfg("execution_section")?;
        let content = TaskDoc::at(&file).text()?;
        let front_matter = frontmatter_lines(&content)
            .ok_or_else(|| anyhow!("a tarefa {task} não tem front-matter: {}", file.display()))?;
        let execution = markdown_section(&content, &section).ok_or_else(|| {
            anyhow!(
                "a tarefa {task} não tem a seção '## {section}': {}",
                file.display()
            )
        })?;
        let profile = models::task_profile(&self.harness, &file);
        let info = models::for_profile(&self.harness, &profile)?;
        if info.tool.is_none() {
            eprintln!("{}", models::tool_warning());
        }
        Ok(format!(
            "---\n{}\n---\n\n## Modelo\n\n- Perfil: {}\n- Descrição: {}\n- Ferramenta: {}\n- Apelido de reserva: {}\n\n## {section}\n{}\n",
            trim_newlines(&front_matter.join("\n")),
            info.profile,
            info.description,
            info.tool.as_deref().unwrap_or("não configurada"),
            info.model,
            trim_newlines(&execution)
        ))
    }

    /// Briefing do subagente: regras, persona, roteiro e tarefa com o registro de execução.
    fn brief(&self, spec: &str, task: &str) -> Result<()> {
        let file = self.find_task(spec, task)?;
        let status = status_of(&file);
        let has_baseline = self.evidence_dir(&file)?.join("baseline").is_dir();
        if status != self.status.in_progress || !has_baseline {
            let hint = if has_baseline {
                String::new()
            } else {
                format!("; execute: vostok task baseline {spec} {task}")
            };
            bail!("a tarefa {task} está com status '{status}'; o briefing só sai para tarefa '{}' com baseline coletado{hint}", self.status.in_progress);
        }
        let root = &self.harness.root;
        let map = fs::read_to_string(self.harness.config_file("context.yml"))?;
        let mut rules: Vec<String> = BRIEF_RULES.iter().map(|rule| rule.to_string()).collect();
        for entry in crate::commands::context::skill_entries(&map, "execute-task") {
            if entry.starts_with("rules/") && !rules.contains(&entry) {
                rules.push(entry);
            }
        }
        let mut text = String::from("# Regras\n");
        for rule in &rules {
            match fs::read_to_string(root.join(rule)) {
                Ok(content) => text.push_str(&format!("\n{}\n", content.trim_end())),
                Err(_) => eprintln!("Aviso: '{rule}' não encontrado em {}", root.display()),
            }
        }
        let persona = match TaskDoc::at(&file)
            .get("persona")
            .unwrap_or_default()
            .as_str()
        {
            "" | "-" | "null" => "desenvolvedor".to_string(),
            persona => persona.to_string(),
        };
        text.push_str("\n# Persona\n");
        let persona_file = format!("personas/{persona}.md");
        match fs::read_to_string(root.join(&persona_file)) {
            Ok(content) => text.push_str(&format!("\n{}\n", content.trim_end())),
            Err(_) => eprintln!(
                "Aviso: '{persona_file}' não encontrado em {}",
                root.display()
            ),
        }
        let route = BRIEF_ROUTE
            .replace("<SPEC_ID>", spec)
            .replace("<TASK_ID>", task);
        text.push_str(&format!("\n# Roteiro\n\n{route}\n# Tarefa\n\n"));
        text.push_str(&self.load_text(spec, task)?);
        let record = self.cfg("record_section")?;
        if let Some(section) = markdown_section(&TaskDoc::at(&file).text()?, &record) {
            text.push_str(&format!("\n## {record}\n{}\n", trim_newlines(&section)));
        }
        print!("{text}");
        Ok(())
    }

    /// Material do executivo e do técnico: a instrução, o que a tarefa pedia, o diff resumido dos
    /// caminhos de "Mudar", as evidências com o resultado e o registro.
    fn describe(&self, spec: &str, task: &str) -> Result<()> {
        let file = self.find_task(spec, task)?;
        let status = status_of(&file);
        let described = [
            &self.status.in_progress,
            &self.status.executed,
            &self.status.validating,
        ];
        if !described.contains(&&status) {
            bail!(
                "a tarefa {task} está com status '{status}'; a descrição só sai para tarefa '{}', '{}' ou '{}'",
                self.status.in_progress,
                self.status.executed,
                self.status.validating
            );
        }
        let doc = TaskDoc::at(&file);
        let layout = layout();
        let content = doc.text()?;
        let field = |key: &str| {
            let value = doc.get(key).unwrap_or_default();
            if matches!(value.as_str(), "" | "-" | "null") {
                "-".to_string()
            } else {
                value
            }
        };
        let (executive, technical) = if doc.folder() {
            (
                format!("`{}`", layout.executive),
                format!("`{}`", layout.technical),
            )
        } else {
            (
                format!("a subseção \"{}\"", layout.executive_heading),
                format!("a subseção \"{}\"", layout.technical_heading),
            )
        };
        let guide = DESCRIBE_GUIDE
            .replace("<EXECUTIVE>", &executive)
            .replace("<TECHNICAL>", &technical);
        let mut text = format!(
            "# Instrução\n\n{guide}\n\n# {task}: {}\n\n{}\n\n## {}\n\n{}\n",
            field("title"),
            field("description"),
            layout.execution,
            trim_newlines(&part_text(&doc, &layout.execution)?.unwrap_or_default())
        );
        let root = self.project_root()?;
        let diff_base = DiffBase::of(file_baseline_commit(&file), &root);
        text.push_str("\n## Arquivos de Mudar\n\n");
        for (action, path) in change_items(&self.subsection(&content, "change_subsection")?) {
            text.push_str(&format!(
                "- `{path}` ({action}): {}\n",
                path_stat(&path, &root, &diff_base.rev)
            ));
        }
        text.push_str(&format!(
            "\n## Evidências\n\n- Testes: {}\n- Regressão: {}\n- Evidências: {}\n",
            field("tests_result"),
            field("regression_result"),
            field("evidence_result")
        ));
        let dir = self.evidence_dir(&file)?;
        let evidences = self.subsection(&content, "evidence_subsection")?;
        for (name, kind, _) in evidences
            .lines()
            .filter(|line| has_text(line) && !is_indented(line))
            .filter_map(evidence_item)
        {
            let state = if dir.join(name).exists() {
                "gravada"
            } else {
                "ausente"
            };
            text.push_str(&format!("- `{name}` ({kind}): {state}\n"));
        }
        let record = part_text(&doc, &layout.record)?.unwrap_or_default();
        text.push_str(&format!("\n{}\n", trim_newlines(&record)));
        print!("{text}");
        Ok(())
    }

    fn status_report(&self, spec: &str) -> Result<String> {
        if !is_spec_id(spec) {
            bail!("ID de spec inválido: '{spec}'");
        }
        let dir = self.tasks_root.join(spec);
        let prefix = format!("{}-", self.cfg("id.prefix")?);
        let files: Vec<PathBuf> = if dir.is_dir() {
            task_entries(&dir)?
                .into_iter()
                .filter(|file| task_id_of(file).starts_with(&prefix))
                .collect()
        } else {
            Vec::new()
        };
        if files.is_empty() {
            bail!("a spec {spec} não tem tarefas em {}", dir.display());
        }
        let field = |file: &Path, key: &str| {
            let value = TaskDoc::at(file).get(key).unwrap_or_default();
            if matches!(value.as_str(), "" | "-" | "null") {
                "-".to_string()
            } else {
                value
            }
        };
        let graph: Vec<(String, Vec<String>)> = files
            .iter()
            .map(|file| (task_id_of(file), dependencies_of(file)))
            .collect();
        let computed = crate::commands::plan::waves(&graph);
        let mut text = format!(
            "### Tarefas da {spec}\n\n| ID | Título | Persona | Perfil | Modelo | Onda | Status |\n|---|---|---|---|---|---|---|\n"
        );
        let mut waves: Vec<(String, String, String)> = Vec::new();
        for file in &files {
            let id = task_id_of(file);
            let profile = models::task_profile(&self.harness, file);
            let status = status_of(file);
            let model = match field(file, "model").as_str() {
                "-" if status == self.status.pending || status == self.status.approved => {
                    models::for_profile(&self.harness, &profile)?.model
                }
                "-" => "-".to_string(),
                model => model.to_string(),
            };
            let wave = match field(file, "wave").as_str() {
                "-" => computed
                    .get(&id)
                    .map_or("-".to_string(), |wave| wave.to_string()),
                wave => wave.to_string(),
            };
            text.push_str(&format!(
                "| {id} | {} | {} | {profile} | {model} | {wave} | {status} |\n",
                escape_pipes(&TaskDoc::at(file).get("title").unwrap_or_default()),
                escape_pipes(&field(file, "persona")),
            ));
            waves.push((wave, id, status));
        }
        if models::harness_tool(&self.harness).is_none() {
            eprintln!("{}", models::tool_warning());
        }
        let mut keys: Vec<String> = waves.iter().map(|(wave, _, _)| wave.clone()).collect();
        keys.sort_by_key(|wave| wave.parse::<u64>().map_or((1, 0), |n| (0, n)));
        keys.dedup();
        text.push_str(&format!(
            "\n### Ondas da {spec}\n\n| Onda | Tarefas | Status |\n|---|---|---|\n"
        ));
        for key in keys {
            let members: Vec<&(String, String, String)> =
                waves.iter().filter(|(wave, _, _)| *wave == key).collect();
            let ids: Vec<&str> = members.iter().map(|(_, id, _)| id.as_str()).collect();
            let statuses: Vec<String> = members.iter().map(|(_, _, s)| s.clone()).collect();
            text.push_str(&format!(
                "| {key} | {} | {} |\n",
                ids.join(", "),
                wave_status(&statuses, &self.status)
            ));
        }
        text.push_str(&self.waits_table(spec, &files));
        text.push_str(&format!(
            "\nSubagentes ao mesmo tempo: {}\n",
            self.max_subagents()
        ));
        Ok(text)
    }

    fn waits_table(&self, spec: &str, files: &[PathBuf]) -> String {
        let mut rows = String::new();
        for file in files {
            let id = task_id_of(file);
            if status_of(file) != self.status.done {
                let pending: Vec<String> = closes_after_of(file)
                    .into_iter()
                    .filter_map(|closing| {
                        let closing_file = self.task_file(spec, &closing);
                        if !closing_file.exists() {
                            return Some(format!("{closing} (não existe)"));
                        }
                        let status = status_of(&closing_file);
                        (status != self.status.done).then(|| format!("{closing} ({status})"))
                    })
                    .collect();
                if !pending.is_empty() {
                    rows.push_str(&format!(
                        "| {id} | fechamento | {} | - |\n",
                        pending.join(", ")
                    ));
                }
            }
            let waiting = waiting_of(file);
            if !waiting.is_empty() {
                let reason = TaskDoc::at(file)
                    .get(WAITING_REASON_FIELD)
                    .unwrap_or_default();
                let reason = match reason.as_str() {
                    "" | "-" | "null" => "-".to_string(),
                    reason => escape_pipes(reason),
                };
                rows.push_str(&format!(
                    "| {id} | dependência | {} | {reason} |\n",
                    waiting.join(", ")
                ));
            }
        }
        if rows.is_empty() {
            return rows;
        }
        format!(
            "\n### Esperas da {spec}\n\n| Tarefa | Espera | Tarefas | Motivo |\n|---|---|---|---|\n{rows}"
        )
    }

    fn check_done(&self, spec: &str, task: &str) -> Result<String> {
        let file = self.existing_task(spec, task)?;
        let sub = self.cfg("done_subsection")?;
        let content = TaskDoc::at(&file).text()?;
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
        for file in &files {
            self.print_dashdash_warnings(file)?;
        }
        self.print_timeout_exceptions(&files)?;
        Ok(())
    }

    /// Evidências com `Tempo-limite:` acima do padrão (ou 0, sem limite): a exceção que o usuário
    /// aprova com a tarefa.
    fn print_timeout_exceptions(&self, files: &[PathBuf]) -> Result<()> {
        let default = self.evidence_baseline_timeout();
        let mut lines = Vec::new();
        for file in files {
            let text = self.subsection(&TaskDoc::at(file).text()?, "evidence_subsection")?;
            let mut timeouts: Vec<(String, u64)> =
                evidence_timeouts(&text, &self.evidence_from_verify())
                    .into_iter()
                    .filter_map(|(name, seconds)| Some((name, seconds?)))
                    .filter(|(_, seconds)| *seconds == 0 || *seconds > default)
                    .collect();
            timeouts.sort();
            for (name, seconds) in timeouts {
                let limit = if seconds == 0 {
                    "sem tempo-limite".to_string()
                } else {
                    format!("com tempo-limite de {seconds} s")
                };
                lines.push(format!(
                    "- {}: {name} {limit} (padrão {default} s)",
                    task_id_of(file)
                ));
            }
        }
        if !lines.is_empty() {
            println!(
                "Evidências com tempo-limite acima do padrão no baseline, aprovadas com a tarefa:"
            );
            println!("{}", lines.join("\n"));
        }
        Ok(())
    }

    /// Avisos dos comandos da "Verificação" com argumentos depois de `--` (RF-02); não recusam.
    fn print_dashdash_warnings(&self, file: &Path) -> Result<()> {
        let block =
            verify_block(&self.subsection(&TaskDoc::at(file).text()?, "verify_subsection")?);
        for command in dashdash_commands(&block) {
            println!("{}", dashdash_warning(&task_id_of(file), &command));
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
            let pending: Vec<PathBuf> = task_entries(&dir)?
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
            if !file.exists() {
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
            problems.extend(self.errors_of(spec, file, EvidenceNames::Strict)?);
        }
        Ok(problems)
    }

    fn approve(&self, files: &[PathBuf]) -> Result<()> {
        let now = utc_now();
        let hash_field = self.spec_cfg("content_hash_field")?;
        let approved_at = self.spec_cfg("approved_at_field")?;
        let approved_hash = self.spec_cfg("approved_hash_field")?;
        for file in files {
            TaskDoc::at(file).set("status", &self.status.approved)?;
            TaskDoc::at(file).put(&approved_at, &now)?;
            TaskDoc::at(file).put(&approved_hash, &TaskDoc::at(file).get(&hash_field)?)?;
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
            &self.status.executed,
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
        let errors = self.errors_of(spec, &file, EvidenceNames::Normalizable)?;
        if !errors.is_empty() {
            println!("Problemas de validação:\n{}", errors.join("\n"));
        }
        Ok(())
    }

    fn validate(&self, spec: &str, task: &str) -> Result<()> {
        let file = self.adjustable_task(spec, task)?;
        let errors = self.errors_of(spec, &file, EvidenceNames::Normalizable)?;
        if !errors.is_empty() {
            bail!("a tarefa {task} tem problemas:\n{}", errors.join("\n"));
        }
        print_renames(
            "Nomes de evidência com caminho, trocados pelo nome do arquivo no vostok task adjust record (nada foi alterado):",
            &self.evidence_renames(&file)?.1,
        );
        println!("Tarefa {task} válida: {}", file.display());
        self.print_dashdash_warnings(&file)
    }

    fn record(&self, spec: &str, task: &str, summary: &str) -> Result<()> {
        let summary = clean_summary(summary)?;
        let file = self.adjustable_task(spec, task)?;
        let errors = self.errors_of(spec, &file, EvidenceNames::Normalizable)?;
        if !errors.is_empty() {
            bail!(
                "a tarefa {task} tem problemas; nenhum ajuste registrado:\n{}",
                errors.join("\n")
            );
        }
        // O nome com caminho vira o nome do arquivo antes do registro, que grava o hash.
        let (content, renames) = self.evidence_renames(&file)?;
        if !renames.is_empty() {
            TaskDoc::at(&file).write_text(&content)?;
        }
        print_renames(
            "Nomes de evidência com caminho trocados pelo nome do arquivo:",
            &renames,
        );
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
        if self.has_baseline(file)? {
            println!(
                "Ajuste em tarefa já iniciada: {task} continua '{}', com a aprovação mantida.",
                self.status.approved
            );
            return Ok(());
        }
        TaskDoc::at(file).set("status", &self.status.pending)?;
        TaskDoc::at(file).put(&self.spec_cfg("approved_at_field")?, "null")?;
        TaskDoc::at(file).put(&self.spec_cfg("approved_hash_field")?, "null")?;
        println!(
            "Aprovação de {task} derrubada: o status voltou para '{}'; aprove-a de novo antes de executar.",
            self.status.pending
        );
        Ok(())
    }

    /// Devolve uma tarefa em execução para in_progress, com tentativas e resultados zerados, mantendo o
    /// baseline e a aprovação. Não faz nada nas demais.
    fn restart_execution(&self, file: &Path, task: &str, summary: &str) -> Result<()> {
        if !self.is_running(&status_of(file)) {
            return Ok(());
        }
        let in_progress = &self.status.in_progress;
        TaskDoc::at(file).set("status", in_progress)?;
        reset_results(file)?;
        self.record_step(file, "ajuste", in_progress, summary)?;
        println!(
            "Ajuste em execução: {task} voltou para '{in_progress}', com tentativas e resultados zerados e o baseline mantido."
        );
        Ok(())
    }

    /// Status de execução: in_progress, executed, validating e blocked.
    fn is_running(&self, status: &str) -> bool {
        [
            &self.status.in_progress,
            &self.status.executed,
            &self.status.validating,
            &self.status.blocked,
        ]
        .contains(&&status.to_string())
    }

    // --- start e baseline ---

    fn has_baseline(&self, file: &Path) -> Result<bool> {
        Ok(self.evidence_dir(file)?.join("baseline").is_dir())
    }

    fn start(&self, spec: &str, task: &str, model: Option<&str>) -> Result<()> {
        let model = match model.map(str::trim) {
            Some(value) if value.is_empty() || value.contains(['\n', '\r']) => {
                bail!("informe o modelo em --model")
            }
            other => other,
        };
        let file = self.find_task(spec, task)?;
        // Tarefa aprovada que já tem baseline saiu de uma espera: recomeça de onde parou.
        let restart = status_of(&file) == self.status.approved && self.has_baseline(&file)?;
        let mut blockers = self.start_blockers(spec, task, &file, restart)?;
        if blockers.is_empty() && !restart {
            blockers = self.created_conflicts(&file)?;
        }
        if !blockers.is_empty() {
            bail!(
                "a tarefa {task} não pode ser iniciada; nada foi alterado:\n{}",
                blockers.join("\n")
            );
        }
        if restart {
            return self.restart_task(&file, task, model);
        }
        // A tarefa aparece em execução antes da coleta, que pode demorar; se a coleta falhar, ela
        // continua em execução sem baseline, e o `vostok task baseline` coleta de novo.
        let in_progress = &self.status.in_progress;
        TaskDoc::at(&file).set("status", in_progress)?;
        let approved_at = TaskDoc::at(&file)
            .get("approved_at")
            .unwrap_or_else(|_| "?".into());
        let mut detail = format!("aprovada em {approved_at}");
        if let Some(model) = model {
            TaskDoc::at(&file).put("model", model)?;
            detail.push_str(&format!("; modelo: {model}"));
        }
        self.record_step(&file, "início", in_progress, &detail)?;
        TaskDoc::at(&file).store_document_hashes()?;
        println!("Tarefa {task} iniciada: status '{in_progress}'. Coletando o baseline:");
        let (dir, baseline_detail) = self.collect_baseline(&file, task)?;
        self.record_step(&file, "baseline", "coletado", &baseline_detail)?;
        println!(
            "Baseline de {task} coletado em {}: {baseline_detail}",
            dir.display()
        );
        Ok(())
    }

    /// Recomeça a tarefa desbloqueada: mantém o baseline, os hashes dos documentos e a aprovação.
    fn restart_task(&self, file: &Path, task: &str, model: Option<&str>) -> Result<()> {
        let in_progress = &self.status.in_progress;
        TaskDoc::at(file).set("status", in_progress)?;
        let mut detail = String::from("baseline mantido");
        if let Some(model) = model {
            TaskDoc::at(file).put("model", model)?;
            detail.push_str(&format!("; modelo: {model}"));
        }
        self.record_step(file, "recomeço", in_progress, &detail)?;
        println!("Tarefa {task} recomeçada: status '{in_progress}', com o baseline mantido.");
        Ok(())
    }

    fn start_blockers(
        &self,
        spec: &str,
        task: &str,
        file: &Path,
        restart: bool,
    ) -> Result<Vec<String>> {
        let status = status_of(file);
        let approved = &self.status.approved;
        if status != *approved {
            return Ok(vec![format!(
                "a tarefa {task} está com status '{status}'; só tarefas com status '{approved}' são executadas. Peça a aprovação ao usuário (skill approve-task)"
            )]);
        }
        let mut blockers = Vec::new();
        let approved_hash = TaskDoc::at(file)
            .get(&self.spec_cfg("approved_hash_field")?)
            .unwrap_or_default();
        if restart {
            if !self.hash_is_current(file)? {
                blockers.push(format!(
                    "a tarefa {task} mudou depois do último ajuste registrado; registre o ajuste (vostok task adjust record-manual) antes de recomeçar"
                ));
            }
        } else if approved_hash != self.task_hash(file)? {
            blockers.push(format!(
                "a tarefa {task} mudou depois da aprovação; registre o ajuste (skill adjust-task) e peça a aprovação de novo"
            ));
        }
        let evidence = self.subsection(&TaskDoc::at(file).text()?, "evidence_subsection")?;
        for problem in evidence
            .lines()
            .filter(|line| !is_indented(line))
            .filter_map(evidence_name_problem)
        {
            blockers.push(format!(
                "{problem}; ajuste a tarefa (skill adjust-task) e peça a aprovação de novo"
            ));
        }
        for dependency in dependencies_of(file) {
            let dependency_file = self.task_file(spec, &dependency);
            if !dependency_file.exists() {
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
        let conflicts = self.created_conflicts(&file)?;
        if !conflicts.is_empty() {
            bail!("{}", conflicts.join("\n"));
        }
        let (dir, detail) = self.collect_baseline(&file, task)?;
        self.record_step(&file, "baseline", "coletado", &detail)?;
        println!("Baseline de {task} coletado em {}: {detail}", dir.display());
        Ok(())
    }

    /// Recoleta o baseline em execução; o anterior fica em `baseline-anterior-N`.
    fn recollect_baseline(&self, spec: &str, task: &str, reason: Option<&str>) -> Result<()> {
        let reason = normalized_reason(reason.unwrap_or_default());
        if reason.is_empty() {
            bail!("informe o motivo da recoleta em --motivo, com a decisão do usuário; nada foi alterado");
        }
        let file = self.find_task(spec, task)?;
        let status = status_of(&file);
        if !self.is_running(&status) {
            bail!(
                "a tarefa {task} está com status '{status}'; só tarefas com status '{}', '{}', '{}' ou '{}' têm o baseline recoletado; nada foi alterado",
                self.status.in_progress,
                self.status.executed,
                self.status.validating,
                self.status.blocked
            );
        }
        let waiting = waiting_of(&file);
        if !waiting.is_empty() {
            bail!(
                "a tarefa {task} espera {}; o baseline é recoletado depois do recomeço (vostok task start); nada foi alterado",
                waiting.join(", ")
            );
        }
        let evidence = self.evidence_dir(&file)?;
        let current = evidence.join("baseline");
        if !current.is_dir() {
            bail!("o baseline de {task} não foi coletado; execute: vostok task baseline {spec} {task}");
        }
        let number = (1..)
            .find(|n| !evidence.join(format!("baseline-anterior-{n}")).exists())
            .unwrap_or(1);
        let previous = evidence.join(format!("baseline-anterior-{number}"));
        fs::rename(&current, &previous)?;
        let (dir, detail) = match self.collect_baseline(&file, task) {
            Ok(collected) => collected,
            Err(error) => {
                fs::rename(&previous, &current)?;
                return Err(error);
            }
        };
        let new_status = if status == self.status.blocked {
            self.status.validating.clone()
        } else {
            status
        };
        let doc = TaskDoc::at(&file);
        doc.set("status", &new_status)?;
        doc.set("regression_result", "null")?;
        doc.set("attempts", "0")?;
        clear_regression_outputs(&evidence.join("validation/regression"))?;
        self.record_step(
            &file,
            "recoleta",
            &new_status,
            &format!("baseline anterior em baseline-anterior-{number}: {reason}; {detail}"),
        )?;
        println!(
            "Baseline de {task} recoletado em {}: {detail}. Anterior guardado em {}. Status: '{new_status}'.",
            dir.display(),
            previous.display()
        );
        Ok(())
    }

    /// Mensagens dos caminhos que a tarefa cria e que já existem no projeto.
    fn created_conflicts(&self, file: &Path) -> Result<Vec<String>> {
        let root = self.project_root()?;
        Ok(self
            .created_paths(file)?
            .into_iter()
            .filter(|path| root.join(path).exists())
            .map(|path| {
                format!(
                    "'{path}' já existe, mas a tarefa o cria: o estado anterior já foi alterado; restaure o estado anterior antes de coletar o baseline"
                )
            })
            .collect())
    }

    /// Executa e grava o baseline; não grava registro nem status.
    fn collect_baseline(&self, file: &Path, task: &str) -> Result<(PathBuf, String)> {
        let dir = self.evidence_dir(file)?.join("baseline");
        if dir.exists() {
            bail!(
                "o baseline de {task} já foi coletado em {}; ele não é sobrescrito",
                dir.display()
            );
        }
        let content = TaskDoc::at(file).text()?;
        let commands = self.baseline_commands(&content)?;
        if commands.is_empty() {
            bail!("a tarefa {task} não tem comandos em 'Baseline'");
        }
        let root = self.project_root()?;
        // Os resultados são gravados juntos no fim: ou o baseline existe inteiro, ou não existe.
        let mut results = Vec::new();
        for (index, command) in commands.iter().enumerate() {
            println!("- [{}] {command}", index + 1);
            results.push(recorded_run(command, &root)?);
        }
        // Evidências automáticas: a saída de antes da mudança; o código de saída não impede a coleta.
        let mut evidences = Vec::new();
        if let Some(evidence_dir) = self.evidence_baseline_dir() {
            let text = self.subsection(&content, "evidence_subsection")?;
            let requirements = evidence_requirements(&text, &self.evidence_from_verify());
            let timeouts = evidence_timeouts(&text, &self.evidence_from_verify());
            let default_timeout = self.evidence_baseline_timeout();
            for (name, command) in self.automatic_evidences(&content)? {
                let missing = requirements
                    .get(&name)
                    .and_then(|paths| paths.iter().find(|path| !root.join(path).exists()));
                if let Some(path) = missing {
                    let reason = format!("{NOT_EXECUTED} {path} ainda não existe");
                    println!("- [evidência {name}] {reason}");
                    let record = format!("$ {command}\nexit: 1\n{reason}\n").into_bytes();
                    evidences.push((evidence_dir.clone(), name, record));
                    continue;
                }
                let seconds = timeouts
                    .get(&name)
                    .copied()
                    .flatten()
                    .unwrap_or(default_timeout);
                if seconds == default_timeout {
                    println!("- [evidência {name}] {command}");
                } else {
                    println!("- [evidência {name}] {command} (tempo-limite {seconds} s)");
                }
                let record = recorded_run_with_timeout(&command, &root, seconds)?;
                evidences.push((evidence_dir.clone(), name, record));
            }
        }
        fs::create_dir_all(&dir)?;
        for (index, (_, output)) in results.iter().enumerate() {
            fs::write(dir.join(output_name(index + 1)), output)?;
        }
        for (evidence_dir, name, output) in &evidences {
            let target = dir.join(evidence_dir);
            fs::create_dir_all(&target)?;
            fs::write(target.join(name), output)?;
        }
        let codes = results
            .iter()
            .map(|(code, _)| code.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        if let Some(sha) = git_text(&root, &["rev-parse", "--verify", "HEAD"]) {
            TaskDoc::at(file).put(BASELINE_COMMIT_KEY, &sha)?;
        }
        let mut detail = format!("{} comando(s); códigos de saída: {codes}", commands.len());
        if !evidences.is_empty() {
            detail.push_str(&format!(
                "; {} evidência(s) automática(s) gravada(s) para comparação",
                evidences.len()
            ));
        }
        Ok((dir, detail))
    }

    /// Motivo de cada comando de "Baseline" com mudança esperada, na ordem dos comandos.
    fn baseline_reasons(&self, content: &str) -> Result<Vec<Option<String>>> {
        Ok(self
            .subsection(content, "baseline_subsection")?
            .lines()
            .filter(|line| baseline_command(line).is_some())
            .map(|line| expected_change(line).map(|(_, reason)| reason.to_string()))
            .collect())
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

    /// Recusa tarefa fora de in_progress, executed e validating; a bloqueada só volta pelo resume.
    fn require_running(&self, file: &Path, task: &str) -> Result<()> {
        let status = status_of(file);
        if status == self.status.blocked {
            bail!("a tarefa {task} está bloqueada; mostre as falhas ao usuário e, com a decisão dele, execute: vostok task resume");
        }
        let validated = [
            &self.status.in_progress,
            &self.status.executed,
            &self.status.validating,
        ];
        if !validated.contains(&&status) {
            bail!(
                "a tarefa {task} está com status '{status}'; só tarefas com status '{}', '{}' ou '{}' são validadas",
                self.status.in_progress,
                self.status.executed,
                self.status.validating
            );
        }
        Ok(())
    }

    /// Saída gravada de cada comando rotulado, pelo texto do comando, só com os testes aprovados.
    fn labeled_outputs(&self, file: &Path, content: &str) -> Result<HashMap<String, Vec<u8>>> {
        let mut outputs = HashMap::new();
        if TaskDoc::at(file).get("tests_result").unwrap_or_default() != "passed" {
            return Ok(outputs);
        }
        let block = verify_block(&self.subsection(content, "verify_subsection")?);
        let dir = self.verify_dir(file)?;
        for (name, command) in labeled_commands(&block, &self.verify_label()) {
            if let Ok(output) = fs::read(dir.join(&name)) {
                outputs.insert(command, output);
            }
        }
        Ok(outputs)
    }

    /// Pasta com a saída de cada comando rotulado da "Verificação".
    fn verify_dir(&self, file: &Path) -> Result<PathBuf> {
        Ok(self.evidence_dir(file)?.join("validation/verify"))
    }

    /// Apaga as saídas de rótulos anteriores e devolve o bloco que grava a saída de cada comando
    /// rotulado em `validation/verify/`; a pasta só é criada quando há rótulo.
    fn block_recording_labels(&self, file: &Path, block: &str) -> Result<String> {
        let verify_dir = self.verify_dir(file)?;
        if verify_dir.exists() {
            fs::remove_dir_all(&verify_dir)?;
        }
        let labels = verify_labels(block, &self.verify_label());
        if labels.is_empty() {
            return Ok(block.to_string());
        }
        fs::create_dir_all(&verify_dir)?;
        Ok(labeled_block(
            block,
            &labels,
            &fs::canonicalize(&verify_dir)?,
        ))
    }

    /// Executa o bloco da "Verificação" e grava a saída em `validation/<name>`.
    fn run_verification(&self, file: &Path, block: &str, name: &str) -> Result<(i32, PathBuf)> {
        let dir = self.evidence_dir(file)?.join("validation");
        fs::create_dir_all(&dir)?;
        let output_path = dir.join(name);
        let output = File::create(&output_path)?;
        let status = Command::new("bash")
            .arg("-c")
            .arg(format!("set -eo pipefail; {VERIFY_TRAP}\n{block}"))
            .current_dir(self.project_root()?)
            .stdin(Stdio::null())
            .stdout(output.try_clone()?)
            .stderr(output)
            .status()
            .context("não foi possível executar o bash")?;
        Ok((exit_code(status), output_path))
    }

    /// Executa a verificação sem mudar o front-matter nem o registro.
    fn check(&self, spec: &str, task: &str) -> Result<()> {
        let file = self.find_task(spec, task)?;
        self.require_running(&file, task)?;
        let block =
            verify_block(&self.subsection(&TaskDoc::at(&file).text()?, "verify_subsection")?);
        if !has_text(&block) {
            bail!(
                "a tarefa {task} não tem um bloco `bash` em '{}'",
                self.cfg("verify_subsection")?
            );
        }
        let (code, path) = self.run_verification(&file, &block, "check.out")?;
        if code == 0 {
            println!("Verificação de {task} passou (saída em {})", path.display());
            return Ok(());
        }
        println!(
            "Verificação de {task} falhou (código {code}); últimas linhas ({}):",
            path.display()
        );
        print!("{}", indented_tail(&fs::read(&path)?, 30));
        bail!("a verificação de {task} falhou; corrija e rode de novo")
    }

    fn tests(&self, spec: &str, task: &str) -> Result<()> {
        let file = self.find_task(spec, task)?;
        self.require_running(&file, task)?;
        if !self.evidence_dir(&file)?.join("baseline").is_dir() {
            bail!("o baseline de {task} não foi coletado; execute: vostok task baseline {spec} {task}");
        }
        let block =
            verify_block(&self.subsection(&TaskDoc::at(&file).text()?, "verify_subsection")?);
        if !has_text(&block) {
            bail!(
                "a tarefa {task} não tem um bloco `bash` em '{}'",
                self.cfg("verify_subsection")?
            );
        }
        let block = self.block_recording_labels(&file, &block)?;
        let (code, output_path) = self.run_verification(&file, &block, "tests.out")?;
        let validating = &self.status.validating;

        if code == 0 {
            TaskDoc::at(&file).set("status", validating)?;
            TaskDoc::at(&file).set("tests_result", "passed")?;
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

        let attempts = number_or_zero(&TaskDoc::at(&file).get("attempts").unwrap_or_default()) + 1;
        let max_attempts = number_or_zero(&self.cfg("max_attempts")?);
        TaskDoc::at(&file).set("tests_result", "failed")?;
        TaskDoc::at(&file).set("regression_result", "null")?;
        TaskDoc::at(&file).set("evidence_result", "null")?;
        TaskDoc::at(&file).set("attempts", &attempts.to_string())?;
        println!(
            "Testes de {task} falharam (código de saída {code}), tentativa {attempts} de {max_attempts}. Últimas linhas da saída ({}):",
            output_path.display()
        );
        print!("{}", indented_tail(&fs::read(&output_path)?, 30));
        if attempts >= max_attempts {
            TaskDoc::at(&file).set("status", &self.status.blocked)?;
            let detail = format!("tentativa {attempts} de {max_attempts}; bloqueada");
            self.record_step(&file, "testes", "failed", &detail)?;
            bail!("a tarefa {task} ficou bloqueada depois de {attempts} tentativas. PARE: responda com o template `blocker`, mostrando as falhas, e aguarde a decisão do usuário");
        }
        TaskDoc::at(&file).set("status", validating)?;
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
        let content = TaskDoc::at(&file).text()?;
        let commands = self.baseline_commands(&content)?;
        let reasons = self.baseline_reasons(&content)?;
        let reused = self.labeled_outputs(&file, &content)?;
        let dir = evidence.join("validation/regression");
        clear_regression_outputs(&dir)?;
        let accepted = latest_acceptances(&dir);
        let mut ignore = self.regression_ignore();
        let created = self.created_paths(&file)?;
        ignore.extend(created.clone());

        let root = self.project_root()?;
        let mut delivery = created;
        delivery.extend(untracked_folders(&root, &delivery.clone()));
        let mut ignored = String::new();
        let mut ignored_count = 0;
        let mut changed = String::new();
        let mut equal = 0;
        let mut accepted_count = 0;
        let alerts = self.regression_alert();
        let mut grown = String::new();
        let mut grown_count = 0;
        let mut expected_changes = String::new();
        let mut expected_count = 0;
        for (index, command) in commands.iter().enumerate() {
            let number = index + 1;
            let name = output_name(number);
            // Comando idêntico a um rotulado da Verificação aprovada: usa a saída já gravada.
            let (code, output) = match reused.get(command.trim()) {
                Some(saved) => {
                    let mut record = format!("$ {command}\nexit: 0\n").into_bytes();
                    record.extend_from_slice(saved);
                    (0, record)
                }
                None => recorded_run(command, &root)?,
            };
            let current = dir.join(&name);
            fs::write(&current, &output)?;
            if let Some(path) = delivery.iter().find(|path| cites_path(command, path)) {
                ignored_count += 1;
                ignored.push_str(&format!(
                    "- [{number}] {command}: ignorado, mede a entrega da tarefa ('{path}')\n"
                ));
                continue;
            }
            let previous = base.join(&name);
            match fs::read(&previous) {
                Err(_) => changed.push_str(&format!("- {command}: sem resultado no baseline\n")),
                Ok(expected)
                    if without_ignored(&expected, &ignore) == without_ignored(&output, &ignore) =>
                {
                    equal += 1
                }
                Ok(_)
                    if accepted
                        .get(&number)
                        .is_some_and(|item| item.covers(code, &output, &ignore)) =>
                {
                    accepted_count += 1
                }
                Ok(_) if reasons.get(index).is_some_and(Option::is_some) => {
                    fs::write(
                        dir.join(diff_name(number)),
                        diff_output(&previous, &current),
                    )?;
                    expected_count += 1;
                    let reason = reasons[index].as_deref().unwrap_or_default();
                    expected_changes
                        .push_str(&format!("- [{number}] {command}: muda ({reason})\n"));
                }
                Ok(expected) => match added_lines(
                    &without_ignored(&expected, &ignore),
                    &without_ignored(&output, &ignore),
                ) {
                    Some(added) => match added.iter().find(|line| has_alert(line, &alerts)) {
                        None => {
                            grown_count += 1;
                            grown.push_str(&format!(
                                "- [{number}] {command}: acréscimo de {} linha(s)\n",
                                added.len()
                            ));
                        }
                        Some(line) => {
                            fs::write(
                                dir.join(diff_name(number)),
                                diff_output(&previous, &current),
                            )?;
                            changed.push_str(&format!(
                                "- [{number}] {command}\nlinha nova com alerta: {line}\n"
                            ));
                        }
                    },
                    None => {
                        fs::write(
                            dir.join(diff_name(number)),
                            diff_output(&previous, &current),
                        )?;
                        changed.push_str(&format!(
                            "- [{number}] {command}\n{}\n",
                            diff_excerpt(&previous, &current)
                        ));
                    }
                },
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
            TaskDoc::at(&file).set("status", &self.status.validating)?;
            TaskDoc::at(&file).set("regression_result", "passed")?;
            let mut detail = if accepted_count == 0 {
                format!("{equal} comando(s) iguais ao baseline")
            } else {
                format!("{equal} igual(is) ao baseline, {accepted_count} aceito(s) pelo usuário")
            };
            if grown_count > 0 {
                detail.push_str(&format!("; {grown_count} com acréscimo"));
            }
            if expected_count > 0 {
                detail.push_str(&format!("; {expected_count} mudança(s) esperada(s)"));
            }
            if ignored_count > 0 {
                detail.push_str(&format!(
                    "; {ignored_count} ignorado(s) por medir a entrega"
                ));
            }
            self.record_step(&file, "regressão", "passed", &detail)?;
            println!("Sem regressão em {task}: {detail}.");
            print!("{grown}{expected_changes}{ignored}");
            return Ok(());
        }
        TaskDoc::at(&file).set("regression_result", "failed")?;
        TaskDoc::at(&file).set("status", &self.status.blocked)?;
        self.record_step(
            &file,
            "regressão",
            "failed",
            "comandos diferentes do baseline; bloqueada",
        )?;
        print!(
            "{ignored}Comandos com resultado diferente do baseline (saídas em {}):\n{changed}Para aceitar uma diferença como esperada, com a decisão do usuário: vostok task resume {spec} {task} --accept-regression <N> \"decisão\"\n",
            dir.display()
        );
        bail!("a tarefa {task} ficou bloqueada por regressão. PARE: responda com o template `blocker`, mostrando os comandos, e aguarde a decisão do usuário")
    }

    fn evidence(
        &self,
        spec: &str,
        task: &str,
        reviewed: &[String],
        rejected: Option<&str>,
        motivo: Option<&str>,
    ) -> Result<()> {
        let file = self.find_task(spec, task)?;
        self.require_running(&file, task)?;
        let known = self.evidence_kinds().all();
        for kind in self.unknown_review_kinds() {
            println!("{}", review_kind_warning(&kind, &known));
        }
        let dir = self.evidence_dir(&file)?;
        let content = TaskDoc::at(&file).text()?;
        if let Some(name) = rejected {
            if !reviewed.is_empty() {
                bail!("use --reviewed e --rejected em chamadas separadas; nada foi registrado");
            }
            let reason = normalized_reason(motivo.unwrap_or_default());
            if reason.is_empty() {
                bail!("informe o motivo da rejeição em --motivo; nada foi registrado");
            }
            let (id, name) = self.resolve_evidence(&content, task, name)?;
            return self.reject(&file, spec, task, (&id, &name), &reason);
        }
        let mut approvals: Vec<(String, String, Vec<u8>)> = Vec::new();
        for given in reviewed {
            let (id, name) = self.resolve_evidence(&content, task, given)?;
            let bytes = self.reviewable(&content, &dir, &name)?;
            approvals.push((id, name, bytes));
        }
        if !approvals.is_empty() {
            for (id, name, bytes) in &approvals {
                self.record_step(
                    &file,
                    "conferência",
                    "usuário",
                    &evidence_review(id, name, &self.review_hashes(bytes)),
                )?;
            }
            let rejected = rejected_of(&file);
            let pending: Vec<String> = rejected
                .iter()
                .filter(|rejected| !approvals.iter().any(|(_, name, _)| name == *rejected))
                .cloned()
                .collect();
            if pending.len() != rejected.len() {
                write_rejected(&file, &pending)?;
            }
            return self.review_outcome(&file, spec, task, &dir);
        }
        let text = self.subsection(&content, "evidence_subsection")?;
        let items: Vec<&str> = text
            .lines()
            .filter(|line| has_text(line) && !is_indented(line))
            .collect();
        let none = self.cfg("evidence_none")?;

        let trimmed: Vec<&str> = items.iter().map(|line| line.trim()).collect();
        if trimmed.join("\n") == none {
            TaskDoc::at(&file).set("evidence_result", "passed")?;
            self.record_step(&file, "evidências", "passed", &none)?;
            println!("Evidências de {task}: {none} Resultado: passed.");
            return Ok(());
        }

        let Generated {
            same,
            missing,
            failed,
            kept,
            changed,
        } = self.generate_evidences(&file, &content, &dir, reviewed, false)?;
        if !failed.is_empty() {
            self.record_step(&file, "evidências", "erro", "evidência automática com erro")?;
            bail!("evidências automáticas de {task} com erro; corrija e execute vostok task evidence de novo:\n{failed}");
        }
        print_kept_and_changed(&kept, &changed);
        let record = TaskDoc::at(&file).text()?;
        let rows = self.record_rows(&record)?;
        let entries = self.evidence_entries(&record)?;
        let review_kinds = self.review_kinds();
        let mut unreviewed: Vec<String> = Vec::new();
        for (index, entry) in entries.iter().enumerate() {
            let path = dir.join(&entry.name);
            if !review_kinds.contains(&entry.kind) || !path.is_file() {
                continue;
            }
            if !review_is_valid(&rows, &entry.name, &self.review_hashes(&fs::read(&path)?)) {
                unreviewed.push(evidence_line(index, &entry.name, &path, ""));
            }
        }
        if !unreviewed.is_empty() {
            TaskDoc::at(&file).set("evidence_result", "pending")?;
            self.record_step(
                &file,
                "evidências",
                "pending",
                &with_kept("evidência sem conferência humana", &kept, &changed),
            )?;
            println!("Evidências sem conferência humana; mostre cada uma ao usuário e, com a resposta dele, execute vostok task evidence {spec} {task} --reviewed <EV-NN|arquivo>:");
            for line in unreviewed {
                println!("{line}");
            }
            print_same_as_baseline(&same);
            return Ok(());
        }
        if !missing.is_empty() {
            TaskDoc::at(&file).set("evidence_result", "pending")?;
            self.record_step(
                &file,
                "evidências",
                "pending",
                &with_kept("evidência manual sem arquivo", &kept, &changed),
            )?;
            print!("Evidências manuais de {task} sem arquivo; peça ao usuário para anexá-las e execute vostok task evidence de novo:\n{missing}");
            print_same_as_baseline(&same);
            return Ok(());
        }
        TaskDoc::at(&file).set("evidence_result", "passed")?;
        let folder = dir
            .strip_prefix(file.parent().unwrap_or(Path::new("")))
            .unwrap_or(&dir)
            .to_string_lossy();
        let mut detail = format!("{} evidência(s) em {folder}/", items.len());
        if !same.is_empty() {
            let names: Vec<&str> = same.iter().map(|(name, _)| name.as_str()).collect();
            detail.push_str(&format!(
                "; {} igual(is) ao baseline: {}",
                same.len(),
                names.join(", ")
            ));
        }
        self.record_step(
            &file,
            "evidências",
            "passed",
            &with_kept(&detail, &kept, &changed),
        )?;
        println!(
            "Evidências de {task} gravadas em {}: {} item(ns). Resultado: passed.",
            dir.display(),
            items.len()
        );
        print_same_as_baseline(&same);
        Ok(())
    }

    /// Depois de gravar conferências, recalcula o resultado sem gerar nem apagar evidências: só
    /// muda `evidence_result` quando ele está `pending`, que é o estado da finalização.
    fn review_outcome(&self, file: &Path, spec: &str, task: &str, dir: &Path) -> Result<()> {
        let record = TaskDoc::at(file).text()?;
        let rows = self.record_rows(&record)?;
        let review_kinds = self.review_kinds();
        let manual = self.cfg("evidence_manual")?;
        let mut unreviewed: Vec<String> = Vec::new();
        let mut missing = String::new();
        let mut total = 0;
        for (index, entry) in self.evidence_entries(&record)?.iter().enumerate() {
            total += 1;
            let path = dir.join(&entry.name);
            if entry.kind == manual && !fs::metadata(&path).is_ok_and(|meta| meta.len() > 0) {
                missing.push_str(&evidence_line(index, &entry.name, &path, &entry.rest));
                missing.push('\n');
            }
            if review_kinds.contains(&entry.kind)
                && path.is_file()
                && !review_is_valid(&rows, &entry.name, &self.review_hashes(&fs::read(&path)?))
            {
                unreviewed.push(evidence_line(index, &entry.name, &path, ""));
            }
        }
        let pending = TaskDoc::at(file).get("evidence_result")? == "pending";
        if pending && unreviewed.is_empty() && missing.is_empty() {
            TaskDoc::at(file).set("evidence_result", "passed")?;
            let folder = dir
                .strip_prefix(file.parent().unwrap_or(Path::new("")))
                .unwrap_or(dir)
                .to_string_lossy();
            self.record_step(
                file,
                "evidências",
                "passed",
                &format!("{total} evidência(s) em {folder}/"),
            )?;
            println!("Conferência registrada. Resultado: passed.");
            return Ok(());
        }
        if pending {
            TaskDoc::at(file).set("evidence_result", "pending")?;
        } else {
            println!("Conferência registrada; resultados sem mudança.");
        }
        if !unreviewed.is_empty() {
            println!("Evidências sem conferência humana; mostre cada uma ao usuário e, com a resposta dele, execute vostok task evidence {spec} {task} --reviewed <EV-NN|arquivo>:");
            for line in unreviewed {
                println!("{line}");
            }
        }
        if !missing.is_empty() {
            print!("Evidências manuais de {task} sem arquivo; peça ao usuário para anexá-las e execute vostok task evidence de novo:\n{missing}");
        }
        Ok(())
    }

    /// Gera as evidências com a tarefa em execução, sem mudar os resultados, lista o que falta o
    /// usuário conferir e marca a tarefa como pronta para finalizar (executed).
    fn evidence_ahead(&self, spec: &str, task: &str) -> Result<()> {
        let file = self.find_task(spec, task)?;
        let status = status_of(&file);
        if status != self.status.in_progress && status != self.status.executed {
            bail!(
                "a tarefa {task} está com status '{status}'; vostok task evidence --antecipada só roda com a tarefa em execução ({} ou {}); nada foi gerado",
                self.status.in_progress,
                self.status.executed
            );
        }
        let dir = self.evidence_dir(&file)?;
        let content = TaskDoc::at(&file).text()?;
        let text = self.subsection(&content, "evidence_subsection")?;
        let none = self.cfg("evidence_none")?;
        if text.trim() == none {
            self.record_step(
                &file,
                "evidências antecipadas",
                "geradas",
                "0 evidências; a conferir: -",
            )?;
            println!("Evidências antecipadas de {task}: {none}");
            return self.mark_executed(&file, task, "sem evidências a gerar");
        }
        let generated = self.generate_evidences(&file, &content, &dir, &[], true)?;
        if !generated.failed.is_empty() {
            bail!(
                "evidências antecipadas de {task} com erro; nada mudou nos resultados:\n{}",
                generated.failed
            );
        }
        let entries = self.evidence_entries(&content)?;
        let rows = self.record_rows(&content)?;
        let review_kinds = self.review_kinds();
        let mut pending: Vec<String> = Vec::new();
        let mut pending_labels: Vec<String> = Vec::new();
        for (index, entry) in entries.iter().enumerate() {
            let path = dir.join(&entry.name);
            if review_kinds.contains(&entry.kind)
                && path.is_file()
                && !review_is_valid(&rows, &entry.name, &self.review_hashes(&fs::read(&path)?))
            {
                pending.push(evidence_line(index, &entry.name, &path, ""));
                pending_labels.push(format!("{} {}", evidence_id(index), entry.name));
            }
        }
        println!(
            "Evidências antecipadas de {task}: {} gerada(s).",
            entries.len()
        );
        print_kept_and_changed(&generated.kept, &generated.changed);
        if !pending.is_empty() {
            println!("Evidências a conferir (mostre ao usuário e, com a resposta dele, execute vostok task evidence {spec} {task} --reviewed <EV-NN|arquivo> ou --rejected <EV-NN|arquivo> --motivo \"...\"):");
            for line in &pending {
                println!("{line}");
            }
        }
        if !generated.missing.is_empty() {
            print!("Evidências manuais sem arquivo:\n{}", generated.missing);
        }
        let listed = if pending_labels.is_empty() {
            "-".to_string()
        } else {
            pending_labels.join(", ")
        };
        self.record_step(
            &file,
            "evidências antecipadas",
            "geradas",
            &format!("{} evidências; a conferir: {listed}", entries.len()),
        )?;
        self.mark_executed(&file, task, "evidências antecipadas geradas sem erro")
    }

    /// Grava a tarefa como pronta para finalizar; só o `--antecipada` sem erro chega aqui.
    fn mark_executed(&self, file: &Path, task: &str, detail: &str) -> Result<()> {
        let executed = &self.status.executed;
        TaskDoc::at(file).set("status", executed)?;
        self.record_step(file, "execução", executed, detail)?;
        println!("Tarefa {task} pronta para finalizar: status '{executed}'.");
        Ok(())
    }

    /// Roda só o comando rotulado de uma evidência da verificação e grava a saída como log.
    fn run_labeled_ahead(
        &self,
        name: &str,
        command: &str,
        target: &Path,
        root: &Path,
        outputs: &mut Vec<(String, Vec<u8>)>,
        failed: &mut String,
    ) -> Result<()> {
        if target.exists() {
            fs::remove_file(target)?;
        }
        if command.is_empty() {
            failed.push_str(&format!(
                "- {name}: a evidência {name} não tem comando rotulado na Verificação\n"
            ));
            return Ok(());
        }
        let (code, output) = shell_output(command, root)?;
        if code != 0 {
            failed.push_str(&format!(
                "- {name}: `{command}` terminou com erro\n{}\n",
                trim_newlines(&indented_tail(&output, 5))
            ));
            return Ok(());
        }
        let mut log = log_header(command, code, root);
        log.extend_from_slice(&output);
        fs::write(target, log)?;
        outputs.push((name.to_string(), output));
        Ok(())
    }

    /// Gera cada evidência do texto de "Evidências". Com `ahead`, a que vem da verificação roda só o
    /// comando rotulado, em vez de copiar a saída do `tests`; a de conferência válida é mantida.
    fn generate_evidences(
        &self,
        file: &Path,
        content: &str,
        dir: &Path,
        reviewed: &[String],
        ahead: bool,
    ) -> Result<Generated> {
        let text = self.subsection(content, "evidence_subsection")?;
        let items: Vec<&str> = text
            .lines()
            .filter(|line| has_text(line) && !is_indented(line))
            .collect();
        let automatic = self.cfg("evidence_automatic")?;
        let manual = self.cfg("evidence_manual")?;
        fs::create_dir_all(dir)?;
        let root = self.project_root()?;
        let ignore = self.regression_ignore();
        let baseline_dir = self
            .evidence_baseline_dir()
            .map(|sub| dir.join("baseline").join(sub));
        let mut same: Vec<(String, String)> = Vec::new();
        let mut missing = String::new();
        let mut failed = String::new();
        let from_verify = self.evidence_from_verify();
        let diff = self.evidence_diff();
        let print = self.evidence_print();
        let expectations = self.evidence_expectations(&text);
        let block = verify_block(&self.subsection(content, "verify_subsection")?);
        let labeled: HashMap<String, String> = labeled_commands(&block, &self.verify_label())
            .into_iter()
            .collect();
        // Saída de cada evidência gravada, sem o cabeçalho, para conferir o texto esperado.
        let mut outputs: Vec<(String, Vec<u8>)> = Vec::new();
        let rows = self.record_rows(content)?;
        for (index, item) in items.iter().enumerate() {
            if let Some(name) = evidence_name(item, &from_verify) {
                match self.review_kept(&rows, dir, name)? {
                    Some(true) => continue,
                    Some(false) => {
                        failed.push_str(&format!(
                            "- {name}: conferida, mas o arquivo {} está vazio; apague-o para gerar de novo\n",
                            dir.join(name).display()
                        ));
                        continue;
                    }
                    None => {}
                }
            }
            if let Some(name) = evidence_name(item, &from_verify) {
                if reviewed.iter().any(|approved| approved == name) {
                    continue;
                }
            }
            if let Some(name) = verify_evidence_name(item, &from_verify) {
                if !is_valid_evidence_name(name) {
                    bail!("nome de evidência inválido: '{name}'");
                }
                let command = labeled.get(name).map_or("", String::as_str);
                if ahead {
                    self.run_labeled_ahead(
                        name,
                        command,
                        &dir.join(name),
                        &root,
                        &mut outputs,
                        &mut failed,
                    )?;
                    continue;
                }
                match self.copy_verify_output(file, name, command, &root)? {
                    Some(output) => outputs.push((name.to_string(), output)),
                    None => failed.push_str(&format!(
                        "- {name}: a evidência {name} vem da verificação; execute vostok task tests\n"
                    )),
                }
                continue;
            }
            let (name, kind, rest) = evidence_item(item)
                .ok_or_else(|| anyhow!("item de evidência fora do formato: {item}"))?;
            if !is_valid_evidence_name(name) {
                bail!("nome de evidência inválido: '{name}'");
            }
            let target = dir.join(name);
            if kind == print {
                let command = rest
                    .strip_prefix('`')
                    .and_then(|rest| rest.strip_suffix('`'))
                    .filter(|command| !command.is_empty())
                    .ok_or_else(|| anyhow!("print sem comando: {item}"))?;
                if target.exists() {
                    fs::remove_file(&target)?;
                }
                let (code, output) =
                    shell_output_with(command, &root, &[("VOSTOK_EVIDENCE", dir)])?;
                if code != 0 {
                    failed.push_str(&format!(
                        "- {name}: `{command}` terminou com erro\n{}\n",
                        trim_newlines(&indented_tail(&output, 5))
                    ));
                } else if !fs::metadata(&target).is_ok_and(|meta| meta.len() > 0) {
                    failed.push_str(&format!(
                        "- {name}: o comando não gerou a imagem em {}\n",
                        target.display()
                    ));
                }
            } else if kind == diff {
                let path = rest
                    .strip_prefix('`')
                    .and_then(|rest| rest.strip_suffix('`'))
                    .filter(|path| !path.is_empty())
                    .ok_or_else(|| anyhow!("evidência diff sem caminho: {item}"))?;
                let base = DiffBase::of(file_baseline_commit(file), &root);
                if let Some(note) = &base.note {
                    println!("Aviso: {note}");
                }
                match path_diff(path, &root, &base)? {
                    Ok((command, output)) => {
                        let mut log = log_header(&command, 0, &root);
                        log.extend_from_slice(&output);
                        fs::write(&target, log)?;
                        outputs.push((name.to_string(), output));
                    }
                    Err(problem) => {
                        failed.push_str(&format!("- {name}: {problem}\n"));
                        if target.exists() {
                            fs::remove_file(&target)?;
                        }
                    }
                }
            } else if kind == automatic {
                let command = rest
                    .strip_prefix('`')
                    .and_then(|rest| rest.strip_suffix('`'))
                    .filter(|command| !command.is_empty())
                    .ok_or_else(|| anyhow!("evidência automática sem comando: {item}"))?;
                // A saída só vira evidência se o comando terminar sem erro; com erro, ela vai para a mensagem.
                let (code, output) = shell_output(command, &root)?;
                if code == 0 {
                    if let Some(recorded) = baseline_dir
                        .as_ref()
                        .and_then(|base| fs::read(base.join(name)).ok())
                    {
                        let before = recorded
                            .splitn(3, |byte| *byte == b'\n')
                            .nth(2)
                            .unwrap_or_default();
                        if recorded_exit_code(&recorded) == Some(0)
                            && !before.starts_with(NOT_EXECUTED.as_bytes())
                            && recorded_exit_code(&recorded) != Some(TIMEOUT_EXIT)
                            && without_ignored(before, &ignore) == without_ignored(&output, &ignore)
                        {
                            same.push((name.to_string(), command.to_string()));
                        }
                    }
                    let mut log = log_header(command, code, &root);
                    log.extend_from_slice(&output);
                    fs::write(&target, log)?;
                    outputs.push((name.to_string(), output));
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
                missing.push_str(&evidence_line(index, name, &target, rest));
                missing.push('\n');
            }
        }

        for (name, output) in &outputs {
            if let Some(expected) = expectations.get(name) {
                if !String::from_utf8_lossy(output).contains(expected.as_str()) {
                    failed.push_str(&format!(
                        "- {name}: texto esperado não encontrado: {expected}\n"
                    ));
                }
            }
        }
        let (kept, changed) = self.reviews_after_generation(content, dir, &rows)?;
        self.clear_changed_rejections(file, dir, &rows)?;
        Ok(Generated {
            same,
            missing,
            failed,
            kept,
            changed,
        })
    }

    /// Evidências com conferência ou rejeição no registro, separadas pela saída atual: as aceitas
    /// com o mesmo hash continuam conferidas; as de hash diferente voltam a pedir conferência (RF-10).
    fn reviews_after_generation(
        &self,
        content: &str,
        dir: &Path,
        rows: &[RecordRow],
    ) -> Result<(Vec<String>, Vec<String>)> {
        let mut kept: Vec<String> = Vec::new();
        let mut changed: Vec<String> = Vec::new();
        for (index, entry) in self.evidence_entries(content)?.iter().enumerate() {
            let path = dir.join(&entry.name);
            if !path.is_file() {
                continue;
            }
            let Some((_, review)) = last_review(rows, &entry.name) else {
                continue;
            };
            let label = format!("{} {}", evidence_id(index), entry.name);
            let same = self.review_hashes(&fs::read(&path)?).matches(&review.hash);
            if same && review.accepted {
                kept.push(label);
            } else if !same {
                changed.push(label);
            }
        }
        Ok((kept, changed))
    }

    /// Tira de `rejected_evidences` a evidência gerada com saída diferente da recusada (RF-09); a que
    /// não foi gerada continua recusada.
    fn clear_changed_rejections(&self, file: &Path, dir: &Path, rows: &[RecordRow]) -> Result<()> {
        let rejected = rejected_of(file);
        let mut pending: Vec<String> = Vec::new();
        for name in &rejected {
            let path = dir.join(name);
            let still = !path.is_file()
                || last_rejection(rows, name).is_some_and(|(_, review)| {
                    self.review_hashes(&fs::read(&path).unwrap_or_default())
                        .matches(&review.hash)
                });
            if still {
                pending.push(name.clone());
            }
        }
        if pending.len() != rejected.len() {
            write_rejected(file, &pending)?;
        }
        Ok(())
    }

    /// Bytes da evidência `name` que o usuário confere ou rejeita; recusa o que não é evidência da
    /// tarefa, não foi gerado ou está vazio.
    fn reviewable(&self, content: &str, dir: &Path, name: &str) -> Result<Vec<u8>> {
        if !self
            .evidence_entries(content)?
            .iter()
            .any(|entry| entry.name == name)
        {
            bail!("'{name}' não é uma evidência desta tarefa; nada foi registrado");
        }
        let path = dir.join(name);
        if !path.is_file() {
            bail!("'{name}' ainda não foi gerada; nada foi registrado");
        }
        let bytes = fs::read(&path)?;
        if bytes.is_empty() {
            bail!("'{name}' está vazia; nada foi registrado");
        }
        Ok(bytes)
    }

    /// ID e arquivo da evidência que o usuário citou, pelo ID (`EV-NN`) ou pelo arquivo.
    fn resolve_evidence(&self, content: &str, task: &str, given: &str) -> Result<(String, String)> {
        let entries = self.evidence_entries(content)?;
        if is_evidence_id(given) {
            if let Some((index, entry)) = entries
                .iter()
                .enumerate()
                .find(|(index, _)| evidence_id(*index) == given)
            {
                return Ok((evidence_id(index), entry.name.clone()));
            }
            let valid: Vec<String> = entries
                .iter()
                .enumerate()
                .map(|(index, entry)| format!("{} {}", evidence_id(index), entry.name))
                .collect();
            bail!(
                "'{given}' não é um ID de evidência de {task}; IDs válidos: {}; nada foi registrado",
                valid.join(", ")
            );
        }
        match entries.iter().position(|entry| entry.name == given) {
            Some(index) => Ok((evidence_id(index), given.to_string())),
            None => bail!("'{given}' não é uma evidência desta tarefa; nada foi registrado"),
        }
    }

    /// Recusa de uma evidência pelo usuário: grava o motivo, marca a evidência em
    /// `rejected_evidences` e devolve a tarefa para `in_progress`, sem bloquear nem contar
    /// tentativa (RF-09, RN-03).
    fn reject(
        &self,
        file: &Path,
        spec: &str,
        task: &str,
        (id, name): (&str, &str),
        reason: &str,
    ) -> Result<()> {
        let dir = self.evidence_dir(file)?;
        let content = TaskDoc::at(file).text()?;
        let bytes = self.reviewable(&content, &dir, name)?;
        let doc = TaskDoc::at(file);
        doc.set("evidence_result", "failed")?;
        let in_progress = &self.status.in_progress;
        doc.set("status", in_progress)?;
        self.record_step(
            file,
            "rejeição",
            in_progress,
            &evidence_rejection(id, name, &self.review_hashes(&bytes), reason),
        )?;
        let mut rejected = rejected_of(file);
        if !rejected.iter().any(|rejected| rejected == name) {
            rejected.push(name.to_string());
        }
        write_rejected(file, &rejected)?;
        let entries = self.evidence_entries(&content)?;
        let pending: Vec<String> = rejected
            .iter()
            .map(|rejected| evidence_label(&entries, rejected))
            .collect();
        println!(
            "Evidência {id} {name} recusada: {reason}. Tarefa {task} em '{in_progress}'; recusas pendentes: {}. Corrija o que cada motivo aponta e gere as evidências de novo com vostok task evidence {spec} {task} --antecipada.",
            pending.join(", ")
        );
        Ok(())
    }

    /// Linhas do "Registro de execução" de `content`, na ordem.
    fn record_rows(&self, content: &str) -> Result<Vec<RecordRow>> {
        let section = self.cfg("record_section")?;
        Ok(markdown_section(content, &section)
            .map(|text| text.lines().filter_map(record_row).collect())
            .unwrap_or_default())
    }

    /// Evidência mantida sem gerar de novo (SPEC-046): a última conferência dela é aceita, com o hash
    /// atual e depois da última mudança da tarefa (ajuste ou retomada; a recusa de outra evidência não
    /// conta, para não gerar de novo as conferidas a cada recusa). `Some(true)` se o
    /// arquivo tem conteúdo, `Some(false)` se está vazio, `None` se ela é gerada de novo.
    fn review_kept(&self, rows: &[RecordRow], dir: &Path, name: &str) -> Result<Option<bool>> {
        let path = dir.join(name);
        if !is_valid_evidence_name(name) || !path.is_file() {
            return Ok(None);
        }
        let changed_at = rows
            .iter()
            .rposition(|row| matches!(row.step.as_str(), "ajuste" | "retomada"));
        let Some((index, review)) = rows.iter().enumerate().rev().find_map(|(index, row)| {
            review_of(row)
                .filter(|review| review.name == name)
                .map(|review| (index, review))
        }) else {
            return Ok(None);
        };
        let bytes = fs::read(&path)?;
        let kept = review.accepted
            && changed_at.is_none_or(|changed_at| index > changed_at)
            && self.review_hashes(&bytes).matches(&review.hash);
        Ok(kept.then_some(!bytes.is_empty()))
    }

    /// Grava na evidência, como log do comando rotulado, a saída do rótulo `name` gravada pelo
    /// `tests`, que só vale com os testes aprovados (RN-04), e devolve a saída. Sem ela, apaga a
    /// evidência antiga e devolve `None`.
    fn copy_verify_output(
        &self,
        file: &Path,
        name: &str,
        command: &str,
        root: &Path,
    ) -> Result<Option<Vec<u8>>> {
        let target = self.evidence_dir(file)?.join(name);
        let tests_passed = TaskDoc::at(file).get("tests_result").unwrap_or_default() == "passed";
        let output = self.verify_dir(file)?.join(name);
        if tests_passed && output.is_file() {
            let output = fs::read(&output)?;
            let mut log = log_header(command, 0, root);
            log.extend_from_slice(&output);
            fs::write(&target, log)?;
            return Ok(Some(output));
        }
        if target.exists() {
            fs::remove_file(&target)?;
        }
        Ok(None)
    }

    // --- finish e resume ---

    /// Uma recusa do `finish` por evidência recusada sem nova geração, com o motivo (RF-11).
    fn rejection_problems(
        &self,
        spec: &str,
        task: &str,
        file: &Path,
    ) -> Result<Vec<(&'static str, String, String)>> {
        let rejected = rejected_of(file);
        if rejected.is_empty() {
            return Ok(Vec::new());
        }
        let content = TaskDoc::at(file).text()?;
        let rows = self.record_rows(&content)?;
        let entries = self.evidence_entries(&content)?;
        let next = format!("vostok task evidence {spec} {task} --antecipada");
        Ok(rejected
            .iter()
            .map(|name| {
                let label = evidence_label(&entries, name);
                let reason =
                    last_rejection(&rows, name).map_or("", |(row, _)| rejection_reason(row));
                (
                    "recusas",
                    format!("{label} recusada pelo usuário: {reason}"),
                    next.clone(),
                )
            })
            .collect())
    }

    /// Impedimentos do `finish` na ordem fixa: nome, detalhe e próximo comando.
    fn finish_problems(
        &self,
        spec: &str,
        task: &str,
        file: &Path,
    ) -> Result<Vec<(&'static str, String, String)>> {
        let mut problems: Vec<(&'static str, String, String)> = Vec::new();
        let results = [
            ("testes", "tests_result", "tests"),
            ("regressão", "regression_result", "regression"),
            ("evidências", "evidence_result", "evidence"),
        ];
        for (name, key, command) in results {
            let value = TaskDoc::at(file).get(key).unwrap_or_default();
            if value == "passed" {
                continue;
            }
            let mut next = format!("vostok task {command} {spec} {task}");
            if key == "evidence_result" && value == "pending" {
                next.push_str(" --reviewed <arquivo>");
            }
            problems.push((name, format!("{key} '{value}'"), next));
        }
        problems.extend(self.rejection_problems(spec, task, file)?);
        if let Err(error) = self.check_done(spec, task) {
            let text = error.to_string();
            let (first, rest) = text.split_once('\n').unwrap_or((&text, ""));
            let mut detail = first.trim_end_matches(':').to_string();
            for line in rest.lines() {
                detail.push_str(&format!("\n   {line}"));
            }
            problems.push((
                "critérios",
                detail,
                format!("vostok task check-done {spec} {task}"),
            ));
        }
        let persist = format!("vostok task persist {spec} {task}");
        for problem in self.persist_problems(spec, task, file)? {
            problems.push(("persistência", problem, persist.clone()));
        }
        let doc = TaskDoc::at(file);
        if doc.folder() {
            let describe = format!("vostok task describe {spec} {task}");
            for (key, name) in layout().document_files() {
                let text = read(&doc.part(name))?;
                if !template_leftovers(&text).is_empty() {
                    problems.push(("documentos", format!("o documento {name} de {task} ainda está no template; escreva-o a partir de vostok task describe"), describe.clone()));
                } else if doc.get(key).ok().as_deref() == Some(sha256_hex(text.as_bytes()).as_str())
                {
                    problems.push(("documentos", format!("o documento {name} de {task} não foi enriquecido desde o início; enriqueça-o a partir de vostok task describe"), describe.clone()));
                }
            }
        }
        let done = &self.status.done;
        for id in closes_after_of(file) {
            if !self.task_file(spec, &id).exists() {
                problems.push((
                    "espera de fechamento",
                    format!("{id} não existe em {spec}"),
                    format!("corrija closes_after de {task}"),
                ));
            } else if status_of(&self.task_file(spec, &id)) != *done {
                problems.push((
                    "espera de fechamento",
                    format!("{id} não está {done}"),
                    format!("conclua {id} antes"),
                ));
            }
        }
        Ok(problems)
    }

    fn run_closes_after(&self, command: ClosesAfterCommand) -> Result<()> {
        let (add, spec_id, task_id, target, reason) = match command {
            ClosesAfterCommand::Add {
                spec_id,
                task_id,
                target,
                reason,
            } => (true, spec_id, task_id, target, reason),
            ClosesAfterCommand::Remove {
                spec_id,
                task_id,
                target,
                reason,
            } => (false, spec_id, task_id, target, reason),
        };
        let (spec, task) = filled(spec_id, task_id, CLOSES_AFTER_USAGE)?;
        let target = target
            .filter(|id| !id.is_empty())
            .ok_or_else(|| usage(CLOSES_AFTER_USAGE))?;
        let reason = clean_summary(&reason.ok_or_else(|| usage(CLOSES_AFTER_USAGE))?)?;
        self.change_closes_after(&spec, &task, &target, &reason, add)
    }

    fn change_closes_after(
        &self,
        spec: &str,
        task: &str,
        target: &str,
        reason: &str,
        add: bool,
    ) -> Result<()> {
        let file = self.find_task(spec, task)?;
        self.check_task_id(target)?;
        let status = status_of(&file);
        if status == self.status.done {
            bail!("a tarefa {task} está com status '{status}'; closes_after não muda em tarefa concluída");
        }
        if target == task {
            bail!("a tarefa {task} não pode fechar depois dela mesma; nada foi gravado");
        }
        if !self.task_file(spec, target).exists() {
            bail!("a tarefa {target} não existe em {spec}; nada foi gravado");
        }
        let mut list = closes_after_of(&file);
        let listed = list.iter().any(|id| id == target);
        if add && listed {
            bail!("a tarefa {task} já fecha depois de {target}; nada foi gravado");
        }
        if !add && !listed {
            bail!("a tarefa {task} não fecha depois de {target}; nada foi gravado");
        }
        if add {
            list.push(target.to_string());
        } else {
            list.retain(|id| id != target);
        }
        let doc = TaskDoc::at(&file);
        doc.put("closes_after", &format!("[{}]", list.join(", ")))?;
        let action = if add { "acrescentada" } else { "retirada" };
        self.record_step(
            &file,
            "ajuste",
            &status,
            &format!("fecha depois de {target} {action}: {reason}"),
        )?;
        if self.is_running(&status) {
            doc.set("evidence_result", "null")?;
        }
        println!("Tarefa {task}: fecha depois de {target} {action}.");
        Ok(())
    }

    // --- depends ---

    fn run_depends(&self, command: DependsCommand) -> Result<()> {
        let (add, spec_id, task_id, dependency, reason) = match command {
            DependsCommand::Add {
                spec_id,
                task_id,
                dependency,
                reason,
            } => (true, spec_id, task_id, dependency, reason),
            DependsCommand::Remove {
                spec_id,
                task_id,
                dependency,
                reason,
            } => (false, spec_id, task_id, dependency, reason),
        };
        let (spec, task) = filled(spec_id, task_id, DEPENDS_USAGE)?;
        let dependency = dependency
            .filter(|id| !id.is_empty())
            .ok_or_else(|| usage(DEPENDS_USAGE))?;
        let reason = clean_summary(&reason.ok_or_else(|| usage(DEPENDS_USAGE))?)?;
        self.change_dependency(&spec, &task, &dependency, &reason, add)
    }

    fn change_dependency(
        &self,
        spec: &str,
        task: &str,
        dependency: &str,
        reason: &str,
        add: bool,
    ) -> Result<()> {
        let file = self.find_task(spec, task)?;
        self.check_task_id(dependency)?;
        self.require_dependency_status(&file, task)?;
        if !self.hash_is_current(&file)? {
            bail!(
                "a tarefa {task} tem edição sem registro; registre-a antes (vostok task adjust record-manual {spec} {task})"
            );
        }
        let mut dependencies = dependencies_of(&file);
        if add {
            self.check_new_dependency(spec, task, dependency, &dependencies)?;
            dependencies.push(dependency.to_string());
        } else {
            let before = dependencies.len();
            dependencies.retain(|id| id != dependency);
            if dependencies.len() == before {
                bail!("a tarefa {task} não depende de {dependency}; nada foi gravado");
            }
        }
        let saved = self.stored_state(&file)?;
        write_dependencies(&file, &dependencies)?;
        let errors = self.errors_of(spec, &file, EvidenceNames::Normalizable)?;
        if !errors.is_empty() {
            self.restore_state(&file, &saved)?;
            bail!(
                "a tarefa {task} ficaria com problemas; nada foi gravado:\n{}",
                errors.join("\n")
            );
        }
        let summary = if add {
            format!("dependência {dependency} acrescentada: {reason}")
        } else {
            format!("dependência {dependency} retirada: {reason}")
        };
        let origin = self.spec_cfg("adjustment_origins.agent")?;
        let count = self.record_adjustment(&file, &origin, &summary)?;
        println!("Ajuste {count} registrado em {task}: {summary}");
        self.settle_adjustment(spec, &file, task, &summary, add.then_some(reason))
    }

    /// A dependência só muda em tarefa em execução, em validação ou bloqueada por dependência.
    fn require_dependency_status(&self, file: &Path, task: &str) -> Result<()> {
        let status = status_of(file);
        let blocked = status == self.status.blocked;
        if [
            &self.status.in_progress,
            &self.status.executed,
            &self.status.validating,
        ]
        .contains(&&status)
        {
            return Ok(());
        }
        if blocked && !waiting_of(file).is_empty() {
            return Ok(());
        }
        if blocked {
            bail!("a tarefa {task} está bloqueada por falha; decida o bloqueio antes (vostok task resume), para os dois motivos não se misturarem");
        }
        bail!("a tarefa {task} está com status '{status}'; a dependência só muda aqui em tarefa em execução, em validação ou bloqueada por dependência; nos demais casos, faça um novo plano");
    }

    fn check_new_dependency(
        &self,
        spec: &str,
        task: &str,
        dependency: &str,
        dependencies: &[String],
    ) -> Result<()> {
        if dependency == task {
            bail!("ciclo: {task} não pode depender dela mesma; nada foi gravado");
        }
        let dependency_file = self.task_file(spec, dependency);
        if !dependency_file.exists() {
            bail!("a dependência {dependency} não existe em {spec}: só vale dependência entre tarefas da mesma spec; nada foi gravado");
        }
        if dependencies.iter().any(|id| id == dependency) {
            bail!("a tarefa {task} já depende de {dependency}; nada foi gravado");
        }
        let mut seen = HashSet::new();
        let mut pending = vec![dependency.to_string()];
        while let Some(id) = pending.pop() {
            if id == task {
                bail!("ciclo: {dependency} já depende de {task}, direta ou indiretamente; nada foi gravado");
            }
            if seen.insert(id.clone()) {
                pending.extend(dependencies_of(&self.task_file(spec, &id)));
            }
        }
        Ok(())
    }

    /// Texto do estado (`estado.yml` da pasta ou o arquivo único) para desfazer uma gravação.
    fn stored_state(&self, file: &Path) -> Result<String> {
        let doc = TaskDoc::at(file);
        if doc.folder() {
            read(&doc.part(&layout().state))
        } else {
            read(file)
        }
    }

    fn restore_state(&self, file: &Path, saved: &str) -> Result<()> {
        let doc = TaskDoc::at(file);
        if doc.folder() {
            write(&doc.part(&layout().state), saved)?;
            return doc.write_readme();
        }
        write(file, saved)
    }

    /// Dependências da tarefa que ainda não foram concluídas.
    fn pending_dependencies(&self, spec: &str, file: &Path) -> Vec<String> {
        dependencies_of(file)
            .into_iter()
            .filter(|id| {
                let dependency = self.task_file(spec, id);
                !dependency.exists() || status_of(&dependency) != self.status.done
            })
            .collect()
    }

    /// Devolve a tarefa em espera a approved, mantendo a aprovação. Não grava o hash, para uma edição
    /// sem registro feita durante a espera continuar aparecendo.
    fn release_waiting(&self, file: &Path, task: &str, detail: &str) -> Result<()> {
        let approved = &self.status.approved;
        TaskDoc::at(file).set("status", approved)?;
        write_waiting(file, &[], "")?;
        self.append_record_row(file, "desbloqueio", approved, detail)?;
        println!("Tarefa {task} desbloqueada: status '{approved}'.");
        Ok(())
    }

    /// Libera as tarefas em espera da `finished` e mantém bloqueadas as que ainda esperam outra.
    fn release_dependents(&self, file: &Path, finished: &str) -> Result<()> {
        for other in task_entries(file.parent().unwrap_or(Path::new("")))? {
            let waiting = waiting_of(&other);
            if !waiting.iter().any(|id| id == finished) {
                continue;
            }
            let id = task_id_of(&other);
            let rest: Vec<String> = waiting.into_iter().filter(|dep| dep != finished).collect();
            if rest.is_empty() {
                self.release_waiting(&other, &id, &format!("{finished} concluída"))?;
                continue;
            }
            let reason = TaskDoc::at(&other)
                .get(WAITING_REASON_FIELD)
                .unwrap_or_default();
            write_waiting(&other, &rest, &reason)?;
            println!(
                "Tarefa {id} continua bloqueada: espera {}.",
                rest.join(", ")
            );
        }
        Ok(())
    }

    /// Depois de um ajuste de dependência: bloqueia a tarefa que ficou com dependência pendente ou
    /// reinicia a execução.
    fn settle_adjustment(
        &self,
        spec: &str,
        file: &Path,
        task: &str,
        summary: &str,
        reason: Option<&str>,
    ) -> Result<()> {
        if !self.is_running(&status_of(file)) {
            return Ok(());
        }
        let waiting = waiting_of(file);
        let pending = self.pending_dependencies(spec, file);
        if pending.is_empty() && waiting.is_empty() {
            return self.restart_execution(file, task, summary);
        }
        reset_results(file)?;
        if pending.is_empty() {
            self.record_step(file, "ajuste", &self.status.approved, summary)?;
            return self.release_waiting(file, task, "sem dependência pendente depois do ajuste");
        }
        let blocked = &self.status.blocked;
        TaskDoc::at(file).set("status", blocked)?;
        self.record_step(file, "ajuste", blocked, summary)?;
        let current = TaskDoc::at(file)
            .get(WAITING_REASON_FIELD)
            .unwrap_or_default();
        let reason = reason
            .map(str::to_string)
            .or_else(|| Some(current).filter(|text| !text.is_empty()))
            .unwrap_or_else(|| summary.to_string());
        write_waiting(file, &pending, &reason)?;
        if pending != waiting {
            self.record_step(
                file,
                "bloqueio",
                blocked,
                &format!("espera {}: {reason}", pending.join(", ")),
            )?;
        }
        println!("Tarefa {task} bloqueada: espera {}.", pending.join(", "));
        Ok(())
    }

    fn finish(&self, spec: &str, task: &str) -> Result<()> {
        let file = self.find_task(spec, task)?;
        self.require_running(&file, task)?;
        let problems = self.finish_problems(spec, task, &file)?;
        if !problems.is_empty() {
            bail!("{}", refusal_message(task, &problems));
        }

        let done = &self.status.done;
        TaskDoc::at(&file).set("status", done)?;
        self.record_step(
            &file,
            "conclusão",
            done,
            "testes, regressão e evidências passed; critérios de conclusão marcados",
        )?;
        println!("Tarefa {task} concluída: status '{done}'.");
        self.release_dependents(&file, task)?;

        let unfinished: Vec<String> = task_entries(file.parent().unwrap_or(Path::new("")))?
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
        crate::commands::spec::record_status(&spec_file, &implemented)?;
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
        TaskDoc::at(&file).set("status", validating)?;
        if acceptances.is_empty() {
            reset_results(&file)?;
            self.record_step(&file, "retomada", validating, &detail)?;
            println!("Tarefa {task} retomada: status '{validating}', tentativas e resultados zerados. Aplique a decisão e valide de novo, começando por vostok task tests.");
            return Ok(());
        }
        // O aceite não muda código nem testes: testes e evidências continuam valendo.
        TaskDoc::at(&file).set("attempts", "0")?;
        TaskDoc::at(&file).set("regression_result", "null")?;
        self.record_step(&file, "retomada", validating, &detail)?;
        println!("Tarefa {task} retomada: status '{validating}', diferença aceita. Execute vostok task regression e, com ela aprovada, vostok task finish.");
        Ok(())
    }

    // --- Persistência das evidências no projeto ---

    fn persist_settings(&self) -> Option<PersistSettings> {
        let dir = self.cfg("persist_dir").ok().filter(|dir| !dir.is_empty())?;
        let readme = self
            .cfg("persist_readme")
            .ok()
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| "README.md".to_string());
        let max_mb = self
            .cfg("persist_max_mb")
            .ok()
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(10.0);
        Some(PersistSettings {
            dir,
            readme,
            max_mb,
        })
    }

    /// Evidências de "Evidências" com os subitens de cada uma.
    fn evidence_entries(&self, content: &str) -> Result<Vec<EvidenceEntry>> {
        let text = self.subsection(content, "evidence_subsection")?;
        let from_verify = self.evidence_from_verify();
        let mut entries: Vec<EvidenceEntry> = Vec::new();
        for line in text.lines().filter(|line| has_text(line)) {
            if let Some((field, value)) = evidence_subitem(line) {
                if let Some(entry) = entries.last_mut() {
                    entry.fields.insert(field.to_string(), value.to_string());
                }
            } else if let Some(name) = verify_evidence_name(line, &from_verify) {
                entries.push(EvidenceEntry {
                    name: name.to_string(),
                    kind: from_verify.clone(),
                    rest: String::new(),
                    fields: HashMap::new(),
                });
            } else if let Some((name, kind, rest)) = evidence_item(line) {
                entries.push(EvidenceEntry {
                    name: name.to_string(),
                    kind: kind.to_string(),
                    rest: rest.to_string(),
                    fields: HashMap::new(),
                });
            }
        }
        Ok(entries)
    }

    fn persist(&self, spec: &str, task: &str, options: &PersistOptions) -> Result<()> {
        let file = self.find_task(spec, task)?;
        let status = status_of(&file);
        let (validating, done) = (&self.status.validating, &self.status.done);
        if status != *validating && status != *done {
            bail!("a tarefa {task} está com status '{status}'; só tarefas com status '{validating}' ou '{done}' têm as evidências persistidas");
        }
        let Some(settings) = self.persist_settings() else {
            bail!("a persistência está desligada: defina persist_dir em config/tasks.yml (ex.: docs/evidencias/<SPEC>/<TASK>)");
        };
        let root = self.project_root()?;
        let target = settings.target(&root, spec, task);
        let source = self.evidence_dir(&file)?;
        let content = TaskDoc::at(&file).text()?;
        let evidences = self.evidence_entries(&content)?;

        let mut files: Vec<String> = Vec::new();
        for evidence in &evidences {
            if !source.join(&evidence.name).is_file() {
                bail!(
                    "evidência {} não encontrada em {}; execute vostok task evidence {spec} {task}",
                    evidence.name,
                    source.display()
                );
            }
            files.push(evidence.name.clone());
        }
        for dir in ["baseline", "validation"] {
            files.extend(relative_files(&source, &source.join(dir))?);
        }
        let decisions = persist_decisions(options, &files)?;
        let reviews = persist_reviews(&options.reviewed, &files)?;

        let limit = (settings.max_mb * 1_048_576.0) as u64;
        let mut copies: Vec<(String, PersistCopy)> = Vec::new();
        let mut findings = String::new();
        for name in &files {
            let path = source.join(name);
            let bytes = fs::read(&path)
                .with_context(|| format!("não foi possível ler {}", path.display()))?;
            let decision = decisions.get(name.as_str()).copied();
            let copy = if bytes.len() as u64 > limit {
                PersistCopy::Large {
                    size: bytes.len() as u64,
                    hash: sha256_hex(&bytes),
                    origin: relative_to(&path, &root),
                }
            } else if decision == Some(PersistDecision::Exclude) {
                PersistCopy::Excluded
            } else if let Some(text) = text_content(&bytes) {
                let text = home_to_tilde(text);
                let found = sensitive_spans(&text);
                match decision {
                    Some(PersistDecision::Redact) => PersistCopy::Text {
                        content: redact_spans(&text, &found),
                        note: Some("dado sensível redigido por decisão do usuário"),
                    },
                    Some(PersistDecision::Accept) => PersistCopy::Text {
                        content: text,
                        note: Some("dado sensível mantido por decisão do usuário"),
                    },
                    _ => {
                        for span in &found {
                            findings.push_str(&format!(
                                "- {name}:{}: {}\n",
                                span.line,
                                span.kind.label()
                            ));
                        }
                        PersistCopy::Text {
                            content: text,
                            note: None,
                        }
                    }
                }
            } else {
                PersistCopy::Binary(bytes)
            };
            copies.push((name.clone(), copy));
        }
        if !findings.is_empty() {
            bail!("dado sensível nas evidências de {task}; nada foi copiado. PARE: mostre os achados ao usuário e, com a decisão dele, execute de novo com --redact, --exclude ou --accept para cada arquivo:\n{findings}");
        }

        let readme_path = target.join(&settings.readme);
        let previous = fs::read_to_string(&readme_path).unwrap_or_default();
        if target.exists() {
            let mut allowed: HashSet<String> = files.iter().cloned().collect();
            allowed.insert(settings.readme.clone());
            let foreign: Vec<String> = relative_files(&target, &target)?
                .into_iter()
                .filter(|name| !allowed.contains(name))
                .collect();
            if !foreign.is_empty() {
                bail!(
                    "a pasta {} tem arquivos que não vieram do vostok task persist; nada foi alterado:\n- {}",
                    target.display(),
                    foreign.join("\n- ")
                );
            }
            fs::remove_dir_all(&target)?;
        }
        fs::create_dir_all(&target)?;
        let mut copied = 0;
        for (name, copy) in &copies {
            let destination = target.join(name);
            let bytes: &[u8] = match copy {
                PersistCopy::Text { content, .. } => content.as_bytes(),
                PersistCopy::Binary(bytes) => bytes,
                PersistCopy::Excluded | PersistCopy::Large { .. } => continue,
            };
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&destination, bytes)?;
            copied += 1;
        }

        let guide =
            self.persist_guide(spec, task, &file, &evidences, &copies, &reviews, &previous)?;
        fs::write(&readme_path, &guide)?;

        let skipped = copies.len() - copied;
        println!(
            "Evidências de {task} persistidas em {}: {copied} arquivo(s) copiado(s){}.",
            relative_to(&target, &root),
            if skipped > 0 {
                format!(
                    ", {skipped} não copiado(s) (excluídos ou acima de {} MB)",
                    settings.max_mb
                )
            } else {
                String::new()
            }
        );
        let pending: Vec<&str> = copies
            .iter()
            .filter(|(name, copy)| {
                matches!(copy, PersistCopy::Binary(_))
                    && !reviews.contains_key(name.as_str())
                    && !previous_review_done(&previous, name)
            })
            .map(|(name, _)| name.as_str())
            .collect();
        if !pending.is_empty() {
            println!("Binários sem revisão: revise cada um e execute de novo com --reviewed <arquivo>=<agente|usuário>:\n- {}", pending.join("\n- "));
        }
        let leftovers = template_leftovers(&guide).len();
        if leftovers > 0 {
            println!(
                "Complete o guia {}: {leftovers} campo(s) a preencher (trechos que provam e campos que faltam na tarefa).",
                relative_to(&readme_path, &root)
            );
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn persist_guide(
        &self,
        spec: &str,
        task: &str,
        file: &Path,
        evidences: &[EvidenceEntry],
        copies: &[(String, PersistCopy)],
        reviews: &HashMap<&str, &str>,
        previous: &str,
    ) -> Result<String> {
        let template_path = self.harness.path("templates")?.join(PERSIST_TEMPLATE);
        let template = read(&template_path)?;
        let template = strip_leading_comment(&template);
        let (top, entry) = template.split_once(PERSIST_ENTRY_MARKER).ok_or_else(|| {
            anyhow!(
                "template sem o marcador de entrada: {}",
                template_path.display()
            )
        })?;
        let today = &utc_now()[..10];
        let note_of = |name: &str| -> String {
            match copies
                .iter()
                .find(|(copied, _)| copied == name)
                .map(|(_, copy)| copy)
            {
                Some(PersistCopy::Text {
                    note: Some(note), ..
                }) => format!(" ({note})"),
                Some(PersistCopy::Excluded) => {
                    " (não copiado: excluído por decisão do usuário)".to_string()
                }
                Some(PersistCopy::Large { size, hash, origin }) => format!(
                    " (não copiado: {}, SHA-256 `{hash}`, original em `{origin}`)",
                    human_size(*size)
                ),
                _ => String::new(),
            }
        };

        let regression = self.evidence_dir(file)?.join("validation/regression");
        let accepted = accepted_regressions(&regression);
        let accepted_text = if accepted.is_empty() {
            "nenhuma.".to_string()
        } else {
            accepted
                .iter()
                .map(|item| {
                    format!(
                        "\n  - `{}`: {} ([diff](validation/regression/{}))",
                        item.command,
                        item.decision,
                        accepted_diff_name(item.number)
                    )
                })
                .collect()
        };
        let evidence_names: HashSet<&str> = evidences.iter().map(|e| e.name.as_str()).collect();
        let support: Vec<String> = copies
            .iter()
            .filter(|(name, _)| !evidence_names.contains(name.as_str()))
            .map(|(name, copy)| match copy {
                PersistCopy::Text { .. } | PersistCopy::Binary(_) => {
                    format!("- [{name}]({name}){}", note_of(name))
                }
                _ => format!("- {name}{}", note_of(name)),
            })
            .collect();
        let title = TaskDoc::at(file).get("title").unwrap_or_default();
        let mut values: HashMap<&str, String> = HashMap::from([
            ("TASK_ID", task.to_string()),
            ("TASK_TITLE", title),
            ("SPEC_ID", spec.to_string()),
            ("DATE", today.to_string()),
            (
                "TESTS_RESULT",
                TaskDoc::at(file).get("tests_result").unwrap_or_default(),
            ),
            (
                "REGRESSION_RESULT",
                TaskDoc::at(file)
                    .get("regression_result")
                    .unwrap_or_default(),
            ),
            ("ACCEPTED", accepted_text),
            ("SUPPORT_FILES", support.join("\n")),
        ]);
        let mut guide = render_known(top, &values);

        let automatic = self.cfg("evidence_automatic")?;
        for evidence in evidences {
            let field = |name: &str, missing: &str| {
                evidence
                    .fields
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| missing.to_string())
            };
            let copy = copies
                .iter()
                .find(|(name, _)| *name == evidence.name)
                .map(|(_, copy)| copy);
            let file_text = match copy {
                Some(PersistCopy::Text { .. } | PersistCopy::Binary(_)) => {
                    format!("[{0}]({0}){1}", evidence.name, note_of(&evidence.name))
                }
                _ => format!("{}{}", evidence.name, note_of(&evidence.name)),
            };
            let command = if evidence.kind == automatic {
                evidence.rest.clone()
            } else {
                format!("evidência manual: {}", evidence.rest)
            };
            let review = match copy {
                Some(PersistCopy::Binary(_)) => match reviews.get(evidence.name.as_str()) {
                    Some(who) => format!("revisado pelo {who} em {today}, sem dado sensível."),
                    None => previous_value(previous, &evidence.name, PERSIST_REVIEW_LABEL)
                        .unwrap_or_else(|| "#{REVISAO}".to_string()),
                },
                Some(PersistCopy::Text { .. }) => {
                    "texto conferido automaticamente; caminhos pessoais trocados por `~`."
                        .to_string()
                }
                _ => "não copiado.".to_string(),
            };
            values = HashMap::from([
                ("NAME", evidence.name.clone()),
                ("PROVA", field("Prova", "#{PROVA}")),
                ("COMMAND", command),
                ("FILE", file_text),
                ("TRECHO_ESPERADO", field("Trecho", "#{TRECHO_ESPERADO}")),
                (
                    "TRECHO",
                    previous_value(previous, &evidence.name, PERSIST_TRECHO_LABEL)
                        .unwrap_or_else(|| "#{TRECHO}".to_string()),
                ),
                ("FALHA", field("Falha", "#{FALHA}")),
                ("ARTEFATO", field("Artefato", "não se aplica.")),
                ("REVISAO", review),
            ]);
            guide.push_str(&render_known(entry, &values));
        }
        Ok(guide)
    }

    /// Problemas da persistência que impedem a conclusão; vazio com a persistência desligada.
    fn persist_problems(&self, spec: &str, task: &str, file: &Path) -> Result<Vec<String>> {
        let Some(settings) = self.persist_settings() else {
            return Ok(Vec::new());
        };
        let target = settings.target(&self.project_root()?, spec, task);
        let readme = target.join(&settings.readme);
        let Ok(guide) = fs::read_to_string(&readme) else {
            return Ok(vec![format!(
                "evidências não persistidas: falta {}; execute vostok task persist {spec} {task}",
                readme.display()
            )]);
        };
        let mut problems = Vec::new();
        for evidence in self.evidence_entries(&TaskDoc::at(file).text()?)? {
            if !guide
                .lines()
                .any(|line| line == format!("## {}", evidence.name))
            {
                problems.push(format!(
                    "o guia {} não tem a entrada da evidência {}",
                    settings.readme, evidence.name
                ));
            }
        }
        for (line, text) in template_leftovers(&guide) {
            problems.push(format!(
                "o guia {} tem campo a preencher: linha {line}: {text}",
                settings.readme
            ));
        }
        Ok(problems)
    }

    // --- Migração para o formato de pasta ---

    /// Converte as tarefas `TASK-NN.md` da spec em pastas `TASK-NN/`, com as evidências dentro.
    /// Tudo ou nada: as pastas são montadas à parte e só entram no lugar depois de todas prontas.
    fn migrate(&self, spec: &str) -> Result<()> {
        if !is_spec_id(spec) {
            bail!("ID de spec inválido: '{spec}'");
        }
        let dir = self.tasks_root.join(spec);
        if !dir.is_dir() {
            bail!(
                "a spec {spec} não tem tarefas em {}; só specs em andamento são migradas",
                dir.display()
            );
        }
        let files = task_entries(&dir)?;
        if files.iter().all(|file| is_task_folder(file)) {
            println!("A spec {spec} já está no formato de pasta; nada mudou.");
            return Ok(());
        }
        for file in &files {
            let stored = TaskDoc::at(file)
                .get(&self.spec_cfg("content_hash_field")?)
                .unwrap_or_default();
            if !matches!(stored.as_str(), "" | "null" | "-") && stored != self.task_hash(file)? {
                bail!(
                    "a tarefa {} tem edição manual sem registro; registre o ajuste (skill adjust-task) antes de migrar; nada foi migrado",
                    task_id_of(file)
                );
            }
        }
        let staging = dir.join(MIGRATION_DIR);
        if staging.exists() {
            fs::remove_dir_all(&staging)?;
        }
        let staged = files
            .iter()
            .map(|file| self.stage_folder(file, &staging))
            .collect::<Result<Vec<PathBuf>>>();
        let staged = match staged {
            Ok(staged) => staged,
            Err(error) => {
                let _ = fs::remove_dir_all(&staging);
                return Err(error.context("nada foi migrado"));
            }
        };
        let evidence_root = dir.join(self.cfg("evidence_dir")?);
        for (file, folder) in files.iter().zip(&staged) {
            let task = task_id_of(file);
            let evidence = evidence_root.join(&task);
            if evidence.is_dir() {
                fs::rename(&evidence, folder.join(&layout().evidence_dir))?;
            }
            fs::remove_file(file)?;
            fs::rename(folder, dir.join(&task))?;
        }
        fs::remove_dir(&staging)?;
        if evidence_root.is_dir() && fs::read_dir(&evidence_root)?.next().is_none() {
            fs::remove_dir(&evidence_root)?;
        }
        println!(
            "Spec {spec} migrada para o formato de pasta: {} tarefa(s).",
            files.len()
        );
        Ok(())
    }

    /// Monta a pasta da tarefa em `staging`: o front-matter vira o estado, cada trecho vai para o
    /// seu arquivo e o card dá lugar ao `README.md`. A aprovação que valia continua valendo.
    fn stage_folder(&self, file: &Path, staging: &Path) -> Result<PathBuf> {
        let layout = layout();
        let task = task_id_of(file);
        let content = read(file)?;
        let mut state: Vec<String> = frontmatter_lines(&content)
            .ok_or_else(|| anyhow!("a tarefa {task} não tem front-matter: {}", file.display()))?
            .into_iter()
            .map(str::to_string)
            .collect();
        let approved_field = self.spec_cfg("approved_hash_field")?;
        let approved = TaskDoc::at(file).get(&approved_field).unwrap_or_default();
        let hash_field = self.spec_cfg("content_hash_field")?;
        let never_hashed = matches!(
            TaskDoc::at(file)
                .get(&hash_field)
                .unwrap_or_default()
                .as_str(),
            "" | "null" | "-"
        );
        let still_approved = approved == self.content_hash(&content)?;
        let description = content
            .lines()
            .find_map(|line| line.trim().strip_prefix(CARD_DESCRIPTION))
            .unwrap_or_default();
        upsert_line(
            &mut state,
            "description",
            &format!("\"{}\"", yaml_escape(description.trim())),
        );
        let folder = staging.join(&task);
        fs::create_dir_all(&folder)
            .with_context(|| format!("não foi possível criar {}", folder.display()))?;
        write(
            &folder.join(&layout.state),
            &format!("{}\n", state.join("\n")),
        )?;
        let parts = split_task_text(&content);
        for name in [
            &layout.execution,
            &layout.verification,
            &layout.criteria,
            &layout.executive,
            &layout.technical,
            &layout.record,
        ] {
            let text = parts
                .iter()
                .find_map(|(part, text)| (part == name).then_some(text.as_str()))
                .unwrap_or_default();
            write(
                &folder.join(name),
                &format!("{}\n", text.trim_matches('\n')),
            )?;
        }
        self.record_step(&folder, "migração", "pasta", "formato antigo convertido")?;
        let doc = TaskDoc::at(&folder);
        if still_approved {
            doc.put(&approved_field, &self.task_hash(&folder)?)?;
        }
        // Sem hash no formato antigo, a tarefa segue à espera do registro da edição manual.
        if never_hashed {
            doc.put(&hash_field, "null")?;
            for (key, _) in layout.hashed_files() {
                doc.put(key, "null")?;
            }
        }
        Ok(folder)
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

    /// Hash de aprovação e de edição manual: na pasta, só os arquivos de execução (o executivo e o
    /// técnico ficam fora); no arquivo único, o conteúdo sem o registro de execução.
    fn task_hash(&self, file: &Path) -> Result<String> {
        let doc = TaskDoc::at(file);
        if doc.folder() {
            doc.check()?;
            return doc.execution_hash();
        }
        self.content_hash(&doc.text()?)
    }

    fn hash_is_current(&self, file: &Path) -> Result<bool> {
        let doc = TaskDoc::at(file);
        let stored = doc
            .get(&self.spec_cfg("content_hash_field")?)
            .unwrap_or_default();
        Ok(stored == self.task_hash(file)? && !(doc.folder() && doc.state_edited()?))
    }

    fn store_hash(&self, file: &Path) -> Result<()> {
        let doc = TaskDoc::at(file);
        let field = self.spec_cfg("content_hash_field")?;
        if doc.folder() {
            return doc.store_hashes(&field);
        }
        doc.put(&field, &self.task_hash(file)?)
    }

    /// Acrescenta uma linha ao "Registro de execução" e grava o hash do conteúdo.
    fn record_step(&self, file: &Path, step: &str, result: &str, detail: &str) -> Result<()> {
        self.append_record_row(file, step, result, detail)?;
        self.store_hash(file)
    }

    /// Acrescenta uma linha ao "Registro de execução" sem gravar o hash, para uma edição feita durante
    /// a espera continuar aparecendo.
    fn append_record_row(&self, file: &Path, step: &str, result: &str, detail: &str) -> Result<()> {
        let section = self.cfg("record_section")?;
        let content = TaskDoc::at(file).text()?;
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
        TaskDoc::at(file).write_text(&updated)
    }

    /// Registra um ajuste: linha no histórico, contagem, datas e, por último, o hash. Devolve o número.
    fn record_adjustment(&self, file: &Path, origin: &str, summary: &str) -> Result<u64> {
        let count = number_or_zero(&TaskDoc::at(file).get("adjustments").unwrap_or_default()) + 1;
        let now = utc_now();
        let heading = self.history_heading()?;
        let row = format!(
            "| {count} | {} | {origin} | {} |",
            &now[..10],
            escape_pipes(summary)
        );
        let updated = insert_table_row(
            &TaskDoc::at(file).text()?,
            &format!("## {heading}"),
            &row,
            true,
        )
        .ok_or_else(|| anyhow!("seção ausente: '## {heading}'; nenhum ajuste registrado"))?;
        TaskDoc::at(file).write_text(&updated)?;
        TaskDoc::at(file).put("adjustments", &count.to_string())?;
        TaskDoc::at(file).put("adjusted_at", &now)?;
        TaskDoc::at(file).put("updated_at", &now)?;
        self.store_hash(file)?;
        if TaskDoc::at(file).folder() {
            TaskDoc::at(file).refresh_state_hash()?;
        }
        Ok(count)
    }

    /// Resumo de um ajuste manual, com as seções alteradas desde o último commit quando houver.
    fn manual_summary(&self, file: &Path) -> Result<String> {
        let summary = self.spec_cfg("manual_adjustment_summary")?;
        let doc = TaskDoc::at(file);
        let sections = if doc.folder() {
            doc.edited_files()?.join(", ")
        } else {
            changed_sections(file, &self.history_heading()?)?
        };
        Ok(if sections.is_empty() {
            summary
        } else {
            format!("{summary}: {sections}")
        })
    }

    // --- Validação da tarefa ---

    /// Problemas da tarefa, prefixados pelo ID. A cobertura é a da própria tarefa, que não muda aqui.
    fn errors_of(&self, spec: &str, file: &Path, names: EvidenceNames) -> Result<Vec<String>> {
        let upstream = upstream_paths(&self.tasks_root.join(spec), &task_id_of(file), |file| {
            self.created_paths(file)
        })?;
        self.task_errors(file, &upstream, names)
    }

    fn task_errors(
        &self,
        file: &Path,
        upstream: &HashSet<String>,
        names: EvidenceNames,
    ) -> Result<Vec<String>> {
        let id = task_id_of(file);
        let content = TaskDoc::at(file).text()?;
        // Na tarefa em pasta, o card é gerado, e o executivo e o técnico são conferidos nos arquivos.
        let folder = is_task_folder(file);
        let checked = if folder {
            before_section(&content, &self.cfg("description_section")?)
        } else {
            content.clone()
        };
        let mut errors: Vec<String> = template_leftovers(&checked)
            .into_iter()
            .map(|(line, text)| format!("variável sem preencher: {line}:{text}"))
            .collect();
        errors.extend(self.fixed_section_errors(&content)?);
        errors.extend(self.subsection_errors(&content, folder)?);
        if folder {
            let doc = TaskDoc::at(file);
            for name in [&layout().executive, &layout().technical] {
                if !template_leftovers(&read(&doc.part(name))?).is_empty() {
                    errors.push(format!(
                        "o documento {name} de {id} ainda está no template; escreva-o com o que a tarefa vai fazer"
                    ));
                }
            }
        } else {
            errors.extend(self.description_errors(&content)?);
        }
        let running = self.is_running(&status_of(file));
        errors.extend(self.change_errors(&content, upstream, running)?);
        errors.extend(self.done_errors(&content)?);
        let created = if running {
            Vec::new()
        } else {
            self.created_paths(file)?
        };
        let folders = new_folders(&self.project_root()?, &created);
        errors.extend(self.baseline_errors(&content, &created, &folders)?);
        errors.extend(self.evidence_errors(&content, names)?);
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

    /// Subseções obrigatórias; na tarefa em pasta, só as de execução (a descrição é escrita depois).
    fn subsection_errors(&self, content: &str, folder: bool) -> Result<Vec<String>> {
        let mut errors = Vec::new();
        let kinds: &[&str] = if folder {
            &["execution"]
        } else {
            &["execution", "description"]
        };
        for kind in kinds {
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
    /// tarefa de que esta depende conta como existente. Em execução (`running`), `criar` e `remover`
    /// não são conferidos: a própria tarefa já pode ter criado ou removido o caminho.
    fn change_errors(
        &self,
        content: &str,
        upstream: &HashSet<String>,
        running: bool,
    ) -> Result<Vec<String>> {
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
            if running && action != ALTER_ACTION {
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
    fn baseline_errors(
        &self,
        content: &str,
        created: &[String],
        folders: &[String],
    ) -> Result<Vec<String>> {
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
                if expected_change(item).is_some_and(|(_, reason)| reason.is_empty()) {
                    errors.push(format!("'{sub}' com item (muda) sem motivo: {item}"));
                }
                if let Some(command) = baseline_command(item) {
                    errors.extend(delivery_errors(&sub, command, created, folders));
                }
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
    /// ou o texto único de `evidence_none`. No ajuste, o nome com caminho normalizável é aceito.
    fn evidence_errors(&self, content: &str, names: EvidenceNames) -> Result<Vec<String>> {
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
        // Nome final de cada item, com o nome escrito quando ele tem caminho.
        let mut finals: Vec<(&str, Option<&str>)> = Vec::new();
        let kinds = self.evidence_kinds();
        for item in text
            .lines()
            .filter(|line| has_text(line) && !is_indented(line))
        {
            if let Some(name) = new_kind_name(item, &kinds) {
                count += 1;
                finals.push((name, None));
                if !is_valid_evidence_name(name) {
                    errors.push(format!("'{sub}' com nome de evidência inválido: `{name}` (o nome é só o nome do arquivo, sem caminho, sem começar por '.' e diferente de 'baseline' e 'validation'): {item}"));
                }
            } else if is_automatic_evidence(item, &automatic) || is_manual_evidence(item, &manual) {
                count += 1;
                let name = evidence_item(item).map_or("", |(name, _, _)| name);
                match normalized_evidence_name(item) {
                    Some(normalized) if names == EvidenceNames::Normalizable => {
                        finals.push((normalized, Some(name)))
                    }
                    _ => {
                        finals.push((name, None));
                        errors.extend(
                            evidence_name_problem(item)
                                .map(|problem| format!("'{sub}' com {problem}")),
                        );
                    }
                }
            } else {
                errors.push(format!("'{sub}' com item fora do formato: {item} (use: - `arquivo` ({automatic}): `comando`, - `arquivo` ({}), - `arquivo` ({}): `caminho`, - `arquivo` ({}): `comando` ou - `arquivo` ({manual}): descrição)", kinds.from_verify, kinds.diff, kinds.print));
            }
        }
        let block = verify_block(&self.subsection(content, "verify_subsection")?);
        let items = change_items(&self.subsection(content, "change_subsection")?);
        let created = created_of(&items);
        let changed: Vec<String> = items.into_iter().map(|(_, path)| path).collect();
        errors.extend(evidence_rule_errors(
            &kinds,
            &sub,
            &self.cfg("verify_subsection")?,
            &text,
            &block,
            &changed,
            &created,
        ));
        if count == 0 {
            errors.push(format!(
                "'{sub}' sem nenhum item no formato e sem o texto '{none}'"
            ));
        }
        errors.extend(
            repeated_evidence_names(&finals)
                .into_iter()
                .map(|error| format!("'{sub}' com {error}")),
        );
        let fields = self.evidence_fields();
        errors.extend(evidence_subitem_errors(&text, &sub, fields.as_ref(), false));
        if let Some((field, max)) = self.evidence_title() {
            errors.extend(evidence_title_errors(&text, &sub, &field, max));
        }
        Ok(errors)
    }

    /// Tarefa com o nome de evidência com caminho trocado pelo nome do arquivo, e os avisos.
    fn evidence_renames(&self, file: &Path) -> Result<(String, Vec<String>)> {
        Ok(renamed_evidence(
            &TaskDoc::at(file).text()?,
            &self.cfg("execution_section")?,
            &self.cfg("evidence_subsection")?,
            &task_id_of(file),
        ))
    }

    /// Caminhos que a tarefa cria ("Mudar" com a ação `criar`).
    fn created_paths(&self, file: &Path) -> Result<Vec<String>> {
        let content = TaskDoc::at(file).text()?;
        Ok(created_of(&change_items(
            &self.subsection(&content, "change_subsection")?,
        )))
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

/// Pasta onde `vostok task migrate` monta as tarefas antes de pô-las no lugar.
const MIGRATION_DIR: &str = ".migracao";

/// Início da linha do card com a descrição da tarefa.
const CARD_DESCRIPTION: &str = "- **Descrição:** ";

/// Instrução do `vostok task describe`.
const DESCRIBE_GUIDE: &str = "Escreva <EXECUTIVE> para quem não é técnico: o que a tarefa entregou e por que importa, sem código. Escreva <TECHNICAL> para quem mantém o código: o que mudou em cada arquivo, as decisões e como foi verificado. Descreva o que foi feito, a partir do material abaixo, sem copiar os passos.";

/// Partes que `vostok task load --part` aceita.
const TASK_PARTS: [&str; 5] = [
    "execucao",
    "verificacao",
    "criterios",
    "executivo",
    "tecnico",
];

/// Campo do estado com o hash do próprio `estado.yml`, sem esta linha.
const STATE_HASH_KEY: &str = "state_hash";

/// Nomes dos arquivos e das seções da tarefa em pasta, lidos de `config/tasks.yml` na criação de
/// `Tasks`; sem as chaves (projetos antigos), os nomes do contrato.
pub(crate) struct TaskLayout {
    pub(crate) state: String,
    pub(crate) execution: String,
    pub(crate) verification: String,
    pub(crate) criteria: String,
    pub(crate) executive: String,
    pub(crate) technical: String,
    pub(crate) record: String,
    pub(crate) readme: String,
    /// Conteúdo do template do `README.md` da tarefa; vazio sem o template, e então o README não é gerado.
    pub(crate) readme_template: String,
    record_section: String,
    card_heading: String,
    pub(crate) evidence_dir: String,
    pub(crate) execution_section: String,
    pub(crate) description_section: String,
    pub(crate) executive_heading: String,
    pub(crate) technical_heading: String,
}

impl TaskLayout {
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
            readme: get("task_files.readme", "README.md"),
            readme_template: String::new(),
            evidence_dir: get("task_evidence_dir", "evidencias"),
            execution_section: get("execution_section", "Execução"),
            description_section: get("description_section", "Descrição"),
            executive_heading: get("executive_summary_subsection", "Resumo executivo"),
            technical_heading: get("technical_details_subsection", "Detalhes técnicos"),
            record_section: get("record_section", "Registro de execução"),
            card_heading: get("card_subsection", "Card"),
        }
    }

    /// Arquivos de execução, na ordem em que entram no texto da tarefa.
    fn execution_files(&self) -> [&str; 3] {
        [&self.execution, &self.verification, &self.criteria]
    }

    /// Campo do estado com o hash de cada documento no início da tarefa, e o arquivo.
    fn document_files(&self) -> [(&'static str, &str); 2] {
        [
            ("executive_hash", &self.executive),
            ("technical_hash", &self.technical),
        ]
    }

    /// Campo do estado que guarda o hash de cada arquivo de execução.
    fn hashed_files(&self) -> [(&'static str, &str); 3] {
        [
            ("execution_hash", &self.execution),
            ("verification_hash", &self.verification),
            ("criteria_hash", &self.criteria),
        ]
    }
}

static TASK_LAYOUT: std::sync::OnceLock<TaskLayout> = std::sync::OnceLock::new();

/// Fixa os nomes dos arquivos da tarefa e o template do README, lidos da configuração do harness.
pub(crate) fn init_layout(harness: &Harness) {
    let config = harness.config_file("tasks.yml");
    let mut layout = TaskLayout::read(&config);
    layout.readme_template = harness
        .path("templates")
        .ok()
        .map(|templates| templates.join(task_template_dir(&config)))
        .and_then(|dir| fs::read_to_string(dir.join(layout.readme.to_lowercase())).ok())
        .unwrap_or_default();
    let _ = TASK_LAYOUT.set(layout);
}

/// Pasta dos templates da tarefa em pasta, relativa a `paths.templates`.
pub(crate) fn task_template_dir(config: &Path) -> String {
    config_get(config, "task_folder_template_dir")
        .ok()
        .filter(|dir| !dir.is_empty())
        .unwrap_or_else(|| "task".to_string())
}

pub(crate) fn layout() -> &'static TaskLayout {
    TASK_LAYOUT.get_or_init(|| TaskLayout::read(Path::new("")))
}

/// Pasta de tarefa no formato novo: tem o arquivo de estado.
pub(crate) fn is_task_folder(path: &Path) -> bool {
    path.join(&layout().state).is_file()
}

/// Uma tarefa no disco. Os comandos leem e gravam o estado, o texto e o registro de uma tarefa só
/// por aqui. No arquivo único `TASK-NN.md`, o estado é o front-matter. Na pasta `TASK-NN/`, o estado
/// fica em `estado.yml`, o registro e o histórico em `registro.md`, e o texto é montado dos arquivos
/// com os mesmos títulos do arquivo único, para os comandos lerem os dois formatos igual.
pub(crate) struct TaskDoc<'a> {
    file: &'a Path,
}

impl<'a> TaskDoc<'a> {
    pub(crate) fn at(file: &'a Path) -> Self {
        Self { file }
    }

    pub(crate) fn folder(&self) -> bool {
        is_task_folder(self.file)
    }

    pub(crate) fn part(&self, name: &str) -> PathBuf {
        self.file.join(name)
    }

    /// Recusa a pasta sem algum arquivo de execução ou sem o registro.
    pub(crate) fn check(&self) -> Result<()> {
        if !self.folder() {
            return Ok(());
        }
        let layout = layout();
        for name in layout
            .execution_files()
            .into_iter()
            .chain([layout.record.as_str()])
        {
            if !self.part(name).is_file() {
                bail!(
                    "a tarefa {} não tem o arquivo {name}: {}",
                    task_id_of(self.file),
                    self.part(name).display()
                );
            }
        }
        Ok(())
    }

    /// Valor de um campo do estado.
    pub(crate) fn get(&self, key: &str) -> Result<String> {
        if !self.folder() {
            return frontmatter_get(self.file, key);
        }
        let path = self.part(&layout().state);
        let prefix = format!("{key}:");
        read(&path)?
            .lines()
            .find_map(|line| line.strip_prefix(&prefix).map(state_value))
            .ok_or_else(|| anyhow!("Chave '{key}' não encontrada em {}", path.display()))
    }

    /// Grava um campo existente do estado.
    fn set(&self, key: &str, value: &str) -> Result<()> {
        if !self.folder() {
            return frontmatter_set(self.file, key, value);
        }
        self.get(key)?;
        self.put(key, value)
    }

    /// Grava um campo do estado, acrescentando-o se ainda não existir. Com o `state_hash` em dia, ele
    /// acompanha a gravação; com uma edição manual pendente, fica como está para ela aparecer.
    fn put(&self, key: &str, value: &str) -> Result<()> {
        if !self.folder() {
            return frontmatter_put(self.file, key, value);
        }
        let path = self.part(&layout().state);
        let in_sync = self.state_in_sync()?;
        let mut lines = state_lines(&read(&path)?);
        upsert_line(&mut lines, key, value);
        if in_sync {
            let digest = state_digest(&lines);
            upsert_line(&mut lines, STATE_HASH_KEY, &digest);
        }
        write(&path, &format!("{}\n", lines.join("\n")))?;
        self.write_readme()
    }

    /// Grava os hashes da tarefa em pasta: o de aprovação, o de cada arquivo de execução e o do
    /// estado. A edição manual do estado ainda não registrada não é apagada.
    pub(crate) fn store_hashes(&self, content_hash_field: &str) -> Result<()> {
        self.put(content_hash_field, &self.execution_hash()?)?;
        for (key, name) in layout().hashed_files() {
            self.put(key, &self.part_hash(name)?)?;
        }
        if !self.state_edited()? {
            self.refresh_state_hash()?;
        }
        Ok(())
    }

    /// Refaz o `README.md` da tarefa pelo template: estado, links dos arquivos e card, com os
    /// critérios de `criterios.md`. Sem o template ou sem os arquivos de execução, não grava nada.
    fn write_readme(&self) -> Result<()> {
        let layout = layout();
        if layout.readme_template.is_empty() || self.check().is_err() {
            return Ok(());
        }
        let field = |key: &str| {
            let value = self.get(key).unwrap_or_default();
            if matches!(value.as_str(), "" | "null") {
                "-".to_string()
            } else {
                value
            }
        };
        let list = |key: &str| {
            let value = self.get(key).unwrap_or_default().replace(['[', ']'], "");
            if value.trim().is_empty() {
                "-".to_string()
            } else {
                value
            }
        };
        let profile = match field("profile").as_str() {
            "-" => field("effort"),
            profile => profile.to_string(),
        };
        let criteria: Vec<String> = read(&self.part(&layout.criteria))?
            .lines()
            .filter(|line| line.starts_with("- ["))
            .map(|line| format!("  {line}"))
            .collect();
        let criteria = if criteria.is_empty() {
            "  - -".to_string()
        } else {
            criteria.join("\n")
        };
        let values = HashMap::from([
            ("TASK_ID", task_id_of(self.file)),
            ("SPEC_ID", field("spec")),
            ("TASK_TITLE", field("title")),
            ("TASK_STATUS", field("status")),
            ("TASK_WAVE", field("wave")),
            ("TASK_DEPENDS", list("depends_on")),
            ("TASK_BLOCKED_BY", list("depends_on")),
            ("TASK_TYPE", field("type")),
            ("TASK_PROFILE", profile),
            ("TASK_LABELS", list("labels")),
            ("TASK_CARD_DESCRIPTION", field("description")),
            ("TASK_CARD_CRITERIA", criteria),
        ]);
        let text = format!("{}\n", render(&layout.readme_template, &values).trim_end());
        let path = self.part(&layout.readme);
        if read(&path).unwrap_or_default() != text {
            write(&path, &text)?;
        }
        Ok(())
    }

    /// Grava no estado o hash do executivo e do técnico, para o `finish` exigir que mudem. Sem
    /// efeito no arquivo único.
    fn store_document_hashes(&self) -> Result<()> {
        if !self.folder() {
            return Ok(());
        }
        for (key, name) in layout().document_files() {
            self.put(key, &self.part_hash(name)?)?;
        }
        Ok(())
    }

    /// Grava o `state_hash` do estado como está.
    fn refresh_state_hash(&self) -> Result<()> {
        let path = self.part(&layout().state);
        let mut lines = state_lines(&read(&path)?);
        let digest = state_digest(&lines);
        upsert_line(&mut lines, STATE_HASH_KEY, &digest);
        write(&path, &format!("{}\n", lines.join("\n")))
    }

    /// `estado.yml` bate com o `state_hash` gravado; sem ele, nunca foi conferido.
    fn state_in_sync(&self) -> Result<bool> {
        Ok(self
            .stored_state_hash()?
            .is_some_and(|stored| stored == self.current_state_hash()))
    }

    /// `estado.yml` mudou desde o último `state_hash` gravado.
    fn state_edited(&self) -> Result<bool> {
        Ok(self
            .stored_state_hash()?
            .is_some_and(|stored| stored != self.current_state_hash()))
    }

    fn stored_state_hash(&self) -> Result<Option<String>> {
        Ok(self
            .get(STATE_HASH_KEY)
            .ok()
            .filter(|value| !matches!(value.as_str(), "" | "null" | "-")))
    }

    fn current_state_hash(&self) -> String {
        let text = read(&self.part(&layout().state)).unwrap_or_default();
        state_digest(&state_lines(&text))
    }

    /// Hash dos arquivos de execução juntos, na ordem; é o hash de aprovação e de edição manual.
    fn execution_hash(&self) -> Result<String> {
        let mut bytes = Vec::new();
        for name in layout().execution_files() {
            bytes.extend(self.part_bytes(name)?);
        }
        Ok(sha256_hex(&bytes))
    }

    fn part_hash(&self, name: &str) -> Result<String> {
        Ok(sha256_hex(&self.part_bytes(name)?))
    }

    fn part_bytes(&self, name: &str) -> Result<Vec<u8>> {
        let path = self.part(name);
        fs::read(&path).with_context(|| format!("não foi possível ler {}", path.display()))
    }

    /// Arquivos de execução cujo hash gravado no estado não bate com o conteúdo, e o `estado.yml`
    /// se ele mudou. Arquivo sem hash gravado nunca foi conferido e fica de fora.
    fn edited_files(&self) -> Result<Vec<String>> {
        let mut edited = Vec::new();
        for (key, name) in layout().hashed_files() {
            let stored = self.get(key).unwrap_or_default();
            if !matches!(stored.as_str(), "" | "null" | "-") && stored != self.part_hash(name)? {
                edited.push(name.to_string());
            }
        }
        if self.state_edited()? {
            edited.push(layout().state.clone());
        }
        Ok(edited)
    }

    /// Nome do arquivo de uma parte (`execucao`, `verificacao`, `criterios`, `executivo`, `tecnico`).
    fn part_file(&self, part: &str) -> Option<&'static str> {
        let layout = layout();
        match part {
            "execucao" => Some(&layout.execution),
            "verificacao" => Some(&layout.verification),
            "criterios" => Some(&layout.criteria),
            "executivo" => Some(&layout.executive),
            "tecnico" => Some(&layout.technical),
            _ => None,
        }
        .map(String::as_str)
    }

    /// Texto da tarefa, com as seções que os comandos leem.
    pub(crate) fn text(&self) -> Result<String> {
        if !self.folder() {
            return read(self.file);
        }
        self.check()?;
        let layout = layout();
        let part = |name: &str| -> Result<String> {
            Ok(trim_newlines(&read(&self.part(name))?).to_string())
        };
        let optional =
            |name: &str| trim_newlines(&read(&self.part(name)).unwrap_or_default()).to_string();
        let state = part(&layout.state)?;
        let title = self.get("title").unwrap_or_default();
        Ok(format!(
            "---\n{state}\n---\n\n# {}: {title}\n\n## {}\n\n{}\n\n{}\n\n{}\n\n## {}\n\n### {}\n\n{}\n\n### {}\n\n{}\n\n{}\n",
            task_id_of(self.file),
            layout.execution_section,
            part(&layout.execution)?,
            part(&layout.verification)?,
            part(&layout.criteria)?,
            layout.description_section,
            layout.executive_heading,
            optional(&layout.executive),
            layout.technical_heading,
            optional(&layout.technical),
            part(&layout.record)?,
        ))
    }

    /// Regrava o texto da tarefa (registro, histórico ou nomes de evidência). Na pasta, cada
    /// trecho volta ao seu arquivo, e só os arquivos que mudaram são gravados.
    pub(crate) fn write_text(&self, content: &str) -> Result<()> {
        if !self.folder() {
            return write(self.file, content);
        }
        let layout = layout();
        for (name, text) in split_task_text(content) {
            // Os comandos nunca escrevem o executivo nem o técnico: regravá-los só poderia estragá-los.
            if name == layout.executive || name == layout.technical {
                continue;
            }
            let path = self.part(&name);
            let current = read(&path).unwrap_or_default();
            let text = text.trim_matches('\n');
            if current.trim_matches('\n') != text {
                write(&path, &format!("{text}\n"))?;
            }
        }
        Ok(())
    }
}

fn state_lines(text: &str) -> Vec<String> {
    text.lines().map(str::to_string).collect()
}

/// Troca o valor da chave, ou a acrescenta no fim se ainda não existir.
fn upsert_line(lines: &mut Vec<String>, key: &str, value: &str) {
    let prefix = format!("{key}:");
    let line = format!("{key}: {value}");
    match lines.iter_mut().find(|line| line.starts_with(&prefix)) {
        Some(found) => *found = line,
        None => lines.push(line),
    }
}

/// Hash do estado sem a linha do próprio hash.
fn state_digest(lines: &[String]) -> String {
    let prefix = format!("{STATE_HASH_KEY}:");
    let body: String = lines
        .iter()
        .filter(|line| !line.starts_with(&prefix))
        .map(|line| format!("{line}\n"))
        .collect();
    sha256_hex(body.as_bytes())
}

/// Valor de uma linha `chave: valor` do estado, sem aspas, como no front-matter.
fn state_value(rest: &str) -> String {
    let value = rest.trim();
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        return value[1..value.len() - 1]
            .replace("\\\"", "\"")
            .replace("\\\\", "\\");
    }
    value.trim_matches('\'').to_string()
}

/// Divide o texto montado por `TaskDoc::text` de volta nos arquivos da pasta, pelos títulos.
pub(crate) fn split_task_text(content: &str) -> Vec<(String, String)> {
    let layout = layout();
    let file_of = |heading: &str| -> Option<&str> {
        match heading {
            "Ler" | "Mudar" | "Passos" => Some(&layout.execution),
            "Baseline" | "Verificação" | "Evidências" => Some(&layout.verification),
            "Concluída quando" => Some(&layout.criteria),
            _ if heading == layout.executive_heading => Some(&layout.executive),
            _ if heading == layout.technical_heading => Some(&layout.technical),
            _ => None,
        }
    };
    let mut parts: Vec<(String, String)> = Vec::new();
    let mut current: Option<String> = None;
    let mut in_code = false;
    let mut in_front_matter = false;
    for (index, line) in content.lines().enumerate() {
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
        let in_document = current
            .as_deref()
            .is_some_and(|name| name == layout.executive || name == layout.technical);
        // O executivo e o técnico são texto livre: só o título do registro, o do card (arquivo
        // único) ou, no executivo, o do técnico encerram o documento; os demais títulos ficam nele.
        let subsection = line.strip_prefix("### ").map(str::trim);
        let ends_document = line
            .strip_prefix("## ")
            .is_some_and(|heading| heading.trim() == layout.record_section)
            || subsection == Some(layout.card_heading.as_str())
            || (current.as_deref() == Some(layout.executive.as_str())
                && subsection == Some(layout.technical_heading.as_str()));
        if !in_code && !(in_document && !ends_document) {
            if let Some(heading) = line.strip_prefix("### ") {
                current = file_of(heading.trim()).map(str::to_string);
                // Executivo e técnico guardam só o texto, sem o título.
                if current
                    .as_deref()
                    .is_some_and(|name| name == layout.executive || name == layout.technical)
                {
                    parts.push((current.clone().unwrap_or_default(), String::new()));
                    continue;
                }
            } else if let Some(heading) = line.strip_prefix("## ") {
                // Depois da execução e da descrição, tudo é registro e histórico.
                let heading = heading.trim();
                current = (heading != layout.execution_section
                    && heading != layout.description_section)
                    .then(|| layout.record.clone());
            } else if line.starts_with("# ") {
                current = None;
            }
        }
        let Some(name) = &current else { continue };
        match parts.iter_mut().rev().find(|(part, _)| part == name) {
            Some((_, text)) => {
                text.push_str(line);
                text.push('\n');
            }
            None => parts.push((name.clone(), format!("{line}\n"))),
        }
    }
    parts
}

/// Texto antes da seção `## <title>`; o texto inteiro se ela não existir.
pub(crate) fn before_section(content: &str, title: &str) -> String {
    let heading = format!("\n## {title}\n");
    content.find(&heading).map_or_else(
        || content.to_string(),
        |index| content[..index + 1].to_string(),
    )
}

/// Tarefas de uma spec afetadas por uma reaprovação, por tratamento.
#[derive(Default)]
pub(crate) struct ChangedTasks {
    pub(crate) redo: Vec<String>,
    pub(crate) adjusted: Vec<String>,
    pub(crate) done: Vec<String>,
}

/// Trata as tarefas que cobrem os IDs mudados: `pending` e `approved` voltam a detalhar, as em
/// execução ganham um ajuste e as `done` só são listadas.
pub(crate) fn apply_changed_ids(
    harness: &Harness,
    spec: &str,
    changed: &[String],
) -> Result<ChangedTasks> {
    let tasks = Tasks::new(harness.clone())?;
    let dir = tasks.tasks_root.join(spec);
    let mut result = ChangedTasks::default();
    if !dir.is_dir() {
        return Ok(result);
    }
    let prefix = format!("{}-", tasks.cfg("id.prefix")?);
    for file in task_entries(&dir)? {
        let id = task_id_of(&file);
        if !id.starts_with(&prefix) {
            continue;
        }
        let covered: Vec<String> =
            inline_list(&TaskDoc::at(&file).get("covers").unwrap_or_default())
                .into_iter()
                .filter(|covered_id| changed.contains(covered_id))
                .collect();
        if covered.is_empty() {
            continue;
        }
        let status = status_of(&file);
        if status == tasks.status.done {
            result.done.push(id);
        } else if status == tasks.status.pending {
            result.redo.push(id);
        } else if status == tasks.status.approved {
            tasks.drop_approval(&file, &id)?;
            result.redo.push(id);
        } else if tasks.is_running(&status) {
            let origin = tasks.spec_cfg("adjustment_origins.agent")?;
            let summary = format!("spec {spec} ajustada: {}", covered.join(", "));
            tasks.record_adjustment(&file, &origin, &summary)?;
            tasks.restart_execution(&file, &id, &summary)?;
            result.adjusted.push(id);
        }
    }
    Ok(result)
}

/// Tarefas de uma pasta de spec, em ordem alfabética: arquivos `TASK-NN.md` ou pastas `TASK-NN/`.
/// Recusa a spec com os dois formatos.
pub(crate) fn task_entries(dir: &Path) -> Result<Vec<PathBuf>> {
    let files = markdown_files(dir)?;
    let mut folders: Vec<PathBuf> = fs::read_dir(dir)
        .with_context(|| format!("não foi possível ler {}", dir.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| is_task_folder(path))
        .collect();
    if !files.is_empty() && !folders.is_empty() {
        let spec = dir.file_name().unwrap_or_default().to_string_lossy();
        bail!("a spec {spec} tem tarefas nos dois formatos; execute: vostok task migrate {spec}");
    }
    if folders.is_empty() {
        return Ok(files);
    }
    folders.sort();
    Ok(folders)
}

/// Campo do estado com os arquivos das evidências recusadas que esperam nova geração (RF-09).
const REJECTED_FIELD: &str = "rejected_evidences";

/// Evidências recusadas pendentes; vazio nas tarefas sem o campo.
fn rejected_of(file: &Path) -> Vec<String> {
    id_list(file, REJECTED_FIELD)
}

/// Grava as recusas pendentes, acrescentando o campo nas tarefas antigas.
fn write_rejected(file: &Path, names: &[String]) -> Result<()> {
    TaskDoc::at(file).put(REJECTED_FIELD, &format!("[{}]", names.join(", ")))
}

fn status_of(file: &Path) -> String {
    TaskDoc::at(file).get("status").unwrap_or_default()
}

/// Lista de IDs de um campo do estado (`chave: [TASK-01, TASK-02]`).
fn id_list(file: &Path, key: &str) -> Vec<String> {
    TaskDoc::at(file)
        .get(key)
        .unwrap_or_default()
        .replace(['[', ']'], "")
        .split(',')
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(str::to_string)
        .collect()
}

/// Dependências do front-matter (`depends_on: [TASK-01, TASK-02]`).
fn dependencies_of(file: &Path) -> Vec<String> {
    id_list(file, "depends_on")
}

/// Tarefas que precisam estar concluídas antes desta (`closes_after: [TASK-02]`).
fn closes_after_of(file: &Path) -> Vec<String> {
    id_list(file, "closes_after")
}

/// Dependências de que a tarefa bloqueada espera.
fn waiting_of(file: &Path) -> Vec<String> {
    id_list(file, WAITING_ON_FIELD)
}

fn write_dependencies(file: &Path, dependencies: &[String]) -> Result<()> {
    let doc = TaskDoc::at(file);
    doc.put("depends_on", &format!("[{}]", dependencies.join(", ")))?;
    if doc.folder() {
        return Ok(());
    }
    let blocked_by = if dependencies.is_empty() {
        "-".to_string()
    } else {
        dependencies.join(", ")
    };
    let content = read(file)?;
    let updated: Vec<String> = content
        .split('\n')
        .map(|line| match line.starts_with(BLOCKED_BY_PREFIX) {
            true => format!("{BLOCKED_BY_PREFIX}{blocked_by}"),
            false => line.to_string(),
        })
        .collect();
    write(file, &updated.join("\n"))
}

fn write_waiting(file: &Path, waiting: &[String], reason: &str) -> Result<()> {
    TaskDoc::at(file).put(WAITING_ON_FIELD, &format!("[{}]", waiting.join(", ")))?;
    TaskDoc::at(file).put(
        WAITING_REASON_FIELD,
        &format!("\"{}\"", yaml_escape(reason)),
    )
}

fn refusal_message(task: &str, problems: &[(&str, String, String)]) -> String {
    let mut message = format!(
        "a tarefa {task} não pode ser concluída; nada foi alterado. Resolva nesta ordem:\n"
    );
    for (index, (name, detail, next)) in problems.iter().enumerate() {
        let (first, extra) = match detail.split_once('\n') {
            Some((first, extra)) => (first, format!("{extra}\n")),
            None => (detail.as_str(), String::new()),
        };
        message.push_str(&format!(
            "{}. {name}: {first} -> {next}\n{extra}",
            index + 1
        ));
    }
    message
}

fn reset_results(file: &Path) -> Result<()> {
    TaskDoc::at(file).set("attempts", "0")?;
    for key in ["tests_result", "regression_result", "evidence_result"] {
        TaskDoc::at(file).set(key, "null")?;
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
    for file in task_entries(dir)? {
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
    let content = TaskDoc::at(file).text()?;
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
    expected_change(line).is_some()
        || line
            .strip_prefix("- ")
            .and_then(quoted_name)
            .is_some_and(|(_, rest)| rest.is_empty())
}

/// Item de "Baseline" com mudança esperada (`` - `comando` (muda): motivo ``): o comando e o
/// motivo, vazio quando falta.
pub(super) fn expected_change(line: &str) -> Option<(&str, &str)> {
    let (command, rest) = line
        .strip_prefix("- `")?
        .split_once(&format!("` {EXPECTED_CHANGE_MARK}"))?;
    if command.is_empty() {
        return None;
    }
    let reason = match rest.strip_prefix(':') {
        Some(reason) => reason.trim(),
        None if rest.trim().is_empty() => "",
        None => return None,
    };
    Some((command, reason))
}

/// Um erro por caminho `criar` citado no comando de "Baseline" e por pasta nova citada: o baseline
/// mede o que a tarefa não pode quebrar, não a entrega.
pub(super) fn delivery_errors(
    sub: &str,
    command: &str,
    created: &[String],
    folders: &[String],
) -> Vec<String> {
    let cited: Vec<&String> = created
        .iter()
        .filter(|path| cites_path(command, path))
        .collect();
    let mut errors: Vec<String> = cited
        .iter()
        .map(|path| {
            format!("'{sub}' com comando que cita '{path}', que a tarefa cria; o baseline mede o que a tarefa não pode quebrar, não a entrega: {command}")
        })
        .collect();
    errors.extend(
        folders
            .iter()
            .filter(|folder| cites_path(command, folder))
            .filter(|folder| !cited.iter().any(|path| path.starts_with(&format!("{folder}/"))))
            .map(|folder| {
                format!("'{sub}' com comando que cita '{folder}', pasta que a tarefa cria; o baseline mede o que a tarefa não pode quebrar, não a entrega: {command}")
            }),
    );
    errors
}

fn is_name_char(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, '_' | '-' | '.' | '/')
}

/// O comando cita o caminho inteiro, sem estar colado a outro nome de arquivo ou pasta.
pub(super) fn cites_path(command: &str, path: &str) -> bool {
    let path = path.trim_end_matches('/');
    if path.is_empty() {
        return false;
    }
    command.match_indices(path).any(|(start, found)| {
        let before = command[..start].chars().next_back();
        let mut after = command[start + found.len()..].chars();
        let ends = match after.next() {
            None => true,
            Some('/') => after.next().is_none_or(|next| !is_name_char(next)),
            Some(next) => !is_name_char(next),
        };
        before.is_none_or(|character| !is_name_char(character)) && ends
    })
}

/// Pastas-pai dos caminhos `criar`, sem repetição e em ordem, de `a/b/c.rs`: `a` e `a/b`.
fn parent_folders(created: &[String]) -> Vec<String> {
    let mut folders: Vec<String> = Vec::new();
    for path in created {
        let parts: Vec<&str> = path.trim_end_matches('/').split('/').collect();
        for end in 1..parts.len() {
            let folder = parts[..end].join("/");
            if !folders.contains(&folder) {
                folders.push(folder);
            }
        }
    }
    folders
}

/// Pastas-pai dos caminhos `criar` que ainda não existem sob a raiz do projeto.
pub(super) fn new_folders(root: &Path, created: &[String]) -> Vec<String> {
    parent_folders(created)
        .into_iter()
        .filter(|folder| !root.join(folder).exists())
        .collect()
}

/// Pastas-pai dos caminhos `criar` sem nenhum arquivo rastreado no `HEAD`; fora do git, nenhuma.
fn untracked_folders(root: &Path, created: &[String]) -> Vec<String> {
    let inside = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .is_ok_and(|output| output.status.success());
    if !inside {
        return Vec::new();
    }
    parent_folders(created)
        .into_iter()
        .filter(|folder| {
            Command::new("git")
                .arg("-C")
                .arg(root)
                .args(["ls-tree", "-r", "--name-only", "HEAD", "--", folder])
                .output()
                .map(|output| !output.status.success() || output.stdout.is_empty())
                .unwrap_or(false)
        })
        .collect()
}

/// Comando de um item de "Baseline" ("- `comando`"); o comando pode ter crases.
fn baseline_command(line: &str) -> Option<&str> {
    if let Some((command, _)) = expected_change(line) {
        return Some(command);
    }
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
/// Nome de evidência válido: só o nome do arquivo, fora das pastas da execução.
fn is_valid_evidence_name(name: &str) -> bool {
    !name.contains('/') && !name.starts_with('.') && name != "baseline" && name != "validation"
}

/// Problema do nome de um item de "Evidências", pelo mesmo critério do `vostok task evidence`.
pub(crate) fn evidence_name_problem(item: &str) -> Option<String> {
    let (name, _, _) = evidence_item(item)?;
    (!is_valid_evidence_name(name)).then(|| {
        format!("nome de evidência inválido: `{name}` (o nome é só o nome do arquivo, sem caminho, sem começar por '.' e diferente de 'baseline' e 'validation'): {item}")
    })
}

/// Nome normalizado de um item com caminho: o texto depois da última `/`, se for válido
/// (caminho terminado em `/` não tem nome de arquivo).
pub(crate) fn normalized_evidence_name(item: &str) -> Option<&str> {
    let (name, _, _) = evidence_item(item)?;
    let (_, file) = name.rsplit_once('/')?;
    (!file.is_empty() && is_valid_evidence_name(file)).then_some(file)
}

/// Nomes repetidos depois de tirar o caminho, dados os pares (nome final, nome escrito com
/// caminho). Dois itens com o mesmo nome gravariam o mesmo arquivo: não há troca segura. Só conta
/// quando ao menos um deles tinha caminho.
pub(crate) fn repeated_evidence_names(finals: &[(&str, Option<&str>)]) -> Vec<String> {
    let mut errors = Vec::new();
    let mut reported = HashSet::new();
    for (name, _) in finals {
        let same: Vec<_> = finals.iter().filter(|(other, _)| other == name).collect();
        if same.len() > 1
            && same.iter().any(|(_, written)| written.is_some())
            && reported.insert(*name)
        {
            let items: Vec<String> = same
                .iter()
                .map(|(final_name, written)| format!("`{}`", written.unwrap_or(final_name)))
                .collect();
            errors.push(format!(
                "nome de evidência repetido depois de tirar o caminho: `{name}` ({})",
                items.join(", ")
            ));
        }
    }
    errors
}

/// Conteúdo com o nome de evidência com caminho trocado pelo nome do arquivo, e um aviso
/// `TASK-NN: antigo → novo` por troca. Só o nome muda; o resto do item e os subitens ficam.
pub(crate) fn renamed_evidence(
    content: &str,
    execution: &str,
    evidence: &str,
    id: &str,
) -> (String, Vec<String>) {
    let (execution, evidence) = (format!("## {execution}"), format!("### {evidence}"));
    let mut warnings = Vec::new();
    let (mut in_execution, mut in_evidence) = (false, false);
    let mut lines = Vec::new();
    for line in content.split('\n') {
        if line.starts_with("## ") {
            in_execution = line.trim_end() == execution;
            in_evidence = false;
        } else if line.starts_with('#') {
            in_evidence = in_execution && line.trim_end() == evidence;
        }
        let renamed = (in_evidence && !is_indented(line))
            .then(|| Some((evidence_item(line)?.0, normalized_evidence_name(line)?)))
            .flatten();
        match renamed {
            Some((old, new)) => {
                warnings.push(format!("{id}: {old} → {new}"));
                lines.push(line.replacen(&format!("`{old}`"), &format!("`{new}`"), 1));
            }
            None => lines.push(line.to_string()),
        }
    }
    (lines.join("\n"), warnings)
}

/// Aviso das evidências automáticas com a mesma saída do baseline; nada quando não há nenhuma.
fn print_same_as_baseline(same: &[(String, String)]) {
    if same.is_empty() {
        return;
    }
    println!(
        "Aviso: evidência(s) com a mesma saída do baseline, que não comprovam a mudança da tarefa:"
    );
    for (name, command) in same {
        println!("- {name}: `{command}`");
    }
}

/// Avisos das trocas de nome de evidência, depois do cabeçalho; nada quando não há troca.
fn print_renames(header: &str, renames: &[String]) {
    if renames.is_empty() {
        return;
    }
    println!("{header}");
    for rename in renames {
        println!("- {rename}");
    }
}

pub(crate) fn evidence_item(line: &str) -> Option<(&str, &str, &str)> {
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

/// Nome de uma evidência da verificação ("- `arquivo` (tipo)"), sem comando.
fn verify_evidence_name<'a>(line: &'a str, from_verify: &str) -> Option<&'a str> {
    let (name, rest) = quoted_name(line.strip_prefix("- ")?)?;
    let kind = rest.strip_prefix(" (")?.strip_suffix(')')?;
    (kind == from_verify).then_some(name)
}

/// Tipos de evidência e o rótulo da "Verificação" lidos de `config/tasks.yml`.
pub(super) struct EvidenceKinds {
    automatic: String,
    manual: String,
    label: String,
    from_verify: String,
    diff: String,
    print: String,
}

impl EvidenceKinds {
    /// Lê as chaves por `get`; a chave ausente (projeto antigo) fica com o valor padrão.
    pub(super) fn read(get: impl Fn(&str) -> Option<String>) -> Self {
        let value = |key: &str, default: &str| get(key).unwrap_or_else(|| default.to_string());
        Self {
            automatic: value("evidence_automatic", "automática"),
            manual: value("evidence_manual", "manual"),
            label: value("verify_label", DEFAULT_VERIFY_LABEL),
            from_verify: value("evidence_from_verify", DEFAULT_EVIDENCE_FROM_VERIFY),
            diff: value("evidence_diff", DEFAULT_EVIDENCE_DIFF),
            print: value("evidence_print", DEFAULT_EVIDENCE_PRINT),
        }
    }

    /// Os cinco tipos que podem pedir aval.
    fn all(&self) -> Vec<String> {
        [
            &self.automatic,
            &self.from_verify,
            &self.diff,
            &self.manual,
            &self.print,
        ]
        .map(String::clone)
        .to_vec()
    }
}

/// Itens de uma lista na linha (`[a, "b, c", d]`): separa por vírgula fora de aspas duplas.
fn inline_list(value: &str) -> Vec<String> {
    let inner = value.trim().trim_start_matches('[').trim_end_matches(']');
    let mut items = Vec::new();
    let mut item = String::new();
    let mut quoted = false;
    for character in inner.chars() {
        match character {
            '"' => quoted = !quoted,
            ',' if !quoted => items.push(std::mem::take(&mut item)),
            _ => item.push(character),
        }
    }
    items.push(item);
    items
        .into_iter()
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

/// Aviso de um tipo desconhecido em `evidence_review_kinds`.
fn review_kind_warning(kind: &str, known: &[String]) -> String {
    format!(
        "Aviso: evidence_review_kinds: tipo '{kind}' desconhecido, ignorado; tipos válidos: {}",
        known.join(", ")
    )
}

/// Caminhos com a ação `criar` de "Mudar".
pub(super) fn created_of(items: &[(String, String)]) -> Vec<String> {
    items
        .iter()
        .filter(|(action, _)| action == CREATE_ACTION)
        .map(|(_, path)| path.clone())
        .collect()
}

/// Valor do subitem `field` de cada evidência, pelo nome da evidência, sem os espaços das pontas.
fn evidence_subitem_values(text: &str, field: &str, from_verify: &str) -> HashMap<String, String> {
    let mut current: Option<String> = None;
    let mut values = HashMap::new();
    for line in text.lines().filter(|line| has_text(line)) {
        if !is_indented(line) {
            current = verify_evidence_name(line, from_verify)
                .or_else(|| evidence_item(line).map(|(name, _, _)| name))
                .map(str::to_string);
            continue;
        }
        let value = line
            .trim()
            .strip_prefix("- ")
            .and_then(|rest| rest.strip_prefix(field))
            .and_then(|rest| rest.strip_prefix(": "))
            .map(str::trim);
        if let (Some(name), Some(value)) = (&current, value) {
            values.insert(name.clone(), value.to_string());
        }
    }
    values
}

/// `Tempo-limite:` de cada evidência: `Some(segundos)` com um número, `None` com outro texto.
pub(super) fn evidence_timeouts(text: &str, from_verify: &str) -> HashMap<String, Option<u64>> {
    evidence_subitem_values(text, EVIDENCE_TIMEOUT, from_verify)
        .into_iter()
        .map(|(name, value)| (name, value.trim_matches('`').trim().parse().ok()))
        .collect()
}

/// Caminhos de `Requer:` de cada evidência: os trechos entre crases, separados por vírgula.
pub(super) fn evidence_requirements(text: &str, from_verify: &str) -> HashMap<String, Vec<String>> {
    evidence_subitem_values(text, EVIDENCE_REQUIRES, from_verify)
        .into_iter()
        .map(|(name, value)| {
            let paths = value
                .split(',')
                .map(|path| path.trim().trim_matches('`').trim().to_string())
                .filter(|path| !path.is_empty())
                .collect();
            (name, paths)
        })
        .collect()
}

/// Nome de um item dos tipos da SPEC-020: da verificação, `diff` ou `print`.
pub(super) fn new_kind_name<'a>(item: &'a str, kinds: &EvidenceKinds) -> Option<&'a str> {
    verify_evidence_name(item, &kinds.from_verify).or_else(|| {
        (is_diff_evidence(item, &kinds.diff) || is_print_evidence(item, &kinds.print))
            .then(|| evidence_item(item).map(|(name, _, _)| name))
            .flatten()
    })
}

/// Regras das evidências que dependem do resto da tarefa: a leitura de um caminho de "Mudar", o
/// print em tarefa com interface e os rótulos da "Verificação" contra as evidências da verificação.
pub(super) fn evidence_rule_errors(
    kinds: &EvidenceKinds,
    sub: &str,
    verify: &str,
    evidence: &str,
    block: &str,
    changed: &[String],
    created: &[String],
) -> Vec<String> {
    let mut errors = Vec::new();
    let mut requirements: Vec<_> = evidence_requirements(evidence, &kinds.from_verify)
        .into_iter()
        .collect();
    requirements.sort();
    for (name, paths) in requirements {
        for path in paths.iter().filter(|path| !created.contains(path)) {
            errors.push(format!(
                "'{sub}': a evidência {name} requer '{path}', que não é criado pela tarefa"
            ));
        }
    }
    let mut timeouts: Vec<_> = evidence_timeouts(evidence, &kinds.from_verify)
        .into_iter()
        .filter(|(_, seconds)| seconds.is_none())
        .map(|(name, _)| name)
        .collect();
    timeouts.sort();
    for name in timeouts {
        errors.push(format!(
            "'{sub}': a evidência {name} tem {EVIDENCE_TIMEOUT} que não é um número de segundos"
        ));
    }
    let mut has_image = false;
    let mut verify_names = Vec::new();
    for item in evidence
        .lines()
        .filter(|line| has_text(line) && !is_indented(line))
    {
        if let Some(name) = verify_evidence_name(item, &kinds.from_verify) {
            verify_names.push(name);
        } else if is_print_evidence(item, &kinds.print) {
            has_image = true;
        } else if is_manual_evidence(item, &kinds.manual) {
            has_image |= evidence_item(item).is_some_and(|(name, _, _)| {
                IMAGE_EXTENSIONS
                    .iter()
                    .any(|ext| name.to_lowercase().ends_with(ext))
            });
        } else if is_automatic_evidence(item, &kinds.automatic) {
            let Some((name, _, rest)) = evidence_item(item) else {
                continue;
            };
            if let Some(path) = read_of_changed_path(rest.trim_matches('`'), changed) {
                errors.push(format!("'{sub}': a evidência {name} só lê '{path}', que a tarefa muda; use - `{name}.diff` ({}): `{path}` ou uma evidência de comportamento", kinds.diff));
            }
        }
    }
    let interface = changed
        .iter()
        .any(|path| INTERFACE_EXTENSIONS.iter().any(|ext| path.ends_with(ext)));
    if interface && !has_image {
        errors.push(format!("'{sub}': tarefa com interface sem print; inclua - `tela.png` ({}): `comando` ou uma evidência manual de imagem", kinds.print));
    }
    errors.extend(label_errors(kinds, verify, block, &verify_names));
    errors
}

/// Rótulos da "Verificação" conferidos contra as evidências da verificação: cada rótulo tem
/// nome válido, um comando logo abaixo, não se repete e tem a sua evidência, e cada evidência
/// da verificação tem o seu rótulo.
fn label_errors(
    kinds: &EvidenceKinds,
    verify: &str,
    block: &str,
    verify_names: &[&str],
) -> Vec<String> {
    let label = &kinds.label;
    let from_verify = &kinds.from_verify;
    let labels = verify_labels(block, label);
    let mut errors = Vec::new();
    let mut seen = HashSet::new();
    for VerifyLabel {
        name,
        command_lines,
        ..
    } in &labels
    {
        let written = format!("`{label} {name}`");
        if name.is_empty() {
            errors.push(format!(
                "'{verify}' com rótulo `{label}` sem o nome do arquivo"
            ));
            continue;
        }
        if !is_valid_evidence_name(name) {
            errors.push(format!("'{verify}' com rótulo {written} com nome de arquivo inválido (só o nome do arquivo, sem caminho)"));
        }
        if *command_lines == 0 {
            errors.push(format!(
                "'{verify}' com rótulo {written} sem comando na linha seguinte"
            ));
        }
        if !seen.insert(name.as_str()) {
            errors.push(format!("'{verify}' com rótulo {written} repetido"));
        } else if !verify_names.contains(&name.as_str()) {
            errors.push(format!(
                "'{verify}' com rótulo {written} sem a evidência `{name}` ({from_verify})"
            ));
        }
    }
    for name in verify_names {
        if !labels.iter().any(|label| label.name == *name) {
            errors.push(format!(
                "evidência `{name}` ({from_verify}) sem o rótulo `{label} {name}` na '{verify}'"
            ));
        }
    }
    errors
}

/// Comandos da "Verificação" que passam argumentos depois de `--` para um script de `pnpm`, `npm`
/// ou `yarn`, em que o filtro costuma ser ignorado; no `cargo`, o `--` é o certo.
pub(super) fn dashdash_commands(block: &str) -> Vec<String> {
    block
        .lines()
        .map(str::trim)
        .filter(|line| {
            let first = line.split_whitespace().next().unwrap_or("");
            ["pnpm", "npm", "yarn"].contains(&first)
                && line
                    .split_once(" -- ")
                    .is_some_and(|(_, rest)| has_text(rest))
        })
        .map(str::to_string)
        .collect()
}

/// Erros dos títulos de evidência acima do limite; o título ausente não é erro aqui.
pub(super) fn evidence_title_errors(
    text: &str,
    section: &str,
    field: &str,
    max: usize,
) -> Vec<String> {
    let mut errors = Vec::new();
    let mut item = "";
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        if !is_indented(line) {
            item = line;
            continue;
        }
        if let Some((name, value)) = evidence_subitem(line) {
            let count = value.trim().chars().count();
            if name == field && count > max {
                errors.push(format!(
                    "'{section}' com '{field}' de mais de {max} caracteres ({count}): {item}"
                ));
            }
        }
    }
    errors
}

/// Aviso de um comando de `dashdash_commands`.
pub(super) fn dashdash_warning(task: &str, command: &str) -> String {
    format!("Aviso: {task}: o comando '{command}' passa argumentos depois de '--' para o script; confira se o filtro é aplicado")
}

/// Item "- `arquivo` (print): `comando`".
fn is_print_evidence(line: &str, print: &str) -> bool {
    evidence_kind_and_rest(line, print)
        .and_then(quoted_name)
        .is_some_and(|(_, rest)| rest.is_empty())
}

/// Nome de um item de "Evidências", o da verificação (`from_verify`) ou o de qualquer outro tipo.
fn evidence_name<'a>(line: &'a str, from_verify: &str) -> Option<&'a str> {
    verify_evidence_name(line, from_verify).or_else(|| evidence_item(line).map(|(name, _, _)| name))
}

/// ID da evidência pela posição do item em "Evidências": `EV-01`, `EV-02`...
fn evidence_id(index: usize) -> String {
    format!("EV-{:02}", index + 1)
}

/// `EV-NN arquivo` pela posição do item em "Evidências"; só o arquivo se ele saiu da lista.
fn evidence_label(entries: &[EvidenceEntry], name: &str) -> String {
    match entries.iter().position(|entry| entry.name == name) {
        Some(index) => format!("{} {name}", evidence_id(index)),
        None => name.to_string(),
    }
}

/// `EV-` seguido só de dígitos, ao menos um.
fn is_evidence_id(text: &str) -> bool {
    text.strip_prefix("EV-")
        .is_some_and(|digits| !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()))
}

/// Item de lista de evidências: `- EV-NN arquivo (caminho)`, com `: descrição` quando há.
fn evidence_line(index: usize, name: &str, path: &Path, description: &str) -> String {
    let line = format!("- {} {name} ({})", evidence_id(index), path.display());
    if description.is_empty() {
        line
    } else {
        format!("{line}: {description}")
    }
}

/// Linha do registro que marca a conferência de uma evidência pelo usuário; o SHA-256 do arquivo
/// faz a conferência valer só para o arquivo conferido.
fn evidence_review(id: &str, name: &str, hashes: &ReviewHashes) -> String {
    format!(
        "{id} {name} conferida pelo usuário (sha256 {})",
        hashes.current
    )
}

/// Linha do registro de uma evidência rejeitada pelo usuário, com o motivo depois dos dois pontos.
fn evidence_rejection(id: &str, name: &str, hashes: &ReviewHashes, reason: &str) -> String {
    format!(
        "{id} {name} rejeitada pelo usuário (sha256 {}): {reason}",
        hashes.current
    )
}

/// Motivo da rejeição numa linha só, sem espaços sobrando.
fn normalized_reason(reason: &str) -> String {
    reason.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Hashes com que uma linha de conferência ou rejeição vale para a evidência atual.
struct ReviewHashes {
    /// Oito primeiros dígitos do SHA-256 de `review_content`, gravado nas linhas novas (RF-08).
    current: String,
    /// Do arquivo inteiro, gravado pelas versões anteriores e ainda aceito (RNF-03).
    legacy: String,
}

impl ReviewHashes {
    fn of(bytes: &[u8], ignore: &[String]) -> Self {
        Self {
            current: sha256_hex(&review_content(bytes, ignore))[..8].to_string(),
            legacy: sha256_hex(bytes)[..8].to_string(),
        }
    }

    fn matches(&self, hash: &str) -> bool {
        hash == self.current || hash == self.legacy
    }
}

/// Bytes que a conferência compara: o log (`$ comando` e cabeçalho até a linha em branco) sem as
/// linhas `# data: ` e `# commit: ` e sem as linhas da saída com um trecho de `regression_ignore`
/// (tempos e durações), que mudam a cada geração sem mudar o resultado; prints, vídeos e manuais
/// como estão.
fn review_content(bytes: &[u8], ignore: &[String]) -> Vec<u8> {
    let Some(end) = bytes
        .starts_with(b"$ ")
        .then(|| bytes.windows(2).position(|pair| pair == b"\n\n"))
        .flatten()
    else {
        return bytes.to_vec();
    };
    let mut content: Vec<u8> = bytes[..end]
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.starts_with(b"# data: ") && !line.starts_with(b"# commit: "))
        .collect::<Vec<_>>()
        .join(&b'\n');
    content.extend_from_slice(&without_ignored(&bytes[end..], ignore));
    content
}

/// Linha do "Registro de execução" com as colunas que a conferência lê.
struct RecordRow {
    step: String,
    detail: String,
}

/// Conferência (aceita ou rejeitada) de uma evidência, como está no registro.
struct Review {
    name: String,
    accepted: bool,
    hash: String,
}

fn record_row(line: &str) -> Option<RecordRow> {
    if !is_numbered_row(line) {
        return None;
    }
    let cells = record_cells(line);
    let [_, _, step, _, detail, ..] = cells.as_slice() else {
        return None;
    };
    Some(RecordRow {
        step: step.clone(),
        detail: detail.clone(),
    })
}

/// Células de uma linha de tabela, com `\|` lido como `|`.
fn record_cells(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let inner = trimmed.strip_prefix('|').unwrap_or(trimmed);
    let inner = inner.strip_suffix('|').unwrap_or(inner);
    let mut cells = Vec::new();
    let mut cell = String::new();
    for character in inner.chars() {
        if character == '|' && !cell.ends_with('\\') {
            cells.push(cell.trim().replace("\\|", "|"));
            cell.clear();
        } else {
            cell.push(character);
        }
    }
    cells.push(cell.trim().replace("\\|", "|"));
    cells
}

/// Conferência ou rejeição de uma evidência, pelas formas `EV-NN <nome> conferida ...`,
/// `EV-NN <nome> rejeitada ...`, `evidência <nome> conferida ...` e a antiga `print <nome> conferido ...`, que continua lida.
fn review_of(row: &RecordRow) -> Option<Review> {
    let rest = row
        .detail
        .strip_prefix("evidência ")
        .or_else(|| row.detail.strip_prefix("print "))
        .or_else(|| {
            let (id, rest) = row.detail.split_once(' ')?;
            is_evidence_id(id).then_some(rest)
        })?;
    let (name, tail) = rest.split_once(' ')?;
    let (accepted, hash) = match row.step.as_str() {
        "conferência" => (
            true,
            [
                "conferida pelo usuário (sha256 ",
                "conferido pelo usuário (sha256 ",
            ]
            .iter()
            .find_map(|prefix| tail.strip_prefix(prefix))?,
        ),
        "rejeição" => (false, tail.strip_prefix("rejeitada pelo usuário (sha256 ")?),
        _ => return None,
    };
    Some(Review {
        name: name.to_string(),
        accepted,
        hash: hash.split(')').next()?.to_string(),
    })
}

/// Última conferência ou rejeição de `name` no registro, com a linha dela.
fn last_review<'a>(rows: &'a [RecordRow], name: &str) -> Option<(&'a RecordRow, Review)> {
    rows.iter().rev().find_map(|row| {
        review_of(row)
            .filter(|review| review.name == name)
            .map(|review| (row, review))
    })
}

/// Conferência válida de `name`: a última linha de conferência ou rejeição dele é uma aceitação com
/// o hash dos bytes atuais, também depois de ajuste e retomada; a rejeição posterior vence (RN-02).
fn review_is_valid(rows: &[RecordRow], name: &str, hashes: &ReviewHashes) -> bool {
    last_review(rows, name)
        .is_some_and(|(_, review)| review.accepted && hashes.matches(&review.hash))
}

/// Última rejeição de `name` no registro, com a linha dela.
fn last_rejection<'a>(rows: &'a [RecordRow], name: &str) -> Option<(&'a RecordRow, Review)> {
    rows.iter().rev().find_map(|row| {
        review_of(row)
            .filter(|review| review.name == name && !review.accepted)
            .map(|review| (row, review))
    })
}

/// Motivo de uma linha de rejeição, depois de `): `.
fn rejection_reason(row: &RecordRow) -> &str {
    row.detail
        .split_once("): ")
        .map_or("", |(_, reason)| reason)
}

#[cfg(feature = "painel")]
/// Estado de conferência de uma evidência, como o painel o mostra (RF-28).
pub(crate) struct ReviewState {
    pub name: String,
    pub kind: String,
    pub status: &'static str,
    pub reason: String,
}

#[cfg(feature = "painel")]
/// Estado de `name` pela última conferência ou rejeição com o hash dos bytes atuais; sem linha
/// válida fica `a conferir`.
fn review_status(rows: &[RecordRow], name: &str, hashes: &ReviewHashes) -> (&'static str, String) {
    match last_review(rows, name) {
        Some((_, review)) if review.accepted && hashes.matches(&review.hash) => {
            ("conferida", String::new())
        }
        Some((row, review)) if hashes.matches(&review.hash) => {
            ("rejeitada", rejection_reason(row).to_string())
        }
        _ => ("a conferir", String::new()),
    }
}

#[cfg(feature = "painel")]
/// Estado de conferência de cada evidência da tarefa em pasta; erro de leitura devolve lista vazia.
pub(crate) fn evidence_review_states(harness: &Path, task_dir: &Path) -> Vec<ReviewState> {
    review_states(harness, task_dir).unwrap_or_default()
}

#[cfg(feature = "painel")]
fn review_states(harness: &Path, task_dir: &Path) -> Result<Vec<ReviewState>> {
    let tasks = Tasks::new(Harness {
        root: harness.to_path_buf(),
    })?;
    let layout = layout();
    let verification = fs::read_to_string(task_dir.join(&layout.verification))?;
    let record = fs::read_to_string(task_dir.join(&layout.record))?;
    let content = format!(
        "## {}\n\n{}\n\n{}\n",
        tasks.cfg("execution_section")?,
        verification.trim_end(),
        record
    );
    let rows = tasks.record_rows(&content)?;
    let review_kinds = tasks.review_kinds();
    let evidence_dir = task_dir.join(&layout.evidence_dir);
    Ok(tasks
        .evidence_entries(&content)?
        .into_iter()
        .map(|entry| {
            let (status, reason) = if !review_kinds.contains(&entry.kind) {
                ("sem aval", String::new())
            } else {
                match fs::read(evidence_dir.join(&entry.name)) {
                    Ok(bytes) if is_valid_evidence_name(&entry.name) => {
                        review_status(&rows, &entry.name, &tasks.review_hashes(&bytes))
                    }
                    _ => ("a conferir", String::new()),
                }
            };
            ReviewState {
                name: entry.name,
                kind: entry.kind,
                status,
                reason,
            }
        })
        .collect())
}

/// Detalhe da linha `evidências` do registro, com as conferências mantidas e as que mudaram.
fn with_kept(detail: &str, kept: &[String], changed: &[String]) -> String {
    let mut text = detail.to_string();
    if !kept.is_empty() {
        text.push_str(&format!("; conferida(s) mantida(s): {}", kept.join(", ")));
    }
    if !changed.is_empty() {
        text.push_str(&format!("; mudou: {}", changed.join(", ")));
    }
    text
}

/// Linha da geração com as conferências mantidas (hash igual) e as que voltaram a pedir
/// conferência (mudou), se houver alguma (RF-10).
fn print_kept_and_changed(kept: &[String], changed: &[String]) {
    if kept.is_empty() && changed.is_empty() {
        return;
    }
    let listed = |items: &[String]| {
        if items.is_empty() {
            "-".to_string()
        } else {
            items.join(", ")
        }
    };
    println!(
        "Conferências mantidas (hash igual): {}; a conferir de novo (mudou): {}",
        listed(kept),
        listed(changed)
    );
}

/// Item "- `arquivo` (diff): `caminho`".
fn is_diff_evidence(line: &str, diff: &str) -> bool {
    evidence_kind_and_rest(line, diff)
        .and_then(quoted_name)
        .is_some_and(|(_, rest)| rest.is_empty())
}

/// Caminho de "Mudar" que um comando de leitura (`grep`, `sed`, `cat`...) cita, se houver.
fn read_of_changed_path<'a>(command: &str, changed: &'a [String]) -> Option<&'a str> {
    let first = command.split_whitespace().next()?;
    if !READING_COMMANDS.contains(&first) {
        return None;
    }
    changed
        .iter()
        .find(|path| cites_path(command, path))
        .map(String::as_str)
}

/// Cabeçalho do log de uma evidência: o comando, a data, o commit (com a marca de mudanças não
/// commitadas) e o código de saída, seguido de uma linha em branco.
fn log_header(command: &str, code: i32, root: &Path) -> Vec<u8> {
    let commit = git_text(root, &["rev-parse", "--short", "HEAD"]).map_or_else(
        || "-".to_string(),
        |hash| {
            let dirty =
                git_text(root, &["status", "--porcelain"]).is_some_and(|out| !out.is_empty());
            if dirty {
                format!("{hash} (com mudanças não commitadas)")
            } else {
                hash
            }
        },
    );
    format!(
        "$ {command}\n# data: {}\n# commit: {commit}\n# código de saída: {code}\n\n",
        utc_now()
    )
    .into_bytes()
}

/// Saída de um comando `git` sem erro, sem os espaços das pontas.
fn git_text(root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .stdin(Stdio::null())
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Chave do estado com o commit em que o baseline foi coletado.
const BASELINE_COMMIT_KEY: &str = "baseline_commit";

/// Commit do baseline gravado na tarefa, se houver.
fn file_baseline_commit(file: &Path) -> Option<String> {
    TaskDoc::at(file)
        .get(BASELINE_COMMIT_KEY)
        .ok()
        .filter(|sha| !sha.is_empty() && sha != "null" && sha != "-")
}

/// Base do diff de uma evidência: o commit do baseline quando ele existe; senão, o `HEAD`.
struct DiffBase {
    rev: String,
    /// Aviso quando o commit gravado não existe mais.
    note: Option<String>,
    /// A tarefa tem o commit do baseline gravado.
    recorded: bool,
}

impl DiffBase {
    fn of(recorded: Option<String>, root: &Path) -> Self {
        match recorded {
            Some(sha)
                if git_text(root, &["cat-file", "-e", &format!("{sha}^{{commit}}")]).is_some() =>
            {
                Self {
                    rev: sha,
                    note: None,
                    recorded: true,
                }
            }
            Some(sha) => Self {
                rev: "HEAD".to_string(),
                note: Some(format!(
                    "commit do baseline não encontrado ({}); diff contra o HEAD",
                    &sha[..sha.len().min(7)]
                )),
                recorded: true,
            },
            None => Self {
                rev: "HEAD".to_string(),
                note: None,
                recorded: false,
            },
        }
    }
}

/// Resumo do diff de `path` contra a base (o commit do baseline ou o `HEAD`) (a última linha do `git diff --stat`), ou por que
/// não há resumo.
fn path_stat(path: &str, root: &Path, rev: &str) -> String {
    if git_text(root, &["rev-parse", "--is-inside-work-tree"]).is_none() {
        return "fora de um repositório git".to_string();
    }
    if git_text(root, &["ls-files", "--error-unmatch", "--", path]).is_none() {
        return if root.join(path).exists() {
            "novo, ainda não rastreado".to_string()
        } else {
            "não existe".to_string()
        };
    }
    match git_text(root, &["diff", "--stat", rev, "--", path]) {
        Some(stat) if !stat.is_empty() => {
            stat.lines().last().unwrap_or_default().trim().to_string()
        }
        Some(_) => "sem alteração".to_string(),
        None => "sem diff".to_string(),
    }
}

/// Texto de um arquivo da tarefa; no arquivo único, as subseções equivalentes. `None` quando falta.
fn part_text(doc: &TaskDoc, name: &str) -> Result<Option<String>> {
    if doc.folder() {
        return Ok(Some(read(&doc.part(name))?));
    }
    Ok(split_task_text(&doc.text()?)
        .into_iter()
        .find_map(|(file_name, text)| (file_name == name).then_some(text)))
}

/// Diff de `path` contra o `HEAD`, ou do conteúdo inteiro quando o arquivo não é rastreado: o
/// comando usado e a saída, ou o problema que impede o diff.
fn path_diff(
    path: &str,
    root: &Path,
    base: &DiffBase,
) -> Result<std::result::Result<(String, Vec<u8>), String>> {
    if git_text(root, &["rev-parse", "--is-inside-work-tree"]).is_none() {
        return Ok(Err(format!(
            "fora de um repositório git; sem diff de {path}"
        )));
    }
    let tracked = git_text(root, &["ls-files", "--error-unmatch", "--", path]).is_some();
    let command = if tracked {
        format!("git diff {} -- {}", base.rev, shell_quoted(path))
    } else {
        format!("git diff --no-index -- /dev/null {}", shell_quoted(path))
    };
    // Com `--no-index`, o código 1 indica que há diferença.
    let (code, output) = shell_output(&command, root)?;
    if code > 1 || (tracked && code != 0) {
        return Ok(Err(format!(
            "`{command}` terminou com erro (código {code})"
        )));
    }
    if output.is_empty() {
        if !base.recorded {
            return Ok(Err(format!(
                "sem diferença em {path} contra o HEAD; o arquivo pode ter sido commitado depois do baseline, e esta tarefa começou antes de o vostok gravar o commit do baseline: ajuste a evidência para uma automática com git diff <commit-anterior> -- {path}"
            )));
        }
        return Ok(Err(format!("sem diferença em {path}")));
    }
    Ok(Ok((command, output)))
}

/// Rótulo da "Verificação": o nome do arquivo, o índice da linha do rótulo no bloco e quantas
/// linhas tem o comando logo abaixo (zero quando não há comando).
struct VerifyLabel {
    name: String,
    line: usize,
    command_lines: usize,
}

/// Rótulos do bloco: a linha que começa por `label` liga o comando da linha seguinte, que
/// continua enquanto a linha termina em `\`.
fn verify_labels(block: &str, label: &str) -> Vec<VerifyLabel> {
    let lines: Vec<&str> = block.lines().collect();
    lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| {
            let name = line.trim().strip_prefix(label)?.trim();
            Some(VerifyLabel {
                name: name.to_string(),
                line: index,
                command_lines: command_lines_after(&lines, index),
            })
        })
        .collect()
}

/// `(arquivo, comando)` de cada rótulo com comando; um comando em várias linhas não é reaproveitado
/// pela regressão, cujos comandos têm uma linha só, e fica com as linhas unidas por `\n`.
fn labeled_commands(block: &str, label: &str) -> Vec<(String, String)> {
    let lines: Vec<&str> = block.lines().collect();
    verify_labels(block, label)
        .into_iter()
        .filter(|item| item.command_lines > 0 && is_valid_evidence_name(&item.name))
        .map(|item| {
            let command = lines[item.line + 1..=item.line + item.command_lines].join("\n");
            (item.name, command.trim().to_string())
        })
        .collect()
}

/// Linhas do comando que começa logo depois de `index`; zero se a linha seguinte estiver vazia,
/// for comentário ou não existir.
fn command_lines_after(lines: &[&str], index: usize) -> usize {
    let starts = lines
        .get(index + 1)
        .is_some_and(|line| has_text(line) && !line.trim_start().starts_with('#'));
    if !starts {
        return 0;
    }
    let mut count = 0;
    for line in &lines[index + 1..] {
        count += 1;
        if !line.trim_end().ends_with('\\') {
            break;
        }
    }
    count
}

/// Bloco com a saída de cada comando rotulado gravada também em `dir/<arquivo>`. O pipe para o
/// `tee` mantém a ordem da saída e, com o `pipefail`, o código de saída do comando; o trap dentro
/// do grupo registra o comando que falhou, já que fora dele o `$BASH_COMMAND` seria o `tee`.
fn labeled_block(block: &str, labels: &[VerifyLabel], dir: &Path) -> String {
    let mut lines: Vec<String> = block.lines().map(str::to_string).collect();
    for label in labels {
        if label.command_lines == 0 || label.name.is_empty() || !is_valid_evidence_name(&label.name)
        {
            continue;
        }
        let first = label.line + 1;
        let last = label.line + label.command_lines;
        let target = shell_quoted(&dir.join(&label.name).to_string_lossy());
        lines[first] = format!("{{ {VERIFY_TRAP}; {}", lines[first]);
        // O fechamento vai na linha seguinte para valer mesmo com um comentário no fim do comando.
        lines[last] = format!("{}\n}} 2>&1 | tee {target}", lines[last]);
    }
    lines.join("\n")
}

/// Texto entre aspas simples para o shell.
fn shell_quoted(text: &str) -> String {
    format!("'{}'", text.replace('\'', "'\\''"))
}

/// Comandos do primeiro bloco `bash` do texto.
pub(super) fn verify_block(text: &str) -> String {
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
    /// Código de saída aceito; `None` nas linhas antigas, que valem pelo hash.
    pub exit_code: Option<i32>,
}

impl AcceptedRegression {
    /// O aceite cobre a saída atual: pelo código de saída, quando gravado; senão, pelo hash da saída
    /// inteira (como foi gravado) ou da saída sem as linhas ignoradas.
    fn covers(&self, code: i32, output: &[u8], ignore: &[String]) -> bool {
        match self.exit_code {
            Some(accepted) => accepted == code,
            None => {
                self.hash == sha256_hex(output)
                    || self.hash == sha256_hex(&without_ignored(output, ignore))
            }
        }
    }
}

/// Aceites gravados em `dir/accepted`, na ordem do arquivo. O código de saída é o último campo,
/// opcional; numa linha antiga com tabulação na decisão, um último campo não numérico volta à decisão.
pub fn accepted_regressions(dir: &Path) -> Vec<AcceptedRegression> {
    fs::read_to_string(dir.join(ACCEPTED_FILE))
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            let mut fields = line.splitn(6, '\t');
            let number = fields.next()?.parse().ok()?;
            let hash = fields.next()?.to_string();
            let date = fields.next()?.to_string();
            let command = fields.next()?.to_string();
            let mut decision = fields.next()?.to_string();
            let exit_code = match fields.next() {
                None => None,
                Some(last) => match last.parse() {
                    Ok(code) => Some(code),
                    Err(_) => {
                        decision = format!("{decision}\t{last}");
                        None
                    }
                },
            };
            Some(AcceptedRegression {
                number,
                hash,
                date,
                command,
                decision,
                exit_code,
            })
        })
        .collect()
}

/// Aceite por comando; o mais recente de um número vale.
fn latest_acceptances(dir: &Path) -> HashMap<usize, AcceptedRegression> {
    accepted_regressions(dir)
        .into_iter()
        .map(|item| (item.number, item))
        .collect()
}

/// Linhas novas de uma saída que só cresceu: as duas primeiras linhas (`$ comando` e `exit: N`)
/// iguais e as linhas de antes presentes na saída atual, na mesma ordem. `None` quando algo de
/// antes sumiu, mudou ou trocou de ordem.
fn added_lines(expected: &[u8], current: &[u8]) -> Option<Vec<String>> {
    let expected = String::from_utf8_lossy(expected);
    let current = String::from_utf8_lossy(current);
    let before: Vec<&str> = expected.lines().collect();
    let after: Vec<&str> = current.lines().collect();
    if before.len() < 2 || after.len() < 2 || before[..2] != after[..2] {
        return None;
    }
    let mut pending = before[2..].iter().peekable();
    let mut added = Vec::new();
    for line in &after[2..] {
        if pending.peek() == Some(&line) {
            pending.next();
        } else {
            added.push(line.to_string());
        }
    }
    pending.peek().is_none().then_some(added)
}

/// A linha contém algum texto de alerta, sem diferenciar maiúsculas.
fn has_alert(line: &str, alerts: &[String]) -> bool {
    let line = line.to_lowercase();
    alerts
        .iter()
        .any(|alert| line.contains(&alert.to_lowercase()))
}

/// Saída sem as linhas que contêm algum dos trechos; sem trechos, a saída inteira.
fn without_ignored(output: &[u8], ignore: &[String]) -> Vec<u8> {
    if ignore.is_empty() {
        return output.to_vec();
    }
    output
        .split_inclusive(|byte| *byte == b'\n')
        .filter(|line| {
            let text = String::from_utf8_lossy(line);
            !ignore.iter().any(|snippet| text.contains(snippet.as_str()))
        })
        .flatten()
        .copied()
        .collect()
}

/// Código de saída gravado por `recorded_run` na segunda linha da saída (`exit: N`).
fn recorded_exit_code(output: &[u8]) -> Option<i32> {
    String::from_utf8_lossy(output)
        .lines()
        .nth(1)
        .and_then(|line| line.strip_prefix("exit: "))
        .and_then(|code| code.trim().parse().ok())
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
        let code = recorded_exit_code(&output).ok_or_else(|| {
            anyhow!(
                "a saída gravada do comando {number} não tem a linha 'exit: N'. Nada foi alterado"
            )
        })?;
        acceptances.push(Acceptance {
            number,
            line: format!(
                "{number}\t{}\t{}\t{}\t{}\t{code}\n",
                sha256_hex(&output),
                utc_now(),
                command.replace('\t', " "),
                decision.replace('\t', " ")
            ),
        });
    }
    Ok(acceptances)
}

/// Executa um comando com `bash -c` a partir de `dir` e devolve o código de saída e a saída, com
/// stdout e stderr juntos na ordem em que foram escritos.
fn shell_output(command: &str, dir: &Path) -> Result<(i32, Vec<u8>)> {
    shell_output_with(command, dir, &[])
}

/// Como `shell_output`, com variáveis de ambiente a mais.
fn shell_output_with(command: &str, dir: &Path, vars: &[(&str, &Path)]) -> Result<(i32, Vec<u8>)> {
    let (mut reader, writer) = std::io::pipe()?;
    let mut child = Command::new("bash")
        .arg("-c")
        .arg(command)
        .current_dir(dir)
        .envs(vars.iter().map(|(key, value)| (*key, *value)))
        .stdin(Stdio::null())
        .stdout(writer.try_clone()?)
        .stderr(writer)
        .spawn()
        .context("não foi possível executar o bash")?;
    let mut output = Vec::new();
    reader.read_to_end(&mut output)?;
    Ok((exit_code(child.wait()?), output))
}

/// Como `recorded_run`, interrompendo o comando, com os processos que ele abriu, depois de
/// `seconds` segundos (0 sem limite): a saída até ali, seguida de `interrompida após N s`.
fn recorded_run_with_timeout(command: &str, dir: &Path, seconds: u64) -> Result<Vec<u8>> {
    if seconds == 0 {
        return Ok(recorded_run(command, dir)?.1);
    }
    let (mut reader, writer) = std::io::pipe()?;
    let mut builder = Command::new("bash");
    builder
        .arg("-c")
        .arg(command)
        .current_dir(dir)
        .stdin(Stdio::null())
        .stdout(writer.try_clone()?)
        .stderr(writer);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        builder.process_group(0);
    }
    let mut child = builder
        .spawn()
        .context("não foi possível executar o bash")?;
    drop(builder);
    let reading = std::thread::spawn(move || {
        let mut output = Vec::new();
        let _ = reader.read_to_end(&mut output);
        output
    });
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(seconds);
    let mut status = None;
    while std::time::Instant::now() < deadline {
        if let Some(done) = child.try_wait()? {
            status = Some(done);
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    let (code, interrupted) = match status {
        Some(done) => (exit_code(done), false),
        None => {
            stop_process_group(&mut child);
            child.wait()?;
            (TIMEOUT_EXIT, true)
        }
    };
    let mut output = reading.join().unwrap_or_default();
    if interrupted {
        if !output.is_empty() && !output.ends_with(b"\n") {
            output.push(b'\n');
        }
        output.extend_from_slice(format!("interrompida após {seconds} s\n").as_bytes());
    }
    let mut record = format!("$ {command}\nexit: {code}\n").into_bytes();
    record.extend(output);
    Ok(record)
}

/// Encerra o grupo de processos do comando: TERM, e KILL no que sobrar.
#[cfg(unix)]
fn stop_process_group(child: &mut std::process::Child) {
    let group = format!("-{}", child.id());
    let signal = |name: &str| {
        let _ = Command::new("kill")
            .args([name, "--", &group])
            .stderr(Stdio::null())
            .status();
    };
    signal("-TERM");
    for _ in 0..20 {
        if matches!(child.try_wait(), Ok(Some(_))) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    signal("-KILL");
}

#[cfg(not(unix))]
fn stop_process_group(child: &mut std::process::Child) {
    let _ = child.kill();
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

struct PersistSettings {
    dir: String,
    readme: String,
    max_mb: f64,
}

impl PersistSettings {
    fn target(&self, root: &Path, spec: &str, task: &str) -> PathBuf {
        root.join(self.dir.replace("<SPEC>", spec).replace("<TASK>", task))
    }
}

struct PersistOptions {
    redact: Vec<String>,
    exclude: Vec<String>,
    accept: Vec<String>,
    reviewed: Vec<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PersistDecision {
    Redact,
    Exclude,
    Accept,
}

enum PersistCopy {
    Text {
        content: String,
        note: Option<&'static str>,
    },
    Binary(Vec<u8>),
    Excluded,
    Large {
        size: u64,
        hash: String,
        origin: String,
    },
}

/// Resultado da geração das evidências de uma tarefa.
struct Generated {
    same: Vec<(String, String)>,
    missing: String,
    failed: String,
    kept: Vec<String>,
    changed: Vec<String>,
}

struct EvidenceEntry {
    name: String,
    kind: String,
    rest: String,
    fields: HashMap<String, String>,
}

/// Decisões do usuário por arquivo; um arquivo desconhecido ou com duas decisões é recusado.
fn persist_decisions<'a>(
    options: &'a PersistOptions,
    files: &[String],
) -> Result<HashMap<&'a str, PersistDecision>> {
    let mut decisions = HashMap::new();
    for (names, decision) in [
        (&options.redact, PersistDecision::Redact),
        (&options.exclude, PersistDecision::Exclude),
        (&options.accept, PersistDecision::Accept),
    ] {
        for name in names {
            if !files.contains(name) {
                bail!("arquivo '{name}' não está entre as evidências da tarefa");
            }
            if decisions.insert(name.as_str(), decision).is_some() {
                bail!("o arquivo '{name}' tem mais de uma decisão");
            }
        }
    }
    Ok(decisions)
}

/// Revisões de binários (`arquivo=agente` ou `arquivo=usuário`).
fn persist_reviews<'a>(
    reviewed: &'a [String],
    files: &[String],
) -> Result<HashMap<&'a str, &'a str>> {
    let mut reviews = HashMap::new();
    for item in reviewed {
        let Some((name, who)) = item.split_once('=') else {
            bail!("revisão fora do formato: '{item}' (use <arquivo>=<agente|usuário>)");
        };
        if !REVIEWERS.contains(&who) {
            bail!("revisão com revisor inválido: '{who}' (use agente ou usuário)");
        }
        if !files.iter().any(|file| file == name) {
            bail!("arquivo '{name}' não está entre as evidências da tarefa");
        }
        reviews.insert(name, who);
    }
    Ok(reviews)
}

/// Arquivos de `dir`, recursivamente, com o caminho relativo a `base`, em ordem.
fn relative_files(base: &Path, dir: &Path) -> Result<Vec<String>> {
    let mut found = Vec::new();
    if !dir.is_dir() {
        return Ok(found);
    }
    let mut pending = vec![dir.to_path_buf()];
    while let Some(current) = pending.pop() {
        for entry in fs::read_dir(&current)?.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if let Ok(relative) = path.strip_prefix(base) {
                found.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    found.sort();
    Ok(found)
}

fn relative_to(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

/// Conteúdo de texto: UTF-8 sem byte nulo.
fn text_content(bytes: &[u8]) -> Option<&str> {
    let text = std::str::from_utf8(bytes).ok()?;
    (!text.contains('\0')).then_some(text)
}

/// Troca `/Users/<usuário>` e `/home/<usuário>` por `~`.
fn home_to_tilde(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    'scan: while !rest.is_empty() {
        for prefix in ["/Users/", "/home/"] {
            if let Some(after) = rest.strip_prefix(prefix) {
                let boundary = out
                    .chars()
                    .last()
                    .is_none_or(|c| !(c.is_alphanumeric() || "_.-/~".contains(c)));
                let user_len = after
                    .find(|c: char| c == '/' || c.is_whitespace() || "\"'`:),;".contains(c))
                    .unwrap_or(after.len());
                if boundary && user_len > 0 {
                    out.push('~');
                    rest = &after[user_len..];
                    continue 'scan;
                }
            }
        }
        let ch = rest.chars().next().unwrap_or_default();
        out.push(ch);
        rest = &rest[ch.len_utf8()..];
    }
    out
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SensitiveKind {
    Secret,
    Ip,
    Email,
    Cpf,
}

impl SensitiveKind {
    fn label(self) -> &'static str {
        match self {
            SensitiveKind::Secret => "segredo",
            SensitiveKind::Ip => "IP",
            SensitiveKind::Email => "e-mail",
            SensitiveKind::Cpf => "CPF",
        }
    }
}

#[derive(Debug)]
struct SensitiveSpan {
    line: usize,
    /// Posição em bytes no texto inteiro.
    start: usize,
    end: usize,
    kind: SensitiveKind,
}

const SECRET_PREFIXES: [&str; 11] = [
    "sk-",
    "sk_live_",
    "ghp_",
    "gho_",
    "ghs_",
    "github_pat_",
    "glpat-",
    "xoxb-",
    "xoxp-",
    "AKIA",
    "AIza",
];
const SECRET_KEYS: [&str; 8] = [
    "password", "passwd", "senha", "secret", "api_key", "apikey", "api-key", "token",
];

/// Trechos com segredo, IP (exceto loopback e 0.0.0.0), e-mail ou CPF.
fn sensitive_spans(text: &str) -> Vec<SensitiveSpan> {
    let mut spans = Vec::new();
    let mut offset = 0;
    for (index, line) in text.split_inclusive('\n').enumerate() {
        let words = words_of(line);
        for (position, &(start, word)) in words.iter().enumerate() {
            let previous = position
                .checked_sub(1)
                .map(|p| words[p].1.to_ascii_lowercase());
            let key_value = word
                .split_once(['=', ':'])
                .filter(|(key, value)| is_secret_key(key) && value.len() >= 6);
            let kind = if is_secret_token(word)
                || previous.as_deref() == Some("bearer") && word.len() >= 8
                || previous
                    .as_deref()
                    .is_some_and(|key| key.strip_suffix([':', '=']).is_some_and(is_secret_key))
                    && word.len() >= 6
            {
                Some((start, word.len(), SensitiveKind::Secret))
            } else if let Some((key, value)) = key_value {
                Some((start + key.len() + 1, value.len(), SensitiveKind::Secret))
            } else if is_email(word) {
                Some((start, word.len(), SensitiveKind::Email))
            } else if is_cpf(word) {
                Some((start, word.len(), SensitiveKind::Cpf))
            } else {
                ip_in(word).map(|(at, len)| (start + at, len, SensitiveKind::Ip))
            };
            if let Some((start, len, kind)) = kind {
                spans.push(SensitiveSpan {
                    line: index + 1,
                    start: offset + start,
                    end: offset + start + len,
                    kind,
                });
            }
        }
        offset += line.len();
    }
    spans
}

/// Palavras da linha, sem pontuação nas pontas, com a posição em bytes.
fn words_of(line: &str) -> Vec<(usize, &str)> {
    let mut words = Vec::new();
    let mut start = None;
    for (index, ch) in line.char_indices().chain([(line.len(), ' ')]) {
        if ch.is_whitespace() {
            if let Some(begin) = start.take() {
                let raw: &str = &line[begin..index];
                let trimmed = raw.trim_start_matches(['"', '\'', '`', '(', '[', '<']);
                let lead = raw.len() - trimmed.len();
                let trimmed =
                    trimmed.trim_end_matches(['"', '\'', '`', ')', ']', '>', ',', ';', '.']);
                if !trimmed.is_empty() {
                    words.push((begin + lead, trimmed));
                }
            }
        } else if start.is_none() {
            start = Some(index);
        }
    }
    words
}

fn is_secret_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    SECRET_KEYS.iter().any(|name| key.ends_with(name))
}

fn is_secret_token(word: &str) -> bool {
    SECRET_PREFIXES.iter().any(|prefix| {
        word.strip_prefix(prefix).is_some_and(|rest| {
            rest.len() >= 16
                && rest
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        })
    })
}

fn is_email(word: &str) -> bool {
    let Some((local, domain)) = word.split_once('@') else {
        return false;
    };
    let valid = |c: char| c.is_ascii_alphanumeric() || "._%+-".contains(c);
    !local.is_empty()
        && local.chars().all(valid)
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && domain
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
}

fn is_cpf(word: &str) -> bool {
    let bytes = word.as_bytes();
    bytes.len() == 14
        && bytes.iter().enumerate().all(|(index, byte)| match index {
            3 | 7 => *byte == b'.',
            11 => *byte == b'-',
            _ => byte.is_ascii_digit(),
        })
}

/// IPv4 na palavra (aceita porta e máscara depois), exceto loopback e 0.0.0.0.
fn ip_in(word: &str) -> Option<(usize, usize)> {
    let candidate = word.split([':', '/']).next()?;
    let octets: Vec<&str> = candidate.split('.').collect();
    let valid = octets.len() == 4
        && octets.iter().all(|octet| {
            !octet.is_empty()
                && octet.len() <= 3
                && octet.chars().all(|c| c.is_ascii_digit())
                && octet.parse::<u16>().is_ok_and(|value| value <= 255)
        });
    let excluded = candidate.starts_with("127.") || candidate == "0.0.0.0";
    (valid && !excluded).then_some((0, candidate.len()))
}

fn redact_spans(text: &str, spans: &[SensitiveSpan]) -> String {
    let mut sorted: Vec<&SensitiveSpan> = spans.iter().collect();
    sorted.sort_by_key(|span| span.start);
    let mut out = String::with_capacity(text.len());
    let mut cursor = 0;
    for span in sorted {
        if span.start < cursor {
            continue;
        }
        out.push_str(&text[cursor..span.start]);
        out.push_str("[REDIGIDO]");
        cursor = span.end;
    }
    out.push_str(&text[cursor..]);
    out
}

fn strip_leading_comment(template: &str) -> &str {
    if let Some(rest) = template.strip_prefix("<!--") {
        if let Some(end) = rest.find("-->") {
            return rest[end + 3..].trim_start_matches('\n');
        }
    }
    template
}

/// Substitui as variáveis conhecidas numa passada só; as desconhecidas ficam no texto.
fn render_known(template: &str, values: &HashMap<&str, String>) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find("#{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        match after.find('}') {
            Some(end) if values.contains_key(&after[..end]) => {
                out.push_str(&values[&after[..end]]);
                rest = &after[end + 1..];
            }
            _ => {
                out.push_str("#{");
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

/// Valor já preenchido de um campo da entrada `## <nome>` do guia anterior.
fn previous_value(guide: &str, name: &str, label: &str) -> Option<String> {
    let heading = format!("## {name}");
    let mut inside = false;
    for line in guide.lines() {
        if line.starts_with("## ") {
            inside = line == heading;
        } else if inside {
            if let Some(value) = line.strip_prefix(label) {
                let filled = !value.trim().is_empty() && template_leftovers(value).is_empty();
                return filled.then(|| value.to_string());
            }
        }
    }
    None
}

fn previous_review_done(guide: &str, name: &str) -> bool {
    previous_value(guide, name, PERSIST_REVIEW_LABEL).is_some()
}

fn human_size(bytes: u64) -> String {
    format!("{:.1} MB", bytes as f64 / 1_048_576.0)
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
    fn wave_status_follows_the_task_statuses() {
        let status = Statuses {
            pending: "pending".into(),
            approved: "approved".into(),
            in_progress: "in_progress".into(),
            executed: "executed".into(),
            validating: "validating".into(),
            blocked: "blocked".into(),
            done: "done".into(),
        };
        let of = |list: &[&str]| {
            let list: Vec<String> = list.iter().map(|s| s.to_string()).collect();
            wave_status(&list, &status)
        };
        assert_eq!(of(&["done", "done"]), "done");
        assert_eq!(of(&["done", "blocked", "pending"]), "blocked");
        assert_eq!(of(&["approved", "in_progress"]), "in_progress");
        assert_eq!(of(&["approved", "validating"]), "in_progress");
        assert_eq!(of(&["approved", "executed"]), "in_progress");
        assert_eq!(of(&["done", "pending"]), "in_progress");
        assert_eq!(of(&["approved", "approved"]), "approved");
        assert_eq!(of(&["approved", "pending"]), "pending");
        assert_eq!(of(&["pending", "pending"]), "pending");
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
        assert_eq!(
            expected_change("- `wc -l` (muda): passa de 5 para 6"),
            Some(("wc -l", "passa de 5 para 6"))
        );
        assert_eq!(expected_change("- `wc -l` (muda)"), Some(("wc -l", "")));
        assert_eq!(expected_change("- `wc -l` (muda) x"), None);
        assert_eq!(baseline_command("- `wc -l` (muda): motivo"), Some("wc -l"));
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
