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
        read(&project, &format!("{PERSIST_DIR}/saida.txt")),
        "~/x e ~/y\n"
    );
    assert_eq!(
        read(&project, &format!("{EVIDENCE_DIR}/saida.txt")),
        "/Users/fulano/x e /home/beltrano/y\n"
    );
}

#[test]
fn sensitive_data_blocks_the_copy_until_the_user_decides() {
    let project = validated_task("token: sk-abcdefghijklmnopqrstuvwx\n");
    let refused = failure_stderr(&mut task(&project, &["persist"]));
    assert!(refused.contains("- saida.txt:1: segredo"), "{refused}");
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
        read(&project, &format!("{PERSIST_DIR}/saida.txt")),
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
            refused.contains(&format!("saida.txt:1: {kind}")),
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
