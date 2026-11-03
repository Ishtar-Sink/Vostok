//! `vostok avais <ID>`: os avais do painel de uma spec, de uma tarefa ou de um pedido.

use anyhow::{bail, Result};

use crate::aval::{self, Aval};
use crate::harness::Harness;

#[derive(Debug, clap::Args)]
pub struct AvaisArgs {
    /// SPEC-NNN, SPEC-NNN/TASK-NN ou PED-NNN.
    pub id: String,
}

pub fn run(args: AvaisArgs) -> Result<()> {
    let id = args.id;
    let is_spec = is_numbered(&id, "SPEC-");
    if !is_spec && !is_task(&id) && !is_numbered(&id, "PED-") {
        bail!("ID inválido '{id}': use SPEC-NNN, SPEC-NNN/TASK-NN ou PED-NNN");
    }
    let harness = Harness::from_current_dir()?;
    let task_prefix = format!("{id}/");
    let mut avais: Vec<Aval> = aval::read_for(&harness, &id)?
        .into_iter()
        .filter(|aval| aval.id == id || (is_spec && aval.id.starts_with(&task_prefix)))
        .collect();
    if avais.is_empty() {
        println!("Nenhum aval de {id}.");
        return Ok(());
    }
    avais.sort_by(|left, right| right.date.cmp(&left.date));
    println!("| Data | ID | Ação | Origem | Hash |");
    println!("|---|---|---|---|---|");
    for aval in avais {
        println!(
            "| {} | {} | {} | {} | {} |",
            aval.date, aval.id, aval.action, aval.origin, aval.hash
        );
    }
    Ok(())
}

fn is_numbered(value: &str, prefix: &str) -> bool {
    value
        .strip_prefix(prefix)
        .is_some_and(|digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
}

fn is_task(id: &str) -> bool {
    id.split_once('/')
        .is_some_and(|(spec, task)| is_numbered(spec, "SPEC-") && is_numbered(task, "TASK-"))
}
