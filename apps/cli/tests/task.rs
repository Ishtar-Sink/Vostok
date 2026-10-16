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

// --- Aceite de diferença de regressão (SPEC-007) ---

const TASK_PATH: &str = "vostok/work/tasks/SPEC-001/TASK-01.md";
const REGRESSION_DIR: &str = "vostok/work/tasks/SPEC-001/evidence/TASK-01/validation/regression";

/// Tarefa em execução com um comando `cat <arquivo>` no baseline para cada arquivo, já coletado.
fn running_task(files: &[(&str, &str)]) -> Project {
    let project = Project::new();
    let baseline: String = files
        .iter()
        .map(|(name, _)| format!("- `cat {name}`\n"))
        .collect();
    for (name, content) in files {
        write(&project, name, content);
    }
    write(
        &project,
        TASK_PATH,
        &format!(
            "---\nid: TASK-01\nspec: SPEC-001\ntitle: \"Tarefa\"\nstatus: in_progress\nattempts: 0\ntests_result: null\nregression_result: null\nevidence_result: null\n---\n\n# TASK-01: Tarefa\n\n## Execução\n\n### Baseline\n\n{baseline}\n### Verificação\n\n```bash\ntrue\n```\n\n### Evidências\n\nNão se aplica.\n\n### Concluída quando\n\n- [x] Pronto.\n\n  > Como comprovar: ver.\n\n## Descrição\n\nTexto.\n\n## Registro de execução\n\n| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n"
        ),
    );
    write(
        &project,
        "vostok/work/tasks/SPEC-001/TASK-02.md",
        "---\nid: TASK-02\nstatus: pending\n---\n",
    );
    task(&project, &["baseline"]).assert().success();
    project
}

fn write(project: &Project, relative: &str, content: &str) {
    let path = project.root().join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn read(project: &Project, relative: &str) -> String {
    std::fs::read_to_string(project.root().join(relative)).unwrap_or_default()
}

fn exists(project: &Project, relative: &str) -> bool {
    project.root().join(relative).exists()
}

/// `vostok task <args...> SPEC-001 TASK-01 [extra...]`.
fn task(project: &Project, args: &[&str]) -> assert_cmd::Command {
    let mut command = project.vostok();
    command
        .arg("task")
        .arg(args[0])
        .args(["SPEC-001", "TASK-01"])
        .args(&args[1..]);
    command
}

fn stdout(command: &mut assert_cmd::Command) -> String {
    String::from_utf8(command.output().unwrap().stdout).unwrap()
}

fn status(project: &Project) -> String {
    read(project, TASK_PATH)
        .lines()
        .find_map(|line| line.strip_prefix("status: "))
        .unwrap_or_default()
        .to_string()
}

/// Valida até a regressão bloquear e aceita os comandos informados.
fn block_and_accept(project: &Project, numbers: &str) {
    task(project, &["tests"]).assert().success();
    task(project, &["regression"]).assert().failure();
    task(
        project,
        &[
            "resume",
            "--accept-regression",
            numbers,
            "diferença esperada",
        ],
    )
    .assert()
    .success();
}

#[test]
fn accepted_difference_lets_the_task_finish() {
    let project = running_task(&[("valor.txt", "a\n")]);
    write(&project, "valor.txt", "b\n");
    task(&project, &["tests"]).assert().success();
    let blocked = stdout(&mut task(&project, &["regression"]));
    assert!(blocked.contains("- [1] cat valor.txt\n"), "{blocked}");
    assert!(blocked.contains("--accept-regression"));
    assert_eq!(status(&project), "blocked");
    assert!(read(&project, &format!("{REGRESSION_DIR}/01.diff")).contains("> b"));

    task(
        &project,
        &["resume", "--accept-regression", "1", "diferença esperada"],
    )
    .assert()
    .success();
    let accepted = read(&project, &format!("{REGRESSION_DIR}/accepted"));
    let fields: Vec<&str> = accepted.trim_end().split('\t').collect();
    assert_eq!(fields.len(), 5, "{accepted}");
    assert_eq!(fields[0], "1");
    assert_eq!(fields[1].len(), 64);
    assert_eq!(fields[3], "cat valor.txt");
    assert_eq!(fields[4], "diferença esperada");

    task(&project, &["tests"]).assert().success();
    let passed = stdout(&mut task(&project, &["regression"]));
    assert!(
        passed.contains("0 igual(is) ao baseline, 1 aceito(s) pelo usuário"),
        "{passed}"
    );
    task(&project, &["evidence"]).assert().success();
    task(&project, &["finish"]).assert().success();
    assert_eq!(status(&project), "done");

    let record = read(&project, TASK_PATH);
    assert!(record.contains("decisão do usuário: diferença esperada; aceito(s): 1 |"));
    assert!(record.contains("0 igual(is) ao baseline, 1 aceito(s) pelo usuário"));
}

#[test]
fn blocked_regression_numbers_only_the_divergent_commands() {
    let project = running_task(&[("um.txt", "1\n"), ("dois.txt", "2\n")]);
    write(&project, "dois.txt", "dois\n");
    task(&project, &["tests"]).assert().success();
    let blocked = stdout(&mut task(&project, &["regression"]));
    assert!(blocked.contains("- [2] cat dois.txt\n"), "{blocked}");
    assert!(!blocked.contains("[1]"));
    assert!(exists(&project, &format!("{REGRESSION_DIR}/02.diff")));
    assert!(!exists(&project, &format!("{REGRESSION_DIR}/01.diff")));
}

#[test]
fn output_changed_after_the_acceptance_blocks_again() {
    let project = running_task(&[("valor.txt", "a\n")]);
    write(&project, "valor.txt", "b\n");
    block_and_accept(&project, "1");
    write(&project, "valor.txt", "c\n");
    task(&project, &["tests"]).assert().success();
    let blocked = stdout(&mut task(&project, &["regression"]));
    assert!(blocked.contains("- [1] cat valor.txt\n"), "{blocked}");
    assert_eq!(status(&project), "blocked");
}

#[test]
fn new_difference_besides_the_accepted_one_blocks() {
    let project = running_task(&[("um.txt", "1\n"), ("dois.txt", "2\n")]);
    write(&project, "um.txt", "um\n");
    block_and_accept(&project, "1");
    write(&project, "dois.txt", "dois\n");
    task(&project, &["tests"]).assert().success();
    let blocked = stdout(&mut task(&project, &["regression"]));
    assert!(blocked.contains("- [2] cat dois.txt\n"), "{blocked}");
    assert!(!blocked.contains("[1]"));
}

#[test]
fn resume_without_acceptance_blocks_again_on_the_same_difference() {
    let project = running_task(&[("valor.txt", "a\n")]);
    write(&project, "valor.txt", "b\n");
    task(&project, &["tests"]).assert().success();
    task(&project, &["regression"]).assert().failure();
    task(&project, &["resume", "corrigir depois"])
        .assert()
        .success();
    task(&project, &["tests"]).assert().success();
    task(&project, &["regression"]).assert().failure();
    assert_eq!(status(&project), "blocked");
    assert!(!exists(&project, &format!("{REGRESSION_DIR}/accepted")));
    assert!(read(&project, TASK_PATH).contains("decisão do usuário: corrigir depois |"));
}

#[test]
fn invalid_acceptances_change_nothing() {
    let project = running_task(&[("um.txt", "1\n"), ("dois.txt", "2\n")]);
    write(&project, "um.txt", "um\n");
    task(&project, &["tests"]).assert().success();
    task(&project, &["regression"]).assert().failure();
    let before = read(&project, TASK_PATH);

    for (numbers, decision, message) in [
        ("3", "aceito", "o comando 3 não divergiu"),
        ("2", "aceito", "o comando 2 não divergiu"),
        ("x", "aceito", "número inválido em --accept-regression: 'x'"),
        ("0", "aceito", "número inválido em --accept-regression: '0'"),
        ("1", "  ", "informe a decisão do usuário"),
    ] {
        let stderr = failure_stderr(&mut task(
            &project,
            &["resume", "--accept-regression", numbers, decision],
        ));
        assert!(stderr.contains(message), "{numbers}: {stderr}");
        assert_eq!(read(&project, TASK_PATH), before);
        assert!(!exists(&project, &format!("{REGRESSION_DIR}/accepted")));
    }
}

#[test]
fn acceptance_of_a_task_that_is_not_blocked_is_refused() {
    let project = running_task(&[("valor.txt", "a\n")]);
    let before = read(&project, TASK_PATH);
    let stderr = failure_stderr(&mut task(
        &project,
        &["resume", "--accept-regression", "1", "aceito"],
    ));
    assert!(
        stderr.contains("só tarefas com status 'blocked' são retomadas"),
        "{stderr}"
    );
    assert_eq!(read(&project, TASK_PATH), before);
}

#[test]
fn acceptance_survives_new_regression_runs() {
    let project = running_task(&[("valor.txt", "a\n")]);
    write(&project, "valor.txt", "b\n");
    block_and_accept(&project, "1");
    let accepted = read(&project, &format!("{REGRESSION_DIR}/accepted"));
    for _ in 0..2 {
        task(&project, &["regression"]).assert().success();
    }
    assert_eq!(
        read(&project, &format!("{REGRESSION_DIR}/accepted")),
        accepted
    );
    assert!(exists(
        &project,
        &format!("{REGRESSION_DIR}/accepted-01.diff")
    ));
    assert!(!exists(&project, &format!("{REGRESSION_DIR}/01.diff")));
}

#[test]
fn output_back_to_the_baseline_counts_as_equal() {
    let project = running_task(&[("valor.txt", "a\n")]);
    write(&project, "valor.txt", "b\n");
    block_and_accept(&project, "1");
    write(&project, "valor.txt", "a\n");
    let passed = stdout(&mut task(&project, &["regression"]));
    assert!(
        passed.contains("1 comando(s) iguais ao baseline"),
        "{passed}"
    );
}

#[test]
fn regression_without_difference_keeps_the_current_behavior() {
    let project = running_task(&[("valor.txt", "a\n")]);
    let passed = stdout(&mut task(&project, &["regression"]));
    assert_eq!(
        passed,
        "Sem regressão em TASK-01: 1 comando(s) iguais ao baseline.\n"
    );
    assert!(!exists(&project, &format!("{REGRESSION_DIR}/01.diff")));
    assert!(!exists(&project, &format!("{REGRESSION_DIR}/accepted")));
}

#[test]
fn later_acceptance_of_the_same_command_replaces_the_earlier_one() {
    let project = running_task(&[("valor.txt", "a\n")]);
    write(&project, "valor.txt", "b\n");
    block_and_accept(&project, "1");
    write(&project, "valor.txt", "c\n");
    block_and_accept(&project, "1");
    assert_eq!(
        read(&project, &format!("{REGRESSION_DIR}/accepted"))
            .lines()
            .count(),
        2
    );
    let passed = stdout(&mut task(&project, &["regression"]));
    assert!(passed.contains("1 aceito(s) pelo usuário"), "{passed}");
}

#[test]
fn evidence_subitems_are_ignored_when_recording() {
    let project = running_task(&[("valor.txt", "a\n")]);
    let content = read(&project, TASK_PATH).replace(
        "### Evidências\n\nNão se aplica.\n",
        "### Evidências\n\n- `saida.txt` (automática): `cat valor.txt`\n  - Prova: o valor é lido.\n  - Trecho: a linha `a`.\n  - Falha: arquivo ausente.\n",
    );
    write(&project, TASK_PATH, &content);
    task(&project, &["tests"]).assert().success();
    task(&project, &["regression"]).assert().success();
    let output = stdout(&mut task(&project, &["evidence"]));
    assert!(
        output.contains("1 item(ns). Resultado: passed."),
        "{output}"
    );
    let evidence = read(
        &project,
        "vostok/work/tasks/SPEC-001/evidence/TASK-01/saida.txt",
    );
    assert_eq!(evidence, "a\n");
}
