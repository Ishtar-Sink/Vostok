//! Testes do grupo `suggest`: validação da sugestão de melhoria ou de recurso do harness e link de
//! nova issue.

mod common;

use std::fs;

use common::{failure_stderr, Project};

const IMPROVEMENT: &str = "---\ntype: improvement\nissue: null\ntitle: \"[harness-improvement] mensagem de regressão mais clara\"\ncreated_at: 2026-10-17T10:00:00Z\n---\n\n<!-- comentário -->\n\n## Versão\n\nvostok 0.1.5, Linux\n\n## Situação atual\n\nA mensagem não cita a tarefa.\n\n## Melhoria proposta\n\nCitar a tarefa & o comando.\n\n## Benefício\n\nMenos tempo procurando.\n\n## Alternativas consideradas\n\nNenhuma.\n";

const FEATURE: &str = "---\ntype: feature\nissue: null\ntitle: \"[harness-feature] comando para listar skills\"\ncreated_at: 2026-10-17T10:00:00Z\n---\n\n<!-- comentário -->\n\n## Versão\n\nvostok 0.1.5, Linux\n\n## Problema ou necessidade\n\nNão há como listar as skills.\n\n## Recurso proposto\n\nUm comando de listagem.\n\n## Exemplo de uso\n\n`vostok skill list`\n\n## Alternativas consideradas\n\nNenhuma.\n";

fn suggestion(project: &Project, content: &str) -> String {
    let path = project.root().join("sugestao.md");
    fs::write(&path, content).unwrap();
    "sugestao.md".to_string()
}

fn stdout(command: &mut assert_cmd::Command) -> String {
    let output = command.assert().success().get_output().stdout.clone();
    String::from_utf8(output).unwrap()
}

#[test]
fn improvement_suggestion_is_valid() {
    let project = Project::new();
    let file = suggestion(&project, IMPROVEMENT);
    let output = stdout(project.vostok().args(["suggest", "check", &file]));
    assert!(output.starts_with("Sugestão válida: "), "{output}");
}

#[test]
fn feature_suggestion_is_valid() {
    let project = Project::new();
    let file = suggestion(&project, FEATURE);
    let output = stdout(project.vostok().args(["suggest", "check", &file]));
    assert!(output.starts_with("Sugestão válida: "), "{output}");
}

#[test]
fn invalid_type_is_refused_citing_the_type() {
    let project = Project::new();
    let file = suggestion(
        &project,
        &IMPROVEMENT.replace("type: improvement", "type: bug"),
    );
    let stderr = failure_stderr(project.vostok().args(["suggest", "check", &file]));
    assert!(stderr.contains("tipo inválido 'bug'"), "{stderr}");
}

#[test]
fn empty_section_is_reported() {
    let project = Project::new();
    let content = IMPROVEMENT.replace("Menos tempo procurando.", "");
    let file = suggestion(&project, &content);
    let stderr = failure_stderr(project.vostok().args(["suggest", "check", &file]));
    assert!(stderr.contains("seção vazia: ## Benefício"), "{stderr}");
    assert_eq!(
        fs::read_to_string(project.root().join(&file)).unwrap(),
        content
    );
}

#[test]
fn missing_section_is_reported() {
    let project = Project::new();
    let content = FEATURE.replace("## Exemplo de uso\n\n`vostok skill list`\n\n", "");
    let file = suggestion(&project, &content);
    let stderr = failure_stderr(project.vostok().args(["suggest", "check", &file]));
    assert!(
        stderr.contains("seção ausente: ## Exemplo de uso"),
        "{stderr}"
    );
}

#[test]
fn title_prefix_follows_the_type() {
    let project = Project::new();
    let content = IMPROVEMENT.replace("[harness-improvement]", "[harness-feature]");
    let file = suggestion(&project, &content);
    let stderr = failure_stderr(project.vostok().args(["suggest", "check", &file]));
    assert!(stderr.contains("[harness-improvement]"), "{stderr}");
}

#[test]
fn leftover_variable_is_reported() {
    let project = Project::new();
    let content = IMPROVEMENT.replace("Nenhuma.", "#{SUGGESTION_ALTERNATIVES}");
    let file = suggestion(&project, &content);
    let stderr = failure_stderr(project.vostok().args(["suggest", "check", &file]));
    assert!(stderr.contains("variável sem preencher"), "{stderr}");
}

#[test]
fn link_encodes_title_and_body_for_the_configured_repository() {
    let project = Project::new();
    let file = suggestion(&project, IMPROVEMENT);
    let output = stdout(project.vostok().args(["suggest", "link", &file]));
    let link = output.lines().next().unwrap();
    assert!(
        link.starts_with(
            "https://github.com/Ishtar-Sink/Vostok/issues/new?title=%5Bharness-improvement%5D"
        ),
        "{link}"
    );
    assert!(link.contains("&body=%23%23%20Vers%C3%A3o"), "{link}");
    assert!(!link.contains("coment%C3%A1rio"), "{link}");
    assert!(!link.contains(' '));

    let config = project.harness().join("config/main.yml");
    let changed = fs::read_to_string(&config).unwrap().replace(
        "repository: Ishtar-Sink/Vostok",
        "repository: exemplo/teste",
    );
    fs::write(&config, changed).unwrap();
    let output = stdout(project.vostok().args(["suggest", "link", &file]));
    assert!(
        output.starts_with("https://github.com/exemplo/teste/issues/new?"),
        "{output}"
    );
}

#[test]
fn feature_link_uses_the_feature_prefix() {
    let project = Project::new();
    let file = suggestion(&project, FEATURE);
    let output = stdout(project.vostok().args(["suggest", "link", &file]));
    assert!(output.contains("title=%5Bharness-feature%5D"), "{output}");
}

#[test]
fn long_suggestion_gets_a_title_only_link_and_a_warning() {
    let project = Project::new();
    let long = IMPROVEMENT.replace("A mensagem não cita a tarefa.", &"mensagem ".repeat(1500));
    let file = suggestion(&project, &long);
    let output = stdout(project.vostok().args(["suggest", "link", &file]));
    let link = output.lines().next().unwrap();
    assert!(!link.contains("&body="), "{link}");
    assert!(link.len() <= 8000);
    assert!(output.contains("cole o corpo de sugestao.md"), "{output}");
}

#[test]
fn link_refuses_an_invalid_suggestion() {
    let project = Project::new();
    let file = suggestion(
        &project,
        &IMPROVEMENT.replace("Menos tempo procurando.", ""),
    );
    let stderr = failure_stderr(project.vostok().args(["suggest", "link", &file]));
    assert!(stderr.contains("seção vazia"), "{stderr}");
}

#[test]
fn missing_suggestion_is_refused() {
    let project = Project::new();
    let stderr = failure_stderr(project.vostok().args(["suggest", "check", "nao-existe.md"]));
    assert!(
        stderr.contains("sugestão não encontrada: nao-existe.md"),
        "{stderr}"
    );
}

#[test]
fn issue_forms_have_the_sections_of_the_templates() {
    for kind in ["improvement", "feature"] {
        let template = vostok::embedded::file(&format!("templates/suggestion/{kind}.md"))
            .unwrap_or_else(|| panic!("template {kind} não embutido"));
        let template = String::from_utf8(template.to_vec()).unwrap();
        let form = fs::read_to_string(format!(
            "{}/../../.github/ISSUE_TEMPLATE/harness-{kind}.yml",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        assert!(
            form.contains(&format!("title: \"[harness-{kind}] \"")),
            "{kind}"
        );
        for heading in template.lines().filter(|line| line.starts_with("## ")) {
            let label = heading.trim_start_matches("## ");
            assert!(
                form.lines()
                    .any(|line| line.trim() == format!("label: {label}")),
                "{kind}: falta label: {label}"
            );
        }
    }
}
