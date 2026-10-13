//! Testes de integração dos comandos `plan`: reprodução das fixtures do grupo e cenários de erro.

mod common;

use common::golden::assert_group;
use common::{failure_stderr, Project};

#[test]
fn golden_plan() {
    assert_group("plan");
}

#[test]
fn create_refuses_unknown_spec() {
    let project = Project::new();
    let stderr = failure_stderr(project.vostok().args(["plan", "create", "SPEC-404"]));
    assert_eq!(stderr, "Erro: spec refinada SPEC-404 não encontrada\n");
}

#[test]
fn score_refuses_spec_without_plan() {
    let project = Project::new();
    let stderr = failure_stderr(project.vostok().args(["plan", "score", "SPEC-404"]));
    assert_eq!(
        stderr,
        "Erro: a spec SPEC-404 não tem plano de tarefas. Crie-o com: plan-tasks.sh --create SPEC-404\n"
    );
}

#[test]
fn adjust_record_without_summary_shows_usage() {
    let project = Project::new();
    let stderr = failure_stderr(
        project
            .vostok()
            .args(["plan", "adjust", "record", "SPEC-001"]),
    );
    assert!(stderr.starts_with("Erro: Uso: adjust-plan.sh"), "{stderr}");
}

#[test]
fn adjust_record_refuses_blank_summary() {
    let project = Project::new();
    let stderr = failure_stderr(
        project
            .vostok()
            .args(["plan", "adjust", "record", "SPEC-001", "  \t "]),
    );
    assert_eq!(stderr, "Erro: o resumo do ajuste está vazio\n");
}

#[test]
fn adjust_record_refuses_multiline_summary() {
    let project = Project::new();
    let stderr = failure_stderr(project.vostok().args([
        "plan",
        "adjust",
        "record",
        "SPEC-001",
        "primeiro\nsegundo",
    ]));
    assert_eq!(
        stderr,
        "Erro: o resumo não pode ter quebra de linha; separe os itens com '; '\n"
    );
}

#[test]
fn approve_validate_refuses_spec_without_plan() {
    let project = Project::new();
    let stderr = failure_stderr(
        project
            .vostok()
            .args(["plan", "approve", "validate", "SPEC-404"]),
    );
    assert_eq!(
        stderr,
        "Erro: a spec SPEC-404 não tem plano de tarefas. Crie-o com a skill plan-tasks\n"
    );
}

#[test]
fn empty_spec_id_shows_usage() {
    let project = Project::new();
    let stderr = failure_stderr(project.vostok().args(["plan", "validate", ""]));
    assert!(stderr.starts_with("Erro: Uso: plan-tasks.sh"), "{stderr}");
}

#[test]
fn plan_outside_a_harness_fails() {
    let dir = tempfile::TempDir::new().unwrap();
    let stderr =
        failure_stderr(common::vostok_at(dir.path()).args(["plan", "validate", "SPEC-001"]));
    assert!(stderr.contains("nenhum harness encontrado"), "{stderr}");
}
