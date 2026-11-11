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

fn run_pre_bash(project: &Project, stdin: &str) -> std::process::Output {
    project
        .vostok()
        .args(["hook", "pre-bash"])
        .write_stdin(stdin.to_string())
        .output()
        .expect("rodar vostok hook pre-bash")
}

fn pre_bash(project: &Project, command: &str) -> std::process::Output {
    let input = serde_json::json!({
        "session_id": "s",
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": {"command": command}
    });
    run_pre_bash(project, &input.to_string())
}

fn assert_denied(project: &Project, command: &str, reason_part: &str) {
    let output = pre_bash(project, command);
    assert_eq!(output.status.code(), Some(0), "comando: {command}");
    let stdout = String::from_utf8(output.stdout).expect("stdout em UTF-8");
    let value: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|error| panic!("stdout sem JSON para '{command}' ({error}): {stdout}"));
    let specific = &value["hookSpecificOutput"];
    assert_eq!(
        specific["hookEventName"], "PreToolUse",
        "comando: {command}"
    );
    assert_eq!(specific["permissionDecision"], "deny", "comando: {command}");
    let reason = specific["permissionDecisionReason"]
        .as_str()
        .unwrap_or_else(|| panic!("sem motivo para '{command}': {stdout}"));
    assert!(
        reason.contains(reason_part),
        "motivo de '{command}' sem '{reason_part}': {reason}"
    );
}

fn assert_silent(output: std::process::Output, label: &str) {
    assert_eq!(output.status.code(), Some(0), "entrada: {label}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "",
        "entrada: {label}"
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "",
        "entrada: {label}"
    );
}

fn assert_allowed(project: &Project, command: &str) {
    assert_silent(pre_bash(project, command), command);
}

#[test]
fn pre_bash_denies_a_redirected_load() {
    let project = Project::new();
    for command in [
        "vostok context load > /dev/null",
        "vostok context load >> ctx.txt",
        "vostok context load &> /dev/null",
        "vostok context load 1>/tmp/x",
        "vostok context load 2>/dev/null",
    ] {
        assert_denied(&project, command, "redirecionada");
    }
}

#[test]
fn pre_bash_denies_a_piped_load() {
    let project = Project::new();
    for command in [
        "vostok context load | head",
        "vostok context load | grep regra",
        "vostok context load |& tail",
        "{ vostok context load; } | head -5",
    ] {
        assert_denied(&project, command, "pipe");
    }
}

#[test]
fn pre_bash_denies_a_captured_load() {
    let project = Project::new();
    for command in [
        "x=$(vostok context load)",
        "x=`vostok context load`",
        "echo \"$(vostok context load)\"",
        "diff <(vostok context load) a.txt",
    ] {
        assert_denied(&project, command, "capturada");
    }
}

#[test]
fn pre_bash_denies_a_skill_load_with_a_pipe() {
    let project = Project::new();
    assert_denied(
        &project,
        "vostok context load --skill refine-spec --persona planejador | head",
        "pipe",
    );
}

#[test]
fn pre_bash_reason_tells_how_to_load() {
    let project = Project::new();
    assert_denied(
        &project,
        "vostok context load | head",
        "`vostok context load`",
    );
    assert_denied(
        &project,
        "vostok context load | head",
        "sem redirecionamento, pipe ou captura",
    );
}

#[test]
fn pre_bash_allows_a_plain_load() {
    let project = Project::new();
    for command in [
        "vostok context load",
        "vostok context load --skill approve-plan --persona arquiteto",
        "vostok context load 2>&1",
        "cd /tmp && vostok context load",
    ] {
        assert_allowed(&project, command);
    }
}

#[test]
fn pre_bash_allows_other_commands() {
    let project = Project::new();
    for command in [
        "ls | head",
        "cargo test 2>&1 | tail",
        "vostok task status SPEC-001 > s.txt",
        "x=$(vostok spec list)",
        "vostok context load; ls | wc -l",
    ] {
        assert_allowed(&project, command);
    }
}

#[test]
fn pre_bash_allows_the_load_quoted_as_text() {
    let project = Project::new();
    for command in [
        "echo 'vostok context load | head'",
        "git commit -m \"vostok context load > x\"",
    ] {
        assert_allowed(&project, command);
    }
}

#[test]
fn pre_bash_allows_unknown_input() {
    let project = Project::new();
    for input in [
        "",
        "não é json",
        r#"{"tool_name": "Edit", "tool_input": {}}"#,
        r#"{"tool_name": "Bash"}"#,
    ] {
        assert_silent(run_pre_bash(&project, input), input);
    }
}
