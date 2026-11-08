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
    assert!(
        stderr.starts_with("Erro: Uso: vostok spec adjust"),
        "{stderr}"
    );
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

#[test]
fn approve_records_the_status_history() {
    let project = Project::new();
    let spec = |args: &[&str]| {
        let mut command = project.vostok();
        command.args(["spec"]).args(args);
        command.assert().success();
    };
    spec(&["new", "Exemplo"]);
    spec(&["refine", "SPEC-001", "improvement"]);
    let file = project
        .harness()
        .join("work/specs/refined/SPEC-001-exemplo.md");
    let filler = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/fill-refined.py"
    );
    std::process::Command::new("python3")
        .arg(filler)
        .arg(&file)
        .status()
        .expect("preenche a spec")
        .success()
        .then_some(())
        .expect("fill-refined.py terminou com erro");
    fs::write(project.root().join("AGENTS.md"), "").unwrap();
    spec(&["adjust", "record-manual", "SPEC-001"]);
    let hash = |content: &str| {
        content
            .lines()
            .find(|line| line.starts_with("content_hash: "))
            .unwrap()
            .to_string()
    };
    let before = hash(&fs::read_to_string(&file).unwrap());
    spec(&["approve", "approve", "SPEC-001"]);
    let content = fs::read_to_string(&file).unwrap();
    let history = content
        .lines()
        .find(|line| line.starts_with("status_history: "))
        .unwrap_or_else(|| panic!("sem status_history: {content}"));
    assert!(history.ends_with(" approved"), "{history}");
    assert_eq!(hash(&content), before);
}

fn refined_spec(project: &Project) -> std::path::PathBuf {
    let spec = |args: &[&str]| {
        let mut command = project.vostok();
        command.args(["spec"]).args(args);
        command.assert().success();
    };
    spec(&["new", "Exemplo"]);
    spec(&["refine", "SPEC-001", "improvement"]);
    let file = project
        .harness()
        .join("work/specs/refined/SPEC-001-exemplo.md");
    let filler = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/fill-refined.py"
    );
    std::process::Command::new("python3")
        .arg(filler)
        .arg(&file)
        .status()
        .expect("preenche a spec")
        .success()
        .then_some(())
        .expect("fill-refined.py terminou com erro");
    fs::write(project.root().join("AGENTS.md"), "").unwrap();
    spec(&["adjust", "record-manual", "SPEC-001"]);
    file
}

fn status_history(file: &std::path::Path) -> String {
    fs::read_to_string(file)
        .unwrap()
        .lines()
        .find_map(|line| line.strip_prefix("status_history: "))
        .unwrap_or_default()
        .to_string()
}

fn run_spec(project: &Project, args: &[&str]) {
    let mut command = project.vostok();
    command.args(["spec"]).args(args);
    command.assert().success();
}

#[test]
fn adjust_history_records_the_return_to_refined() {
    let project = Project::new();
    let file = refined_spec(&project);
    run_spec(&project, &["approve", "approve", "SPEC-001"]);
    run_spec(
        &project,
        &["adjust", "record", "SPEC-001", "Ajuste de teste"],
    );
    let content = fs::read_to_string(&file).unwrap();
    assert!(content.contains("\nstatus: refined\n"), "{content}");
    let history = status_history(&file);
    let entries: Vec<&str> = history.split("; ").collect();
    assert_eq!(entries.len(), 2, "{history}");
    assert!(entries[0].ends_with(" approved"), "{history}");
    assert!(entries[1].ends_with(" refined"), "{history}");
}

#[test]
fn adjust_history_manual_adjust_records_the_return_to_refined() {
    let project = Project::new();
    let file = refined_spec(&project);
    run_spec(&project, &["approve", "approve", "SPEC-001"]);
    let content = fs::read_to_string(&file).unwrap();
    fs::write(
        &file,
        content.replacen("Objetivo.", "Objetivo editado à mão.", 1),
    )
    .unwrap();
    run_spec(&project, &["adjust", "record-manual", "SPEC-001"]);
    let history = status_history(&file);
    assert!(history.ends_with(" refined"), "{history}");
    assert_eq!(history.split("; ").count(), 2, "{history}");
}

#[test]
fn adjust_history_unchanged_for_a_refined_spec() {
    let project = Project::new();
    let file = refined_spec(&project);
    let before = status_history(&file);
    run_spec(
        &project,
        &["adjust", "record", "SPEC-001", "Ajuste de teste"],
    );
    assert_eq!(status_history(&file), before);
    let content = fs::read_to_string(&file).unwrap();
    assert!(content.contains("\nstatus: refined\n"), "{content}");
}

#[test]
fn chat_aval_spec_approve_without_aval_file() {
    let project = Project::new();
    let file = refined_spec(&project);
    let mut config = fs::OpenOptions::new()
        .append(true)
        .open(project.harness().join("config/main.yml"))
        .unwrap();
    std::io::Write::write_all(&mut config, b"approval_lock: true\n").unwrap();
    run_spec(&project, &["approve", "approve", "SPEC-001"]);
    let content = fs::read_to_string(&file).unwrap();
    assert!(content.contains("\nstatus: approved\n"), "{content}");
    assert!(!content.contains("approved_by"), "{content}");
    assert!(!project.harness().join("work/avais").exists());
}

#[test]
fn chat_aval_spec_with_approved_by_painel_is_read() {
    let project = Project::new();
    let file = refined_spec(&project);
    run_spec(&project, &["approve", "approve", "SPEC-001"]);
    let content = fs::read_to_string(&file).unwrap();
    fs::write(
        &file,
        content.replacen(
            "\nstatus: approved\n",
            "\nstatus: approved\napproved_by: painel\n",
            1,
        ),
    )
    .unwrap();
    run_spec(&project, &["adjust", "validate", "SPEC-001"]);
    let output = project.vostok().args(["spec", "list"]).assert().success();
    let listed = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    assert!(listed.contains("SPEC-001"), "{listed}");
}

// --- Componentes afetados prontos para o plano (SPEC-015) ---

fn refined_with_components(project: &Project, components: &str, contracts: &str) {
    project
        .vostok()
        .args(["spec", "new", "Exemplo"])
        .assert()
        .success();
    project
        .vostok()
        .args(["spec", "refine", "SPEC-001", "improvement"])
        .assert()
        .success();
    let file = project
        .harness()
        .join("work/specs/refined/SPEC-001-exemplo.md");
    let content = fs::read_to_string(&file).expect("spec refinada");
    let content = content
        .replace("#{AFFECTED_COMPONENTS}", components)
        .replace("#{CONTRACTS}", contracts);
    fs::write(&file, content).expect("grava a spec refinada");
}

fn existing(project: &Project, paths: &[&str]) {
    for path in paths {
        let full = project.root().join(path);
        if path.ends_with('/') {
            fs::create_dir_all(&full).expect("cria a pasta");
        } else {
            fs::create_dir_all(full.parent().expect("pasta-pai")).expect("cria a pasta-pai");
            fs::write(&full, "").expect("cria o arquivo");
        }
    }
}

fn component_row(path: &str, action: &str) -> String {
    format!("| `{path}` | {action} | Motivo. |")
}

fn validate_stderr(project: &Project) -> String {
    failure_stderr(project.vostok().args(["spec", "validate", "SPEC-001"]))
}

const FOLDER_ALTER_MESSAGE: &str =
    "'apps/cli/src' é uma pasta; com a ação 'alterar', liste os arquivos ou as subpastas que mudam";

fn alter_folder_project() -> Project {
    let project = Project::new();
    existing(&project, &["apps/cli/src/main.rs"]);
    refined_with_components(
        &project,
        &component_row("apps/cli/src", "alterar"),
        "Sem caminhos.",
    );
    project
}

#[test]
fn components_folder_with_a_cited_path_is_refused() {
    let project = Project::new();
    existing(&project, &["apps/web/src/billing.ts"]);
    refined_with_components(
        &project,
        &component_row("apps/web", "alterar"),
        "O arquivo `apps/web/src/billing.ts` muda.",
    );
    let stderr = validate_stderr(&project);
    assert!(stderr.lines().any(|line| line == "'apps/web' contém caminhos citados na spec (apps/web/src/billing.ts); liste os caminhos específicos no lugar da pasta"), "{stderr}");
    assert!(!stderr.contains("'apps/web' é uma pasta"), "{stderr}");
}

#[test]
fn components_folder_that_contains_another_component_is_refused() {
    let project = Project::new();
    let rows = format!(
        "{}\n{}",
        component_row("apps/server", "criar"),
        component_row("apps/server/src/notes", "criar")
    );
    refined_with_components(&project, &rows, "Sem caminhos.");
    let stderr = validate_stderr(&project);
    assert!(stderr.lines().any(|line| line == "'apps/server' contém caminhos citados na spec (apps/server/src/notes); liste os caminhos específicos no lugar da pasta"), "{stderr}");
    assert!(!stderr.contains("'apps/server/src/notes'"), "{stderr}");
}

#[test]
fn components_existing_folder_with_alter_is_refused() {
    let project = alter_folder_project();
    let stderr = validate_stderr(&project);
    assert!(
        stderr.lines().any(|line| line == FOLDER_ALTER_MESSAGE),
        "{stderr}"
    );
    assert!(!stderr.contains("contém caminhos citados"), "{stderr}");
}

#[test]
fn components_new_folder_and_removed_folder_stay_accepted() {
    let project = Project::new();
    existing(&project, &["apps/antigo/legado.ts"]);
    let rows = format!(
        "{}\n{}",
        component_row("apps/novo", "criar"),
        component_row("apps/antigo", "remover")
    );
    refined_with_components(
        &project,
        &rows,
        "O arquivo `apps/antigo/legado.ts` sai junto.",
    );
    let stderr = validate_stderr(&project);
    assert!(!stderr.contains("'apps/novo'"), "{stderr}");
    assert!(!stderr.contains("'apps/antigo'"), "{stderr}");
}

#[test]
fn components_message_lists_three_cited_paths_in_alphabetical_order() {
    let project = Project::new();
    refined_with_components(
        &project,
        &component_row("apps/web", "criar"),
        "`apps/web/e.ts` `apps/web/b.ts` `apps/web/d.ts` `apps/web/a.ts` `apps/web/c.ts` `apps/web/a.ts`",
    );
    let stderr = validate_stderr(&project);
    assert!(stderr.contains("'apps/web' contém caminhos citados na spec (apps/web/a.ts, apps/web/b.ts, apps/web/c.ts, ...); liste os caminhos específicos no lugar da pasta"), "{stderr}");
}

#[test]
fn components_adjust_and_approve_validation_use_the_same_rules() {
    let project = alter_folder_project();
    let adjust = failure_stderr(
        project
            .vostok()
            .args(["spec", "adjust", "validate", "SPEC-001"]),
    );
    assert!(adjust.contains(FOLDER_ALTER_MESSAGE), "{adjust}");
    let line = format!("validação: {FOLDER_ALTER_MESSAGE}");
    let validate = failure_stderr(
        project
            .vostok()
            .args(["spec", "approve", "validate", "SPEC-001"]),
    );
    assert!(validate.contains(&line), "{validate}");
    let approve = failure_stderr(
        project
            .vostok()
            .args(["spec", "approve", "approve", "SPEC-001"]),
    );
    assert!(approve.contains("nada foi alterado"), "{approve}");
    assert!(approve.contains(&line), "{approve}");
}

#[test]
fn components_trailing_slash_is_treated_as_the_folder() {
    let project = Project::new();
    existing(&project, &["apps/web/a.ts"]);
    refined_with_components(
        &project,
        &component_row("apps/web/", "alterar"),
        "Muda `apps/web/a.ts`.",
    );
    let stderr = validate_stderr(&project);
    assert!(
        stderr.contains("'apps/web' contém caminhos citados na spec (apps/web/a.ts)"),
        "{stderr}"
    );
}

#[test]
fn components_own_path_and_prefix_lookalike_do_not_count() {
    let project = Project::new();
    refined_with_components(
        &project,
        &component_row("apps/web", "criar"),
        "A pasta `apps/web` nasce aqui; `apps/web/` também é citada; `apps/webapp/x.ts` é outra coisa.",
    );
    let stderr = validate_stderr(&project);
    assert!(!stderr.contains("contém caminhos citados"), "{stderr}");
}

#[test]
fn components_existing_file_with_alter_and_dash_row_stay_accepted() {
    let project = Project::new();
    existing(&project, &["apps/cli/Cargo.toml"]);
    let rows = format!(
        "{}\n| - | - | - |",
        component_row("apps/cli/Cargo.toml", "alterar")
    );
    refined_with_components(&project, &rows, "Sem caminhos.");
    let stderr = validate_stderr(&project);
    assert!(!stderr.contains("é uma pasta"), "{stderr}");
    assert!(!stderr.contains("contém caminhos citados"), "{stderr}");
    assert!(!stderr.contains("ação inválida para '-'"), "{stderr}");
    assert!(!stderr.contains("'-' não existe"), "{stderr}");
}

#[test]
fn components_cited_paths_with_spaces_templates_or_in_code_blocks_do_not_count() {
    let project = Project::new();
    refined_with_components(
        &project,
        &component_row("apps/web", "criar"),
        "Comando `apps/web/a b.ts` com espaço, moldes `apps/web/<modulo>` e `apps/web/*.ts`.\n```\n`apps/web/x.ts`\n```",
    );
    let stderr = validate_stderr(&project);
    assert!(!stderr.contains("contém caminhos citados"), "{stderr}");
}
