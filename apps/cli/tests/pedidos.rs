//! Testes de `vostok pedidos`: os pedidos do painel mudam de estado só pelo comando.

mod common;

use std::fs;
use std::path::PathBuf;

use common::{failure_stderr, Project};

fn request_file(project: &Project, id: &str) -> PathBuf {
    project
        .harness()
        .join("work/pedidos")
        .join(format!("{id}.md"))
}

fn write_request(project: &Project, id: &str, action: &str, state: &str) -> PathBuf {
    let path = request_file(project, id);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        format!(
            "---\nid: {id}\nacao: {action}\nspec: SPEC-001\ntarefa: TASK-02\nestado: {state}\norigem: painel\ncriado_em: 2026-01-01T00:00:00Z\natualizado_em: 2026-01-01T00:00:00Z\n---\n\n## Pedido\n\nAjuste o texto.\n\n## Resultado\n\n"
        ),
    )
    .unwrap();
    path
}

fn stdout_of(command: &mut assert_cmd::Command) -> String {
    String::from_utf8(command.assert().success().get_output().stdout.clone()).unwrap()
}

#[test]
fn listar_shows_pending_requests() {
    let project = Project::new();
    write_request(&project, "PED-002", "aprovar-tarefa", "feito");
    write_request(&project, "PED-001", "ajustar-tarefa", "pendente");

    let all = stdout_of(project.vostok().args(["pedidos", "listar"]));
    assert!(all.contains("| ID | Ação | Spec | Tarefa | Estado |"));
    assert!(all.contains("| PED-001 | ajustar-tarefa | SPEC-001 | TASK-02 | pendente |"));
    assert!(all.find("PED-001").unwrap() < all.find("PED-002").unwrap());

    let done = stdout_of(
        project
            .vostok()
            .args(["pedidos", "listar", "--estado", "feito"]),
    );
    assert!(done.contains("PED-002"));
    assert!(!done.contains("PED-001"));
}

#[test]
fn iniciar_moves_pending_to_em_andamento() {
    let project = Project::new();
    let path = write_request(&project, "PED-001", "ajustar-tarefa", "pendente");

    project
        .vostok()
        .args(["pedidos", "iniciar", "PED-001"])
        .assert()
        .success();

    let content = fs::read_to_string(path).unwrap();
    assert!(content.contains("estado: em_andamento"));
    assert!(!content.contains("atualizado_em: 2026-01-01T00:00:00Z"));
    assert!(content.contains("criado_em: 2026-01-01T00:00:00Z"));
}

#[test]
fn iniciar_refuses_a_request_not_pending() {
    let project = Project::new();
    let path = write_request(&project, "PED-001", "ajustar-tarefa", "em_andamento");
    let before = fs::read_to_string(&path).unwrap();

    let stderr = failure_stderr(project.vostok().args(["pedidos", "iniciar", "PED-001"]));

    assert!(stderr.contains("em_andamento"));
    assert_eq!(fs::read_to_string(path).unwrap(), before);
}

#[test]
fn concluir_writes_the_result() {
    let project = Project::new();
    let path = write_request(&project, "PED-001", "ajustar-tarefa", "em_andamento");

    project
        .vostok()
        .args(["pedidos", "concluir", "PED-001"])
        .write_stdin("Tarefa ajustada.\n")
        .assert()
        .success();

    let content = fs::read_to_string(path).unwrap();
    assert!(content.contains("estado: feito"));
    assert!(content.contains("## Resultado\n\nTarefa ajustada.\n"));
    assert!(content.contains("## Pedido\n\nAjuste o texto."));
}

#[test]
fn recusar_writes_the_reason() {
    let project = Project::new();
    let path = write_request(&project, "PED-001", "ajustar-tarefa", "em_andamento");

    project
        .vostok()
        .args(["pedidos", "recusar", "PED-001"])
        .write_stdin("Fora do escopo.")
        .assert()
        .success();

    let content = fs::read_to_string(path).unwrap();
    assert!(content.contains("estado: recusado"));
    assert!(content.contains("## Resultado\n\nFora do escopo.\n"));
}

#[test]
fn concluir_requires_em_andamento() {
    let project = Project::new();
    let path = write_request(&project, "PED-001", "ajustar-tarefa", "pendente");
    let before = fs::read_to_string(&path).unwrap();

    let stderr = failure_stderr(
        project
            .vostok()
            .args(["pedidos", "concluir", "PED-001"])
            .write_stdin("x"),
    );

    assert!(stderr.contains("pendente"));
    assert_eq!(fs::read_to_string(path).unwrap(), before);
}

#[test]
fn rejects_an_id_outside_the_pattern() {
    let project = Project::new();
    write_request(&project, "PED-001", "ajustar-tarefa", "pendente");

    failure_stderr(project.vostok().args(["pedidos", "iniciar", "../PED-001"]));
    failure_stderr(project.vostok().args(["pedidos", "iniciar", "PED-9"]));
}

#[test]
fn iniciar_fails_for_a_missing_request() {
    let project = Project::new();
    failure_stderr(project.vostok().args(["pedidos", "iniciar", "PED-001"]));
}
