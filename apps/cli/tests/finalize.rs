//! Testes do grupo `finalize`: equivalência com `finalize-spec.sh` pelas fixtures (CT-10, CT-12) e
//! recusas fora delas.

mod common;

use std::fs;

use common::golden::assert_group;
use common::{failure_stderr, Project};

#[test]
fn golden_finalize() {
    assert_group("finalize");
}

fn write_spec(project: &Project, status: &str) {
    let refined = project.harness().join("work/specs/refined");
    fs::create_dir_all(&refined).unwrap();
    fs::write(
        refined.join("SPEC-001-exemplo.md"),
        format!("---\nid: SPEC-001\ntitle: Exemplo\nstatus: {status}\n---\n\n# Exemplo\n"),
    )
    .unwrap();
}

fn write_task(project: &Project, id: &str, status: &str) {
    let tasks = project.harness().join("work/tasks/SPEC-001");
    fs::create_dir_all(&tasks).unwrap();
    fs::write(
        tasks.join(format!("{id}.md")),
        format!("---\nid: {id}\ntitle: Tarefa\nstatus: {status}\n---\n"),
    )
    .unwrap();
}

#[test]
fn every_subcommand_requires_a_spec_id() {
    let project = Project::new();
    for subcommand in ["sources", "create", "validate", "build", "archive"] {
        project
            .vostok()
            .args(["finalize", subcommand])
            .assert()
            .failure();
    }
}

#[test]
fn invalid_spec_id_is_refused() {
    let project = Project::new();
    let stderr = failure_stderr(project.vostok().args(["finalize", "sources", "spec-1"]));
    assert_eq!(stderr, "Erro: ID de spec inválido: 'spec-1'\n");
}

#[test]
fn unknown_spec_is_refused() {
    let project = Project::new();
    let stderr = failure_stderr(project.vostok().args(["finalize", "build", "SPEC-999"]));
    assert_eq!(
        stderr,
        "Erro: spec SPEC-999 não encontrada entre as refinadas\n"
    );
}

#[test]
fn spec_without_tasks_is_refused() {
    let project = Project::new();
    write_spec(&project, "implemented");
    let stderr = failure_stderr(project.vostok().args(["finalize", "create", "SPEC-001"]));
    assert!(stderr.starts_with("Erro: a spec SPEC-001 não pode ser finalizada:\n"));
    assert!(stderr.contains("  - nenhuma tarefa encontrada em "));
    assert!(!project.harness().join("work/finalize").exists());
}

#[test]
fn every_pending_task_is_listed() {
    let project = Project::new();
    write_spec(&project, "approved");
    write_task(&project, "TASK-01", "done");
    write_task(&project, "TASK-02", "in_progress");
    let stderr = failure_stderr(project.vostok().args(["finalize", "archive", "SPEC-001"]));
    assert_eq!(
        stderr,
        "Erro: a spec SPEC-001 não pode ser finalizada:\n  \
         - a spec está com status 'approved'; só specs com status 'implemented' são finalizadas\n  \
         - TASK-02: status 'in_progress', não 'done'\n"
    );
}

#[test]
fn validate_without_content_points_to_create() {
    let project = Project::new();
    write_spec(&project, "implemented");
    write_task(&project, "TASK-01", "done");
    let stderr = failure_stderr(project.vostok().args(["finalize", "validate", "SPEC-001"]));
    assert!(stderr.starts_with("Erro: o conteúdo de SPEC-001 tem problemas:\n"));
    assert!(stderr.contains("conteúdo não encontrado: "));
    assert!(stderr
        .trim_end()
        .ends_with("; crie com vostok finalize create"));
}
