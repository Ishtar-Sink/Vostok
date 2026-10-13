//! Comando `init`: instala o harness em `vostok/`, na raiz do projeto, com o roteamento das
//! ferramentas escolhidas (RF-03 a RF-08).

use std::io::{self, BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::Args;

use crate::embedded;
use crate::harness::Harness;
use crate::manifest::{self, Manifest};
use crate::routing::{self, Tool, TOOLS};
use crate::support::config_set;

const INSTALL_DIR: &str = "vostok";
/// Raiz do projeto vista de `vostok/` (RN-05).
const PROJECT_ROOT: &str = "../";
const DEFAULT_DOCS: &str = "docs";
const ALL_TOOLS_ANSWER: &str = "todas";
/// Força as perguntas mesmo com o stdin fora de um terminal, para responder por um pipe (testes, scripts).
const FORCE_INTERACTIVE_ENV: &str = "VOSTOK_FORCE_INTERACTIVE";

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Usa os valores padrão, sem perguntas.
    #[arg(long)]
    pub yes: bool,
    /// Ferramentas que recebem roteamento, separadas por vírgula (ex.: claude-code,codex).
    #[arg(long)]
    pub tools: Option<String>,
}

/// Respostas da inicialização. Os caminhos já estão relativos à raiz do harness.
struct Choices {
    docs: String,
    spec_docs: String,
    tools: Vec<&'static Tool>,
}

pub fn run(args: InitArgs) -> Result<()> {
    let project = std::env::current_dir()?;
    let harness = Harness {
        root: project.join(INSTALL_DIR),
    };
    if harness.root.exists() {
        bail!(
            "o projeto já tem o harness em {INSTALL_DIR}/; para atualizá-lo, execute `vostok update`"
        );
    }
    let choices = if args.yes {
        default_choices(args.tools.as_deref())?
    } else {
        ask_choices(args.tools.as_deref())?
    };
    install(&harness, &choices)
}

fn default_choices(tools: Option<&str>) -> Result<Choices> {
    Ok(Choices {
        docs: harness_relative(DEFAULT_DOCS),
        spec_docs: harness_relative(&spec_docs_default(DEFAULT_DOCS)),
        tools: chosen_tools(tools)?,
    })
}

/// Sem `--tools`, todas as ferramentas suportadas (inferência M).
fn chosen_tools(tools: Option<&str>) -> Result<Vec<&'static Tool>> {
    match tools {
        Some(list) => routing::parse_tools(list),
        None => Ok(TOOLS.iter().collect()),
    }
}

fn ask_choices(tools: Option<&str>) -> Result<Choices> {
    if !is_interactive() {
        bail!(
            "sem terminal para as perguntas da inicialização; execute `vostok init --yes`, com `--tools` para escolher as ferramentas"
        );
    }
    let preset = tools.map(routing::parse_tools).transpose()?;
    let mut input = io::stdin().lock();
    let docs = prompt(
        &mut input,
        "Pasta dos documentos do projeto (paths.docs)",
        DEFAULT_DOCS,
    )?;
    let spec_docs = prompt(
        &mut input,
        "Pasta dos documentos publicados das specs (paths.spec_docs)",
        &spec_docs_default(&docs),
    )?;
    let tools = match preset {
        Some(tools) => tools,
        None => ask_tools(&mut input)?,
    };
    Ok(Choices {
        docs: harness_relative(&docs),
        spec_docs: harness_relative(&spec_docs),
        tools,
    })
}

fn is_interactive() -> bool {
    io::stdin().is_terminal()
        || std::env::var_os(FORCE_INTERACTIVE_ENV).is_some_and(|value| value == "1")
}

/// Seleção múltipla: números ou nomes separados por vírgula; vazio escolhe todas.
fn ask_tools(input: &mut impl BufRead) -> Result<Vec<&'static Tool>> {
    println!("Ferramentas suportadas:");
    for (index, tool) in TOOLS.iter().enumerate() {
        println!("  {}) {} ({})", index + 1, tool.name, tool.label);
    }
    let answer = prompt(
        input,
        "Ferramentas do projeto (números ou nomes, separados por vírgula)",
        ALL_TOOLS_ANSWER,
    )?;
    if answer == ALL_TOOLS_ANSWER {
        return Ok(TOOLS.iter().collect());
    }
    let names: Vec<&str> = answer
        .split([',', ' '])
        .map(|token| tool_name(token.trim()))
        .collect();
    routing::parse_tools(&names.join(","))
}

/// Nome da ferramenta para um número da lista; qualquer outro texto continua como está.
fn tool_name(token: &str) -> &str {
    token
        .parse::<usize>()
        .ok()
        .and_then(|number| number.checked_sub(1))
        .and_then(|index| TOOLS.get(index))
        .map_or(token, |tool| tool.name)
}

/// Lê a resposta de uma pergunta; uma linha vazia fica com o valor padrão.
fn prompt(input: &mut impl BufRead, question: &str, default: &str) -> Result<String> {
    print!("{question} [{default}]: ");
    io::stdout().flush()?;
    let mut answer = String::new();
    if input.read_line(&mut answer)? == 0 {
        bail!("a entrada terminou sem a resposta para '{question}'");
    }
    let answer = answer.trim();
    Ok(if answer.is_empty() { default } else { answer }.to_string())
}

fn spec_docs_default(docs: &str) -> String {
    format!("{}/specs", docs.trim_end_matches('/'))
}

/// Caminho respondido a partir da raiz do projeto, convertido para relativo à raiz do harness.
fn harness_relative(answer: &str) -> String {
    if Path::new(answer).is_absolute() {
        return answer.to_string();
    }
    let relative = answer.trim_start_matches("./").trim_end_matches('/');
    format!("{PROJECT_ROOT}{relative}")
}

fn install(harness: &Harness, choices: &Choices) -> Result<()> {
    embedded::extract_to(&harness.root)
        .with_context(|| format!("não foi possível instalar em {}", harness.root.display()))?;
    let main = harness.config_file("main.yml");
    config_set(&main, "paths.root", PROJECT_ROOT)?;
    config_set(&main, "paths.docs", &choices.docs)?;
    config_set(&main, "paths.spec_docs", &choices.spec_docs)?;
    routing::save_tools(harness, &choices.tools)?;
    let routing = routing::generate(harness)?;
    Manifest::current(harness)?.write(harness)?;

    let root = routing::project_root(harness)?;
    println!("Harness {} instalado em {INSTALL_DIR}/.", manifest::VERSION);
    let names: Vec<&str> = choices.tools.iter().map(|tool| tool.name).collect();
    println!("Ferramentas: {}", names.join(", "));
    for local in &routing.kept_locals {
        println!("Versão anterior guardada: {}", relative(&root, local));
    }
    print_routing(&root, &routing.changed, &choices.tools);
    println!("Manifesto: {INSTALL_DIR}/{}", manifest::FILE_NAME);
    Ok(())
}

/// Arquivos de roteamento um por linha e os redirecionamentos resumidos por pasta.
fn print_routing(root: &Path, changed: &[PathBuf], tools: &[&Tool]) {
    let in_redirect_dir = |path: &Path, tool: &Tool| path.starts_with(root.join(tool.redirect_dir));
    for path in changed {
        if !tools.iter().any(|tool| in_redirect_dir(path, tool)) {
            println!("Roteamento: {}", relative(root, path));
        }
    }
    for tool in tools {
        let count = changed
            .iter()
            .filter(|path| in_redirect_dir(path, tool))
            .count();
        println!("Redirecionamentos: {} ({count} skills)", tool.redirect_dir);
    }
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}
