//! Hooks das ferramentas de agente (SPEC-050).

mod common;

use std::fs;

use common::Project;

fn run_session_start(project: &Project) -> (String, String) {
    let output = project
        .vostok()
        .args(["hook", "session-start"])
        .assert()
        .success()
        .get_output()
        .clone();
    (
        String::from_utf8(output.stdout).expect("stdout em UTF-8"),
        String::from_utf8(output.stderr).expect("stderr em UTF-8"),
    )
}

fn session_start(project: &Project) -> String {
    run_session_start(project).0
}

fn position(text: &str, needle: &str) -> usize {
    text.find(needle)
        .unwrap_or_else(|| panic!("falta '{needle}' em:\n{text}"))
}

fn write_security(project: &Project, content: &str) {
    let path = project.harness().join("rules/project/security.md");
    fs::create_dir_all(path.parent().unwrap()).expect("criar rules/project");
    fs::write(path, content).expect("gravar security.md");
}

#[test]
fn session_start_prints_the_short_core() {
    let project = Project::new();
    let output = session_start(&project);
    assert!(output.starts_with("# Núcleo do contexto obrigatório do harness\n"));
    assert!(output.contains("--- arquivo: rules/agent/boundaries.md ---"));
    assert!(output.contains("Rode agora `vostok context load`"));
    assert!(!output.contains("--- arquivo: templates/responses/"));
    assert!(!output.contains("--- arquivo: personas/"));
    assert!(!output.contains("skill-catalog.md"));
}

#[test]
fn session_start_includes_the_project_security_rules() {
    let project = Project::new();
    write_security(&project, "# Segurança\n\n[project/security-01]: Teste.\n");
    let output = session_start(&project);
    let boundaries = position(&output, "--- arquivo: rules/agent/boundaries.md ---");
    let security = position(&output, "--- arquivo: rules/project/security.md ---");
    let rule = position(&output, "[project/security-01]: Teste.");
    let next = position(&output, "Rode agora `vostok context load`");
    assert!(boundaries < security && security < rule && rule < next);
}

#[test]
fn session_start_without_security_rules_has_no_warning() {
    let project = Project::new();
    let path = project.harness().join("rules/project/security.md");
    let _ = fs::remove_file(path);
    let (output, stderr) = run_session_start(&project);
    assert!(!output.contains("rules/project/security.md"));
    assert_eq!(stderr, "");
}

#[test]
fn session_start_fits_the_claude_code_limit() {
    let project = Project::new();
    let output = session_start(&project);
    assert!(
        output.chars().count() < 10_000,
        "{} caracteres",
        output.chars().count()
    );
}

#[test]
fn session_start_warns_when_over_the_limit() {
    let project = Project::new();
    write_security(&project, &"é".repeat(10_000));
    let (output, stderr) = run_session_start(&project);
    let warning = stderr.trim_end();
    assert!(
        warning.starts_with("Aviso: o núcleo tem "),
        "stderr: {stderr}"
    );
    assert!(warning.ends_with(
        " caracteres, acima do limite de 10.000 do hook; rode `vostok context load` agora."
    ));
    let count: usize = warning["Aviso: o núcleo tem ".len()..]
        .split(' ')
        .next()
        .unwrap()
        .parse()
        .expect("número de caracteres");
    assert!(count >= 10_000);
    assert!(output.trim_end().ends_with(warning));
}

#[test]
fn session_start_leaves_context_load_unchanged() {
    let project = Project::new();
    let output = project
        .vostok()
        .args(["context", "load"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output = String::from_utf8(output).unwrap();
    assert!(output.contains("--- arquivo: context/skill-catalog.md ---"));
    assert!(output.contains("# Contexto obrigatório do harness"));
}
