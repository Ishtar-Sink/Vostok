//! Testes de ponta a ponta do disparo sob demanda (`request_dispatch`) do `vostok painel`.
#![cfg(feature = "painel")]

mod common;

use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStderr, Command, Stdio};
use std::time::{Duration, Instant};

use common::Project;

struct Running {
    child: Child,
    stderr: Option<ChildStderr>,
}

impl Running {
    /// Encerra o painel e devolve o que ele escreveu no stderr.
    fn stop(mut self) -> String {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let mut text = String::new();
        if let Some(mut stderr) = self.stderr.take() {
            let _ = stderr.read_to_string(&mut text);
        }
        text
    }
}

impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn set_dispatch(project: &Project, value: &str) {
    let main = project.harness().join("config/main.yml");
    let content = fs::read_to_string(&main).unwrap();
    let changed: String = content
        .lines()
        .map(|line| {
            if line.starts_with("request_dispatch:") {
                format!("request_dispatch: {value}\n")
            } else {
                format!("{line}\n")
            }
        })
        .collect();
    assert_ne!(changed, content, "sem a linha request_dispatch");
    fs::write(main, changed).unwrap();
}

fn write_pending(project: &Project, id: &str) {
    let path = project
        .harness()
        .join(format!("work/pedidos/pendente/{id}.md"));
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path,
        format!("---\nid: {id}\nacao: ajustar-tarefa\nspec: SPEC-001\ntarefa: TASK-01\nestado: pendente\norigem: painel\ncriado_em: 2026-11-02T10:00:00Z\natualizado_em: 2026-11-02T10:00:00Z\n---\n\n## Pedido\n\nAcrescentar um critério.\n\n## Resultado\n\n"),
    )
    .unwrap();
}

fn fake_agent(project: &Project, name: &str, log: &Path) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let bin = project.root().join("bin");
    fs::create_dir_all(&bin).unwrap();
    let script = bin.join(name);
    fs::write(
        &script,
        format!(
            "#!/bin/sh\nid=$(echo \"$*\" | grep -o 'PED-[0-9]*' | head -n 1)\necho \"inicio $id\" >> '{log}'\necho \"{name} $*\" >> '{log}'\nsleep 0.3\ncd '{root}' && vostok pedidos iniciar \"$id\" >/dev/null && printf 'feito pelo {name}' | vostok pedidos concluir \"$id\" >/dev/null\necho \"fim $id\" >> '{log}'\n",
            log = log.display(),
            root = project.root().display()
        ),
    )
    .unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    bin
}

fn path_with(first: &Path) -> String {
    let vostok_dir = Path::new(env!("CARGO_BIN_EXE_vostok"))
        .parent()
        .unwrap()
        .to_path_buf();
    format!("{}:{}:/usr/bin:/bin", first.display(), vostok_dir.display())
}

fn start(project: &Project, path: &str) -> Running {
    let mut child = Command::new(env!("CARGO_BIN_EXE_vostok"))
        .args(["painel", "--sem-navegador", "--porta", "0", "--pasta"])
        .arg(project.root())
        .env("PATH", path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap());
    let mut line = String::new();
    output.read_line(&mut line).unwrap();
    assert!(line.contains("http://"), "sem URL na saída: {line:?}");
    // O painel segue escrevendo na saída; um pipe fechado o derrubaria.
    std::thread::spawn(move || std::io::copy(&mut output, &mut std::io::sink()));
    let stderr = child.stderr.take();
    Running { child, stderr }
}

fn state_of(project: &Project, id: &str) -> String {
    let base = project.harness().join("work/pedidos");
    for folder in ["pendente", "em_andamento", "feito", "recusado"] {
        if let Ok(content) = fs::read_to_string(base.join(folder).join(format!("{id}.md"))) {
            return content
                .lines()
                .find_map(|line| line.strip_prefix("estado: ").map(str::to_string))
                .unwrap_or_default();
        }
    }
    String::new()
}

fn wait_until(limit: Duration, done: impl Fn() -> bool) -> bool {
    let start = Instant::now();
    while start.elapsed() < limit {
        if done() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    done()
}

#[test]
fn disparo_com_codex_chama_o_agente_uma_vez_por_pedido_em_ordem() {
    let project = Project::new();
    set_dispatch(&project, "codex");
    write_pending(&project, "PED-002");
    write_pending(&project, "PED-001");
    let log = project.root().join("agente.log");
    let bin = fake_agent(&project, "codex", &log);
    let running = start(&project, &path_with(&bin));
    let finished = wait_until(Duration::from_secs(20), || {
        state_of(&project, "PED-001") == "feito" && state_of(&project, "PED-002") == "feito"
    });
    running.stop();
    assert!(finished, "pedidos não terminaram");
    let log = fs::read_to_string(log).unwrap();
    let marks: Vec<&str> = log
        .lines()
        .filter(|line| line.starts_with("inicio ") || line.starts_with("fim "))
        .collect();
    assert_eq!(
        marks,
        [
            "inicio PED-001",
            "fim PED-001",
            "inicio PED-002",
            "fim PED-002"
        ],
        "{log}"
    );
    let calls: Vec<&str> = log
        .lines()
        .filter(|line| line.starts_with("codex exec --full-auto"))
        .collect();
    assert_eq!(calls.len(), 2, "{log}");
    assert!(
        calls[0].contains("PED-001") && calls[1].contains("PED-002"),
        "{log}"
    );
}

#[test]
fn disparo_com_claude_usa_o_modo_nao_interativo() {
    let project = Project::new();
    set_dispatch(&project, "claude");
    write_pending(&project, "PED-001");
    let log = project.root().join("agente.log");
    let bin = fake_agent(&project, "claude", &log);
    let running = start(&project, &path_with(&bin));
    let finished = wait_until(Duration::from_secs(20), || {
        state_of(&project, "PED-001") == "feito"
    });
    running.stop();
    assert!(finished, "o pedido não terminou");
    let log = fs::read_to_string(log).unwrap();
    assert!(
        log.lines()
            .any(|line| line.starts_with("claude -p") && line.contains("PED-001")),
        "{log}"
    );
}

#[test]
fn disparo_desligado_nao_chama_o_agente() {
    let project = Project::new();
    set_dispatch(&project, "\"\"");
    write_pending(&project, "PED-001");
    let log = project.root().join("agente.log");
    let bin = fake_agent(&project, "codex", &log);
    let running = start(&project, &path_with(&bin));
    std::thread::sleep(Duration::from_secs(4));
    running.stop();
    assert!(!log.exists());
    assert_eq!(state_of(&project, "PED-001"), "pendente");
}

#[test]
fn disparo_sem_o_agente_no_path_avisa_e_nao_recusa() {
    let project = Project::new();
    set_dispatch(&project, "codex");
    write_pending(&project, "PED-001");
    let empty = project.root().join("vazio");
    fs::create_dir_all(&empty).unwrap();
    let running = start(&project, &path_with(&empty));
    std::thread::sleep(Duration::from_secs(4));
    let stderr = running.stop();
    assert!(stderr.contains("codex"), "{stderr}");
    assert!(stderr.contains("PATH"), "{stderr}");
    assert_eq!(state_of(&project, "PED-001"), "pendente");
}

#[test]
fn disparo_com_valor_invalido_recusa_o_painel() {
    let project = Project::new();
    set_dispatch(&project, "gemini");
    let output = Command::new(env!("CARGO_BIN_EXE_vostok"))
        .args(["painel", "--sem-navegador", "--porta", "0", "--pasta"])
        .arg(project.root())
        .output()
        .unwrap();
    assert!(!output.status.success(), "{output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("codex") && stderr.contains("claude"),
        "{stderr}"
    );
}
