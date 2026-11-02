//! Testes da trava de aval (SPEC-030) na aprovação do plano, no arquivamento da spec e nos commits
//! (CT-05, CT-06, CT-07, CT-08, CB-04).

mod common;

use std::fs;
use std::path::Path;
use std::process::Command as Process;

use assert_cmd::Command;
use common::{failure_stderr, Project};
use vostok::support::{config_set, sha256_hex};

const AVAL_REFUSAL: &str = "sem o aval do painel";

fn write_file(project: &Project, relative: &str, content: &str) {
    let path = project.root().join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn lock(project: &Project) {
    config_set(
        &project.harness().join("config/main.yml"),
        "approval_lock",
        "true",
    )
    .unwrap();
}

fn add_aval(project: &Project, id: &str, hash: &str, action: &str) {
    let log = project.harness().join("work/avais.log");
    fs::create_dir_all(log.parent().unwrap()).unwrap();
    let mut content = fs::read_to_string(&log).unwrap_or_default();
    content.push_str(&format!(
        "2026-11-02T10:00:00Z\t{id}\t{hash}\tpainel\t{action}\n"
    ));
    fs::write(log, content).unwrap();
}

// --- Plano (CT-05) ---

/// Spec aprovada, plano e TASK-01 válidos; devolve o `content_hash` do plano.
fn approved_plan(project: &Project) -> String {
    let spec_body = "\n# Exemplo\n\n## Requisitos funcionais\n\n| ID | Requisito |\n|---|---|\n| RF-01 | Entregar. |\n";
    let plan_body = "\n# Plano\n\n## Visão geral\n\nUma tarefa.\n\n## Tarefas\n\n| ID | Título | Descrição | Tipo | Nível | Dependências | Cobre |\n|---|---|---|---|---|---|---|\n| TASK-01 | Tarefa | Entrega. | tarefa | simples | - | RF-01 |\n\n## Fatos de esforço\n\n| ID | Arquivos a criar | Arquivos a alterar | Contrato público | Inferência | Risco alto |\n|---|---|---|---|---|---|\n| TASK-01 | 0 | 1 | não | não | não |\n";
    let spec_hash = sha256_hex(spec_body.as_bytes());
    write_file(
        project,
        "vostok/work/specs/refined/SPEC-001-exemplo.md",
        &format!("---\nid: SPEC-001\ntitle: \"Exemplo\"\nstatus: approved\ntype: improvement\nupdated_at: null\ncontent_hash: {spec_hash}\napproved_hash: {spec_hash}\n---\n{spec_body}"),
    );
    let plan_hash = sha256_hex(plan_body.as_bytes());
    write_file(
        project,
        "vostok/work/plans/SPEC-001.md",
        &format!("---\nspec: SPEC-001\nspec_hash: {spec_hash}\ncontent_hash: {plan_hash}\n---\n{plan_body}"),
    );
    write_file(
        project,
        "vostok/work/tasks/SPEC-001/TASK-01.md",
        "---\nid: TASK-01\nspec: SPEC-001\ntitle: \"Tarefa\"\neffort: simples\ndepends_on: []\ncovers: [RF-01]\ntype: tarefa\nlabels: [SPEC-001, simples]\nstatus: pending\ncontent_hash: null\nadjustments: 0\nadjusted_at: null\napproved_at: null\napproved_hash: null\nattempts: 0\ntests_result: null\nregression_result: null\nevidence_result: null\n---\n\n# TASK-01: Tarefa\n\n## Execução\n\n### Ler\n\n- `README.md`\n\n### Mudar\n\n- `novo.txt` (criar): arquivo novo.\n\n### Passos\n\n1. Crie o arquivo.\n\n### Baseline\n\n- `true`\n\n### Verificação\n\n```bash\ntrue\n```\n\n### Evidências\n\n- `saida.txt` (automática): `echo ok`\n  - Prova: ok.\n  - Trecho: ok.\n  - Falha: erro.\n\n### Concluída quando\n\n- [ ] Pronto (RF-01).\n\n  > Como comprovar: ver o arquivo.\n\n## Descrição\n\n### Resumo executivo\n\nResumo.\n\n### Detalhes técnicos\n\nDetalhes.\n\n### Card\n\n- **Título:** Tarefa\n- **Tipo:** tarefa\n- **Descrição:** Descrição.\n- **Critérios de aceite:**\n  - [ ] Pronto.\n- **Rótulos:** SPEC-001, simples\n- **Estimativa:** simples\n- **Bloqueada por:** -\n\n## Registro de execução\n\n| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n\n## Histórico de ajustes\n\n| # | Data | Origem | Resumo |\n|---|---|---|---|\n",
    );
    plan_hash
}

fn mark_planned(project: &Project) -> Command {
    let mut command = project.vostok();
    command.args(["plan", "approve", "mark-planned", "SPEC-001"]);
    command
}

#[test]
fn mark_planned_needs_the_plan_aval() {
    let project = Project::new();
    let plan_hash = approved_plan(&project);
    lock(&project);
    let plan = project.harness().join("work/plans/SPEC-001.md");

    let stderr = failure_stderr(&mut mark_planned(&project));
    assert!(stderr.contains(AVAL_REFUSAL), "{stderr}");
    assert!(stderr.contains("aprovar-plano de SPEC-001"), "{stderr}");
    assert!(plan.is_file());

    add_aval(&project, "SPEC-001", "outro-hash", "aprovar-plano");
    add_aval(&project, "SPEC-001", &plan_hash, "finalizar-spec");
    let stderr = failure_stderr(&mut mark_planned(&project));
    assert!(
        stderr.contains("o aval existente é do hash outro-hash"),
        "{stderr}"
    );
    assert!(plan.is_file());

    add_aval(&project, "SPEC-001", &plan_hash, "aprovar-plano");
    mark_planned(&project).assert().success();
    assert!(!plan.exists());
}

#[test]
fn mark_planned_without_the_lock_needs_no_aval() {
    let project = Project::new();
    approved_plan(&project);
    mark_planned(&project).assert().success();
}

// --- Arquivamento (CT-06) ---

const SPEC_HASH: &str = "hash-da-spec";

/// Spec implementada com `content_hash` e uma tarefa em pasta, com os documentos já gerados.
fn built_spec() -> Project {
    let project = Project::new();
    write_file(
        &project,
        "vostok/work/specs/refined/SPEC-001-exemplo.md",
        &format!("---\nid: SPEC-001\ntitle: Exemplo\nstatus: implemented\ncontent_hash: {SPEC_HASH}\n---\n\n# Exemplo\n"),
    );
    let task = "vostok/work/tasks/SPEC-001/TASK-01";
    let files = [
        ("estado.yml", "id: TASK-01\ntitle: Tarefa em pasta\nstatus: done\n"),
        ("execucao.md", "### Ler\n\n- `a.rs`\n\n### Mudar\n\n- `src/novo.rs` (criar): novo\n\n### Passos\n\n1. Fazer.\n"),
        ("verificacao.md", "### Baseline\n\n- `true`\n\n### Verificação\n\n```bash\ntrue\n```\n\n### Evidências\n\n- `saida.txt` (automática): `echo depois`\n"),
        ("criterios.md", "### Concluída quando\n\n- [x] Feito.\n"),
        ("executivo.md", "Resumo.\n"),
        ("tecnico.md", "Detalhe.\n"),
        ("registro.md", "## Registro de execução\n\n| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n| 1 | 2026-10-14 | conclusão | done | ok |\n"),
        ("evidencias/saida.txt", "saida\n"),
    ];
    for (name, text) in files {
        write_file(&project, &format!("{task}/{name}"), text);
    }
    project
        .vostok()
        .args(["finalize", "create", "SPEC-001"])
        .assert()
        .success();
    let content_path = project.harness().join("work/finalize/SPEC-001/content.md");
    let content = fs::read_to_string(&content_path)
        .unwrap()
        .replace("#{EVIDENCE_01_PROOF}", "a saída de CT-01.")
        .replace("#{EVIDENCE_01_HOW}", "rodar.")
        .replace("#{EVIDENCE_01_READ}", "ler.");
    fs::write(&content_path, fill_variables(&content)).unwrap();
    project
        .vostok()
        .args(["finalize", "build", "SPEC-001"])
        .assert()
        .success();
    project
}

/// Troca cada variável `#{...}` por um texto válido para o tipo da seção.
fn fill_variables(content: &str) -> String {
    let mut out = String::new();
    for line in content.lines() {
        if let Some(start) = line.find("#{") {
            let name = &line[start + 2..line.find('}').unwrap()];
            out.push_str(match name {
                "EXEC_NUMBERS" => "- **2** tarefas concluídas",
                "TECH_NUMBERS" => "Não se aplica.",
                "TECH_OPERATION" => "1. Rodar `vostok finalize build`.",
                "TECH_CODE_MAP" => "| Caminho | Papel | Testes |\n|---|---|---|\n| `src/novo.rs` | novo | `novo_` |",
                "EVIDENCE_COVERAGE" => "| ID | O que foi verificado | Evidência |\n|---|---|---|\n| CT-01 | saída | TASK-01 / saida.txt |",
                _ => "Texto.",
            });
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

fn archive(project: &Project) -> Command {
    let mut command = project.vostok();
    command.args(["finalize", "archive", "SPEC-001"]);
    command
}

#[test]
fn archive_needs_the_spec_aval() {
    let project = built_spec();
    lock(&project);
    let tasks = project.harness().join("work/tasks/SPEC-001");

    let stderr = failure_stderr(&mut archive(&project));
    assert!(stderr.contains(AVAL_REFUSAL), "{stderr}");
    assert!(stderr.contains("finalizar-spec de SPEC-001"), "{stderr}");
    assert!(tasks.is_dir());

    add_aval(&project, "SPEC-001", SPEC_HASH, "aprovar-plano");
    assert!(failure_stderr(&mut archive(&project)).contains(AVAL_REFUSAL));
    assert!(tasks.is_dir());

    add_aval(&project, "SPEC-001", SPEC_HASH, "finalizar-spec");
    archive(&project).assert().success();
    assert!(!tasks.exists());
    assert!(project.harness().join("work/.archive/SPEC-001").is_dir());
}

// --- Commit (CT-07, CT-08, CB-04) ---

const REQUEST_TEXT: &str = "Commit da onda 1.";

fn git(root: &Path, args: &[&str]) {
    let status = Process::new("git")
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

fn commit_cmd(project: &Project, args: &[&str]) -> Command {
    let mut command = project.vostok();
    command.arg("commit").args(args).envs(git_identity());
    command
}

fn stdout(command: &mut Command) -> String {
    let output = command.assert().success().get_output().stdout.clone();
    String::from_utf8(output).unwrap()
}

/// Repositório git com a referência de commits gravada e a trava ligada.
fn locked_git_project() -> Project {
    let project = Project::new();
    config_set(
        &project.harness().join("config/main.yml"),
        "paths.root",
        "../",
    )
    .unwrap();
    git(project.root(), &["init", "-q", "-b", "main"]);
    stdout(&mut commit_cmd(&project, &["check"]));
    lock(&project);
    project
}

fn write_request(project: &Project, id: &str, action: &str, state: &str) {
    write_file(
        project,
        &format!("vostok/work/pedidos/{id}.md"),
        &format!("---\nid: {id}\nacao: {action}\nspec: \ntarefa: \nestado: {state}\norigem: painel\ncriado_em: 2026-11-02T10:00:00Z\natualizado_em: 2026-11-02T10:00:00Z\n---\n\n## Pedido\n\n{REQUEST_TEXT}\n\n## Resultado\n\n"),
    );
}

fn request_hash() -> String {
    sha256_hex(REQUEST_TEXT.as_bytes())
}

fn head_count(project: &Project) -> usize {
    let output = Process::new("git")
        .arg("-C")
        .arg(project.root())
        .args(["rev-list", "--all", "--count"])
        .output()
        .unwrap();
    String::from_utf8(output.stdout)
        .unwrap()
        .trim()
        .parse()
        .unwrap()
}

#[test]
fn commit_needs_a_request_with_aval() {
    let project = locked_git_project();
    write_file(&project, "a.txt", "a\n");

    let stderr =
        failure_stderr(commit_cmd(&project, &["commit", "a.txt"]).write_stdin("feat: a\n"));
    assert!(stderr.contains("--pedido"), "{stderr}");
    assert_eq!(head_count(&project), 0);

    write_request(&project, "PED-001", "fazer-commit", "em_andamento");
    let stderr = failure_stderr(
        commit_cmd(&project, &["commit", "--pedido", "PED-001", "a.txt"]).write_stdin("feat: a\n"),
    );
    assert!(stderr.contains(AVAL_REFUSAL), "{stderr}");
    assert!(stderr.contains("fazer-commit de PED-001"), "{stderr}");
    assert_eq!(head_count(&project), 0);

    add_aval(&project, "PED-001", "texto-antigo", "fazer-commit");
    assert!(failure_stderr(
        commit_cmd(&project, &["commit", "--pedido", "PED-001", "a.txt"]).write_stdin("feat: a\n"),
    )
    .contains(AVAL_REFUSAL));

    add_aval(&project, "PED-001", &request_hash(), "fazer-commit");
    let out = stdout(
        commit_cmd(&project, &["commit", "--pedido", "PED-001", "a.txt"]).write_stdin("feat: a\n"),
    );
    assert!(out.starts_with("Commit feito e liberado: "), "{out}");
    assert_eq!(head_count(&project), 1);
}

#[test]
fn commit_refuses_a_finished_request() {
    let project = locked_git_project();
    write_file(&project, "a.txt", "a\n");
    add_aval(&project, "PED-001", &request_hash(), "fazer-commit");
    for state in ["feito", "pendente", "recusado"] {
        write_request(&project, "PED-001", "fazer-commit", state);
        let stderr = failure_stderr(
            commit_cmd(&project, &["commit", "--pedido", "PED-001", "a.txt"])
                .write_stdin("feat: a\n"),
        );
        assert!(stderr.contains("em_andamento"), "{state}: {stderr}");
        assert!(stderr.contains(&format!("'{state}'")), "{state}: {stderr}");
    }
    assert_eq!(head_count(&project), 0);
}

#[test]
fn commit_refuses_a_request_of_another_action_or_missing() {
    let project = locked_git_project();
    write_file(&project, "a.txt", "a\n");
    let attempt = |project: &Project| {
        failure_stderr(
            commit_cmd(project, &["commit", "--pedido", "PED-001", "a.txt"])
                .write_stdin("feat: a\n"),
        )
    };

    let stderr = attempt(&project);
    assert!(stderr.contains("pedido PED-001 não encontrado"), "{stderr}");

    for invalid in ["../PED-001", "PED-", "PED-1a", ""] {
        let stderr = failure_stderr(
            commit_cmd(&project, &["commit", "--pedido", invalid, "a.txt"])
                .write_stdin("feat: a\n"),
        );
        assert!(stderr.contains("pedido inválido"), "{invalid}: {stderr}");
    }

    write_request(&project, "PED-001", "ajustar-spec", "em_andamento");
    add_aval(&project, "PED-001", &request_hash(), "fazer-commit");
    let stderr = attempt(&project);
    assert!(stderr.contains("fazer-commit"), "{stderr}");
    assert!(stderr.contains("'ajustar-spec'"), "{stderr}");
    assert_eq!(head_count(&project), 0);
}

#[test]
fn commit_approve_needs_the_request() {
    let project = locked_git_project();
    write_file(&project, "a.txt", "a\n");
    git(project.root(), &["add", "a.txt"]);
    git(project.root(), &["commit", "-q", "-m", "feat: a"]);
    let shown = stdout(&mut commit_cmd(&project, &["show"]));
    let code = shown
        .lines()
        .last()
        .unwrap()
        .strip_prefix("Código de liberação: ")
        .unwrap()
        .to_string();

    let stderr = failure_stderr(&mut commit_cmd(&project, &["approve", &code]));
    assert!(stderr.contains("--pedido"), "{stderr}");

    write_request(&project, "PED-001", "fazer-commit", "em_andamento");
    let stderr = failure_stderr(&mut commit_cmd(
        &project,
        &["approve", "--pedido", "PED-001", &code],
    ));
    assert!(stderr.contains(AVAL_REFUSAL), "{stderr}");
    assert!(failure_stderr(&mut commit_cmd(&project, &["check"])).contains("não liberados"));

    add_aval(&project, "PED-001", &request_hash(), "fazer-commit");
    let out = stdout(&mut commit_cmd(
        &project,
        &["approve", "--pedido", "PED-001", &code],
    ));
    assert!(out.starts_with("Liberação gravada: 1 commit"), "{out}");
    assert_eq!(
        stdout(&mut commit_cmd(&project, &["check"])),
        "Nenhum commit sem liberação.\n"
    );
}

#[test]
fn commit_without_the_lock_needs_no_request() {
    let project = Project::new();
    config_set(
        &project.harness().join("config/main.yml"),
        "paths.root",
        "../",
    )
    .unwrap();
    git(project.root(), &["init", "-q", "-b", "main"]);
    stdout(&mut commit_cmd(&project, &["check"]));
    write_file(&project, "a.txt", "a\n");
    let out = stdout(commit_cmd(&project, &["commit", "a.txt"]).write_stdin("feat: a\n"));
    assert!(out.starts_with("Commit feito e liberado: "), "{out}");
}
