//! Equivalência dos grupos `context`, `memory` e `rules` com os scripts bash (RF-13, CT-10), pelas
//! fixtures de `tests/fixtures/golden`, e cenários de erro que as fixtures não cobrem.

mod common;

use std::fs;

use common::golden::assert_group;
use common::{failure_stderr, Project};

#[test]
fn golden_context() {
    assert_group("context");
}

#[test]
fn golden_memory() {
    assert_group("memory");
}

#[test]
fn golden_rules() {
    assert_group("rules");
}

#[test]
fn context_load_rejects_an_empty_skill_name() {
    let project = Project::new();
    let stderr = failure_stderr(project.vostok().args(["context", "load", "--skill", ""]));
    assert_eq!(stderr, "Uso: vostok context load --skill <nome>\n");
}

#[test]
fn context_load_keeps_loading_after_a_missing_path() {
    let project = Project::new();
    let output = project
        .vostok()
        .args(["context", "load", "nao-existe.md", "config/skills.yml"])
        .assert()
        .failure()
        .code(1)
        .get_output()
        .clone();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("--- arquivo: config/skills.yml ---"));
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "Aviso: 'nao-existe.md' não encontrado\n"
    );
}

#[test]
fn memory_search_without_arguments_is_a_usage_error() {
    let project = Project::new();
    let stderr = failure_stderr(project.vostok().args(["memory", "search"]));
    assert_eq!(
        stderr,
        "Erro: Uso: vostok memory search <tipo|all> [termo...] | --id <ID>\n"
    );
}

#[test]
fn memory_search_with_an_empty_id_is_a_usage_error() {
    let project = Project::new();
    let stderr = failure_stderr(project.vostok().args(["memory", "search", "--id", ""]));
    assert!(stderr.starts_with("Erro: Uso: vostok memory search"));
}

#[test]
fn memory_update_without_the_memory_file_fails() {
    let project = Project::new();
    let stderr = failure_stderr(
        project
            .vostok()
            .args(["memory", "update", "DEC-001", "Status", "ativa"]),
    );
    assert!(
        stderr.starts_with("Erro: arquivo de memória não encontrado: "),
        "{stderr}"
    );
}

#[test]
fn memory_add_rejects_a_corrupted_sequence() {
    let project = Project::new();
    let sequences = project.harness().join("config/sequences.yml");
    let content = fs::read_to_string(&sequences).unwrap();
    fs::write(&sequences, content.replace("issues: 1", "issues: x")).unwrap();
    let stderr = failure_stderr(project.vostok().args(["memory", "add", "issue", "Falha"]));
    assert!(stderr.contains("issues inválido em "), "{stderr}");
    assert!(stderr.ends_with(": 'x'\n"), "{stderr}");
}

#[test]
fn memory_search_omits_entries_beyond_the_limit() {
    let project = Project::new();
    for index in 1..=6 {
        let title = format!("Comum {index}");
        project
            .vostok()
            .args(["memory", "add", "knowledge", &title])
            .assert()
            .success();
        let file = project.harness().join("memory/knowledge.md");
        let content = fs::read_to_string(&file).unwrap();
        let filled = content
            .lines()
            .map(|line| match line.find("#{") {
                Some(start) => format!("{}x", &line[..start]),
                None => line.to_string(),
            })
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&file, filled + "\n").unwrap();
    }
    let output = project
        .vostok()
        .args(["memory", "search", "knowledge", "comum"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();
    assert_eq!(stdout.matches("> Termos encontrados: 1 de 1").count(), 5);
    assert!(stdout
        .ends_with("\nEntradas omitidas: 1. Use termos mais específicos para refinar a busca.\n"));
}

#[test]
fn rules_create_category_rejects_an_invalid_purpose() {
    let project = Project::new();
    for (purpose, message) in [
        ("   ", "Erro: o propósito está vazio\n"),
        ("a\nb", "Erro: o propósito deve ter uma linha só\n"),
        (
            "tem #{X}",
            "Erro: o propósito não pode conter '#{', reservado para variáveis de template\n",
        ),
    ] {
        let stderr = failure_stderr(project.vostok().args([
            "rules",
            "create-category",
            "api",
            "API",
            purpose,
        ]));
        assert_eq!(stderr, message);
    }
    assert!(!project.harness().join("rules/project/api.md").exists());
}

#[test]
fn rules_create_category_rejects_an_empty_title() {
    let project = Project::new();
    let stderr = failure_stderr(project.vostok().args([
        "rules",
        "create-category",
        "api",
        " ",
        "Propósito.",
    ]));
    assert_eq!(stderr, "O título está vazio\n");
}

#[test]
fn rules_load_validates_every_target_before_writing() {
    let project = Project::new();
    let context = project.harness().join("config/context.yml");
    let before = fs::read_to_string(&context).unwrap();
    for (target, message) in [
        (
            "execute-task,Bad",
            "Erro: destino inválido 'Bad': use 'always' ou o nome de uma skill\n",
        ),
        ("", "Erro: informe 'always' ou ao menos uma skill\n"),
    ] {
        let stderr = failure_stderr(project.vostok().args(["rules", "load", "language", target]));
        assert_eq!(stderr, message);
    }
    assert_eq!(fs::read_to_string(&context).unwrap(), before);
}

#[test]
fn rules_write_rejects_a_removal_without_a_number() {
    let project = Project::new();
    let rules = project.root().join("regras.txt");
    fs::write(&rules, "remover: dois\n").unwrap();
    let stderr = failure_stderr(project.vostok().args([
        "rules",
        "write",
        "language",
        rules.to_str().unwrap(),
    ]));
    assert_eq!(
        stderr,
        "Erro: remover: espera o número da regra: 'remover: dois'\n"
    );
}

#[test]
fn rules_validate_reports_a_rule_before_the_purpose() {
    let project = Project::new();
    fs::write(
        project.harness().join("rules/project/extra.md"),
        "# Extra\n\n[project/extra-01]: Regra.\n",
    )
    .unwrap();
    let output = project
        .vostok()
        .args(["rules", "validate"])
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    assert!(String::from_utf8(output).unwrap().contains(
        "rules/project/extra.md:3: falta a frase de propósito antes da primeira regra\n"
    ));
}
