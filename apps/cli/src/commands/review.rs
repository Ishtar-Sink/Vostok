//! Comandos `review`: roteiro da revisão sem contexto de spec, plano e tarefa (SPEC-067).

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Result};
use clap::Subcommand;

use crate::harness::Harness;
use crate::models;
use crate::support::config_get;

const FLOWS: [&str; 3] = ["spec", "plan", "task"];
const MAX_ROUNDS: u32 = 5;
const DEFAULT_ROUNDS: u32 = 1;
const TASK_PARTS: [&str; 4] = [
    "execucao.md",
    "verificacao.md",
    "criterios.md",
    "tecnico.md",
];

const ROUTE_START: &str = "1. Leia os arquivos abaixo e o código que eles citam, por busca e trecho num arquivo com mais de 400 linhas.
2. Não edite nenhum arquivo.
";
const ROUTE_END: &str =
    "Aponte só o que causaria retrabalho na execução; estilo e preferência ficam de fora.";

const SPEC_LOOKUP: &str = "3. Procure: requisito com duas leituras, caso de teste que não prova o requisito, componente afetado com caminho errado ou faltando, critério de aceite que não se verifica, seções que se contradizem.";
const PLAN_LOOKUP: &str = "3. Procure: tarefa sem objetivo único, dependência que falta ou sobra, ID da spec sem tarefa, componente na tarefa errada, duas tarefas da mesma onda mudando o mesmo arquivo.";
const TASK_LOOKUP: &str = "3. Procure: caminho de \"Ler\" ou \"Mudar\" que não existe ou que falta, passo ambíguo ou que deixa decisão, comando da \"Verificação\" que não roda ou não prova o requisito, texto do Trecho que não aparece na saída do comando, Baseline que mede a entrega, critério sem comprovação. Entre tarefas, procure: mesmo arquivo criado por duas tarefas, passo que usa o resultado de outra tarefa sem depender dela, duas tarefas da mesma onda mudando o mesmo arquivo, nome ou contrato diferente para a mesma coisa, ID da spec que nenhuma tarefa cobre.";

const CONTEXT_INTRO: &str = "## Contexto\n\nLeia estas tarefas só para achar inconsistências com as revisadas; todo achado cita uma tarefa revisada.\n\n";
const DEFAULT_TABLE: &str =
    "| # | Local | Problema | Correção | Muda o escopo |\n|---|---|---|---|---|";
const TASK_TABLE: &str = "| # | Tarefa | Local | Problema | Correção | Muda o escopo |\n|---|---|---|---|---|---|\n\nUm achado entre tarefas cita todas as envolvidas na coluna Tarefa.";

#[derive(Debug, Subcommand)]
pub enum ReviewCommand {
    /// Imprime o roteiro da revisão sem contexto: modelo, persona, o que procurar e os arquivos.
    Brief {
        /// Fluxo revisado: spec, plan ou task.
        flow: String,
        spec_id: String,
        /// IDs das tarefas revisadas (só no fluxo task); sem ID, todas.
        task_ids: Vec<String>,
        /// Rodada da revisão, a partir de 1; acima do limite de review_rounds, a revisão termina.
        #[arg(long, value_name = "N", default_value_t = 1)]
        rodada: u32,
    },
}

pub fn run(command: ReviewCommand) -> Result<()> {
    let harness = Harness::from_current_dir()?;
    match command {
        ReviewCommand::Brief {
            flow,
            spec_id,
            task_ids,
            rodada,
        } => brief(&harness, &flow, &spec_id, &task_ids, rodada),
    }
}

fn brief(
    harness: &Harness,
    flow: &str,
    spec_id: &str,
    task_ids: &[String],
    round: u32,
) -> Result<()> {
    if !FLOWS.contains(&flow) {
        bail!("fluxo inválido: '{flow}'; use spec, plan ou task");
    }
    if !enabled(harness, flow)? {
        println!("Revisão desligada para {flow}.");
        return Ok(());
    }
    if round == 0 {
        bail!("rodada inválida: {round}; use um número a partir de 1");
    }
    let limit = rounds(harness, flow)?;
    if round > limit {
        println!("Limite de {limit} rodada(s) de revisão para {flow} atingido.");
        return Ok(());
    }
    let task_ids = unique(task_ids);
    let (files, context) = files_of(harness, flow, spec_id, &task_ids)?;
    let info = models::for_profile(harness, "julgamento")?;
    if info.tool.is_none() {
        eprintln!("{}", models::tool_warning());
    }
    let persona = fs::read_to_string(harness.root.join("personas/revisor.md")).map_err(|_| {
        anyhow!(
            "persona 'revisor' não encontrada em {}",
            harness.root.display()
        )
    })?;
    let project_root = harness.root.parent().unwrap_or(&harness.root);
    let listing_of = |files: &[PathBuf]| -> String {
        files
            .iter()
            .map(|file| format!("- `{}`\n", relative(project_root, file)))
            .collect()
    };
    let mut listing = listing_of(&files);
    if !context.is_empty() {
        listing.push_str(&format!("\n{CONTEXT_INTRO}{}", listing_of(&context)));
    }
    let target = match (flow, task_ids.is_empty()) {
        ("task", true) => format!("{spec_id} (todas as tarefas)"),
        ("task", false) => format!("{spec_id} {}", task_ids.join(" ")),
        _ => spec_id.to_string(),
    };
    let table = if flow == "task" {
        TASK_TABLE
    } else {
        DEFAULT_TABLE
    };
    println!(
        "# Revisão de {flow} {target}\n\n## Modelo\n\n- Perfil: {}\n- Ferramenta: {}\n- Apelido de reserva: {}\n- Rodada: {round} de {limit}\n\n# Persona\n\n{}\n\n# Roteiro\n\n{ROUTE_START}{}\n{ROUTE_END}\n\n# Arquivos\n\n{listing}\n# Resposta\n\n{table}\n\nSem achados, responda só: Nenhum achado.",
        info.profile,
        info.tool.as_deref().unwrap_or("não configurada"),
        info.model,
        persona.trim_end(),
        lookup_of(flow),
    );
    Ok(())
}

fn lookup_of(flow: &str) -> &'static str {
    match flow {
        "spec" => SPEC_LOOKUP,
        "plan" => PLAN_LOOKUP,
        _ => TASK_LOOKUP,
    }
}

/// Ausente ou vazia vale 1; fora de 1 a 5 é recusado.
fn rounds(harness: &Harness, flow: &str) -> Result<u32> {
    let key = format!("review_rounds.{flow}");
    let Ok(value) = config_get(&harness.config_file("main.yml"), &key) else {
        return Ok(DEFAULT_ROUNDS);
    };
    let value = value.trim();
    if value.is_empty() {
        return Ok(DEFAULT_ROUNDS);
    }
    match value.parse::<u32>() {
        Ok(number) if (1..=MAX_ROUNDS).contains(&number) => Ok(number),
        _ => bail!(
            "valor inválido em config/main.yml ({key}): '{value}'; use um número de 1 a {MAX_ROUNDS}"
        ),
    }
}

/// Ausente ou vazia: só a revisão de task fica ligada; true liga; false desliga; outro valor é recusado.
fn enabled(harness: &Harness, flow: &str) -> Result<bool> {
    let key = format!("review.{flow}");
    match config_get(&harness.config_file("main.yml"), &key) {
        Err(_) => Ok(flow == "task"),
        Ok(value) => match value.as_str() {
            "" => Ok(flow == "task"),
            "true" => Ok(true),
            "false" => Ok(false),
            other => {
                bail!("valor inválido em config/main.yml ({key}): '{other}'; use true ou false")
            }
        },
    }
}

fn unique(ids: &[String]) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    for id in ids {
        if !seen.contains(id) {
            seen.push(id.clone());
        }
    }
    seen
}

/// Devolve os arquivos revisados e os de contexto (só no fluxo task com tarefas indicadas).
fn files_of(
    harness: &Harness,
    flow: &str,
    spec_id: &str,
    task_ids: &[String],
) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    match flow {
        "spec" => Ok((vec![spec_file(harness, spec_id)?], Vec::new())),
        "plan" => {
            let plan = existing(
                harness.path("plans")?.join(format!("{spec_id}.md")),
                &format!("o plano da {spec_id}"),
            )?;
            Ok((vec![plan, spec_file(harness, spec_id)?], Vec::new()))
        }
        _ => {
            let all = task_ids_of(harness, spec_id)?;
            let reviewed_ids = if task_ids.is_empty() { &all } else { task_ids };
            let mut reviewed = Vec::new();
            for id in reviewed_ids {
                reviewed.extend(task_files(harness, spec_id, id)?);
            }
            reviewed.push(spec_file(harness, spec_id)?);
            let mut context = Vec::new();
            if !task_ids.is_empty() {
                for id in all.iter().filter(|id| !task_ids.contains(id)) {
                    context.extend(task_files(harness, spec_id, id)?);
                }
            }
            Ok((reviewed, context))
        }
    }
}

fn task_ids_of(harness: &Harness, spec_id: &str) -> Result<Vec<String>> {
    let dir = harness.path("tasks")?.join(spec_id);
    let prefix = config_get(&harness.config_file("tasks.yml"), "id.prefix")
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "TASK".to_string());
    let start = format!("{prefix}-");
    let mut ids: Vec<String> = fs::read_dir(&dir)
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .filter(|name| name.starts_with(&start))
                .map(|name| name.strip_suffix(".md").unwrap_or(&name).to_string())
                .collect()
        })
        .unwrap_or_default();
    if ids.is_empty() {
        bail!("a {spec_id} não tem tarefas: {}", dir.display());
    }
    ids.sort();
    Ok(ids)
}

fn spec_file(harness: &Harness, spec_id: &str) -> Result<PathBuf> {
    let dir = harness.path("specs")?.join("refined");
    let prefix = format!("{spec_id}-");
    let found = fs::read_dir(&dir).ok().and_then(|entries| {
        let mut names: Vec<PathBuf> = entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension().is_some_and(|ext| ext == "md")
                    && path
                        .file_name()
                        .is_some_and(|name| name.to_string_lossy().starts_with(&prefix))
            })
            .collect();
        names.sort();
        names.into_iter().next()
    });
    found.ok_or_else(|| {
        anyhow!(
            "a spec {spec_id} não foi encontrada: {}/{prefix}*.md",
            dir.display()
        )
    })
}

fn task_files(harness: &Harness, spec_id: &str, task_id: &str) -> Result<Vec<PathBuf>> {
    let dir = harness.path("tasks")?.join(spec_id);
    let folder = dir.join(task_id);
    if folder.is_dir() {
        return TASK_PARTS
            .iter()
            .map(|part| existing(folder.join(part), &format!("a tarefa {task_id}")))
            .collect();
    }
    Ok(vec![existing(
        dir.join(format!("{task_id}.md")),
        &format!("a tarefa {task_id}"),
    )?])
}

fn existing(path: PathBuf, what: &str) -> Result<PathBuf> {
    if path.is_file() {
        Ok(path)
    } else {
        bail!("{what} não foi encontrada: {}", path.display())
    }
}

fn relative(root: &Path, file: &Path) -> String {
    file.strip_prefix(root)
        .unwrap_or(file)
        .display()
        .to_string()
}
