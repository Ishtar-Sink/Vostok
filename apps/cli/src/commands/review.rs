//! Comandos `review`: roteiro da revisão sem contexto de spec, plano e tarefa (SPEC-067).

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Result};
use clap::Subcommand;

use crate::harness::Harness;
use crate::models;
use crate::support::config_get;

const FLOWS: [&str; 3] = ["spec", "plan", "task"];
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
const TASK_LOOKUP: &str = "3. Procure: caminho de \"Ler\" ou \"Mudar\" que não existe ou que falta, passo ambíguo ou que deixa decisão, comando da \"Verificação\" que não roda ou não prova o requisito, texto do Trecho que não aparece na saída do comando, Baseline que mede a entrega, critério sem comprovação.";

#[derive(Debug, Subcommand)]
pub enum ReviewCommand {
    /// Imprime o roteiro da revisão sem contexto: modelo, persona, o que procurar e os arquivos.
    Brief {
        /// Fluxo revisado: spec, plan ou task.
        flow: String,
        spec_id: String,
        task_id: Option<String>,
    },
}

pub fn run(command: ReviewCommand) -> Result<()> {
    let harness = Harness::from_current_dir()?;
    match command {
        ReviewCommand::Brief {
            flow,
            spec_id,
            task_id,
        } => brief(&harness, &flow, &spec_id, task_id.as_deref()),
    }
}

fn brief(harness: &Harness, flow: &str, spec_id: &str, task_id: Option<&str>) -> Result<()> {
    if !FLOWS.contains(&flow) {
        bail!("fluxo inválido: '{flow}'; use spec, plan ou task");
    }
    if flow == "task" && task_id.is_none() {
        bail!("o fluxo task exige o ID da tarefa: vostok review brief task {spec_id} <TASK_ID>");
    }
    if !enabled(harness, flow)? {
        println!("Revisão desligada para {flow}.");
        return Ok(());
    }
    let files = files_of(harness, flow, spec_id, task_id)?;
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
    let listing: String = files
        .iter()
        .map(|file| format!("- `{}`\n", relative(project_root, file)))
        .collect();
    let target = task_id.map_or(spec_id.to_string(), |task| format!("{spec_id} {task}"));
    println!(
        "# Revisão de {flow} {target}\n\n## Modelo\n\n- Perfil: {}\n- Ferramenta: {}\n- Apelido de reserva: {}\n\n# Persona\n\n{}\n\n# Roteiro\n\n{ROUTE_START}{}\n{ROUTE_END}\n\n# Arquivos\n\n{listing}\n# Resposta\n\n| # | Local | Problema | Correção | Muda o escopo |\n|---|---|---|---|---|\n\nSem achados, responda só: Nenhum achado.",
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

/// Ausente ou `true` liga; `false` desliga; outro valor é recusado.
fn enabled(harness: &Harness, flow: &str) -> Result<bool> {
    let key = format!("review.{flow}");
    match config_get(&harness.config_file("main.yml"), &key) {
        Err(_) => Ok(true),
        Ok(value) => match value.as_str() {
            "" | "true" => Ok(true),
            "false" => Ok(false),
            other => {
                bail!("valor inválido em config/main.yml ({key}): '{other}'; use true ou false")
            }
        },
    }
}

fn files_of(
    harness: &Harness,
    flow: &str,
    spec_id: &str,
    task_id: Option<&str>,
) -> Result<Vec<PathBuf>> {
    match flow {
        "spec" => Ok(vec![spec_file(harness, spec_id)?]),
        "plan" => {
            let plan = existing(
                harness.path("plans")?.join(format!("{spec_id}.md")),
                &format!("o plano da {spec_id}"),
            )?;
            Ok(vec![plan, spec_file(harness, spec_id)?])
        }
        _ => {
            let task = task_id.unwrap_or_default();
            let mut files = task_files(harness, spec_id, task)?;
            files.push(spec_file(harness, spec_id)?);
            Ok(files)
        }
    }
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
