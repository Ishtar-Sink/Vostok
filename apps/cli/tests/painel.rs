//! Testes de ponta a ponta do `vostok painel`: dois painéis ao mesmo tempo, cada um com a marca
//! da sua pasta, e o fluxo de aprovar, editar, pedir e receber o aviso ao vivo (RF-01, CT-01).
#![cfg(feature = "painel")]

mod common;

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use common::Project;

const FOLDER: &str = "vostok/work/tasks/SPEC-001/TASK-01";

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

fn start(folder: &Path) -> Running {
    let mut child = Command::new(env!("CARGO_BIN_EXE_vostok"))
        .args(["painel", "--sem-navegador", "--porta", "0", "--pasta"])
        .arg(folder)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap());
    let mut line = String::new();
    output.read_line(&mut line).unwrap();
    // O painel segue escrevendo na saída (o registro de ajuste imprime); um pipe fechado o derrubaria.
    std::thread::spawn(move || std::io::copy(&mut output, &mut std::io::sink()));
    let address = line
        .split_whitespace()
        .last()
        .and_then(|url| url.strip_prefix("http://"))
        .and_then(|rest| rest.strip_suffix('/'))
        .unwrap_or_else(|| panic!("sem URL na saída: {line:?}"))
        .parse()
        .unwrap();
    Running { child, address }
}

fn request(address: SocketAddr, method: &str, target: &str, token: &str, body: &str) -> String {
    let mut stream = TcpStream::connect(address).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(20)))
        .unwrap();
    let header = if token.is_empty() {
        String::new()
    } else {
        format!("X-Painel-Token: {token}\r\n")
    };
    let raw = format!(
        "{method} {target} HTTP/1.1\r\nHost: x\r\n{header}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(raw.as_bytes()).unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}

fn get(address: SocketAddr, target: &str) -> String {
    request(address, "GET", target, "", "")
}

fn assert_status(response: &str, status: &str) {
    let line = response.lines().next().unwrap_or_default();
    assert!(line.contains(&format!(" {status} ")), "{response}");
}

/// Valor de um campo texto do JSON (`"campo":"valor"`), sem depender de um parser.
fn json_field(response: &str, field: &str) -> String {
    let marker = format!("\"{field}\":\"");
    let start = response
        .find(&marker)
        .unwrap_or_else(|| panic!("sem {field}: {response}"))
        + marker.len();
    let end = start + response[start..].find('"').unwrap();
    response[start..end].to_string()
}

fn write(project: &Project, relative: &str, content: &str) {
    let path = project.root().join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn read(project: &Project, relative: &str) -> String {
    fs::read_to_string(project.root().join(relative)).unwrap_or_default()
}

/// Tarefa em pasta, pendente e válida, pronta para ser aprovada pelo painel.
fn pending_task() -> Project {
    let project = Project::new();
    let files = [
        ("estado.yml", "id: TASK-01\nspec: SPEC-001\ntitle: \"Tarefa\"\neffort: simples\ndepends_on: []\ncovers: [RF-01]\ntype: tarefa\nlabels: [SPEC-001, simples]\nstatus: pending\ncontent_hash: null\nadjustments: 0\nadjusted_at: null\napproved_at: null\napproved_hash: null\nattempts: 0\ntests_result: null\nregression_result: null\nevidence_result: null\n"),
        ("execucao.md", "### Ler\n\n- `README.md`\n\n### Mudar\n\n- `novo.txt` (criar): arquivo novo.\n\n### Passos\n\n1. Crie o arquivo.\n"),
        ("verificacao.md", "### Baseline\n\n- `true`\n\n### Verificação\n\n```bash\ntrue\n```\n\n### Evidências\n\nNão se aplica.\n"),
        ("criterios.md", "### Concluída quando\n\n- [ ] Pronto (RF-01).\n\n  > Como comprovar: ver o arquivo.\n"),
        ("executivo.md", "Resumo.\n"),
        ("tecnico.md", "Detalhes.\n"),
        ("registro.md", "## Registro de execução\n\n| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n\n## Histórico de ajustes\n\n| # | Data | Origem | Resumo |\n|---|---|---|---|\n"),
    ];
    for (name, content) in files {
        write(&project, &format!("{FOLDER}/{name}"), content);
    }
    project
        .vostok()
        .args(["task", "adjust", "record-manual", "SPEC-001", "TASK-01"])
        .assert()
        .success();
    project
}

#[test]
fn ct01_two_panels_run_together_each_with_its_own_brand() {
    let ai_hub = Project::new();
    let atria = Project::new();
    let documents = "vostok/config/documents.yml";
    let branded =
        read(&atria, documents).replacen("primary: \"#6d28d9\"", "primary: \"#0f766e\"", 1);
    write(&atria, documents, &branded);

    let first = start(ai_hub.root());
    let second = start(atria.root());
    assert_ne!(first.address.port(), second.address.port());
    assert_eq!(first.address.ip().to_string(), "127.0.0.1");
    assert_eq!(second.address.ip().to_string(), "127.0.0.1");

    let first_brand = get(first.address, "/api/marca");
    let second_brand = get(second.address, "/api/marca");
    assert_status(&first_brand, "200");
    assert_status(&second_brand, "200");
    assert!(first_brand.contains("#6d28d9"), "{first_brand}");
    assert!(!first_brand.contains("#0f766e"), "{first_brand}");
    assert!(second_brand.contains("#0f766e"), "{second_brand}");
}

/// Abre o WebSocket do aviso ao vivo e confere a mensagem `conectado`.
fn open_live(address: SocketAddr) -> TcpStream {
    let mut stream = TcpStream::connect(address).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    stream
        .write_all(b"GET /api/ao-vivo HTTP/1.1\r\nHost: x\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nSec-WebSocket-Version: 13\r\n\r\n")
        .unwrap();
    let mut head = Vec::new();
    let mut byte = [0u8; 1];
    while !head.ends_with(b"\r\n\r\n") {
        stream.read_exact(&mut byte).unwrap();
        head.push(byte[0]);
    }
    let head = String::from_utf8_lossy(&head);
    assert!(head.starts_with("HTTP/1.1 101"), "{head}");
    assert_eq!(next_message(&mut stream), "conectado");
    stream
}

/// Lê um quadro de texto curto do servidor (sem máscara, menos de 126 bytes).
fn next_message(stream: &mut TcpStream) -> String {
    let mut header = [0u8; 2];
    stream.read_exact(&mut header).unwrap();
    let mut payload = vec![0u8; usize::from(header[1] & 0x7f)];
    stream.read_exact(&mut payload).unwrap();
    String::from_utf8(payload).unwrap()
}

#[cfg(painel_bundle)]
#[test]
fn approve_edit_request_and_live_update_end_to_end() {
    let project = pending_task();
    let painel = start(project.root());

    let page = get(painel.address, "/");
    assert_status(&page, "200");
    let marker = "<meta name=\"painel-token\" content=\"";
    let start_at = page.find(marker).expect("token na página") + marker.len();
    let token = &page[start_at..start_at + page[start_at..].find('"').unwrap()];

    let mut live = open_live(painel.address);

    let approval = request(
        painel.address,
        "POST",
        "/api/aprovacoes?spec=SPEC-001&tarefas=TASK-01",
        token,
        "",
    );
    assert_status(&approval, "200");
    assert!(read(&project, &format!("{FOLDER}/estado.yml")).contains("\napproved_by: painel\n"));
    let avais = read(&project, "vostok/work/avais.log");
    assert!(
        avais.contains("SPEC-001/TASK-01\t") && avais.trim_end().ends_with("\tpainel"),
        "{avais}"
    );

    let criteria = format!("{FOLDER}/criterios.md");
    let source = get(painel.address, &format!("/api/fonte?caminho={criteria}"));
    assert_status(&source, "200");
    let hash = json_field(&source, "hash");
    let edited = "### Concluída quando\n\n- [ ] Pronto, com o arquivo criado (RF-01).\n\n  > Como comprovar: ver o arquivo.\n";
    let saved = request(
        painel.address,
        "PUT",
        &format!("/api/arquivo?caminho={criteria}&hash={hash}"),
        token,
        edited,
    );
    assert_status(&saved, "200");
    assert_eq!(read(&project, &criteria), edited);
    project
        .vostok()
        .args(["task", "adjust", "validate", "SPEC-001", "TASK-01"])
        .assert()
        .success();

    let asked = request(
        painel.address,
        "POST",
        "/api/pedidos?acao=ajustar-tarefa&spec=SPEC-001&tarefa=TASK-01",
        token,
        "Acrescentar um critério.",
    );
    assert_status(&asked, "200");
    let order = read(&project, "vostok/work/pedidos/PED-001.md");
    assert!(
        order.contains("acao: ajustar-tarefa") && order.contains("Acrescentar um critério."),
        "{order}"
    );

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if next_message(&mut live) == "mudou" {
            break;
        }
        assert!(Instant::now() < deadline, "sem aviso de mudança");
    }
}
