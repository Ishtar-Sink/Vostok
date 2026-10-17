//! Testes do grupo `bug`: validação do relato de defeito do harness e link de nova issue.

mod common;

use std::fs;

use common::{failure_stderr, Project};

const COMPLETE: &str = "---\nissue: null\ntitle: \"[harness-bug] regressão aceita volta a bloquear\"\ncreated_at: 2026-10-17T10:00:00Z\n---\n\n<!-- comentário -->\n\n## Versão\n\nvostok 0.1.2, Linux\n\n## Comportamento atual\n\nA tarefa bloqueia de novo.\n\n## Comportamento esperado\n\nA tarefa segue.\n\n## Como reproduzir\n\n1. Rode `vostok task regression`.\n\n## Causa provável\n\nDesconhecida.\n\n## O que já foi tentado\n\nRetomar a tarefa & rodar de novo.\n";

fn report(project: &Project, content: &str) -> String {
    let path = project.root().join("relato.md");
    fs::write(&path, content).unwrap();
    "relato.md".to_string()
}

fn stdout(command: &mut assert_cmd::Command) -> String {
    let output = command.assert().success().get_output().stdout.clone();
    String::from_utf8(output).unwrap()
}

#[test]
fn complete_report_is_valid() {
    let project = Project::new();
    let file = report(&project, COMPLETE);
    let output = stdout(project.vostok().args(["bug", "check", &file]));
    assert!(output.starts_with("Relato válido: "), "{output}");
}

#[test]
fn incomplete_report_lists_every_problem_and_changes_nothing() {
    let project = Project::new();
    let content = COMPLETE
        .replace("A tarefa segue.", "")
        .replace("Desconhecida.", "#{BUG_CAUSE}")
        .replace("[harness-bug] ", "");
    let file = report(&project, &content);
    let stderr = failure_stderr(project.vostok().args(["bug", "check", &file]));
    assert!(
        stderr.contains("seção vazia: ## Comportamento esperado"),
        "{stderr}"
    );
    assert!(stderr.contains("variável sem preencher"), "{stderr}");
    assert!(stderr.contains("[harness-bug]"), "{stderr}");
    assert_eq!(
        fs::read_to_string(project.root().join(&file)).unwrap(),
        content
    );
}

#[test]
fn missing_section_is_reported() {
    let project = Project::new();
    let content = COMPLETE.replace("## Causa provável\n\nDesconhecida.\n\n", "");
    let file = report(&project, &content);
    let stderr = failure_stderr(project.vostok().args(["bug", "check", &file]));
    assert!(
        stderr.contains("seção ausente: ## Causa provável"),
        "{stderr}"
    );
}

#[test]
fn link_encodes_title_and_body_for_the_configured_repository() {
    let project = Project::new();
    let file = report(&project, COMPLETE);
    let output = stdout(project.vostok().args(["bug", "link", &file]));
    let link = output.lines().next().unwrap();
    assert!(
        link.starts_with("https://github.com/Ishtar-Sink/Vostok/issues/new?title=%5Bharness-bug%5D%20regress%C3%A3o"),
        "{link}"
    );
    assert!(link.contains("&body=%23%23%20Vers%C3%A3o"), "{link}");
    assert!(link.contains("tarefa%20%26%20rodar"), "{link}");
    assert!(!link.contains("coment%C3%A1rio"), "{link}");
    assert!(!link.contains(' '));

    let config = project.harness().join("config/main.yml");
    let changed = fs::read_to_string(&config).unwrap().replace(
        "repository: Ishtar-Sink/Vostok",
        "repository: exemplo/teste",
    );
    fs::write(&config, changed).unwrap();
    let output = stdout(project.vostok().args(["bug", "link", &file]));
    assert!(
        output.starts_with("https://github.com/exemplo/teste/issues/new?"),
        "{output}"
    );
}

#[test]
fn long_report_gets_a_title_only_link_and_a_warning() {
    let project = Project::new();
    let long = COMPLETE.replace("A tarefa bloqueia de novo.", &"bloqueia ".repeat(1500));
    let file = report(&project, &long);
    let output = stdout(project.vostok().args(["bug", "link", &file]));
    let mut lines = output.lines();
    let link = lines.next().unwrap();
    assert!(!link.contains("&body="), "{link}");
    assert!(link.len() <= 8000);
    assert!(output.contains("cole o corpo de relato.md"), "{output}");
}

#[test]
fn link_refuses_an_invalid_report() {
    let project = Project::new();
    let file = report(&project, &COMPLETE.replace("A tarefa segue.", ""));
    let stderr = failure_stderr(project.vostok().args(["bug", "link", &file]));
    assert!(stderr.contains("seção vazia"), "{stderr}");
}

#[test]
fn missing_report_is_refused() {
    let project = Project::new();
    let stderr = failure_stderr(project.vostok().args(["bug", "check", "nao-existe.md"]));
    assert!(
        stderr.contains("relato não encontrado: nao-existe.md"),
        "{stderr}"
    );
}
