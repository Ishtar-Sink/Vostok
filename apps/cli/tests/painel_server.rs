//! Testes do `vostok painel` com o binário real: localização do harness, escuta só em
//! `127.0.0.1`, porta ocupada e recusa das ações sem o token de sessão (RF-01, RF-08, CT-02, CT-05).
#![cfg(feature = "painel")]

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::Path;
use std::process::{Child, Command, Output, Stdio};
use std::time::Duration;

use tempfile::TempDir;

/// O processo do painel, encerrado ao fim do teste.
struct Running {
    child: Child,
    address: SocketAddr,
}

impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn vostok() -> Command {
    Command::new(env!("CARGO_BIN_EXE_vostok"))
}

fn project() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("vostok/config");
    fs::create_dir_all(&config).unwrap();
    fs::write(config.join("main.yml"), "paths:\n  work: ./work\n").unwrap();
    dir
}

fn start(folder: &Path) -> Running {
    let mut child = vostok()
        .args(["painel", "--sem-navegador", "--porta", "0", "--pasta"])
        .arg(folder)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut line = String::new();
    BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut line)
        .unwrap();
    let url = line
        .split_whitespace()
        .last()
        .unwrap_or_else(|| panic!("sem URL na saída: {line:?}"));
    let address = url
        .strip_prefix("http://")
        .and_then(|rest| rest.strip_suffix('/'))
        .unwrap_or_else(|| panic!("URL inesperada: {url}"))
        .parse()
        .unwrap();
    Running { child, address }
}

fn request(address: SocketAddr, raw: &str) -> String {
    let mut stream = TcpStream::connect(address).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    stream.write_all(raw.as_bytes()).unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}

fn status_line(response: &str) -> &str {
    response.lines().next().unwrap_or_default()
}

fn run_painel(args: &[&str]) -> Output {
    vostok().arg("painel").args(args).output().unwrap()
}

#[test]
fn ct01_listens_only_on_loopback_and_announces_the_folder() {
    let project = project();
    let painel = start(project.path());
    assert!(painel.address.ip().is_loopback(), "{}", painel.address);
    assert_eq!(painel.address.ip().to_string(), "127.0.0.1");
    assert_ne!(painel.address.port(), 0);
}

#[test]
fn ct05_action_without_token_is_403() {
    let project = project();
    let painel = start(project.path());
    let response = request(
        painel.address,
        "POST /api/aprovacoes?spec=SPEC-001 HTTP/1.1\r\nHost: x\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
    );
    assert!(status_line(&response).contains(" 403 "), "{response}");
}

#[test]
fn ct05_action_with_wrong_token_is_403() {
    let project = project();
    let painel = start(project.path());
    let response = request(
        painel.address,
        "PUT /api/arquivo?caminho=work/x.md HTTP/1.1\r\nHost: x\r\nX-Painel-Token: errado\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok",
    );
    assert!(status_line(&response).contains(" 403 "), "{response}");
}

#[test]
fn unknown_api_route_is_404() {
    let project = project();
    let painel = start(project.path());
    let response = request(
        painel.address,
        "GET /api/rota-desconhecida HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n",
    );
    assert!(status_line(&response).contains(" 404 "), "{response}");
}

#[test]
fn page_carries_the_token_or_explains_the_missing_bundle() {
    let project = project();
    let painel = start(project.path());
    let response = request(
        painel.address,
        "GET /specs/SPEC-001 HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n",
    );
    let served = status_line(&response).contains(" 200 ")
        && response.contains("<meta name=\"painel-token\" content=\"");
    let explained =
        status_line(&response).contains(" 503 ") && response.contains("--features painel");
    assert!(served || explained, "{response}");
}

#[test]
fn harness_is_found_from_a_subfolder() {
    let project = project();
    let nested = project.path().join("apps/web");
    fs::create_dir_all(&nested).unwrap();
    let painel = start(&nested);
    assert!(painel.address.ip().is_loopback());
}

#[test]
fn ct02_folder_without_harness_fails_citing_the_config() {
    let output = run_painel(&["--pasta", "/", "--sem-navegador"]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("vostok/config/main.yml"), "{stderr}");
    assert!(output.stdout.is_empty());
}

#[test]
fn missing_folder_fails_with_its_path() {
    let output = run_painel(&["--pasta", "/caminho/que/nao/existe", "--sem-navegador"]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("/caminho/que/nao/existe"), "{stderr}");
}

#[test]
fn cb01_busy_port_fails_citing_the_port() {
    let project = project();
    let busy = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = busy.local_addr().unwrap().port().to_string();
    let output = run_painel(&[
        "--pasta",
        project.path().to_str().unwrap(),
        "--porta",
        &port,
        "--sem-navegador",
    ]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(&port) && stderr.contains("ocupada"),
        "{stderr}"
    );
}

#[test]
fn invalid_port_is_refused_by_the_arguments() {
    let output = run_painel(&["--porta", "70000", "--sem-navegador"]);
    assert!(!output.status.success());
}
