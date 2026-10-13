//! Testes de integração dos comandos `task` contra as fixtures de equivalência e cenários de erro.

mod common;

use common::golden::assert_group;
use common::{failure_stderr, vostok_at, Project};
use tempfile::TempDir;

#[test]
fn golden_task() {
    assert_group("task");
}

#[test]
fn load_without_ids_shows_the_usage() {
    let project = Project::new();
    let stderr = failure_stderr(project.vostok().args(["task", "load"]));
    assert_eq!(stderr, "Erro: Uso: vostok task load <SPEC_ID> <TASK_ID>\n");
}

#[test]
fn load_of_a_missing_task_names_the_path() {
    let project = Project::new();
    let stderr = failure_stderr(
        project
            .vostok()
            .args(["task", "load", "SPEC-404", "TASK-01"]),
    );
    assert!(stderr.starts_with("Erro: tarefa TASK-01 da spec SPEC-404 não encontrada: "));
    assert!(stderr
        .trim_end()
        .ends_with("work/tasks/SPEC-404/TASK-01.md"));
}

#[test]
fn start_rejects_an_invalid_spec_id() {
    let project = Project::new();
    let stderr = failure_stderr(
        project
            .vostok()
            .args(["task", "start", "../SPEC-1", "TASK-01"]),
    );
    assert_eq!(stderr, "Erro: ID de spec inválido: '../SPEC-1'\n");
}

#[test]
fn start_without_task_id_shows_the_usage() {
    let project = Project::new();
    let stderr = failure_stderr(project.vostok().args(["task", "start", "SPEC-001"]));
    assert!(stderr.starts_with("Erro: Uso: vostok task start"));
}

#[test]
fn approve_of_a_spec_without_tasks_is_refused() {
    let project = Project::new();
    let stderr = failure_stderr(
        project
            .vostok()
            .args(["task", "approve", "approve", "SPEC-404", "TASK-01"]),
    );
    assert!(stderr.starts_with("Erro: a spec SPEC-404 não tem tarefas em "));
}

#[test]
fn approve_without_tasks_asks_for_them() {
    let project = Project::new();
    std::fs::create_dir_all(project.harness().join("work/tasks/SPEC-001")).unwrap();
    let stderr = failure_stderr(
        project
            .vostok()
            .args(["task", "approve", "validate", "SPEC-001"]),
    );
    assert!(stderr.starts_with("Erro: informe as tarefas a aprovar ou --all"));
}

#[test]
fn resume_rejects_an_invalid_task_id() {
    let project = Project::new();
    let stderr = failure_stderr(
        project
            .vostok()
            .args(["task", "resume", "SPEC-001", "TASK-x", "decisão"]),
    );
    assert_eq!(stderr, "Erro: ID de tarefa inválido: 'TASK-x'\n");
}

#[test]
fn adjust_record_with_blank_summary_is_refused_before_the_lookup() {
    let project = Project::new();
    let stderr = failure_stderr(
        project
            .vostok()
            .args(["task", "adjust", "record", "SPEC-404", "TASK-01", "  "]),
    );
    assert_eq!(stderr, "Erro: o resumo do ajuste está vazio\n");
}

#[test]
fn task_commands_fail_outside_a_harness() {
    let dir = TempDir::new().unwrap();
    let stderr =
        failure_stderr(vostok_at(dir.path()).args(["task", "load", "SPEC-001", "TASK-01"]));
    assert!(stderr.contains("nenhum harness encontrado"));
}
