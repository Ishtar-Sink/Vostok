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
    running_task_with_evidence(files, "Não se aplica.")
}

/// Como `running_task`, com o texto dado na subseção "Evidências" antes de coletar o baseline.
fn running_task_with_evidence(files: &[(&str, &str)], evidence: &str) -> Project {
    let project = task_before_baseline(files, evidence);
    task(&project, &["baseline"]).assert().success();
    project
}

/// Como `running_task_with_evidence`, mas sem coletar o baseline.
fn task_before_baseline(files: &[(&str, &str)], evidence: &str) -> Project {
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
            "---\nid: TASK-01\nspec: SPEC-001\ntitle: \"Tarefa\"\nstatus: in_progress\nattempts: 0\ntests_result: null\nregression_result: null\nevidence_result: null\n---\n\n# TASK-01: Tarefa\n\n## Execução\n\n### Baseline\n\n{baseline}\n### Verificação\n\n```bash\ntrue\n```\n\n### Evidências\n\n{evidence}\n\n### Concluída quando\n\n- [x] Pronto.\n\n  > Como comprovar: ver.\n\n## Descrição\n\nTexto.\n\n## Registro de execução\n\n| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n"
        ),
    );
    write(
        &project,
        "vostok/work/tasks/SPEC-001/TASK-02.md",
        "---\nid: TASK-02\nstatus: pending\n---\n",
    );
    project
}

/// Saída gravada numa evidência, sem o cabeçalho do log (até a primeira linha em branco).
fn log_body(text: &str) -> &str {
    text.split_once("\n\n").map_or(text, |(_, body)| body)
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
    assert_eq!(fields.len(), 6, "{accepted}");
    assert_eq!(fields[0], "1");
    assert_eq!(fields[1].len(), 64);
    assert_eq!(fields[3], "cat valor.txt");
    assert_eq!(fields[4], "diferença esperada");
    assert_eq!(fields[5], "0");

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
    // Outro código de saída bloqueia de novo; o aceite novo passa a valer pelo código 1.
    std::fs::remove_file(project.root().join("valor.txt")).unwrap();
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

// --- Linhas ignoradas e aceite por código de saída (SPEC-013) ---

/// Remove a chave `regression_ignore` da configuração, como num projeto antigo.
fn without_regression_ignore(project: &Project) {
    let path = "vostok/config/tasks.yml";
    let mut skipping = false;
    let content: String = read(project, path)
        .lines()
        .filter(|line| {
            if line.starts_with("regression_ignore:") {
                skipping = true;
                return false;
            }
            if skipping && line.trim_start().starts_with("- ") {
                return false;
            }
            skipping = false;
            true
        })
        .map(|line| format!("{line}\n"))
        .collect();
    assert!(!content.contains("regression_ignore"));
    write(project, path, &content);
}

#[test]
fn regression_ignore_skips_lines_with_the_configured_snippets() {
    let project = running_task(&[("valor.txt", "ok\nDuration 1ms\nTime: 10:00:01\n")]);
    write(&project, "valor.txt", "ok\nDuration 2ms\nTime: 10:00:02\n");
    task(&project, &["tests"]).assert().success();
    let passed = stdout(&mut task(&project, &["regression"]));
    assert_eq!(
        passed,
        "Sem regressão em TASK-01: 1 comando(s) iguais ao baseline.\n"
    );
    assert!(!exists(&project, &format!("{REGRESSION_DIR}/01.diff")));
}

#[test]
fn regression_ignore_without_the_key_compares_the_whole_output() {
    let project = running_task(&[("valor.txt", "ok\nDuration 1ms\n")]);
    without_regression_ignore(&project);
    write(&project, "valor.txt", "ok\nDuration 2ms\n");
    task(&project, &["tests"]).assert().success();
    let blocked = stdout(&mut task(&project, &["regression"]));
    assert!(blocked.contains("- [1] cat valor.txt\n"), "{blocked}");
    assert_eq!(status(&project), "blocked");
}

#[test]
fn regression_ignore_keeps_the_whole_output_in_the_diff() {
    let project = running_task(&[("valor.txt", "ok\nDuration 1ms\n")]);
    write(&project, "valor.txt", "mudou\nDuration 2ms\n");
    task(&project, &["tests"]).assert().success();
    let blocked = stdout(&mut task(&project, &["regression"]));
    assert!(blocked.contains("- [1] cat valor.txt\n"), "{blocked}");
    assert_eq!(status(&project), "blocked");
    let diff = read(&project, &format!("{REGRESSION_DIR}/01.diff"));
    assert!(diff.contains("> mudou"), "{diff}");
    assert!(diff.contains("> Duration 2ms"), "{diff}");
    assert!(diff.contains("< Duration 1ms"), "{diff}");
}

#[test]
fn regression_accept_holds_while_the_exit_code_is_the_same() {
    let project = running_task(&[("valor.txt", "a\n")]);
    write(&project, "valor.txt", "b\n");
    block_and_accept(&project, "1");
    let accepted = read(&project, &format!("{REGRESSION_DIR}/accepted"));
    assert!(
        accepted.ends_with("\tdiferença esperada\t0\n"),
        "{accepted}"
    );

    for value in ["c\n", "d\ne\n"] {
        write(&project, "valor.txt", value);
        task(&project, &["tests"]).assert().success();
        let passed = stdout(&mut task(&project, &["regression"]));
        assert!(
            passed.contains("0 igual(is) ao baseline, 1 aceito(s) pelo usuário"),
            "{passed}"
        );
    }
    task(&project, &["evidence"]).assert().success();
    task(&project, &["finish"]).assert().success();
    assert_eq!(status(&project), "done");
}

#[test]
fn regression_accept_blocks_when_the_exit_code_changes() {
    let project = running_task(&[("valor.txt", "a\n")]);
    write(&project, "valor.txt", "b\n");
    block_and_accept(&project, "1");
    std::fs::remove_file(project.root().join("valor.txt")).unwrap();
    task(&project, &["tests"]).assert().success();
    let blocked = stdout(&mut task(&project, &["regression"]));
    assert!(blocked.contains("- [1] cat valor.txt\n"), "{blocked}");
    assert!(blocked.contains("> exit: 1"), "{blocked}");
    assert_eq!(status(&project), "blocked");
}

#[test]
fn regression_accept_old_line_without_exit_code_holds_by_the_hash() {
    let project = running_task(&[("valor.txt", "a\n")]);
    write(&project, "valor.txt", "b\n");
    block_and_accept(&project, "1");
    // Volta a linha ao formato antigo, de 5 campos, sem o código de saída.
    let path = format!("{REGRESSION_DIR}/accepted");
    let line = read(&project, &path);
    let old = line.trim_end().rsplit_once('\t').unwrap().0.to_string();
    assert_eq!(old.split('\t').count(), 5, "{old}");
    write(&project, &path, &format!("{old}\n"));

    task(&project, &["tests"]).assert().success();
    let passed = stdout(&mut task(&project, &["regression"]));
    assert!(passed.contains("1 aceito(s) pelo usuário"), "{passed}");
    assert_eq!(read(&project, &path), format!("{old}\n"));

    // Sem o código de saída, o aceite antigo vale só para a saída aceita.
    write(&project, "valor.txt", "c\n");
    task(&project, &["tests"]).assert().success();
    let blocked = stdout(&mut task(&project, &["regression"]));
    assert!(blocked.contains("- [1] cat valor.txt\n"), "{blocked}");
}

#[test]
fn regression_accept_old_line_is_listed_in_the_persist_guide() {
    let project = validated_task("b\n");
    let path = format!("{EVIDENCE_DIR}/validation/regression/accepted");
    let line = read(&project, &path);
    let old = line.trim_end().rsplit_once('\t').unwrap().0.to_string();
    write(&project, &path, &format!("{old}\n"));
    task(&project, &["persist"]).assert().success();
    assert!(
        guide(&project).contains("`cat valor.txt`: valor novo esperado ([diff]"),
        "{}",
        guide(&project)
    );
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
    assert_eq!(log_body(&evidence), "a\n");
}

// --- Evidências que só passam com a mudança (SPEC-014) ---

/// Remove a linha `evidence_baseline_dir:` da configuração, como num projeto antigo.
fn without_evidence_baseline(project: &Project) {
    let path = "vostok/config/tasks.yml";
    let content: String = read(project, path)
        .lines()
        .filter(|line| !line.starts_with("evidence_baseline_dir:"))
        .map(|line| format!("{line}\n"))
        .collect();
    write(project, path, &content);
}

const AUTOMATIC_EVIDENCE: &str = "- `saida.txt` (automática): `cat valor.txt`\n  - Prova: o valor é lido.\n  - Trecho: a linha `a`.\n  - Falha: arquivo ausente.";

fn evidence_baseline_file(project: &Project, name: &str) -> String {
    read(project, &format!("{EVIDENCE_DIR}/baseline/evidence/{name}"))
}

#[test]
fn evidence_baseline_records_the_automatic_evidence_output() {
    let evidence = format!("{AUTOMATIC_EVIDENCE}\n- `tela.png` (manual): print");
    let project = running_task_with_evidence(&[("valor.txt", "a\n")], &evidence);
    assert_eq!(
        evidence_baseline_file(&project, "saida.txt"),
        "$ cat valor.txt\nexit: 0\na\n"
    );
    assert!(!exists(
        &project,
        &format!("{EVIDENCE_DIR}/baseline/evidence/tela.png")
    ));
    let registro = read(&project, TASK_PATH);
    assert!(
        registro.contains("1 evidência(s) automática(s) gravada(s) para comparação"),
        "{registro}"
    );
}

#[test]
fn evidence_baseline_same_output_warns_and_passes() {
    let project = running_task_with_evidence(&[("valor.txt", "a\n")], AUTOMATIC_EVIDENCE);
    task(&project, &["tests"]).assert().success();
    task(&project, &["regression"]).assert().success();
    let output = stdout(&mut task(&project, &["evidence"]));
    assert!(
        output.contains("Aviso: evidência(s) com a mesma saída do baseline, que não comprovam a mudança da tarefa:"),
        "{output}"
    );
    assert!(output.contains("- saida.txt: `cat valor.txt`"), "{output}");
    assert!(read(&project, TASK_PATH).contains("evidence_result: passed"));
    assert_eq!(
        log_body(&read(&project, &format!("{EVIDENCE_DIR}/saida.txt"))),
        "a\n"
    );
}

#[test]
fn evidence_baseline_changed_output_does_not_warn() {
    let evidence = AUTOMATIC_EVIDENCE.replace("valor.txt", "outro.txt");
    let project = task_before_baseline(&[("valor.txt", "a\n")], &evidence);
    write(&project, "outro.txt", "a\n");
    task(&project, &["baseline"]).assert().success();
    write(&project, "outro.txt", "b\n");
    task(&project, &["tests"]).assert().success();
    task(&project, &["regression"]).assert().success();
    let output = stdout(&mut task(&project, &["evidence"]));
    assert!(output.contains("Resultado: passed."), "{output}");
    assert!(!output.contains("Aviso:"), "{output}");
}

#[test]
fn evidence_baseline_command_that_failed_before_does_not_warn() {
    let evidence = AUTOMATIC_EVIDENCE.replace("valor.txt", "novo.txt");
    let project = running_task_with_evidence(&[("valor.txt", "a\n")], &evidence);
    assert!(evidence_baseline_file(&project, "saida.txt").contains("exit: 1"));
    write(&project, "novo.txt", "a\n");
    task(&project, &["tests"]).assert().success();
    task(&project, &["regression"]).assert().success();
    let output = stdout(&mut task(&project, &["evidence"]));
    assert!(output.contains("Resultado: passed."), "{output}");
    assert!(!output.contains("Aviso:"), "{output}");
}

#[test]
fn evidence_baseline_ignored_lines_count_as_equal() {
    let evidence = AUTOMATIC_EVIDENCE.replace("valor.txt", "tempo.txt");
    let project = running_task_with_evidence(&[("tempo.txt", "ok\nDuration: 1s\n")], &evidence);
    write(&project, "tempo.txt", "ok\nDuration: 2s\n");
    task(&project, &["tests"]).assert().success();
    task(&project, &["regression"]).assert().success();
    let output = stdout(&mut task(&project, &["evidence"]));
    assert!(output.contains("Aviso:"), "{output}");
    assert!(output.contains("- saida.txt: `cat tempo.txt`"), "{output}");
}

#[test]
fn evidence_baseline_without_the_key_records_nothing() {
    let project = task_before_baseline(&[("valor.txt", "a\n")], AUTOMATIC_EVIDENCE);
    without_evidence_baseline(&project);
    let output = stdout(&mut task(&project, &["baseline"]));
    assert!(
        output
            .trim_end()
            .ends_with("1 comando(s); códigos de saída: 0"),
        "{output}"
    );
    assert!(!exists(
        &project,
        &format!("{EVIDENCE_DIR}/baseline/evidence")
    ));
    task(&project, &["tests"]).assert().success();
    task(&project, &["regression"]).assert().success();
    let output = stdout(&mut task(&project, &["evidence"]));
    assert!(output.contains("Resultado: passed."), "{output}");
    assert!(!output.contains("Aviso:"), "{output}");
}

// --- Persistência das evidências no projeto (SPEC-009) ---

const PERSIST_DIR: &str = "docs/evidencias/SPEC-001/TASK-01";
const EVIDENCE_DIR: &str = "vostok/work/tasks/SPEC-001/evidence/TASK-01";
const PNG: &[u8] = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR";

fn set_config(project: &Project, key: &str, value: &str) {
    let path = "vostok/config/tasks.yml";
    let content: String = read(project, path)
        .lines()
        .map(|line| {
            if line.starts_with(&format!("{key}:")) {
                format!("{key}: {value}\n")
            } else {
                format!("{line}\n")
            }
        })
        .collect();
    write(project, path, &content);
}

/// Tarefa validada, com uma evidência automática (`saida.txt`, de `cat valor.txt`) e uma manual
/// (`tela.png`), com a persistência ligada.
fn validated_task(value: &str) -> Project {
    let project = running_task(&[("valor.txt", "a\n")]);
    set_config(&project, "persist_dir", "docs/evidencias/<SPEC>/<TASK>");
    let content = read(&project, TASK_PATH).replace(
        "### Evidências\n\nNão se aplica.\n",
        "### Evidências\n\n- `saida.txt` (automática): `cat valor.txt`\n  - Prova: o valor é lido.\n  - Trecho: a linha com o valor.\n  - Falha: arquivo ausente.\n- `tela.png` (manual): print da tela\n  - Prova: a tela abre.\n  - Trecho: o título.\n  - Falha: tela em branco.\n",
    );
    write(&project, TASK_PATH, &content);
    write(&project, "valor.txt", value);
    write(&project, &format!("{EVIDENCE_DIR}/tela.png"), "");
    std::fs::write(project.root().join(format!("{EVIDENCE_DIR}/tela.png")), PNG).unwrap();
    task(&project, &["tests"]).assert().success();
    let _ = task(&project, &["regression"]).output();
    if status(&project) == "blocked" {
        task(
            &project,
            &["resume", "--accept-regression", "1", "valor novo esperado"],
        )
        .assert()
        .success();
        task(&project, &["tests"]).assert().success();
        task(&project, &["regression"]).assert().success();
    }
    task(&project, &["evidence"]).assert().success();
    project
}

fn guide(project: &Project) -> String {
    read(project, &format!("{PERSIST_DIR}/README.md"))
}

fn fill_trechos(project: &Project) {
    let filled = guide(project).replace("#{TRECHO}", "a linha `a`.");
    write(project, &format!("{PERSIST_DIR}/README.md"), &filled);
}

#[test]
fn persisted_evidence_has_files_and_a_complete_guide() {
    let project = validated_task("a\n");
    let output = stdout(&mut task(&project, &["persist"]));
    assert!(output.contains("Binários sem revisão"), "{output}");
    assert!(output.contains("- tela.png"));
    for file in [
        "saida.txt",
        "tela.png",
        "baseline/01.out",
        "validation/tests.out",
        "README.md",
    ] {
        assert!(exists(&project, &format!("{PERSIST_DIR}/{file}")), "{file}");
    }
    let text = guide(&project);
    assert!(text.starts_with("# TASK-01: Tarefa\n"), "{text}");
    assert!(text.contains("## saida.txt\n"));
    assert!(text.contains("- **Prova:** o valor é lido."));
    assert!(text.contains("- **Como gerar:** `cat valor.txt`"));
    assert!(text.contains("- **Arquivo:** [saida.txt](saida.txt)"));
    assert!(text.contains("- **O que procurar:** a linha com o valor."));
    assert!(text.contains("- **Sinal de falha:** arquivo ausente."));
    assert!(text.contains("- **Revisão:** #{REVISAO}"));
    assert!(text.contains("[baseline/01.out](baseline/01.out)"));

    let refused = failure_stderr(&mut task(&project, &["finish"]));
    assert!(refused.contains("campo a preencher"), "{refused}");

    task(&project, &["persist", "--reviewed", "tela.png=agente"])
        .assert()
        .success();
    fill_trechos(&project);
    task(&project, &["finish"]).assert().success();
    assert_eq!(status(&project), "done");
    assert!(guide(&project).contains("revisado pelo agente em"));
}

#[test]
fn filled_fields_survive_a_new_persist() {
    let project = validated_task("a\n");
    task(&project, &["persist", "--reviewed", "tela.png=usuário"])
        .assert()
        .success();
    fill_trechos(&project);
    task(&project, &["persist"]).assert().success();
    let text = guide(&project);
    assert!(
        text.contains("- **Trecho que prova:** a linha `a`."),
        "{text}"
    );
    assert!(text.contains("revisado pelo usuário em"));
}

#[test]
fn personal_paths_become_a_tilde_only_in_the_copy() {
    let project = validated_task("/Users/fulano/x e /home/beltrano/y\n");
    task(&project, &["persist"]).assert().success();
    assert_eq!(
        log_body(&read(&project, &format!("{PERSIST_DIR}/saida.txt"))),
        "~/x e ~/y\n"
    );
    assert_eq!(
        log_body(&read(&project, &format!("{EVIDENCE_DIR}/saida.txt"))),
        "/Users/fulano/x e /home/beltrano/y\n"
    );
}

#[test]
fn sensitive_data_blocks_the_copy_until_the_user_decides() {
    let project = validated_task("token: sk-abcdefghijklmnopqrstuvwx\n");
    let refused = failure_stderr(&mut task(&project, &["persist"]));
    // A linha 6 é a primeira da saída, depois das 5 linhas do cabeçalho do log.
    assert!(refused.contains("- saida.txt:6: segredo"), "{refused}");
    assert!(
        refused.contains("- validation/regression/01.out:3: segredo"),
        "{refused}"
    );
    assert!(!exists(&project, PERSIST_DIR));

    let diff = "validation/regression/accepted-01.diff";
    assert!(
        refused.contains(&format!("- {diff}:4: segredo")),
        "{refused}"
    );
    let files = ["saida.txt", "validation/regression/01.out"];
    task(
        &project,
        &[
            "persist", "--redact", files[0], "--redact", files[1], "--redact", diff,
        ],
    )
    .assert()
    .success();
    assert_eq!(
        log_body(&read(&project, &format!("{PERSIST_DIR}/saida.txt"))),
        "token: [REDIGIDO]\n"
    );
    assert!(guide(&project).contains("redigido por decisão do usuário"));

    task(
        &project,
        &[
            "persist",
            "--exclude",
            files[0],
            "--accept",
            files[1],
            "--accept",
            diff,
        ],
    )
    .assert()
    .success();
    assert!(!exists(&project, &format!("{PERSIST_DIR}/saida.txt")));
    assert!(guide(&project).contains("excluído por decisão do usuário"));
    assert!(read(&project, &format!("{PERSIST_DIR}/{}", files[1])).contains("sk-abcdefghij"));
}

#[test]
fn ips_emails_and_cpfs_block_but_loopback_does_not() {
    for (value, kind) in [
        ("servidor 10.1.2.3:8080\n", "IP"),
        ("contato fulano@exemplo.com\n", "e-mail"),
        ("cpf 123.456.789-09\n", "CPF"),
        ("Authorization: Bearer abcdefgh12345678\n", "segredo"),
    ] {
        let project = validated_task(value);
        let refused = failure_stderr(&mut task(&project, &["persist"]));
        assert!(
            refused.contains(&format!("saida.txt:6: {kind}")),
            "{value}: {refused}"
        );
    }
    let project = validated_task("ouvindo em 127.0.0.1:8080 e 0.0.0.0\n");
    task(&project, &["persist"]).assert().success();
}

#[test]
fn finish_refuses_without_the_guide() {
    let project = validated_task("a\n");
    task(&project, &["persist", "--reviewed", "tela.png=agente"])
        .assert()
        .success();
    fill_trechos(&project);
    std::fs::remove_file(project.root().join(format!("{PERSIST_DIR}/README.md"))).unwrap();
    let refused = failure_stderr(&mut task(&project, &["finish"]));
    assert!(refused.contains("evidências não persistidas"), "{refused}");
    assert_eq!(status(&project), "validating");
}

#[test]
fn finish_without_persist_dir_works_as_before() {
    let project = validated_task("a\n");
    set_config(&project, "persist_dir", "\"\"");
    task(&project, &["finish"]).assert().success();
    assert!(!exists(&project, "docs"));
}

#[test]
fn done_tasks_can_be_persisted() {
    let project = validated_task("a\n");
    set_config(&project, "persist_dir", "\"\"");
    task(&project, &["finish"]).assert().success();
    set_config(&project, "persist_dir", "docs/evidencias/<SPEC>/<TASK>");
    task(&project, &["persist"]).assert().success();
    assert!(exists(&project, &format!("{PERSIST_DIR}/README.md")));
}

#[test]
fn guide_lists_the_accepted_regressions() {
    let project = validated_task("b\n");
    task(&project, &["persist"]).assert().success();
    let text = guide(&project);
    assert!(
        text.contains(
            "`cat valor.txt`: valor novo esperado ([diff](validation/regression/accepted-01.diff))"
        ),
        "{text}"
    );
    assert!(exists(
        &project,
        &format!("{PERSIST_DIR}/validation/regression/accepted-01.diff")
    ));
}

#[test]
fn files_above_the_limit_are_registered_but_not_copied() {
    let project = validated_task("a\n");
    set_config(&project, "persist_max_mb", "0.000005");
    task(&project, &["persist"]).assert().success();
    assert!(!exists(&project, &format!("{PERSIST_DIR}/tela.png")));
    let text = guide(&project);
    assert!(text.contains("tela.png (não copiado:"), "{text}");
    assert!(text.contains("SHA-256 `"));
    assert!(text.contains(&format!("original em `{EVIDENCE_DIR}/tela.png`")));
}

#[test]
fn guide_name_comes_from_the_configuration() {
    let project = validated_task("a\n");
    set_config(&project, "persist_readme", "LEIA-ME.md");
    task(&project, &["persist", "--reviewed", "tela.png=agente"])
        .assert()
        .success();
    let path = format!("{PERSIST_DIR}/LEIA-ME.md");
    let filled = read(&project, &path).replace("#{TRECHO}", "ok.");
    write(&project, &path, &filled);
    assert!(!exists(&project, &format!("{PERSIST_DIR}/README.md")));
    task(&project, &["finish"]).assert().success();
}

#[test]
fn persist_refuses_foreign_files_and_unfinished_tasks() {
    let project = validated_task("a\n");
    write(&project, &format!("{PERSIST_DIR}/outro.txt"), "do time\n");
    let refused = failure_stderr(&mut task(&project, &["persist"]));
    assert!(refused.contains("- outro.txt"), "{refused}");
    assert!(exists(&project, &format!("{PERSIST_DIR}/outro.txt")));

    let running = running_task(&[("valor.txt", "a\n")]);
    set_config(&running, "persist_dir", "docs/evidencias/<SPEC>/<TASK>");
    let refused = failure_stderr(&mut task(&running, &["persist"]));
    assert!(refused.contains("status 'in_progress'"), "{refused}");
}

#[test]
fn persist_is_refused_when_disabled() {
    let project = validated_task("a\n");
    set_config(&project, "persist_dir", "\"\"");
    let refused = failure_stderr(&mut task(&project, &["persist"]));
    assert!(
        refused.contains("a persistência está desligada"),
        "{refused}"
    );
}

// --- Nome de evidência inválido recusado antes da execução (SPEC-010) ---

/// Tarefa completa e válida com o status e a lista de "Evidências" informados.
fn complete_task(status: &str, evidence: &str) -> String {
    format!(
        "---\nid: TASK-01\nspec: SPEC-001\ntitle: \"Tarefa\"\neffort: simples\ndepends_on: []\ncovers: [RF-01]\ntype: tarefa\nlabels: [SPEC-001, simples]\nstatus: {status}\ncontent_hash: null\nadjustments: 0\nadjusted_at: null\napproved_at: null\napproved_hash: null\nattempts: 0\ntests_result: null\nregression_result: null\nevidence_result: null\n---\n\n# TASK-01: Tarefa\n\n## Execução\n\n### Ler\n\n- `README.md`\n\n### Mudar\n\n- `novo.txt` (criar): arquivo novo.\n\n### Passos\n\n1. Crie o arquivo.\n\n### Baseline\n\n- `true`\n\n### Verificação\n\n```bash\ntrue\n```\n\n### Evidências\n\n{evidence}\n\n### Concluída quando\n\n- [ ] Pronto (RF-01).\n\n  > Como comprovar: ver o arquivo.\n\n## Descrição\n\n### Resumo executivo\n\nResumo.\n\n### Detalhes técnicos\n\nDetalhes.\n\n### Card\n\n- **Título:** Tarefa\n- **Tipo:** tarefa\n- **Descrição:** Descrição.\n- **Critérios de aceite:**\n  - [ ] Pronto.\n- **Rótulos:** SPEC-001, simples\n- **Estimativa:** simples\n- **Bloqueada por:** -\n\n## Registro de execução\n\n| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n\n## Histórico de ajustes\n\n| # | Data | Origem | Resumo |\n|---|---|---|---|\n"
    )
}

fn task_with_evidence(status: &str, evidence: &str) -> Project {
    let project = Project::new();
    write(&project, TASK_PATH, &complete_task(status, evidence));
    project
}

/// `vostok task <grupo> <comando> SPEC-001 TASK-01`.
fn nested(project: &Project, group: &str, command: &str) -> assert_cmd::Command {
    let mut vostok = project.vostok();
    vostok.args(["task", group, command, "SPEC-001", "TASK-01"]);
    vostok
}

/// Saída de um comando que precisa terminar sem erro.
fn success_stdout(command: &mut assert_cmd::Command) -> String {
    let output = command.assert().success().get_output().stdout.clone();
    String::from_utf8(output).unwrap()
}

const PATH_EVIDENCE: &str = "- `docs/x/saida.txt` (automática): `echo ok`";

#[test]
fn evidence_name_with_a_path_is_accepted_with_a_warning_by_the_task_adjust_validation() {
    let project = task_with_evidence("pending", PATH_EVIDENCE);
    let before = read(&project, TASK_PATH);
    let output = success_stdout(&mut nested(&project, "adjust", "validate"));
    assert!(
        output.contains("TASK-01: docs/x/saida.txt → saida.txt"),
        "{output}"
    );
    assert!(output.contains("nada foi alterado"), "{output}");
    assert_eq!(read(&project, TASK_PATH), before);
}

#[test]
fn evidence_name_with_a_path_is_renamed_by_the_task_adjust_record() {
    let subitems = "  - Prova: o comando roda.\n  - Trecho: a linha `ok`.\n  - Falha: saída vazia.";
    let project = task_with_evidence(
        "pending",
        &format!("{PATH_EVIDENCE}\n{subitems}\n- `notas.md` (manual): conferir a tela."),
    );
    let mut record = nested(&project, "adjust", "record");
    record.arg("Nome de evidência trocado");
    let output = success_stdout(&mut record);
    assert!(
        output.contains("TASK-01: docs/x/saida.txt → saida.txt"),
        "{output}"
    );
    let content = read(&project, TASK_PATH);
    // Só o nome muda: o tipo, o comando e os subitens ficam.
    let expected = format!(
        "### Evidências\n\n- `saida.txt` (automática): `echo ok`\n{subitems}\n- `notas.md` (manual): conferir a tela.\n"
    );
    assert!(content.contains(&expected), "{content}");
    assert!(!content.contains("docs/x/"), "{content}");
    // O ajuste foi registrado no histórico, com o hash do conteúdo já trocado.
    assert!(content.contains("adjustments: 1\n"), "{content}");
    let history = content.split("## Histórico de ajustes").nth(1).unwrap();
    assert!(
        history
            .lines()
            .any(|line| line.starts_with("| 1 |") && line.contains("Nome de evidência trocado")),
        "{history}"
    );
    nested(&project, "approve", "validate").assert().success();
}

#[test]
fn evidence_name_with_a_path_is_renamed_in_an_approved_task_by_the_adjust_record() {
    let project = task_with_evidence("approved", PATH_EVIDENCE);
    let mut record = nested(&project, "adjust", "record");
    record.arg("Nome de evidência trocado");
    let output = success_stdout(&mut record);
    assert!(
        output.contains("TASK-01: docs/x/saida.txt → saida.txt"),
        "{output}"
    );
    assert!(read(&project, TASK_PATH).contains("- `saida.txt` (automática): `echo ok`"));
    // A troca reescreve a tarefa: a aprovação cai.
    assert_eq!(status(&project), "pending");
}

#[test]
fn evidence_name_ambiguous_or_reserved_is_refused_by_the_task_adjust() {
    let cases = [
        (
            "- `a/saida.txt` (automática): `echo a`\n- `b/saida.txt` (automática): `echo b`",
            "saida.txt",
        ),
        ("- `docs/baseline` (automática): `echo ok`", "docs/baseline"),
        (
            "- `docs/evidencias/` (automática): `echo ok`",
            "docs/evidencias/",
        ),
    ];
    for (evidence, name) in cases {
        let project = task_with_evidence("pending", evidence);
        let before = read(&project, TASK_PATH);
        let refused = failure_stderr(&mut nested(&project, "adjust", "validate"));
        assert!(refused.contains(name), "{name}: {refused}");
        assert!(refused.contains("TASK-01:"), "{name}: {refused}");
        let mut record = nested(&project, "adjust", "record");
        record.arg("Ajuste");
        let refused = failure_stderr(&mut record);
        assert!(
            refused.contains("nenhum ajuste registrado"),
            "{name}: {refused}"
        );
        assert_eq!(read(&project, TASK_PATH), before, "{name}");
    }
}

#[test]
fn evidence_name_with_a_path_is_refused_by_the_task_approval() {
    let project = task_with_evidence("pending", PATH_EVIDENCE);
    let before = read(&project, TASK_PATH);
    let refused = failure_stderr(&mut nested(&project, "approve", "approve"));
    assert!(refused.contains("docs/x/saida.txt"), "{refused}");
    assert!(refused.contains("sem caminho"), "{refused}");
    assert_eq!(status(&project), "pending");
    assert_eq!(read(&project, TASK_PATH), before);
}

#[test]
fn evidence_name_with_a_path_is_refused_by_the_task_start() {
    let project = task_with_evidence("approved", PATH_EVIDENCE);
    let before = read(&project, TASK_PATH);
    let refused = failure_stderr(&mut task(&project, &["start"]));
    assert!(refused.contains("docs/x/saida.txt"), "{refused}");
    assert!(refused.contains("adjust-task"), "{refused}");
    assert_eq!(status(&project), "approved");
    assert_eq!(read(&project, TASK_PATH), before);
}

#[test]
fn evidence_name_valid_is_accepted_by_the_task_validation() {
    let project = task_with_evidence("pending", "- `saida.txt` (automática): `echo ok`");
    let output = success_stdout(&mut nested(&project, "adjust", "validate"));
    assert!(!output.contains('→'), "{output}");
}

#[test]
fn evidence_name_reserved_or_hidden_is_refused_by_the_task_validation() {
    for name in [".saida.txt", "baseline", "validation"] {
        let project = task_with_evidence("pending", &format!("- `{name}` (automática): `echo ok`"));
        let refused = failure_stderr(&mut nested(&project, "adjust", "validate"));
        assert!(refused.contains(&format!("`{name}`")), "{name}: {refused}");
    }
}

// --- Ajuste de tarefa em execução com criar e remover (SPEC-012) ---

const VALID_EVIDENCE: &str = "- `saida.txt` (automática): `echo ok`";

/// Tarefa completa com o status e o item de "Mudar" informados.
fn task_with_change(status: &str, change: &str) -> Project {
    let project = Project::new();
    let content = complete_task(status, VALID_EVIDENCE)
        .replace("- `novo.txt` (criar): arquivo novo.", change);
    write(&project, TASK_PATH, &content);
    project
}

#[test]
fn change_in_execution_accepts_create_of_an_existing_file() {
    let project = task_with_change("in_progress", "- `novo.txt` (criar): arquivo novo.");
    write(&project, "novo.txt", "criado pela tarefa\n");
    success_stdout(&mut nested(&project, "adjust", "validate"));
    let mut record = nested(&project, "adjust", "record");
    record.arg("Passos detalhados");
    let output = success_stdout(&mut record);
    assert!(output.contains("Ajuste 1 registrado"), "{output}");
    let content = read(&project, TASK_PATH);
    assert!(content.contains("adjustments: 1\n"), "{content}");
    assert_eq!(status(&project), "in_progress");
}

#[test]
fn change_in_execution_accepts_remove_of_a_missing_file() {
    let project = task_with_change("validating", "- `velho.txt` (remover): arquivo antigo.");
    success_stdout(&mut nested(&project, "adjust", "validate"));
}

#[test]
fn change_in_execution_still_checks_alter_of_a_missing_path() {
    let project = task_with_change("in_progress", "- `ausente.txt` (alterar): arquivo.");
    let refused = failure_stderr(&mut nested(&project, "adjust", "validate"));
    assert!(refused.contains("'ausente.txt' não existe"), "{refused}");
}

#[test]
fn change_in_execution_does_not_relax_a_pending_task() {
    let project = task_with_change("pending", "- `novo.txt` (criar): arquivo novo.");
    write(&project, "novo.txt", "já existia\n");
    let refused = failure_stderr(&mut nested(&project, "adjust", "validate"));
    assert!(refused.contains("'novo.txt' já existe"), "{refused}");
}

// --- Perfil e modelo da tarefa (SPEC-014) ---

fn task_with_profile(line: &str) -> Project {
    let project = Project::new();
    let content = complete_task("pending", "Não se aplica.").replace("effort: simples", line);
    write(&project, TASK_PATH, &content);
    project
}

fn set_overrides(project: &Project, overrides: &str) {
    let path = "vostok/config/models.yml";
    let content =
        read(project, path).replace("overrides:\n", &format!("overrides:\n{overrides}\n"));
    write(project, path, &content);
}

fn load_output(project: &Project) -> String {
    success_stdout(&mut task(project, &["load"]))
}

#[test]
fn model_load_shows_profile_description_and_reserve_alias() {
    let project = task_with_profile("profile: julgamento");
    let output = load_output(&project);
    for expected in [
        "## Modelo",
        "- Perfil: julgamento",
        "- Descrição: Desenho, ambiguidade e decisões amplas; prefere o modelo mais capaz.",
        "- Ferramenta: claude-code",
        "- Apelido de reserva: opus",
    ] {
        assert!(output.contains(expected), "{expected}\n{output}");
    }
    assert!(output.find("## Modelo").unwrap() < output.find("## Execução").unwrap());
}

#[test]
fn model_load_maps_the_old_effort_level() {
    let project = task_with_profile("effort: moderada");
    let output = load_output(&project);
    assert!(output.contains("- Perfil: padrao"), "{output}");
    assert!(output.contains("- Apelido de reserva: sonnet"), "{output}");
}

#[test]
fn model_load_uses_the_project_override() {
    let project = task_with_profile("profile: julgamento");
    set_overrides(&project, "  julgamento:\n    claude-code: sonnet");
    let output = load_output(&project);
    assert!(output.contains("- Apelido de reserva: sonnet"), "{output}");
}

#[test]
fn model_load_refuses_an_invalid_override() {
    let project = task_with_profile("profile: julgamento");
    set_overrides(&project, "  julgamento:\n    claude-code: \"Opus 4!\"");
    let stderr = failure_stderr(&mut task(&project, &["load"]));
    assert!(
        stderr.contains(
            "apelido inválido em config/models.yml (overrides.julgamento.claude-code): 'Opus 4!'"
        ),
        "{stderr}"
    );
}

#[test]
fn model_load_without_tool_shows_the_session_model() {
    let project = task_with_profile("profile: julgamento");
    let main = read(&project, "vostok/config/main.yml");
    let main: String = main
        .lines()
        .map(|line| {
            if line.starts_with("  tool:") {
                "  tool: \"\"".to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    write(&project, "vostok/config/main.yml", &main);
    let mut command = task(&project, &["load"]);
    let assert = command.assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    let stderr = String::from_utf8(output.stderr.clone()).unwrap();
    assert!(stdout.contains("- Ferramenta: não configurada"), "{stdout}");
    assert!(
        stdout.contains("- Apelido de reserva: modelo da sessão"),
        "{stdout}"
    );
    assert!(
        stderr.contains("Aviso: harness.tool vazio ou desconhecido"),
        "{stderr}"
    );
}

fn started_task(extra: &[&str]) -> Project {
    let project = Project::new();
    write(
        &project,
        TASK_PATH,
        &complete_task("pending", "Não se aplica."),
    );
    success_stdout(&mut nested(&project, "adjust", "record-manual"));
    success_stdout(&mut nested(&project, "approve", "approve"));
    let mut args = vec!["start"];
    args.extend_from_slice(extra);
    success_stdout(&mut task(&project, &args));
    project
}

#[test]
fn model_start_records_the_chosen_model() {
    let project = started_task(&["--model", "sonnet"]);
    let content = read(&project, TASK_PATH);
    assert!(content.contains("\nmodel: sonnet\n"), "{content}");
    let line = content.lines().find(|l| l.contains("| início |")).unwrap();
    assert!(line.ends_with("; modelo: sonnet |"), "{line}");
}

#[test]
fn model_start_without_model_keeps_the_record() {
    let project = started_task(&[]);
    let content = read(&project, TASK_PATH);
    assert!(!content.contains("\nmodel:"), "{content}");
    let line = content.lines().find(|l| l.contains("| início |")).unwrap();
    assert!(!line.contains("modelo:"), "{line}");
}

// --- Baseline garantido no início (SPEC-016) ---

const BASELINE_DIR: &str = "vostok/work/tasks/SPEC-001/evidence/TASK-01/baseline";

fn approved_task(baseline: &str) -> Project {
    let project = Project::new();
    let content = complete_task("pending", "Não se aplica.").replace(
        "### Baseline\n\n- `true`\n",
        &format!("### Baseline\n\n{baseline}\n"),
    );
    write(&project, TASK_PATH, &content);
    success_stdout(&mut nested(&project, "adjust", "record-manual"));
    success_stdout(&mut nested(&project, "approve", "approve"));
    project
}

fn in_progress_without_baseline() -> Project {
    let project = Project::new();
    write(
        &project,
        TASK_PATH,
        &complete_task("in_progress", "Não se aplica."),
    );
    project
}

#[test]
fn start_baseline_is_collected_by_the_start() {
    let project = approved_task("- `true`\n- `echo antes`");
    let output = success_stdout(&mut task(&project, &["start"]));
    let content = read(&project, TASK_PATH);
    assert!(content.contains("\nstatus: in_progress\n"), "{content}");
    assert!(exists(&project, &format!("{BASELINE_DIR}/01.out")));
    assert!(exists(&project, &format!("{BASELINE_DIR}/02.out")));
    let start = content.find("| início |").expect("início");
    let baseline = content.find("| baseline |").expect("baseline");
    assert!(start < baseline, "{content}");
    assert!(output.contains("Baseline de TASK-01 coletado"), "{output}");
    assert!(output.contains("códigos de saída: 0, 0"), "{output}");
}

#[test]
fn start_baseline_refuses_an_existing_created_path() {
    let project = approved_task("- `true`");
    write(&project, "novo.txt", "já existe");
    let stderr = failure_stderr(&mut task(&project, &["start"]));
    assert!(stderr.contains("'novo.txt'"), "{stderr}");
    assert!(stderr.contains("a tarefa o cria"), "{stderr}");
    let content = read(&project, TASK_PATH);
    assert!(content.contains("\nstatus: approved\n"), "{content}");
    assert!(!exists(&project, BASELINE_DIR));
    assert!(!content.contains("| início |"), "{content}");
}

#[test]
fn start_progress_failure_leaves_the_task_running() {
    let project = approved_task("- `true`");
    write(&project, "vostok/work/tasks/SPEC-001/evidence", "arquivo");
    failure_stderr(&mut task(&project, &["start"]));
    let content = read(&project, TASK_PATH);
    assert!(content.contains("\nstatus: in_progress\n"), "{content}");
    assert!(content.contains("| início |"), "{content}");
    assert!(!content.contains("| baseline |"), "{content}");
}

#[test]
fn start_baseline_legacy_task_still_collects() {
    let project = task_before_baseline(&[("a.txt", "x")], "Não se aplica.");
    task(&project, &["baseline"]).assert().success();
    assert!(exists(&project, BASELINE_DIR));
}

#[test]
fn start_baseline_legacy_task_refuses_an_existing_created_path() {
    let project = in_progress_without_baseline();
    write(&project, "novo.txt", "já existe");
    let stderr = failure_stderr(&mut task(&project, &["baseline"]));
    assert!(stderr.contains("'novo.txt'"), "{stderr}");
    assert!(!exists(&project, BASELINE_DIR));
}

#[test]
fn start_baseline_tests_refuse_a_task_without_baseline() {
    let project = in_progress_without_baseline();
    let stderr = failure_stderr(&mut task(&project, &["tests"]));
    assert!(
        stderr.contains(
            "o baseline de TASK-01 não foi coletado; execute: vostok task baseline SPEC-001 TASK-01"
        ),
        "{stderr}"
    );
    let content = read(&project, TASK_PATH);
    assert!(content.contains("\nattempts: 0\n"), "{content}");
    assert!(!exists(
        &project,
        "vostok/work/tasks/SPEC-001/evidence/TASK-01/validation/tests.out"
    ));
}

#[test]
fn start_baseline_second_baseline_is_refused() {
    let project = approved_task("- `true`");
    success_stdout(&mut task(&project, &["start"]));
    let stderr = failure_stderr(&mut task(&project, &["baseline"]));
    assert!(stderr.contains("já foi coletado"), "{stderr}");
}

// --- Status das tarefas (SPEC-014) ---

fn status_task(project: &Project, id: &str, lines: &[&str]) {
    let content = format!("---\nid: {id}\n{}\n---\n", lines.join("\n"));
    write(
        project,
        &format!("vostok/work/tasks/SPEC-001/{id}.md"),
        &content,
    );
}

fn status_run(project: &Project) -> assert_cmd::Command {
    let mut command = project.vostok();
    command.args(["task", "status", "SPEC-001"]);
    command
}

#[test]
fn status_without_spec_shows_the_usage() {
    let project = Project::new();
    let stderr = failure_stderr(project.vostok().args(["task", "status"]));
    assert_eq!(stderr, "Erro: Uso: vostok task status <SPEC_ID>\n");
}

#[test]
fn status_rejects_an_invalid_spec_id() {
    let project = Project::new();
    let stderr = failure_stderr(project.vostok().args(["task", "status", "../SPEC-1"]));
    assert_eq!(stderr, "Erro: ID de spec inválido: '../SPEC-1'\n");
}

#[test]
fn status_of_a_spec_without_tasks_is_refused() {
    let project = Project::new();
    let stderr = failure_stderr(project.vostok().args(["task", "status", "SPEC-404"]));
    assert!(
        stderr.starts_with("Erro: a spec SPEC-404 não tem tarefas em "),
        "{stderr}"
    );
}

#[test]
fn status_prints_the_tasks_and_waves_tables() {
    let project = Project::new();
    status_task(
        &project,
        "TASK-01",
        &[
            "title: \"Primeira\"",
            "persona: desenvolvedor",
            "profile: mecanica",
            "wave: 1",
            "status: done",
        ],
    );
    status_task(
        &project,
        "TASK-02",
        &[
            "title: \"Segunda\"",
            "persona: arquiteto",
            "profile: julgamento",
            "model: sonnet",
            "wave: 1",
            "status: in_progress",
        ],
    );
    status_task(
        &project,
        "TASK-03",
        &[
            "title: \"Terceira\"",
            "persona: qa",
            "profile: padrao",
            "wave: 2",
            "status: approved",
        ],
    );
    status_task(
        &project,
        "TASK-04",
        &[
            "title: \"Quarta | com barra\"",
            "effort: critica",
            "wave: 2",
            "status: approved",
        ],
    );
    let output = success_stdout(&mut status_run(&project));
    let expected = "### Tarefas da SPEC-001

| ID | Título | Persona | Perfil | Modelo | Onda | Status |
|---|---|---|---|---|---|---|
| TASK-01 | Primeira | desenvolvedor | mecanica | - | 1 | done |
| TASK-02 | Segunda | arquiteto | julgamento | sonnet | 1 | in_progress |
| TASK-03 | Terceira | qa | padrao | sonnet | 2 | approved |
| TASK-04 | Quarta \\| com barra | - | julgamento | opus | 2 | approved |

### Ondas da SPEC-001

| Onda | Tarefas | Status |
|---|---|---|
| 1 | TASK-01, TASK-02 | in_progress |
| 2 | TASK-03, TASK-04 | approved |

Subagentes ao mesmo tempo: 3
";
    assert_eq!(output, expected);
}

#[test]
fn status_tasks_without_wave_use_the_dependencies() {
    let project = Project::new();
    status_task(&project, "TASK-01", &["title: \"A\"", "status: done"]);
    status_task(
        &project,
        "TASK-02",
        &["title: \"B\"", "depends_on: [TASK-01]", "status: pending"],
    );
    status_task(
        &project,
        "TASK-03",
        &["title: \"C\"", "depends_on: [TASK-02]", "status: pending"],
    );
    let output = success_stdout(&mut status_run(&project));
    for (id, tail) in [
        ("TASK-01", "| 1 | done |"),
        ("TASK-02", "| 2 | pending |"),
        ("TASK-03", "| 3 | pending |"),
    ] {
        let row = output
            .lines()
            .find(|l| l.starts_with(&format!("| {id} |")))
            .unwrap();
        assert!(row.ends_with(tail), "{row}");
    }
    for row in [
        "| 1 | TASK-01 | done |",
        "| 2 | TASK-02 | pending |",
        "| 3 | TASK-03 | pending |",
    ] {
        assert!(output.contains(row), "{output}");
    }
}

#[test]
fn status_tasks_in_a_dependency_cycle_go_to_the_dash_wave() {
    let project = Project::new();
    status_task(
        &project,
        "TASK-01",
        &["title: \"A\"", "depends_on: [TASK-02]", "status: pending"],
    );
    status_task(
        &project,
        "TASK-02",
        &["title: \"B\"", "depends_on: [TASK-01]", "status: pending"],
    );
    let output = success_stdout(&mut status_run(&project));
    for id in ["TASK-01", "TASK-02"] {
        let row = output
            .lines()
            .find(|l| l.starts_with(&format!("| {id} |")))
            .unwrap();
        assert!(row.ends_with("| - | pending |"), "{row}");
    }
    assert!(
        output.contains("| - | TASK-01, TASK-02 | pending |\n"),
        "{output}"
    );
}

#[test]
fn status_model_is_shown_only_when_recorded_or_suggested() {
    let project = Project::new();
    for (id, status) in [
        ("TASK-01", "done"),
        ("TASK-02", "in_progress"),
        ("TASK-03", "pending"),
        ("TASK-04", "approved"),
    ] {
        status_task(
            &project,
            id,
            &[
                "title: \"T\"",
                "profile: padrao",
                "wave: 1",
                &format!("status: {status}"),
            ],
        );
    }
    let output = success_stdout(&mut status_run(&project));
    for (id, model) in [
        ("TASK-01", "-"),
        ("TASK-02", "-"),
        ("TASK-03", "sonnet"),
        ("TASK-04", "sonnet"),
    ] {
        let row = output
            .lines()
            .find(|l| l.starts_with(&format!("| {id} |")))
            .unwrap();
        assert!(row.contains(&format!("| padrao | {model} | 1 |")), "{row}");
    }
}

#[test]
fn status_without_tool_shows_the_session_model() {
    let project = Project::new();
    status_task(
        &project,
        "TASK-01",
        &[
            "title: \"Primeira\"",
            "profile: padrao",
            "wave: 1",
            "status: approved",
        ],
    );
    let main = read(&project, "vostok/config/main.yml");
    let main: String = main
        .lines()
        .map(|line| {
            if line.starts_with("  tool:") {
                "  tool: \"\"".to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    write(&project, "vostok/config/main.yml", &main);
    let mut command = status_run(&project);
    let assert = command.assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    let stderr = String::from_utf8(output.stderr.clone()).unwrap();
    assert!(stdout.contains("| modelo da sessão |"), "{stdout}");
    assert!(
        stderr.contains("Aviso: harness.tool vazio ou desconhecido"),
        "{stderr}"
    );
}

#[test]
fn status_refuses_an_invalid_override() {
    let project = Project::new();
    status_task(
        &project,
        "TASK-01",
        &[
            "title: \"Primeira\"",
            "profile: julgamento",
            "wave: 1",
            "status: approved",
        ],
    );
    set_overrides(&project, "  julgamento:\n    claude-code: \"Opus 4!\"");
    let stderr = failure_stderr(&mut status_run(&project));
    assert!(
        stderr.contains("apelido inválido em config/models.yml"),
        "{stderr}"
    );
}

// --- Baseline sem a entrega (SPEC-017) ---

const DELIVERY_COMMAND: &str = "test -e novo.txt && echo existe";

fn with_delivery_baseline(status: &str) -> String {
    complete_task(status, "Não se aplica.").replace(
        "### Baseline\n\n- `true`",
        &format!("### Baseline\n\n- `{DELIVERY_COMMAND}`"),
    )
}

#[test]
fn baseline_delivery_is_refused_by_the_task_validation() {
    let project = Project::new();
    write(&project, TASK_PATH, &with_delivery_baseline("pending"));
    let output = nested(&project, "adjust", "validate")
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let stderr = String::from_utf8(output).unwrap();
    assert!(
        stderr.contains("'Baseline' com comando que cita 'novo.txt', que a tarefa cria"),
        "{stderr}"
    );
}

#[test]
fn baseline_delivery_running_task_is_not_refused() {
    let project = Project::new();
    write(&project, TASK_PATH, &with_delivery_baseline("in_progress"));
    write(
        &project,
        "vostok/work/tasks/SPEC-001/evidence/TASK-01/baseline/.keep",
        "",
    );
    let output = nested(&project, "adjust", "validate").output().unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(!stderr.contains("que a tarefa cria"), "{stderr}");
}

const CREATED_BASELINE: &str = "for p in src/novo.rs LICENSE; do test -e \"$p\" && echo \"$p existe\" || echo \"$p ausente\"; done";

/// Tarefa em execução que cria `src/novo.rs`, com `LICENSE` gravado e baseline já coletado.
fn task_with_created_path() -> Project {
    let project = Project::new();
    write(&project, "LICENSE", "texto");
    let text = complete_task("in_progress", "Não se aplica.")
        .replace("- `novo.txt` (criar)", "- `src/novo.rs` (criar)")
        .replace(
            "### Baseline\n\n- `true`",
            &format!("### Baseline\n\n- `{CREATED_BASELINE}`"),
        );
    write(&project, TASK_PATH, &text);
    task(&project, &["baseline"]).assert().success();
    project
}

#[test]
fn baseline_delivery_created_path_is_not_a_regression() {
    let project = task_with_created_path();
    write(&project, "src/novo.rs", "fn main() {}");
    task(&project, &["tests"]).assert().success();
    task(&project, &["regression"]).assert().success();
}

#[test]
fn delivery_command_cited_command_is_ignored_entirely() {
    let project = task_with_created_path();
    write(&project, "src/novo.rs", "fn main() {}");
    std::fs::remove_file(project.root().join("LICENSE")).unwrap();
    task(&project, &["tests"]).assert().success();
    task(&project, &["regression"]).assert().success();
}

#[test]
fn baseline_delivery_task_without_created_paths_is_unchanged() {
    let project = running_task(&[("a.txt", "antes")]);
    write(&project, "a.txt", "depois");
    task(&project, &["tests"]).assert().success();
    task(&project, &["regression"]).assert().failure();
}

// --- Comando de baseline que mede a entrega (SPEC-018) ---

fn git(root: &std::path::Path, args: &[&str]) {
    let status = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .envs(git_identity())
        .output()
        .unwrap()
        .status;
    assert!(status.success(), "git {args:?}");
}

fn git_identity() -> [(&'static str, &'static str); 4] {
    [
        ("GIT_AUTHOR_NAME", "T"),
        ("GIT_AUTHOR_EMAIL", "t@t"),
        ("GIT_COMMITTER_NAME", "T"),
        ("GIT_COMMITTER_EMAIL", "t@t"),
    ]
}

/// Tarefa em execução que cria `created` e tem `baseline` como único comando, sem coletar o baseline.
fn delivery_task(created: &str, baseline: &str, files: &[(&str, &str)]) -> Project {
    let project = Project::new();
    for (name, content) in files {
        write(&project, name, content);
    }
    let text = complete_task("in_progress", "Não se aplica.")
        .replace("- `novo.txt` (criar)", &format!("- `{created}` (criar)"))
        .replace(
            "### Baseline\n\n- `true`",
            &format!("### Baseline\n\n- `{baseline}`"),
        );
    write(&project, TASK_PATH, &text);
    project
}

fn init_git(project: &Project) {
    git(project.root(), &["init", "-q"]);
    git(project.root(), &["add", "-A"]);
    git(project.root(), &["commit", "-qm", "base"]);
}

fn regression_output(project: &Project) -> (bool, String) {
    let output = task(project, &["regression"]).output().unwrap();
    (
        output.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    )
}

#[test]
fn delivery_command_issue_roadmap_does_not_block() {
    let command = "test -e src/exemplo.rs && echo existe || echo ausente";
    let project = delivery_task("src/exemplo.rs", command, &[]);
    task(&project, &["baseline"]).assert().success();
    write(&project, "src/exemplo.rs", "fn main() {}");
    task(&project, &["tests"]).assert().success();
    let (ok, all) = regression_output(&project);
    assert!(ok, "{all}");
    assert!(all.contains("ignorado(s) por medir a entrega"), "{all}");
    assert!(
        all.contains(&format!(
            "- [1] {command}: ignorado, mede a entrega da tarefa ('src/exemplo.rs')"
        )),
        "{all}"
    );
}

#[test]
fn delivery_command_new_folder_does_not_block() {
    let command = "test -e src/modulo && echo existe || echo ausente";
    let project = delivery_task("src/modulo/a.rs", command, &[("README.md", "x")]);
    init_git(&project);
    task(&project, &["baseline"]).assert().success();
    write(&project, "src/modulo/a.rs", "fn main() {}");
    task(&project, &["tests"]).assert().success();
    let (ok, all) = regression_output(&project);
    assert!(ok, "{all}");
    assert!(
        all.contains("ignorado, mede a entrega da tarefa ('src/modulo')"),
        "{all}"
    );
}

#[test]
fn delivery_command_existing_folder_is_still_compared() {
    let project = delivery_task("src/novo.rs", "ls src", &[("src/velho.rs", "x")]);
    init_git(&project);
    task(&project, &["baseline"]).assert().success();
    write(&project, "src/novo.rs", "fn main() {}");
    task(&project, &["tests"]).assert().success();
    let (ok, all) = regression_output(&project);
    // A pasta já existia: o comando é comparado, e a linha nova passa como acréscimo (SPEC-021).
    assert!(ok, "{all}");
    assert!(
        all.contains("- [1] ls src: acréscimo de 1 linha(s)"),
        "{all}"
    );
    assert!(!all.contains("ignorado"), "{all}");
}

#[test]
fn delivery_command_without_git_only_exact_paths() {
    let command = "test -e src/modulo && echo existe || echo ausente";
    let project = delivery_task("src/modulo/a.rs", command, &[("README.md", "x")]);
    task(&project, &["baseline"]).assert().success();
    write(&project, "src/modulo/a.rs", "fn main() {}");
    task(&project, &["tests"]).assert().success();
    let (ok, all) = regression_output(&project);
    assert!(!ok, "{all}");
}

fn pending_delivery_task(created: &str, baselines: &[&str]) -> Project {
    let project = Project::new();
    let items: String = baselines.iter().map(|c| format!("- `{c}`\n")).collect();
    let text = complete_task("pending", "Não se aplica.")
        .replace("- `novo.txt` (criar)", &format!("- `{created}` (criar)"))
        .replace(
            "### Baseline\n\n- `true`\n",
            &format!("### Baseline\n\n{items}"),
        );
    write(&project, TASK_PATH, &text);
    project
}

#[test]
fn delivery_command_new_folder_is_refused_by_the_task_validation() {
    let project = pending_delivery_task("src/modulo/a.rs", &["test -e src/modulo && echo existe"]);
    let stderr = failure_stderr(&mut nested(&project, "adjust", "validate"));
    assert!(
        stderr.contains("'Baseline' com comando que cita 'src/modulo', pasta que a tarefa cria"),
        "{stderr}"
    );
}

#[test]
fn delivery_command_lookalike_names_do_not_count() {
    let project = pending_delivery_task(
        "src/modulo/a.rs",
        &["test -e src/modulos", "cat src/exemplo.rs.bak"],
    );
    let output = nested(&project, "adjust", "validate").output().unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(!stderr.contains("que a tarefa cria"), "{stderr}");
    assert!(!stderr.contains("pasta que a tarefa cria"), "{stderr}");
}

// --- Consumo de tokens (SPEC-019) ---

fn harness_file(project: &Project, relative: &str) -> String {
    std::fs::read_to_string(project.harness().join(relative)).unwrap()
}

fn brief_of(project: &Project) -> String {
    success_stdout(&mut task(project, &["brief"]))
}

#[test]
fn brief_prints_rules_persona_route_and_task() {
    let project = started_task(&[]);
    let brief = brief_of(&project);
    let positions: Vec<usize> = ["# Regras", "# Persona", "# Roteiro", "# Tarefa"]
        .iter()
        .map(|title| brief.find(title).unwrap_or_else(|| panic!("sem {title}")))
        .collect();
    assert!(
        positions.windows(2).all(|pair| pair[0] < pair[1]),
        "{brief}"
    );
    assert!(brief.contains(harness_file(&project, "rules/agent/boundaries.md").trim()));
    if project
        .harness()
        .join("rules/project/security.md")
        .is_file()
    {
        assert!(brief.contains(harness_file(&project, "rules/project/security.md").trim()));
    }
    assert!(brief.contains("## Execução"), "{brief}");
    assert!(
        brief.contains("vostok task check SPEC-001 TASK-01"),
        "{brief}"
    );
    assert!(!brief.contains("<SPEC_ID>"), "{brief}");
    assert!(brief.contains("| baseline |"), "{brief}");
}

#[test]
fn brief_is_smaller_than_the_current_load() {
    let project = started_task(&[]);
    let brief = brief_of(&project);
    let persona =
        success_stdout(
            project
                .vostok()
                .args(["context", "load", "--persona", "desenvolvedor"]),
        );
    let skill =
        success_stdout(
            project
                .vostok()
                .args(["context", "load", "--skill", "execute-task"]),
        );
    let load = success_stdout(&mut task(&project, &["load"]));
    assert!(
        brief.len() * 100 < 40 * (persona.len() + skill.len() + load.len()),
        "briefing {} contra {}",
        brief.len(),
        persona.len() + skill.len() + load.len()
    );
}

#[test]
fn brief_refuses_a_task_not_started_or_without_baseline() {
    let expected = "o briefing só sai para tarefa 'in_progress' com baseline coletado";
    let project = Project::new();
    write(
        &project,
        TASK_PATH,
        &complete_task("pending", "Não se aplica."),
    );
    success_stdout(&mut nested(&project, "adjust", "record-manual"));
    success_stdout(&mut nested(&project, "approve", "approve"));
    let stderr = failure_stderr(&mut task(&project, &["brief"]));
    assert!(stderr.contains(expected), "{stderr}");

    let project = started_task(&[]);
    std::fs::remove_dir_all(project.root().join(BASELINE_DIR)).unwrap();
    let stderr = failure_stderr(&mut task(&project, &["brief"]));
    assert!(stderr.contains(expected), "{stderr}");
}

#[test]
fn brief_without_persona_uses_desenvolvedor() {
    let project = started_task(&[]);
    assert!(!read(&project, TASK_PATH).contains("\npersona:"));
    let brief = brief_of(&project);
    let persona = harness_file(&project, "personas/desenvolvedor.md");
    let section = brief.split("\n# Persona\n").nth(1).unwrap();
    assert!(section.contains(persona.trim()), "{brief}");
}

/// Troca o bloco da "Verificação" da tarefa pelo comando dado.
fn set_verification(project: &Project, command: &str) {
    let content =
        read(project, TASK_PATH).replace("```bash\ntrue\n```", &format!("```bash\n{command}\n```"));
    write(project, TASK_PATH, &content);
}

#[test]
fn check_passes_and_fails_without_touching_the_task() {
    let project = started_task(&[]);
    let before = read(&project, TASK_PATH);
    let stdout = success_stdout(&mut task(&project, &["check"]));
    assert!(
        stdout.contains("Verificação de TASK-01 passou (saída em "),
        "{stdout}"
    );
    assert!(exists(
        &project,
        &format!("{EVIDENCE_DIR}/validation/check.out")
    ));
    assert_eq!(read(&project, TASK_PATH), before);

    set_verification(&project, "false");
    let before = read(&project, TASK_PATH);
    let output = task(&project, &["check"])
        .assert()
        .failure()
        .get_output()
        .clone();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("Verificação de TASK-01 falhou (código 1); últimas linhas"),
        "{stdout}"
    );
    assert_eq!(read(&project, TASK_PATH), before);
}

#[test]
fn check_refuses_a_blocked_task() {
    let project = started_task(&[]);
    let content = read(&project, TASK_PATH).replace("status: in_progress", "status: blocked");
    write(&project, TASK_PATH, &content);
    let stderr = failure_stderr(&mut task(&project, &["check"]));
    assert!(stderr.contains("está bloqueada"), "{stderr}");
}

#[test]
fn max_subagents_is_shown_and_invalid_values_warn() {
    let project = Project::new();
    status_task(
        &project,
        "TASK-01",
        &["title: \"A\"", "wave: 1", "status: pending"],
    );
    let stdout_of = |project: &Project| {
        let output = status_run(project).output().unwrap();
        (
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap(),
        )
    };
    let (out, err) = stdout_of(&project);
    assert!(
        out.trim_end().ends_with("Subagentes ao mesmo tempo: 3"),
        "{out}"
    );
    assert!(!err.contains("Aviso: max_subagents"), "{err}");

    let path = "vostok/config/tasks.yml";
    let base = read(&project, path);
    assert!(base.contains("max_subagents: 3"));
    for (value, shown) in [("3", "3"), ("0", "6"), ("9", "6")] {
        write(
            &project,
            path,
            &base.replace("max_subagents: 3", &format!("max_subagents: {value}")),
        );
        let (out, err) = stdout_of(&project);
        assert!(
            out.trim_end()
                .ends_with(&format!("Subagentes ao mesmo tempo: {shown}")),
            "{out}"
        );
        assert_eq!(
            err.contains("Aviso: max_subagents"),
            shown != value,
            "{err}"
        );
    }
    write(&project, path, &base.replace("max_subagents: 3\n", ""));
    let (out, _) = stdout_of(&project);
    assert!(
        out.trim_end().ends_with("Subagentes ao mesmo tempo: 3"),
        "{out}"
    );
}

// --- Rótulos de evidência na Verificação (SPEC-020) ---

const VERIFY_DIR: &str = "vostok/work/tasks/SPEC-001/evidence/TASK-01/validation/verify";

fn from_verify(name: &str) -> String {
    format!("- `{name}` (automática, da verificação)\n  - Prova: o comando roda.\n  - Trecho: a saída.\n  - Falha: saída ausente.")
}

/// Tarefa em execução, com o baseline coletado, as evidências dadas e a Verificação informada.
fn labeled_task(verification: &str, evidence: &str) -> Project {
    let project = running_task_with_evidence(&[("valor.txt", "a\n")], evidence);
    set_verification(&project, verification);
    project
}

fn counter_lines(project: &Project) -> usize {
    read(project, "contador.txt").lines().count()
}

#[test]
fn verify_label_records_each_labeled_output_running_the_block_once() {
    let evidence = format!(
        "{}\n{}",
        from_verify("primeira.txt"),
        from_verify("segunda.txt")
    );
    let project = labeled_task(
        "echo rodou >> contador.txt\n# evidência: primeira.txt\necho um\necho fora\n# evidência: segunda.txt\necho dois >&2",
        &evidence,
    );
    task(&project, &["tests"]).assert().success();
    assert_eq!(
        read(&project, &format!("{VERIFY_DIR}/primeira.txt")),
        "um\n"
    );
    assert_eq!(
        read(&project, &format!("{VERIFY_DIR}/segunda.txt")),
        "dois\n"
    );
    assert_eq!(counter_lines(&project), 1);
    let output = read(&project, &format!("{EVIDENCE_DIR}/validation/tests.out"));
    assert_eq!(output, "um\nfora\ndois\n");
}

#[test]
fn verify_label_clears_old_outputs_before_running() {
    let project = labeled_task("# evidência: saida.txt\necho um", &from_verify("saida.txt"));
    write(&project, &format!("{VERIFY_DIR}/antiga.txt"), "velha\n");
    task(&project, &["tests"]).assert().success();
    assert!(!exists(&project, &format!("{VERIFY_DIR}/antiga.txt")));
    assert_eq!(read(&project, &format!("{VERIFY_DIR}/saida.txt")), "um\n");
}

#[test]
fn verify_label_covers_a_command_continued_with_backslash() {
    let project = labeled_task(
        "# evidência: saida.txt\necho um \\\n  dois \\\n  tres\necho fora",
        &from_verify("saida.txt"),
    );
    task(&project, &["tests"]).assert().success();
    assert_eq!(
        read(&project, &format!("{VERIFY_DIR}/saida.txt")),
        "um dois tres\n"
    );
}

#[test]
fn verify_label_keeps_the_exit_code_of_the_labeled_command() {
    let project = labeled_task(
        "# evidência: saida.txt\nbash -c 'exit 3'\necho depois",
        &from_verify("saida.txt"),
    );
    task(&project, &["tests"]).assert().failure();
    let output = read(&project, &format!("{EVIDENCE_DIR}/validation/tests.out"));
    assert!(!output.contains("depois"), "{output}");
    assert!(
        output.contains("comando com falha (código 3): bash -c 'exit 3'"),
        "{output}"
    );
    assert!(read(&project, TASK_PATH).contains("tests_result: failed"));
}

#[test]
fn verify_label_evidence_is_copied_without_running_again() {
    let project = labeled_task(
        "# evidência: saida.txt\necho rodou >> contador.txt && cat contador.txt",
        &from_verify("saida.txt"),
    );
    task(&project, &["tests"]).assert().success();
    task(&project, &["regression"]).assert().success();
    let output = success_stdout(&mut task(&project, &["evidence"]));
    assert!(
        output.contains("1 item(ns). Resultado: passed."),
        "{output}"
    );
    assert_eq!(
        log_body(&read(&project, &format!("{EVIDENCE_DIR}/saida.txt"))),
        "rodou\n"
    );
    assert_eq!(counter_lines(&project), 1);
    assert!(read(&project, TASK_PATH).contains("evidence_result: passed"));
}

#[test]
fn verify_label_evidence_fails_when_the_tests_failed() {
    let project = labeled_task(
        "false\n# evidência: saida.txt\necho um",
        &from_verify("saida.txt"),
    );
    task(&project, &["tests"]).assert().failure();
    assert!(!exists(&project, &format!("{VERIFY_DIR}/saida.txt")));
    let stderr = failure_stderr(&mut task(&project, &["evidence"]));
    assert!(
        stderr.contains("a evidência saida.txt vem da verificação; execute vostok task tests"),
        "{stderr}"
    );
    assert!(!exists(&project, &format!("{EVIDENCE_DIR}/saida.txt")));
}

#[test]
fn verify_label_evidence_fails_without_the_label_output() {
    let project = labeled_task("# evidência: saida.txt\necho um", &from_verify("saida.txt"));
    task(&project, &["tests"]).assert().success();
    std::fs::remove_file(project.root().join(format!("{VERIFY_DIR}/saida.txt"))).unwrap();
    let stderr = failure_stderr(&mut task(&project, &["evidence"]));
    assert!(
        stderr.contains("a evidência saida.txt vem da verificação; execute vostok task tests"),
        "{stderr}"
    );
}

/// Tarefa pendente e completa com a Verificação e as evidências informadas.
fn validated_labels(verification: &str, evidence: &str) -> Project {
    let project = task_with_evidence("pending", evidence);
    set_verification(&project, verification);
    project
}

fn refused(verification: &str, evidence: &str) -> String {
    let project = validated_labels(verification, evidence);
    failure_stderr(&mut nested(&project, "adjust", "validate"))
}

#[test]
fn verify_label_with_its_evidence_is_accepted() {
    let project = validated_labels(
        "# evidência: saida.txt\ncargo test \\\n  --test task",
        &from_verify("saida.txt"),
    );
    success_stdout(&mut nested(&project, "adjust", "validate"));
}

#[test]
fn verify_label_without_evidence_is_refused() {
    let stderr = refused(
        "# evidência: orfao.txt\necho um",
        "- `notas.md` (manual): conferir a tela.",
    );
    assert!(
        stderr.contains("rótulo `# evidência: orfao.txt` sem a evidência `orfao.txt` (automática, da verificação)"),
        "{stderr}"
    );
}

#[test]
fn verify_label_evidence_without_label_is_refused() {
    let stderr = refused("echo um", &from_verify("saida.txt"));
    assert!(
        stderr.contains("evidência `saida.txt` (automática, da verificação) sem o rótulo `# evidência: saida.txt`"),
        "{stderr}"
    );
}

#[test]
fn verify_label_without_a_command_is_refused() {
    for verification in [
        "# evidência: saida.txt\n\necho um",
        "# evidência: saida.txt\n# comentário\necho um",
        "echo um\n# evidência: saida.txt",
    ] {
        let stderr = refused(verification, &from_verify("saida.txt"));
        assert!(
            stderr.contains("rótulo `# evidência: saida.txt` sem comando na linha seguinte"),
            "{verification}\n{stderr}"
        );
    }
}

#[test]
fn verify_label_repeated_is_refused() {
    let stderr = refused(
        "# evidência: saida.txt\necho um\n# evidência: saida.txt\necho dois",
        &from_verify("saida.txt"),
    );
    assert!(
        stderr.contains("rótulo `# evidência: saida.txt` repetido"),
        "{stderr}"
    );
}

#[test]
fn verify_label_with_an_invalid_name_is_refused() {
    let stderr = refused(
        "# evidência: ../fora.txt\necho um",
        "- `notas.md` (manual): conferir a tela.",
    );
    assert!(
        stderr.contains("rótulo `# evidência: ../fora.txt` com nome de arquivo inválido"),
        "{stderr}"
    );
    let stderr = refused(
        "# evidência:\necho um",
        "- `notas.md` (manual): conferir a tela.",
    );
    assert!(
        stderr.contains("rótulo `# evidência:` sem o nome do arquivo"),
        "{stderr}"
    );
}

// --- Regressão que reaproveita a saída da Verificação (SPEC-020) ---

const COUNTED: &str = "echo x >> contador.txt && cat valor.txt";

/// Tarefa em execução cujo único comando de Baseline incrementa `contador.txt`, com a
/// Verificação informada e o baseline já coletado (contador em 1).
fn reused_task(verification: &str) -> Project {
    let project = task_before_baseline(&[("valor.txt", "a\n")], &from_verify("saida.txt"));
    let content = read(&project, TASK_PATH).replace("- `cat valor.txt`", &format!("- `{COUNTED}`"));
    write(&project, TASK_PATH, &content);
    set_verification(&project, verification);
    task(&project, &["baseline"]).assert().success();
    project
}

#[test]
fn regression_reuses_the_output_of_an_identical_labeled_command() {
    let project = reused_task(&format!("# evidência: saida.txt\n  {COUNTED}  "));
    task(&project, &["tests"]).assert().success();
    assert_eq!(counter_lines(&project), 2);
    let stdout = success_stdout(&mut task(&project, &["regression"]));
    assert!(
        stdout.contains("1 comando(s) iguais ao baseline"),
        "{stdout}"
    );
    assert_eq!(counter_lines(&project), 2);
}

#[test]
fn regression_reuses_nothing_for_a_different_command() {
    let project = reused_task("# evidência: saida.txt\necho x >> contador.txt &&  cat valor.txt");
    task(&project, &["tests"]).assert().success();
    task(&project, &["regression"]).assert().success();
    assert_eq!(counter_lines(&project), 3);
}

#[test]
fn regression_reuses_nothing_without_passed_tests() {
    let project = reused_task(&format!("# evidência: saida.txt\n{COUNTED}\nfalse"));
    task(&project, &["tests"]).assert().failure();
    task(&project, &["regression"]).assert().success();
    assert_eq!(counter_lines(&project), 3);
}

// --- Evidência como log, Espera, diff e leitura do próprio texto (SPEC-020) ---

fn automatic(name: &str, command: &str, expect: Option<&str>) -> String {
    let mut item = format!("- `{name}` (automática): `{command}`\n  - Prova: o comando roda.\n  - Trecho: a saída.\n  - Falha: saída ausente.");
    if let Some(text) = expect {
        item.push_str(&format!("\n  - Espera: `{text}`"));
    }
    item
}

fn diff_item(name: &str, path: &str) -> String {
    format!("- `{name}` (diff): `{path}`\n  - Prova: a mudança.\n  - Trecho: as linhas novas.\n  - Falha: diff vazio.")
}

/// Projeto git com `a.txt` commitado, tarefa em execução e a evidência dada.
fn git_task(evidence: &str) -> Project {
    let project = running_task_with_evidence(&[("a.txt", "um\n")], evidence);
    git(project.root(), &["init", "-q"]);
    git(project.root(), &["add", "-A"]);
    git(project.root(), &["commit", "-qm", "inicial"]);
    project
}

#[test]
fn evidence_log_starts_with_the_command_date_commit_and_exit_code() {
    let project = running_task_with_evidence(
        &[("a.txt", "um\n")],
        &automatic("saida.txt", "echo ola", None),
    );
    task(&project, &["evidence"]).assert().success();
    let log = read(&project, &format!("{EVIDENCE_DIR}/saida.txt"));
    let lines: Vec<&str> = log.lines().collect();
    assert_eq!(lines[0], "$ echo ola", "{log}");
    assert!(
        lines[1].starts_with("# data: 20") && lines[1].ends_with('Z'),
        "{log}"
    );
    assert_eq!(lines[2], "# commit: -", "{log}");
    assert_eq!(lines[3], "# código de saída: 0", "{log}");
    assert_eq!(lines[4], "", "{log}");
    assert_eq!(lines[5], "ola", "{log}");
}

#[test]
fn evidence_log_cites_the_commit_and_uncommitted_changes() {
    let project = git_task(&automatic("saida.txt", "echo ola", None));
    write(&project, "a.txt", "dois\n");
    task(&project, &["evidence"]).assert().success();
    let log = read(&project, &format!("{EVIDENCE_DIR}/saida.txt"));
    let commit = log.lines().nth(2).unwrap();
    assert!(
        commit.starts_with("# commit: ") && commit.ends_with(" (com mudanças não commitadas)"),
        "{log}"
    );
}

#[test]
fn evidence_log_of_the_verification_has_the_labeled_command() {
    let project = labeled_task("# evidência: saida.txt\necho um", &from_verify("saida.txt"));
    task(&project, &["tests"]).assert().success();
    task(&project, &["evidence"]).assert().success();
    let log = read(&project, &format!("{EVIDENCE_DIR}/saida.txt"));
    assert!(log.starts_with("$ echo um\n# data: "), "{log}");
    assert!(log.contains("# código de saída: 0\n\num\n"), "{log}");
}

#[test]
fn evidence_expect_present_passes() {
    let project = running_task_with_evidence(
        &[("a.txt", "um\n")],
        &automatic("saida.txt", "echo ola mundo", Some("ola")),
    );
    task(&project, &["evidence"]).assert().success();
}

#[test]
fn evidence_expect_missing_fails_with_the_evidence_and_the_text() {
    let project = running_task_with_evidence(
        &[("a.txt", "um\n")],
        &automatic("saida.txt", "echo ola", Some("tchau")),
    );
    let stderr = failure_stderr(&mut task(&project, &["evidence"]));
    assert!(
        stderr.contains("- saida.txt: texto esperado não encontrado: tchau"),
        "{stderr}"
    );
}

#[test]
fn evidence_expect_does_not_look_at_the_header() {
    let project = running_task_with_evidence(
        &[("a.txt", "um\n")],
        &automatic("saida.txt", "true", Some("código de saída")),
    );
    task(&project, &["evidence"]).assert().failure();
}

#[test]
fn evidence_diff_of_a_changed_file() {
    let project = git_task(&diff_item("a.diff", "a.txt"));
    write(&project, "a.txt", "dois\n");
    task(&project, &["evidence"]).assert().success();
    let log = read(&project, &format!("{EVIDENCE_DIR}/a.diff"));
    assert!(log.contains("-um\n+dois"), "{log}");
}

#[test]
fn evidence_diff_of_a_new_untracked_file() {
    let project = git_task(&diff_item("novo.diff", "novo.txt"));
    write(&project, "novo.txt", "linha nova\n");
    task(&project, &["evidence"]).assert().success();
    let log = read(&project, &format!("{EVIDENCE_DIR}/novo.diff"));
    assert!(log.contains("+linha nova"), "{log}");
}

#[test]
fn evidence_diff_without_changes_fails() {
    let project = git_task(&diff_item("a.diff", "a.txt"));
    let stderr = failure_stderr(&mut task(&project, &["evidence"]));
    assert!(stderr.contains("sem diferença em a.txt"), "{stderr}");
}

#[test]
fn evidence_diff_outside_git_fails() {
    let project = running_task_with_evidence(&[("a.txt", "um\n")], &diff_item("a.diff", "a.txt"));
    let stderr = failure_stderr(&mut task(&project, &["evidence"]));
    assert!(stderr.contains("fora de um repositório git"), "{stderr}");
}

#[test]
fn evidence_reads_of_a_changed_path_are_refused() {
    let stderr = refused(
        "true",
        &automatic("leitura.txt", "grep -n x novo.txt", None),
    );
    assert!(
        stderr.contains("a evidência leitura.txt só lê 'novo.txt', que a tarefa muda"),
        "{stderr}"
    );
}

#[test]
fn evidence_reads_of_other_files_are_accepted() {
    let project = validated_labels(
        "true",
        &automatic("leitura.txt", "grep -n x README.md", None),
    );
    success_stdout(&mut nested(&project, "adjust", "validate"));
}

#[test]
fn evidence_diff_item_is_accepted_by_the_validation() {
    let project = validated_labels("true", &diff_item("novo.diff", "novo.txt"));
    success_stdout(&mut nested(&project, "adjust", "validate"));
}

// --- Print com conferência humana (SPEC-020) ---

fn print_item(name: &str, command: &str) -> String {
    format!("- `{name}` (print): `{command}`\n  - Prova: a tela.\n  - Trecho: o botão.\n  - Falha: tela vazia.")
}

fn print_task(command: &str) -> Project {
    running_task_with_evidence(&[("a.txt", "um\n")], &print_item("tela.png", command))
}

const SHOT: &str = "printf 'PNG' > \"$VOSTOK_EVIDENCE/tela.png\"";

#[test]
fn evidence_print_is_pending_until_the_user_reviews_it() {
    let project = print_task(SHOT);
    let stdout = success_stdout(&mut task(&project, &["evidence"]));
    assert!(stdout.contains("Prints sem conferência humana"), "{stdout}");
    assert!(stdout.contains("tela.png"), "{stdout}");
    assert_eq!(read(&project, &format!("{EVIDENCE_DIR}/tela.png")), "PNG");
    assert!(read(&project, TASK_PATH).contains("evidence_result: pending"));
    assert!(read(&project, TASK_PATH).contains("print sem conferência humana"));
}

#[test]
fn evidence_print_without_the_image_fails() {
    let project = print_task("true");
    let stderr = failure_stderr(&mut task(&project, &["evidence"]));
    assert!(
        stderr.contains("- tela.png: o comando não gerou a imagem"),
        "{stderr}"
    );
}

#[test]
fn evidence_print_reviewed_by_the_user_passes() {
    let project = print_task(SHOT);
    task(&project, &["evidence"]).assert().success();
    let stdout = success_stdout(&mut task(&project, &["evidence", "--reviewed", "tela.png"]));
    assert!(stdout.contains("Resultado: passed."), "{stdout}");
    let content = read(&project, TASK_PATH);
    assert!(
        content.contains("print tela.png conferido pelo usuário (sha256 "),
        "{content}"
    );
    assert!(content.contains("evidence_result: passed"));
}

#[test]
fn evidence_print_changed_after_the_review_is_pending_again() {
    let project = print_task("printf \"$(date +%s%N)\" > \"$VOSTOK_EVIDENCE/tela.png\"");
    task(&project, &["evidence", "--reviewed", "tela.png"])
        .assert()
        .success();
    let stdout = success_stdout(&mut task(&project, &["evidence"]));
    assert!(stdout.contains("Prints sem conferência humana"), "{stdout}");
}

#[test]
fn evidence_print_review_of_an_unknown_print_fails() {
    let project = print_task(SHOT);
    let stderr = failure_stderr(&mut task(
        &project,
        &["evidence", "--reviewed", "outra.png"],
    ));
    assert!(
        stderr.contains("'outra.png' não é um print desta tarefa"),
        "{stderr}"
    );
}

#[test]
fn evidence_print_required_for_an_interface_task() {
    let project = validated_labels("true", &automatic("saida.txt", "echo ok", None));
    let content = read(&project, TASK_PATH).replace(
        "- `novo.txt` (criar): arquivo novo.",
        "- `tela.tsx` (criar): tela nova.",
    );
    write(&project, TASK_PATH, &content);
    let stderr = failure_stderr(&mut nested(&project, "adjust", "validate"));
    assert!(
        stderr.contains("tarefa com interface sem print"),
        "{stderr}"
    );

    let content = read(&project, TASK_PATH).replace(
        "- `saida.txt` (automática): `echo ok`",
        "- `tela.png` (print): `npx playwright screenshot`",
    );
    write(&project, TASK_PATH, &content);
    success_stdout(&mut nested(&project, "adjust", "validate"));
}

#[test]
fn evidence_legacy_task_finishes_as_before() {
    let project = running_task_with_evidence(
        &[("valor.txt", "a\n")],
        &automatic("saida.txt", "grep -n a valor.txt", None),
    );
    task(&project, &["tests"]).assert().success();
    task(&project, &["regression"]).assert().success();
    let stdout = success_stdout(&mut task(&project, &["evidence"]));
    assert!(stdout.contains("Resultado: passed."), "{stdout}");
}

// --- Acréscimo e alerta na regressão (SPEC-021) ---

fn growth_regression(before: &str, after: &str) -> (Project, bool, String) {
    let project = running_task(&[("valor.txt", before)]);
    write(&project, "valor.txt", after);
    task(&project, &["tests"]).assert().success();
    let (ok, output) = regression_output(&project);
    (project, ok, output)
}

#[test]
fn growth_of_a_list_passes_as_an_addition() {
    let (project, ok, output) = growth_regression("a\nb\n", "a\nnovo\nb\nfim\n");
    assert!(ok, "{output}");
    assert!(
        output.contains(
            "Sem regressão em TASK-01: 0 comando(s) iguais ao baseline; 1 com acréscimo."
        ),
        "{output}"
    );
    assert!(
        output.contains("- [1] cat valor.txt: acréscimo de 2 linha(s)\n"),
        "{output}"
    );
    assert_eq!(status(&project), "validating");
    assert!(read(&project, TASK_PATH).contains("1 com acréscimo"));
}

#[test]
fn growth_with_a_removed_line_blocks() {
    let (project, ok, output) = growth_regression("a\nb\n", "a\nnovo\n");
    assert!(!ok);
    assert!(output.contains("- [1] cat valor.txt\n"), "{output}");
    assert_eq!(status(&project), "blocked");
}

#[test]
fn growth_with_an_alert_line_blocks_and_cites_it() {
    let (project, ok, output) = growth_regression("a\n", "a\ntest x ... FAILED\n");
    assert!(!ok);
    assert!(
        output.contains("linha nova com alerta: test x ... FAILED"),
        "{output}"
    );
    assert_eq!(status(&project), "blocked");
    assert!(exists(&project, &format!("{REGRESSION_DIR}/01.diff")));
}

#[test]
fn growth_alert_ignores_case() {
    let (_, ok, output) = growth_regression("a\n", "a\nWarning: algo\n");
    assert!(!ok, "{output}");
    assert!(
        output.contains("linha nova com alerta: Warning: algo"),
        "{output}"
    );
}

#[test]
fn growth_with_a_different_exit_code_blocks() {
    let project = running_task(&[("valor.txt", "a\n")]);
    std::fs::remove_file(project.root().join("valor.txt")).unwrap();
    task(&project, &["tests"]).assert().success();
    let (ok, output) = regression_output(&project);
    assert!(!ok, "{output}");
    assert_eq!(status(&project), "blocked");
}

#[test]
fn growth_with_reordered_lines_blocks() {
    let (project, ok, output) = growth_regression("a\nb\n", "b\na\n");
    assert!(!ok, "{output}");
    assert_eq!(status(&project), "blocked");
}

#[test]
fn growth_alert_list_comes_from_the_configuration() {
    let project = running_task(&[("valor.txt", "a\n")]);
    let path = "vostok/config/tasks.yml";
    let config = read(&project, path).replace(
        "regression_alert:\n  - FAILED\n  - error\n  - warning\n  - panicked\n",
        "regression_alert:\n  - ERRO\n",
    );
    assert!(config.contains("  - ERRO\n"), "{config}");
    write(&project, path, &config);
    write(&project, "valor.txt", "a\nerro: algo\nwarning: outro\n");
    task(&project, &["tests"]).assert().success();
    let (ok, output) = regression_output(&project);
    assert!(!ok);
    assert!(
        output.contains("linha nova com alerta: erro: algo"),
        "{output}"
    );
}

#[test]
fn growth_over_a_baseline_recorded_before_the_change_passes() {
    let project = running_task(&[("valor.txt", "a\n")]);
    write(
        &project,
        &format!("{BASELINE_DIR}/01.out"),
        "$ cat valor.txt\nexit: 0\na\n",
    );
    write(&project, "valor.txt", "a\nb\n");
    task(&project, &["tests"]).assert().success();
    let (ok, output) = regression_output(&project);
    assert!(ok, "{output}");
    assert!(output.contains("1 com acréscimo"), "{output}");
}

// --- Mudança esperada no Baseline (SPEC-021) ---

const EXPECTED_CHANGE: &str = "- `cat valor.txt` (muda): o valor passa de a para b\n";

#[test]
fn expected_change_passes_with_the_reason_recorded() {
    let project = running_task(&[("valor.txt", "a\n")]);
    let text = read(&project, TASK_PATH).replace("- `cat valor.txt`\n", EXPECTED_CHANGE);
    write(&project, TASK_PATH, &text);
    write(&project, "valor.txt", "b\n");
    task(&project, &["tests"]).assert().success();
    let (ok, output) = regression_output(&project);
    assert!(ok, "{output}");
    assert!(output.contains("1 mudança(s) esperada(s)"), "{output}");
    assert!(
        output.contains("- [1] cat valor.txt: muda (o valor passa de a para b)\n"),
        "{output}"
    );
    assert!(read(&project, &format!("{REGRESSION_DIR}/01.diff")).contains("> b"));
    assert!(read(&project, TASK_PATH).contains("1 mudança(s) esperada(s)"));
}

#[test]
fn expected_change_without_a_difference_counts_as_equal() {
    let project = running_task(&[("valor.txt", "a\n")]);
    let text = read(&project, TASK_PATH).replace("- `cat valor.txt`\n", EXPECTED_CHANGE);
    write(&project, TASK_PATH, &text);
    task(&project, &["tests"]).assert().success();
    let (ok, output) = regression_output(&project);
    assert!(ok, "{output}");
    assert!(
        output.contains("1 comando(s) iguais ao baseline"),
        "{output}"
    );
}

#[test]
fn expected_change_without_a_reason_is_refused_by_the_task_validation() {
    let project = Project::new();
    let text = complete_task("pending", "Não se aplica.").replace(
        "### Baseline\n\n- `true`",
        "### Baseline\n\n- `true` (muda):",
    );
    write(&project, TASK_PATH, &text);
    let output = nested(&project, "adjust", "validate")
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let stderr = String::from_utf8(output).unwrap();
    assert!(
        stderr.contains("'Baseline' com item (muda) sem motivo: - `true` (muda):"),
        "{stderr}"
    );
}

#[test]
fn expected_change_with_a_reason_is_valid_for_the_task() {
    let project = Project::new();
    let text = complete_task("pending", "Não se aplica.").replace(
        "### Baseline\n\n- `true`",
        "### Baseline\n\n- `true` (muda): a saída muda",
    );
    write(&project, TASK_PATH, &text);
    nested(&project, "adjust", "validate").assert().success();
}

#[test]
fn expected_change_on_a_delivery_command_is_ignored_as_delivery() {
    let command = "test -e novo.txt && echo existe || echo ausente";
    let project = delivery_task("novo.txt", command, &[("README.md", "x")]);
    let text = read(&project, TASK_PATH).replace(
        &format!("- `{command}`"),
        &format!("- `{command}` (muda): a entrega aparece"),
    );
    write(&project, TASK_PATH, &text);
    init_git(&project);
    task(&project, &["baseline"]).assert().success();
    write(&project, "novo.txt", "x");
    task(&project, &["tests"]).assert().success();
    let (ok, output) = regression_output(&project);
    assert!(ok, "{output}");
    assert!(
        output.contains("ignorado, mede a entrega da tarefa ('novo.txt')"),
        "{output}"
    );
    assert!(!output.contains("muda ("), "{output}");
}

// --- Aceite que mantém testes e evidências (SPEC-021) ---

fn field(project: &Project, key: &str) -> String {
    let prefix = format!("{key}: ");
    read(project, TASK_PATH)
        .lines()
        .find_map(|line| line.strip_prefix(&prefix).map(str::to_string))
        .unwrap_or_default()
}

/// Tarefa com testes e evidências aprovados, bloqueada pela regressão do comando 1.
fn kept_results_blocked() -> Project {
    let project = running_task(&[("valor.txt", "a\n")]);
    write(&project, "valor.txt", "b\n");
    task(&project, &["tests"]).assert().success();
    task(&project, &["evidence"]).assert().success();
    task(&project, &["regression"]).assert().failure();
    assert_eq!(status(&project), "blocked");
    project
}

#[test]
fn kept_results_finish_with_the_regression_only() {
    let project = kept_results_blocked();
    let resumed = stdout(&mut task(
        &project,
        &["resume", "--accept-regression", "1", "diferença esperada"],
    ));
    assert!(
        resumed.contains("diferença aceita. Execute vostok task regression"),
        "{resumed}"
    );
    assert_eq!(field(&project, "tests_result"), "passed");
    assert_eq!(field(&project, "evidence_result"), "passed");
    assert_eq!(field(&project, "regression_result"), "null");
    assert_eq!(field(&project, "attempts"), "0");
    task(&project, &["regression"]).assert().success();
    task(&project, &["finish"]).assert().success();
    assert_eq!(status(&project), "done");
}

#[test]
fn kept_results_are_reset_by_a_plain_restart() {
    let project = kept_results_blocked();
    let resumed = stdout(&mut task(&project, &["resume", "corrigir o valor"]));
    assert!(
        resumed.contains("tentativas e resultados zerados"),
        "{resumed}"
    );
    assert_eq!(field(&project, "tests_result"), "null");
    assert_eq!(field(&project, "evidence_result"), "null");
    assert_eq!(field(&project, "regression_result"), "null");
}

#[test]
fn kept_results_keep_a_failed_test_result() {
    let project = kept_results_blocked();
    let text = read(&project, TASK_PATH).replace("tests_result: passed", "tests_result: failed");
    write(&project, TASK_PATH, &text);
    task(
        &project,
        &["resume", "--accept-regression", "1", "diferença esperada"],
    )
    .assert()
    .success();
    assert_eq!(field(&project, "tests_result"), "failed");
    task(&project, &["regression"]).assert().success();
    task(&project, &["finish"]).assert().failure();
}

// --- Progresso no início da tarefa (SPEC-022) ---

#[test]
fn start_progress_prints_each_command_before_the_summary() {
    let project = Project::new();
    let content = complete_task(
        "pending",
        "- `saida.txt` (automática): `echo depois`\n  - Prova: o valor.\n  - Trecho: a linha.\n  - Falha: vazio.",
    )
    .replace(
        "### Baseline\n\n- `true`\n",
        "### Baseline\n\n- `true`\n- `echo dois`\n",
    );
    write(&project, TASK_PATH, &content);
    success_stdout(&mut nested(&project, "adjust", "record-manual"));
    success_stdout(&mut nested(&project, "approve", "approve"));
    let output = stdout(&mut task(&project, &["start"]));
    let header = output
        .find("Tarefa TASK-01 iniciada: status 'in_progress'. Coletando o baseline:\n")
        .expect(&output);
    let first = output.find("- [1] true\n").expect(&output);
    let second = output.find("- [2] echo dois\n").expect(&output);
    let evidence = output
        .find("- [evidência saida.txt] echo depois\n")
        .expect(&output);
    let summary = output
        .find("Baseline de TASK-01 coletado em ")
        .expect(&output);
    assert!(header < first && first < second && second < evidence && evidence < summary);
    let content = read(&project, TASK_PATH);
    assert!(content.find("| início |").unwrap() < content.find("| baseline |").unwrap());
}

// --- Requer nas evidências (SPEC-022) ---

const REQUIRES_EVIDENCE: &str = "- `saida.txt` (automática): `echo x >> contador.txt; test -s novo.txt && echo pronto`\n  - Prova: o arquivo novo é lido.\n  - Trecho: o conteúdo.\n  - Falha: arquivo ausente.\n  - Requer: `novo.txt`";

#[test]
fn requires_missing_path_skips_the_evidence_in_the_baseline() {
    let project = running_task_with_evidence(&[("valor.txt", "a\n")], REQUIRES_EVIDENCE);
    assert_eq!(
        evidence_baseline_file(&project, "saida.txt"),
        "$ echo x >> contador.txt; test -s novo.txt && echo pronto\nexit: 1\nnão executada: novo.txt ainda não existe\n"
    );
    assert!(!exists(&project, "contador.txt"));
}

#[test]
fn requires_existing_path_runs_the_evidence_in_the_baseline() {
    let project = running_task_with_evidence(
        &[("valor.txt", "a\n"), ("novo.txt", "pronto\n")],
        REQUIRES_EVIDENCE,
    );
    assert_eq!(
        evidence_baseline_file(&project, "saida.txt"),
        "$ echo x >> contador.txt; test -s novo.txt && echo pronto\nexit: 0\npronto\n"
    );
    assert!(exists(&project, "contador.txt"));
}

#[test]
fn requires_skipped_evidence_does_not_warn_about_the_same_output() {
    let project = running_task_with_evidence(&[("valor.txt", "a\n")], REQUIRES_EVIDENCE);
    write(&project, "novo.txt", "pronto\n");
    task(&project, &["tests"]).assert().success();
    let output = stdout(&mut task(&project, &["evidence"]));
    assert!(!output.contains("mesma saída do baseline"), "{output}");
    assert!(read(&project, TASK_PATH).contains("evidence_result: passed"));
}

fn requires_validation(evidence: &str) -> (bool, String) {
    let project = Project::new();
    write(&project, TASK_PATH, &complete_task("pending", evidence));
    let output = nested(&project, "adjust", "validate").output().unwrap();
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn requires_a_created_path_is_valid_for_the_task() {
    let (ok, stderr) = requires_validation(REQUIRES_EVIDENCE);
    assert!(ok, "{stderr}");
}

#[test]
fn requires_a_path_the_task_does_not_create_is_refused() {
    let (ok, stderr) = requires_validation(
        &REQUIRES_EVIDENCE.replace("Requer: `novo.txt`", "Requer: `novo.txt`, `README.md`"),
    );
    assert!(!ok);
    assert!(
        stderr.contains("a evidência saida.txt requer 'README.md', que não é criado pela tarefa"),
        "{stderr}"
    );
}

// --- Tempo-limite das evidências no baseline (SPEC-022) ---

fn timeout_evidence(command: &str, extra: &str) -> String {
    format!("- `saida.txt` (automática): `{command}`\n  - Prova: o comando roda.\n  - Trecho: fim.\n  - Falha: vazio.{extra}")
}

/// Tarefa com a evidência dada, com o tempo-limite padrão em `seconds`, depois do baseline; devolve
/// também quanto a coleta demorou.
fn timeout_task(seconds: &str, evidence: &str) -> (Project, std::time::Duration) {
    let project = task_before_baseline(&[("valor.txt", "a\n")], evidence);
    set_config(&project, "evidence_baseline_timeout", seconds);
    assert!(read(&project, "vostok/config/tasks.yml")
        .contains(&format!("evidence_baseline_timeout: {seconds}\n")));
    let start = std::time::Instant::now();
    task(&project, &["baseline"]).assert().success();
    (project, start.elapsed())
}

#[test]
fn evidence_timeout_interrupts_and_records_the_evidence() {
    let (project, elapsed) =
        timeout_task("1", &timeout_evidence("echo começo; sleep 5; echo fim", ""));
    assert!(elapsed < std::time::Duration::from_secs(4), "{elapsed:?}");
    assert_eq!(
        evidence_baseline_file(&project, "saida.txt"),
        "$ echo começo; sleep 5; echo fim\nexit: 124\ncomeço\ninterrompida após 1 s\n"
    );
}

#[test]
fn evidence_timeout_stops_the_child_processes() {
    let (project, elapsed) = timeout_task(
        "1",
        &timeout_evidence("sh -c 'sleep 30; echo tarde >> filho.txt' & wait", ""),
    );
    assert!(elapsed < std::time::Duration::from_secs(5), "{elapsed:?}");
    assert!(evidence_baseline_file(&project, "saida.txt").ends_with("interrompida após 1 s\n"));
}

#[test]
fn evidence_timeout_zero_runs_without_a_limit() {
    let (project, _) = timeout_task(
        "1",
        &timeout_evidence("sleep 2; echo fim", "\n  - Tempo-limite: 0"),
    );
    assert_eq!(
        evidence_baseline_file(&project, "saida.txt"),
        "$ sleep 2; echo fim\nexit: 0\nfim\n"
    );
}

#[test]
fn evidence_timeout_of_the_evidence_replaces_the_default() {
    let (project, _) = timeout_task(
        "1",
        &timeout_evidence("sleep 2; echo fim", "\n  - Tempo-limite: 10"),
    );
    assert!(evidence_baseline_file(&project, "saida.txt").ends_with("fim\n"));
}

#[test]
fn evidence_timeout_legacy_task_collects_as_before() {
    let (project, _) = timeout_task("120", AUTOMATIC_EVIDENCE);
    assert_eq!(
        evidence_baseline_file(&project, "saida.txt"),
        "$ cat valor.txt\nexit: 0\na\n"
    );
}

#[test]
fn evidence_timeout_that_is_not_a_number_is_refused() {
    let project = Project::new();
    write(
        &project,
        TASK_PATH,
        &complete_task(
            "pending",
            &timeout_evidence("echo ok", "\n  - Tempo-limite: abc"),
        ),
    );
    let output = nested(&project, "adjust", "validate").output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(
        stderr.contains("a evidência saida.txt tem Tempo-limite que não é um número de segundos"),
        "{stderr}"
    );
}

fn timeout_approval(evidence: &str) -> String {
    let project = Project::new();
    write(&project, TASK_PATH, &complete_task("pending", evidence));
    success_stdout(&mut nested(&project, "adjust", "record-manual"));
    success_stdout(&mut nested(&project, "approve", "validate"))
}

#[test]
fn evidence_timeout_above_the_default_is_listed_for_approval() {
    let output = timeout_approval(&timeout_evidence("echo ok", "\n  - Tempo-limite: 900"));
    assert!(
        output.contains("- TASK-01: saida.txt com tempo-limite de 900 s (padrão 120 s)"),
        "{output}"
    );
}

#[test]
fn evidence_timeout_below_the_default_is_not_listed() {
    let output = timeout_approval(&timeout_evidence("echo ok", "\n  - Tempo-limite: 30"));
    assert!(!output.contains("tempo-limite"), "{output}");
}

// --- Camada da tarefa (SPEC-023) ---

/// Texto da tarefa depois do front-matter, sem o registro.
fn task_body(project: &Project) -> String {
    let text = read(project, TASK_PATH);
    let body = text
        .splitn(3, "---\n")
        .nth(2)
        .unwrap_or_default()
        .to_string();
    body.split("## Registro de execução")
        .next()
        .unwrap_or_default()
        .to_string()
}

#[test]
fn task_layer_state_change_keeps_the_rest_of_the_task() {
    let project = running_task(&[("valor.txt", "a\n")]);
    let before = task_body(&project);
    task(&project, &["tests"]).assert().success();
    assert!(read(&project, TASK_PATH).contains("\ntests_result: passed\n"));
    assert_eq!(task_body(&project), before);
}

#[test]
fn task_layer_record_appends_a_numbered_row() {
    let project = running_task(&[("valor.txt", "a\n")]);
    task(&project, &["tests"]).assert().success();
    task(&project, &["regression"]).assert().success();
    let text = read(&project, TASK_PATH);
    let rows: Vec<&str> = text
        .lines()
        .filter(|line| {
            line.starts_with("| ") && line.as_bytes().get(2).is_some_and(u8::is_ascii_digit)
        })
        .collect();
    assert_eq!(rows.len(), 3, "{text}");
    assert!(rows[1].starts_with("| 2 |") && rows[1].contains("| testes | passed |"));
    assert!(rows[2].starts_with("| 3 |") && rows[2].contains("| regressão | passed |"));
}

#[test]
fn task_layer_evidence_folder_is_the_current_one() {
    let project = running_task(&[("valor.txt", "a\n")]);
    assert!(exists(&project, &format!("{EVIDENCE_DIR}/baseline/01.out")));
    task(&project, &["tests"]).assert().success();
    assert!(exists(
        &project,
        &format!("{EVIDENCE_DIR}/validation/tests.out")
    ));
}

#[test]
fn task_layer_status_lists_the_tasks_of_the_spec() {
    let project = running_task(&[("valor.txt", "a\n")]);
    let output = stdout(project.vostok().args(["task", "status", "SPEC-001"]));
    assert!(output.contains("| TASK-01 | Tarefa |"), "{output}");
}

// --- Formato de pasta (SPEC-023) ---

const FOLDER: &str = "vostok/work/tasks/SPEC-001/TASK-01";

/// Grava a TASK-01 em pasta, com o status dado e um `cat valor.txt` no Baseline.
fn folder_task(status: &str) -> Project {
    let project = Project::new();
    write(&project, "valor.txt", "a\n");
    let files = [
        ("estado.yml", format!("id: TASK-01\nspec: SPEC-001\ntitle: \"Tarefa\"\neffort: simples\ndepends_on: []\ncovers: [RF-01]\ntype: tarefa\nlabels: [SPEC-001, simples]\nstatus: {status}\ncontent_hash: null\nadjustments: 0\nadjusted_at: null\napproved_at: null\napproved_hash: null\nattempts: 0\ntests_result: null\nregression_result: null\nevidence_result: null\n")),
        ("execucao.md", "### Ler\n\n- `README.md`\n\n### Mudar\n\n- `novo.txt` (criar): arquivo novo.\n\n### Passos\n\n1. Crie o arquivo.\n".to_string()),
        ("verificacao.md", "### Baseline\n\n- `cat valor.txt`\n\n### Verificação\n\n```bash\ntrue\n```\n\n### Evidências\n\n- `saida.txt` (automática): `cat valor.txt`\n  - Prova: o valor é lido.\n  - Trecho: a linha.\n  - Falha: arquivo ausente.\n".to_string()),
        ("criterios.md", "### Concluída quando\n\n- [ ] Pronto (RF-01).\n\n  > Como comprovar: ver o arquivo.\n".to_string()),
        ("executivo.md", "Resumo.\n".to_string()),
        ("tecnico.md", "Detalhes.\n".to_string()),
        ("registro.md", "## Registro de execução\n\n| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n\n## Histórico de ajustes\n\n| # | Data | Origem | Resumo |\n|---|---|---|---|\n".to_string()),
    ];
    for (name, content) in files {
        write(&project, &format!("{FOLDER}/{name}"), &content);
    }
    project
}

fn folder_started() -> Project {
    let project = folder_task("pending");
    nested(&project, "adjust", "record-manual")
        .assert()
        .success();
    nested(&project, "approve", "approve").assert().success();
    task(&project, &["start"]).assert().success();
    project
}

#[test]
fn folder_format_keeps_the_state_in_estado_yml() {
    let project = folder_started();
    let state = read(&project, &format!("{FOLDER}/estado.yml"));
    assert!(state.contains("\nstatus: in_progress\n"), "{state}");
    let approved = state
        .lines()
        .find_map(|line| line.strip_prefix("approved_hash: "))
        .unwrap_or_default();
    assert_eq!(approved.len(), 64, "{state}");
    for name in ["execucao.md", "verificacao.md", "criterios.md"] {
        assert!(!read(&project, &format!("{FOLDER}/{name}")).starts_with("---"));
    }
    assert!(!exists(&project, "vostok/work/tasks/SPEC-001/TASK-01.md"));
}

#[test]
fn folder_format_records_the_steps_in_registro_md() {
    let project = folder_started();
    task(&project, &["tests"]).assert().success();
    let record = read(&project, &format!("{FOLDER}/registro.md"));
    assert!(
        record.contains("| 1 |") && record.contains("| início | in_progress |"),
        "{record}"
    );
    assert!(record.contains("| baseline | coletado |"), "{record}");
    assert!(record.contains("| testes | passed |"), "{record}");
    assert!(record.contains("## Histórico de ajustes\n"), "{record}");
    assert_eq!(
        read(&project, &format!("{FOLDER}/executivo.md")),
        "Resumo.\n"
    );
}

#[test]
fn folder_format_keeps_the_evidence_inside_the_task() {
    let project = folder_started();
    task(&project, &["tests"]).assert().success();
    task(&project, &["regression"]).assert().success();
    task(&project, &["evidence"]).assert().success();
    assert!(exists(
        &project,
        &format!("{FOLDER}/evidencias/baseline/01.out")
    ));
    assert!(exists(
        &project,
        &format!("{FOLDER}/evidencias/validation/tests.out")
    ));
    assert!(exists(&project, &format!("{FOLDER}/evidencias/saida.txt")));
    assert!(!exists(&project, "vostok/work/tasks/SPEC-001/evidence"));
    write(
        &project,
        "vostok/work/specs/refined/SPEC-001-exemplo.md",
        "---\nid: SPEC-001\ntitle: \"Exemplo\"\nstatus: planned\nupdated_at: null\n---\n\n# Exemplo\n",
    );
    let criteria = read(&project, &format!("{FOLDER}/criterios.md")).replace("- [ ]", "- [x]");
    write(&project, &format!("{FOLDER}/criterios.md"), &criteria);
    task(&project, &["finish"]).assert().success();
    assert!(read(&project, &format!("{FOLDER}/estado.yml")).contains("\nstatus: done\n"));
    assert!(read(&project, &format!("{FOLDER}/registro.md")).contains("| conclusão | done |"));
    let verification = read(&project, &format!("{FOLDER}/verificacao.md"));
    assert!(
        verification.starts_with("### Baseline\n\n- `cat valor.txt`\n"),
        "{verification}"
    );
    assert!(
        verification.ends_with("  - Falha: arquivo ausente.\n"),
        "{verification}"
    );
    assert!(read(&project, &format!("{FOLDER}/execucao.md")).starts_with("### Ler\n"));
}

#[test]
fn folder_format_refuses_a_spec_with_both_formats() {
    let project = folder_task("pending");
    write(
        &project,
        "vostok/work/tasks/SPEC-001/TASK-02.md",
        &complete_task("pending", "Não se aplica.").replace("TASK-01", "TASK-02"),
    );
    let stderr = failure_stderr(project.vostok().args(["task", "status", "SPEC-001"]));
    assert!(
        stderr.contains(
            "a spec SPEC-001 tem tarefas nos dois formatos; execute: vostok task migrate SPEC-001"
        ),
        "{stderr}"
    );
}

#[test]
fn folder_format_refuses_a_missing_execution_file() {
    let project = folder_task("pending");
    std::fs::remove_file(project.root().join(format!("{FOLDER}/verificacao.md"))).unwrap();
    let stderr = failure_stderr(&mut nested(&project, "approve", "validate"));
    assert!(
        stderr.contains("não tem o arquivo verificacao.md"),
        "{stderr}"
    );
}

// --- Hash por arquivo, edição manual e carregamento por arquivo (SPEC-023) ---

fn folder_approved() -> Project {
    let project = folder_task("pending");
    nested(&project, "adjust", "record-manual")
        .assert()
        .success();
    nested(&project, "approve", "approve").assert().success();
    project
}

fn append(project: &Project, relative: &str, extra: &str) {
    let current = read(project, relative);
    write(project, relative, &format!("{current}{extra}"));
}

#[test]
fn part_hash_ignores_the_files_of_people() {
    let project = folder_approved();
    append(
        &project,
        &format!("{FOLDER}/executivo.md"),
        "Mais um parágrafo.\n",
    );
    append(
        &project,
        &format!("{FOLDER}/tecnico.md"),
        "Mais um detalhe.\n",
    );
    task(&project, &["start"]).assert().success();
    let output = success_stdout(&mut nested(&project, "adjust", "record-manual"));
    assert!(output.contains("Nenhuma edição manual."), "{output}");
}

#[test]
fn part_hash_stores_the_hash_of_each_execution_file_in_the_state() {
    let project = folder_approved();
    let state = read(&project, &format!("{FOLDER}/estado.yml"));
    for key in [
        "execution_hash",
        "verification_hash",
        "criteria_hash",
        "state_hash",
    ] {
        let value = state
            .lines()
            .find_map(|line| line.strip_prefix(&format!("{key}: ")))
            .unwrap_or_default();
        assert_eq!(value.len(), 64, "{key}: {state}");
    }
}

#[test]
fn part_hash_record_manual_cites_the_edited_execution_file() {
    let project = folder_approved();
    append(
        &project,
        &format!("{FOLDER}/execucao.md"),
        "2. Confira o arquivo.\n",
    );
    let output = success_stdout(&mut nested(&project, "adjust", "record-manual"));
    assert!(output.contains("Edição manual: execucao.md"), "{output}");
    assert!(!output.contains("verificacao.md"), "{output}");
    let record = read(&project, &format!("{FOLDER}/registro.md"));
    assert!(
        record.contains("| manual | Edição manual: execucao.md |"),
        "{record}"
    );
    let again = success_stdout(&mut nested(&project, "adjust", "record-manual"));
    assert!(again.contains("Nenhuma edição manual."), "{again}");
}

#[test]
fn part_hash_record_manual_cites_every_edited_execution_file() {
    let project = folder_approved();
    append(&project, &format!("{FOLDER}/verificacao.md"), "\n");
    append(
        &project,
        &format!("{FOLDER}/criterios.md"),
        "- [ ] Outro (RF-01).\n",
    );
    let output = success_stdout(&mut nested(&project, "adjust", "record-manual"));
    assert!(output.contains("criterios.md"), "{output}");
    assert!(!output.contains("execucao.md"), "{output}");
}

#[test]
fn part_hash_treats_an_edited_state_file_as_a_manual_edit() {
    let project = folder_approved();
    let state = read(&project, &format!("{FOLDER}/estado.yml"))
        .replace("effort: simples", "effort: padrao");
    write(&project, &format!("{FOLDER}/estado.yml"), &state);
    let output = success_stdout(&mut nested(&project, "adjust", "record-manual"));
    assert!(output.contains("Edição manual: estado.yml"), "{output}");
    let again = success_stdout(&mut nested(&project, "adjust", "record-manual"));
    assert!(again.contains("Nenhuma edição manual."), "{again}");
}

#[test]
fn part_hash_keeps_the_state_edit_visible_after_a_script_writes_the_state() {
    let project = folder_approved();
    let state = read(&project, &format!("{FOLDER}/estado.yml"))
        .replace("effort: simples", "effort: padrao");
    write(&project, &format!("{FOLDER}/estado.yml"), &state);
    task(&project, &["start"]).assert().success();
    let output = success_stdout(&mut nested(&project, "adjust", "record-manual"));
    assert!(output.contains("estado.yml"), "{output}");
}

#[test]
fn part_hash_does_not_cite_files_in_the_single_file_format() {
    let project = task_with_evidence("pending", "Não se aplica.");
    nested(&project, "adjust", "record-manual")
        .assert()
        .success();
    let edited =
        read(&project, TASK_PATH).replace("1. Crie o arquivo.", "1. Crie o arquivo agora.");
    write(&project, TASK_PATH, &edited);
    let output = success_stdout(&mut nested(&project, "adjust", "record-manual"));
    assert!(
        output.contains("registrado em TASK-01: Edição manual"),
        "{output}"
    );
    assert!(
        !output.contains(".yml") && !output.contains("execucao.md"),
        "{output}"
    );
}

#[test]
fn part_hash_load_brings_only_the_files_of_the_agent() {
    let project = folder_started();
    let output = success_stdout(&mut task(&project, &["load"]));
    assert!(output.contains("status: in_progress"), "{output}");
    assert!(output.contains("### Ler"), "{output}");
    assert!(output.contains("### Baseline"), "{output}");
    assert!(output.contains("### Concluída quando"), "{output}");
    assert!(!output.contains("Detalhes."), "{output}");
    assert!(!output.contains("Resumo."), "{output}");
    assert!(!output.contains("Registro de execução"), "{output}");
}

#[test]
fn part_hash_load_part_prints_only_that_file() {
    let project = folder_started();
    let technical = success_stdout(&mut task(&project, &["load", "--part", "tecnico"]));
    assert_eq!(technical, "Detalhes.\n");
    let execution = success_stdout(&mut task(&project, &["load", "--part", "execucao"]));
    assert!(execution.starts_with("### Ler\n"), "{execution}");
    assert!(!execution.contains("### Baseline"), "{execution}");
    let criteria = success_stdout(&mut task(&project, &["load", "--part", "criterios"]));
    assert!(criteria.starts_with("### Concluída quando\n"), "{criteria}");
}

#[test]
fn part_hash_load_part_prints_the_equivalent_subsections_of_the_single_file_format() {
    let project = task_with_evidence("pending", "Não se aplica.");
    let executive = success_stdout(&mut task(&project, &["load", "--part", "executivo"]));
    assert_eq!(executive, "Resumo.\n");
    let verification = success_stdout(&mut task(&project, &["load", "--part", "verificacao"]));
    assert!(verification.starts_with("### Baseline\n"), "{verification}");
    assert!(verification.contains("### Evidências"), "{verification}");
    assert!(
        !verification.contains("### Concluída quando"),
        "{verification}"
    );
}

#[test]
fn part_hash_load_refuses_an_unknown_part() {
    let project = folder_started();
    let stderr = failure_stderr(&mut task(&project, &["load", "--part", "registro"]));
    assert!(stderr.contains("execucao"), "{stderr}");
}

// --- Material do executivo e do técnico (SPEC-023) ---

/// Tarefa em pasta pronta para concluir: verificação, regressão e evidências passed e critérios
/// marcados, com a spec refinada que o `finish` marca como implementada.
fn folder_validated() -> Project {
    let project = folder_started();
    task(&project, &["tests"]).assert().success();
    task(&project, &["regression"]).assert().success();
    task(&project, &["evidence"]).assert().success();
    write(
        &project,
        "vostok/work/specs/refined/SPEC-001-exemplo.md",
        "---\nid: SPEC-001\ntitle: \"Exemplo\"\nstatus: planned\nupdated_at: null\n---\n\n# Exemplo\n",
    );
    let criteria = read(&project, &format!("{FOLDER}/criterios.md")).replace("- [ ]", "- [x]");
    write(&project, &format!("{FOLDER}/criterios.md"), &criteria);
    project
}

#[test]
fn describe_prints_the_steps_the_diff_the_evidence_and_the_record() {
    let project = folder_started();
    git(project.root(), &["init", "-q"]);
    git(project.root(), &["add", "-A"]);
    git(project.root(), &["commit", "-qm", "inicial"]);
    write(&project, "novo.txt", "um\n");
    task(&project, &["tests"]).assert().success();
    task(&project, &["evidence"]).assert().success();
    let output = stdout(&mut task(&project, &["describe"]));
    assert!(
        output.starts_with("# Instrução\n\nEscreva `executivo.md`"),
        "{output}"
    );
    assert!(output.contains("# TASK-01: Tarefa\n"), "{output}");
    assert!(
        output.contains("### Passos\n\n1. Crie o arquivo.\n"),
        "{output}"
    );
    assert!(
        output.contains("- `novo.txt` (criar): novo, ainda não rastreado\n"),
        "{output}"
    );
    assert!(output.contains("- Testes: passed\n"), "{output}");
    assert!(
        output.contains("- `saida.txt` (automática): gravada\n"),
        "{output}"
    );
    assert!(output.contains("| testes | passed |"), "{output}");
    assert!(output.contains("## Histórico de ajustes\n"), "{output}");
}

#[test]
fn describe_summarizes_the_diff_of_a_tracked_file() {
    let project = folder_started();
    write(&project, "novo.txt", "um\n");
    git(project.root(), &["init", "-q"]);
    git(project.root(), &["add", "-A"]);
    git(project.root(), &["commit", "-qm", "inicial"]);
    write(&project, "novo.txt", "um\ndois\n");
    let output = stdout(&mut task(&project, &["describe"]));
    assert!(
        output.contains("- `novo.txt` (criar): 1 file changed, 1 insertion(+)\n"),
        "{output}"
    );
}

#[test]
fn describe_works_in_the_single_file_format() {
    let project = running_task(&[("valor.txt", "a\n")]);
    let output = stdout(&mut task(&project, &["describe"]));
    assert!(
        output.starts_with("# Instrução\n\nEscreva a subseção \"Resumo executivo\""),
        "{output}"
    );
    assert!(output.contains("# TASK-01: Tarefa\n"), "{output}");
    assert!(output.contains("- Testes: -\n"), "{output}");
    assert!(output.contains("## Registro de execução\n"), "{output}");
}

#[test]
fn describe_refuses_a_task_that_is_not_running() {
    let project = folder_task("pending");
    let stderr = failure_stderr(&mut task(&project, &["describe"]));
    assert!(
        stderr.contains("a descrição só sai para tarefa 'in_progress' ou 'validating'"),
        "{stderr}"
    );
}

#[test]
fn describe_finish_refuses_the_technical_document_still_in_the_template() {
    let project = folder_validated();
    write(
        &project,
        &format!("{FOLDER}/tecnico.md"),
        "<!-- Escrito na finalização da tarefa. -->\n\n#{TASK_TECHNICAL}\n",
    );
    let stderr = failure_stderr(&mut task(&project, &["finish"]));
    assert!(
        stderr.contains("o documento tecnico.md de TASK-01 ainda está no template; escreva-o a partir de vostok task describe"),
        "{stderr}"
    );
    assert!(!read(&project, &format!("{FOLDER}/estado.yml")).contains("\nstatus: done\n"));
    write(
        &project,
        &format!("{FOLDER}/tecnico.md"),
        "Criado o novo.txt.\n",
    );
    task(&project, &["finish"]).assert().success();
    assert!(read(&project, &format!("{FOLDER}/estado.yml")).contains("\nstatus: done\n"));
}

#[test]
fn describe_finish_refuses_the_executive_document_still_in_the_template() {
    let project = folder_validated();
    write(
        &project,
        &format!("{FOLDER}/executivo.md"),
        "#{TASK_EXECUTIVE}\n",
    );
    let stderr = failure_stderr(&mut task(&project, &["finish"]));
    assert!(
        stderr.contains("o documento executivo.md de TASK-01"),
        "{stderr}"
    );
}

#[test]
fn describe_finish_accepts_a_migrated_task_with_the_documents_written() {
    let project = folder_validated();
    task(&project, &["finish"]).assert().success();
    assert_eq!(
        read(&project, &format!("{FOLDER}/executivo.md")),
        "Resumo.\n"
    );
    assert_eq!(
        read(&project, &format!("{FOLDER}/tecnico.md")),
        "Detalhes.\n"
    );
}
