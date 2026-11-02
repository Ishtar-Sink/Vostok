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

/// Pasta com um `codex` simulado: anota início e fim de cada execução no log e processa o pedido
/// citado na instrução pelo próprio `vostok pedidos`.
fn fake_codex(project: &Project, log: &std::path::Path) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let bin = project.root().join("bin");
    fs::create_dir_all(&bin).unwrap();
    let script = bin.join("codex");
    fs::write(
        &script,
        format!(
            "#!/bin/sh\nid=$(echo \"$*\" | grep -o 'PED-[0-9]*' | head -n 1)\necho \"inicio $id\" >> '{log}'\nsleep 0.3\ncd '{root}' && vostok pedidos iniciar \"$id\" >/dev/null && printf 'feito pelo codex' | vostok pedidos concluir \"$id\" >/dev/null\necho \"fim $id\" >> '{log}'\n",
            log = log.display(),
            root = project.root().display()
        ),
    )
    .unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    bin
}

fn path_with(first: &std::path::Path) -> String {
    let vostok_dir = std::path::Path::new(env!("CARGO_BIN_EXE_vostok"))
        .parent()
        .unwrap()
        .to_path_buf();
    format!("{}:{}:/usr/bin:/bin", first.display(), vostok_dir.display())
}

fn state_of(project: &Project, id: &str) -> String {
    fs::read_to_string(request_file(project, id))
        .unwrap()
        .lines()
        .find_map(|line| line.strip_prefix("estado: ").map(str::to_string))
        .unwrap_or_default()
}

#[test]
fn vigiar_runs_codex_once_per_request_in_order() {
    let project = Project::new();
    write_request(&project, "PED-002", "ajustar-tarefa", "pendente");
    write_request(&project, "PED-001", "ajustar-tarefa", "pendente");
    let log = project.root().join("codex.log");
    let bin = fake_codex(&project, &log);

    let mut watcher = std::process::Command::new(env!("CARGO_BIN_EXE_vostok"))
        .args(["pedidos", "vigiar", "--agente", "codex", "--intervalo", "1"])
        .current_dir(project.root())
        .env("PATH", path_with(&bin))
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    while ["PED-001", "PED-002"]
        .iter()
        .any(|id| state_of(&project, id) != "feito")
    {
        assert!(
            std::time::Instant::now() < deadline,
            "os pedidos não ficaram feitos a tempo"
        );
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    watcher.kill().unwrap();
    watcher.wait().unwrap();

    let lines: Vec<String> = fs::read_to_string(&log)
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect();
    assert_eq!(
        lines,
        [
            "inicio PED-001",
            "fim PED-001",
            "inicio PED-002",
            "fim PED-002"
        ]
    );
}

#[test]
fn vigiar_refuses_a_request_codex_left_pending() {
    use std::os::unix::fs::PermissionsExt;

    let project = Project::new();
    write_request(&project, "PED-001", "ajustar-tarefa", "pendente");
    let bin = project.root().join("bin");
    fs::create_dir_all(&bin).unwrap();
    let script = bin.join("codex");
    fs::write(&script, "#!/bin/sh\nexit 0\n").unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();

    let mut watcher = std::process::Command::new(env!("CARGO_BIN_EXE_vostok"))
        .args(["pedidos", "vigiar", "--agente", "codex", "--intervalo", "1"])
        .current_dir(project.root())
        .env("PATH", path_with(&bin))
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    while state_of(&project, "PED-001") != "recusado" {
        assert!(
            std::time::Instant::now() < deadline,
            "o pedido não foi recusado"
        );
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    watcher.kill().unwrap();
    watcher.wait().unwrap();

    let content = fs::read_to_string(request_file(&project, "PED-001")).unwrap();
    assert!(content.contains("o codex terminou sem processar o pedido"));
}

#[test]
fn vigiar_without_codex_stops_with_instructions() {
    let project = Project::new();
    let empty = project.root().join("vazio");
    fs::create_dir_all(&empty).unwrap();

    let stderr = failure_stderr(
        project
            .vostok()
            .env("PATH", empty.display().to_string())
            .args(["pedidos", "vigiar", "--agente", "codex"]),
    );

    assert!(stderr.contains("codex"), "{stderr}");
    assert!(stderr.contains("PATH"), "{stderr}");
}

#[test]
fn vigiar_rejects_an_unknown_agent() {
    let project = Project::new();
    let stderr = failure_stderr(
        project
            .vostok()
            .args(["pedidos", "vigiar", "--agente", "claude"]),
    );
    assert!(stderr.contains("codex"), "{stderr}");
}
