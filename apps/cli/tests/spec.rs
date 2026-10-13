//! Testes de integração dos comandos `spec`: equivalência com as fixtures dos scripts bash
//! (`tests/fixtures/golden/spec`) e cenários de erro fora delas.

mod common;

use std::fs;

use common::golden::assert_group;
use common::{failure_stderr, Project};

#[test]
fn golden_spec() {
    assert_group("spec");
}

fn stdout_of(command: &mut assert_cmd::Command) -> String {
    let output = command.assert().success().get_output().stdout.clone();
    String::from_utf8(output).expect("stdout em UTF-8")
}

#[test]
fn list_without_specs_reports_none() {
    let project = Project::new();
    let stdout = stdout_of(project.vostok().args(["spec", "list"]));
    assert_eq!(stdout, "Nenhuma spec encontrada.\n");
}

#[test]
fn new_with_blank_title_fails_without_creating_files() {
    let project = Project::new();
    let stderr = failure_stderr(project.vostok().args(["spec", "new", "   "]));
    assert!(stderr.contains("Erro: título inválido"), "{stderr}");
    assert!(!project.harness().join("work/specs/draft").exists());
}

#[test]
fn new_with_title_without_letters_fails() {
    let project = Project::new();
    let stderr = failure_stderr(project.vostok().args(["spec", "new", "!!!"]));
    assert!(
        stderr.contains("o título precisa ter ao menos uma letra ou número"),
        "{stderr}"
    );
}

#[test]
fn new_advances_the_sequence_and_lists_from_a_subfolder() {
    let project = Project::new();
    let first = stdout_of(project.vostok().args(["spec", "new", "Primeira"]));
    let second = stdout_of(project.vostok().args(["spec", "new", "Segunda"]));
    assert!(first.starts_with("SPEC-001 criada em "), "{first}");
    assert!(second.starts_with("SPEC-002 criada em "), "{second}");

    fs::create_dir_all(project.root().join("a/b")).unwrap();
    let listed = stdout_of(project.vostok_in("a/b").args(["spec", "list", "draft"]));
    assert!(
        listed.contains("| SPEC-001 | Primeira | draft |"),
        "{listed}"
    );
    assert!(
        listed.contains("| SPEC-002 | Segunda | draft |"),
        "{listed}"
    );
}

#[test]
fn refine_unknown_spec_fails() {
    let project = Project::new();
    let stderr = failure_stderr(project.vostok().args(["spec", "refine", "SPEC-404", "fix"]));
    assert!(
        stderr.contains("Erro: spec SPEC-404 não encontrada em "),
        "{stderr}"
    );
}

#[test]
fn refine_twice_refuses_the_existing_refined_spec() {
    let project = Project::new();
    project
        .vostok()
        .args(["spec", "new", "Dupla"])
        .assert()
        .success();
    project
        .vostok()
        .args(["spec", "refine", "SPEC-001", "study"])
        .assert()
        .success();
    let stderr = failure_stderr(
        project
            .vostok()
            .args(["spec", "refine", "SPEC-001", "study"]),
    );
    assert!(
        stderr.contains("Erro: a spec refinada já existe: "),
        "{stderr}"
    );
}

#[test]
fn record_without_summary_shows_usage() {
    let project = Project::new();
    let stderr = failure_stderr(
        project
            .vostok()
            .args(["spec", "adjust", "record", "SPEC-001"]),
    );
    assert!(stderr.starts_with("Erro: Uso: adjust-spec.sh"), "{stderr}");
}

#[test]
fn approve_unknown_spec_fails() {
    let project = Project::new();
    let stderr = failure_stderr(
        project
            .vostok()
            .args(["spec", "approve", "approve", "SPEC-404"]),
    );
    assert!(
        stderr.contains("Erro: spec refinada SPEC-404 não encontrada em "),
        "{stderr}"
    );
}
