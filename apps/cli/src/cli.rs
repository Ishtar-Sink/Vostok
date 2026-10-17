//! Definição da linha de comando.

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::commands::{
    bug, commit, context, finalize, init, memory, plan, rules, skill, spec, task, update,
};

#[derive(Debug, Parser)]
#[command(
    name = "vostok",
    version,
    about = "Harness do Vostok para agentes de IA"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Carregamento de contexto para o agente.
    #[command(subcommand)]
    Context(context::ContextCommand),
    /// Memória do projeto.
    #[command(subcommand)]
    Memory(memory::MemoryCommand),
    /// Regras do projeto e mapa de carregamento.
    #[command(subcommand)]
    Rules(rules::RulesCommand),
    /// Specs.
    #[command(subcommand)]
    Spec(spec::SpecCommand),
    /// Planos de tarefas.
    #[command(subcommand)]
    Plan(plan::PlanCommand),
    /// Tarefas.
    #[command(subcommand)]
    Task(task::TaskCommand),
    /// Commits liberados pelo usuário.
    #[command(subcommand)]
    Commit(commit::CommitCommand),
    /// Skills.
    #[command(subcommand)]
    Skill(skill::SkillCommand),
    /// Finalização de specs.
    #[command(subcommand)]
    Finalize(finalize::FinalizeCommand),
    /// Relato de defeitos do próprio harness.
    #[command(subcommand)]
    Bug(bug::BugCommand),
    /// Instala o harness no projeto.
    Init(init::InitArgs),
    /// Atualiza o harness instalado.
    Update,
}

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Context(command) => context::run(command),
        Command::Memory(command) => memory::run(command),
        Command::Rules(command) => rules::run(command),
        Command::Spec(command) => spec::run(command),
        Command::Plan(command) => plan::run(command),
        Command::Task(command) => task::run(command),
        Command::Commit(command) => commit::run(command),
        Command::Skill(command) => skill::run(command),
        Command::Finalize(command) => finalize::run(command),
        Command::Bug(command) => bug::run(command),
        Command::Init(args) => init::run(args),
        Command::Update => update::run(),
    }
}
