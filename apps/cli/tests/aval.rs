//! Testes de integração da trava de aval (SPEC-030) na conclusão e na aprovação de tarefas e specs.

mod common;

use std::fs;

use common::{failure_stderr, Project};

const TASK_PATH: &str = "vostok/work/tasks/SPEC-001/TASK-01.md";
const SPEC_PATH: &str = "vostok/work/specs/refined/SPEC-001-exemplo.md";
const AVAIS_LOG: &str = "vostok/work/avais.log";
const APPROVED_HASH: &str = "hashaprovado";
const PANEL_INSTRUCTION: &str = "vostok painel";
const PANEL_WITHOUT_AVAL: &str = "approved_by: painel sem aval correspondente em avais.log";

fn write(project: &Project, relative: &str, content: &str) {
    let path = project.root().join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn read(project: &Project, relative: &str) -> String {
    fs::read_to_string(project.root().join(relative)).unwrap_or_default()
}

fn field(project: &Project, relative: &str, key: &str) -> String {
    read(project, relative)
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{key}: ")))
        .unwrap_or_default()
        .to_string()
}

fn replace_in(project: &Project, relative: &str, from: &str, to: &str) {
    let content = read(project, relative);
    assert!(content.contains(from), "{from} ausente em {relative}");
    write(project, relative, &content.replacen(from, to, 1));
}

fn lock(project: &Project) {
    replace_in(
        project,
        "vostok/config/main.yml",
        "approval_lock: false",
        "approval_lock: true",
    );
}

fn add_aval(project: &Project, id: &str, hash: &str, action: &str) {
    let mut log = read(project, AVAIS_LOG);
    log.push_str(&format!(
        "2026-11-02T10:00:00Z\t{id}\t{hash}\tpainel\t{action}\n"
    ));
    write(project, AVAIS_LOG, &log);
}

fn vostok(project: &Project, args: &[&str]) -> assert_cmd::Command {
    let mut command = project.vostok();
    command.args(args);
    command
}

fn task(project: &Project, args: &[&str]) -> assert_cmd::Command {
    let mut command = project.vostok();
    command.arg("task").args(args).args(["SPEC-001", "TASK-01"]);
    command
}

/// Tarefa em execução, aprovada com `APPROVED_HASH`, já validada e pronta para concluir.
fn task_ready_to_finish() -> Project {
    let project = Project::new();
    write(
        &project,
        TASK_PATH,
        &format!(
            "---\nid: TASK-01\nspec: SPEC-001\ntitle: \"Tarefa\"\nstatus: in_progress\napproved_hash: {APPROVED_HASH}\nattempts: 0\ntests_result: null\nregression_result: null\nevidence_result: null\n---\n\n# TASK-01: Tarefa\n\n## Execução\n\n### Baseline\n\n- `true`\n\n### Verificação\n\n```bash\ntrue\n```\n\n### Evidências\n\nNão se aplica.\n\n### Concluída quando\n\n- [x] Pronto.\n\n  > Como comprovar: ver.\n\n## Descrição\n\nTexto.\n\n## Registro de execução\n\n| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n"
        ),
    );
    // Uma segunda tarefa pendente mantém a spec aberta depois da conclusão.
    write(
        &project,
        "vostok/work/tasks/SPEC-001/TASK-02.md",
        "---\nid: TASK-02\nstatus: pending\n---\n",
    );
    for step in ["baseline", "tests", "regression", "evidence"] {
        task(&project, &[step]).assert().success();
    }
    project
}

/// Tarefa pendente, completa e válida, pronta para aprovar.
fn pending_task() -> Project {
    let project = Project::new();
    write(
        &project,
        TASK_PATH,
        "---\nid: TASK-01\nspec: SPEC-001\ntitle: \"Tarefa\"\neffort: simples\ndepends_on: []\ncovers: [RF-01]\ntype: tarefa\nlabels: [SPEC-001, simples]\nstatus: pending\ncontent_hash: null\nadjustments: 0\nadjusted_at: null\napproved_at: null\napproved_hash: null\nattempts: 0\ntests_result: null\nregression_result: null\nevidence_result: null\n---\n\n# TASK-01: Tarefa\n\n## Execução\n\n### Ler\n\n- `README.md`\n\n### Mudar\n\n- `novo.txt` (criar): arquivo novo.\n\n### Passos\n\n1. Crie o arquivo.\n\n### Baseline\n\n- `true`\n\n### Verificação\n\n```bash\ntrue\n```\n\n### Evidências\n\nNão se aplica.\n\n### Concluída quando\n\n- [ ] Pronto (RF-01).\n\n  > Como comprovar: ver o arquivo.\n\n## Descrição\n\n### Resumo executivo\n\nResumo.\n\n### Detalhes técnicos\n\nDetalhes.\n\n### Card\n\n- **Título:** Tarefa\n- **Tipo:** tarefa\n- **Descrição:** Descrição.\n- **Critérios de aceite:**\n  - [ ] Pronto.\n- **Rótulos:** SPEC-001, simples\n- **Estimativa:** simples\n- **Bloqueada por:** -\n\n## Registro de execução\n\n| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n\n## Histórico de ajustes\n\n| # | Data | Origem | Resumo |\n|---|---|---|---|\n",
    );
    task(&project, &["adjust", "record-manual"])
        .assert()
        .success();
    project
}

/// Spec refinada, preenchida e registrada, pronta para aprovar.
fn refined_spec() -> Project {
    let project = Project::new();
    vostok(&project, &["spec", "new", "Exemplo"])
        .assert()
        .success();
    vostok(&project, &["spec", "refine", "SPEC-001", "improvement"])
        .assert()
        .success();
    let filler = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/fill-refined.py"
    );
    let filled = std::process::Command::new("python3")
        .arg(filler)
        .arg(project.root().join(SPEC_PATH))
        .status()
        .expect("preenche a spec");
    assert!(filled.success(), "fill-refined.py terminou com erro");
    write(&project, "AGENTS.md", "");
    vostok(&project, &["spec", "adjust", "record-manual", "SPEC-001"])
        .assert()
        .success();
    project
}

#[test]
fn finish_without_lock_works_as_before() {
    let project = task_ready_to_finish();
    task(&project, &["finish"]).assert().success();
    assert_eq!(field(&project, TASK_PATH, "status"), "done");
}

#[test]
fn finish_with_lock_needs_the_aval() {
    let project = task_ready_to_finish();
    lock(&project);
    add_aval(&project, "SPEC-001/TASK-01", APPROVED_HASH, "aprovar");
    let refused = failure_stderr(&mut task(&project, &["finish"]));
    assert!(refused.contains("finalizar-tarefa"), "{refused}");
    assert!(refused.contains(PANEL_INSTRUCTION), "{refused}");
    assert_eq!(field(&project, TASK_PATH, "status"), "validating");

    add_aval(
        &project,
        "SPEC-001/TASK-01",
        APPROVED_HASH,
        "finalizar-tarefa",
    );
    task(&project, &["finish"]).assert().success();
    assert_eq!(field(&project, TASK_PATH, "status"), "done");
}

#[test]
fn aval_of_another_hash_does_not_count() {
    let project = task_ready_to_finish();
    lock(&project);
    add_aval(
        &project,
        "SPEC-001/TASK-01",
        "hashantigo",
        "finalizar-tarefa",
    );
    let refused = failure_stderr(&mut task(&project, &["finish"]));
    assert!(refused.contains(APPROVED_HASH), "{refused}");
    assert!(refused.contains("hashantigo"), "{refused}");
    assert_eq!(field(&project, TASK_PATH, "status"), "validating");
}

#[test]
fn lock_without_avais_log_refuses() {
    let project = task_ready_to_finish();
    lock(&project);
    assert!(!project.root().join(AVAIS_LOG).exists());
    let refused = failure_stderr(&mut task(&project, &["finish"]));
    assert!(refused.contains(PANEL_INSTRUCTION), "{refused}");
    assert_eq!(field(&project, TASK_PATH, "status"), "validating");
}

#[test]
fn chat_approvals_need_the_aval_with_the_lock() {
    let project = pending_task();
    lock(&project);
    let refused = failure_stderr(&mut task(&project, &["approve", "approve"]));
    assert!(refused.contains("aprovar"), "{refused}");
    assert!(refused.contains(PANEL_INSTRUCTION), "{refused}");
    assert_eq!(field(&project, TASK_PATH, "status"), "pending");
    let hash = field(&project, TASK_PATH, "content_hash");
    add_aval(&project, "SPEC-001/TASK-01", &hash, "aprovar");
    task(&project, &["approve", "approve"]).assert().success();
    assert_eq!(field(&project, TASK_PATH, "status"), "approved");

    let project = refined_spec();
    lock(&project);
    let refused = failure_stderr(&mut vostok(
        &project,
        &["spec", "approve", "approve", "SPEC-001"],
    ));
    assert!(refused.contains("aprovar"), "{refused}");
    assert!(refused.contains(PANEL_INSTRUCTION), "{refused}");
    assert_eq!(field(&project, SPEC_PATH, "status"), "refined");
    let hash = field(&project, SPEC_PATH, "content_hash");
    add_aval(&project, "SPEC-001", &hash, "aprovar");
    vostok(&project, &["spec", "approve", "approve", "SPEC-001"])
        .assert()
        .success();
    assert_eq!(field(&project, SPEC_PATH, "status"), "approved");
}

#[test]
fn panel_approval_without_aval_is_reported() {
    let project = pending_task();
    task(&project, &["approve", "approve"]).assert().success();
    replace_in(
        &project,
        TASK_PATH,
        "approved_by: chat",
        "approved_by: painel",
    );
    let refused = failure_stderr(&mut task(&project, &["adjust", "validate"]));
    assert!(refused.contains(PANEL_WITHOUT_AVAL), "{refused}");
    let hash = field(&project, TASK_PATH, "approved_hash");
    add_aval(&project, "SPEC-001/TASK-01", &hash, "aprovar");
    task(&project, &["adjust", "validate"]).assert().success();

    let project = refined_spec();
    vostok(&project, &["spec", "approve", "approve", "SPEC-001"])
        .assert()
        .success();
    replace_in(
        &project,
        SPEC_PATH,
        "approved_by: chat",
        "approved_by: painel",
    );
    for command in [
        &["spec", "validate", "SPEC-001"][..],
        &["spec", "adjust", "validate", "SPEC-001"][..],
    ] {
        let refused = failure_stderr(&mut vostok(&project, command));
        assert!(refused.contains(PANEL_WITHOUT_AVAL), "{refused}");
    }
    let hash = field(&project, SPEC_PATH, "approved_hash");
    add_aval(&project, "SPEC-001", &hash, "aprovar");
    vostok(&project, &["spec", "validate", "SPEC-001"])
        .assert()
        .success();
    vostok(&project, &["spec", "adjust", "validate", "SPEC-001"])
        .assert()
        .success();
}
