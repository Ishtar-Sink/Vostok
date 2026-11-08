//! Testes de ponta a ponta do `vostok painel` somente leitura: rotas GET, 405, rotas removidas, plano desatualizado e aviso ao vivo (SPEC-042).
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

fn request(address: SocketAddr, method: &str, target: &str, body: &str) -> String {
    let mut stream = TcpStream::connect(address).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(20)))
        .unwrap();
    let raw = format!(
        "{method} {target} HTTP/1.1\r\nHost: x\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(raw.as_bytes()).unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}

fn get(address: SocketAddr, target: &str) -> String {
    request(address, "GET", target, "")
}

fn assert_status(response: &str, status: &str) {
    let line = response.lines().next().unwrap_or_default();
    assert!(line.contains(&format!(" {status} ")), "{response}");
}

fn write(project: &Project, relative: &str, content: &str) {
    let path = project.root().join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn read(project: &Project, relative: &str) -> String {
    fs::read_to_string(project.root().join(relative)).unwrap_or_default()
}

fn spec_with_plan(approved_hash: &str, status: &str, plan_hash: Option<&str>) -> Project {
    let project = Project::new();
    write(
        &project,
        "vostok/work/specs/refined/SPEC-001-plano.md",
        &format!("---\nid: SPEC-001\ntitle: \"Plano\"\nstatus: {status}\ntype: fix\napproved_hash: {approved_hash}\n---\n\n# Plano\n"),
    );
    if let Some(plan_hash) = plan_hash {
        write(
            &project,
            "vostok/work/plans/SPEC-001.md",
            &format!("---\nspec: SPEC-001\nspec_hash: {plan_hash}\n---\n\n# Plano de tarefas\n"),
        );
    }
    project
}

fn assert_plan_fields(project: &Project, expected: &str) {
    let running = start(project.root());
    for target in ["/api/visao", "/api/specs/SPEC-001"] {
        let response = get(running.address, target);
        assert_status(&response, "200");
        assert!(response.contains(expected), "{response}");
    }
}

#[test]
fn plan_outdated_true_when_the_spec_was_approved_again() {
    let project = spec_with_plan("novo", "approved", Some("antigo"));
    assert_plan_fields(&project, "\"plan\":true,\"plan_outdated\":true");
}

#[test]
fn plan_outdated_false_when_the_hashes_match() {
    let project = spec_with_plan("abc", "approved", Some("abc"));
    assert_plan_fields(&project, "\"plan\":true,\"plan_outdated\":false");
}

#[test]
fn plan_outdated_true_when_the_spec_lost_the_approval() {
    let project = spec_with_plan("null", "refined", Some("abc"));
    assert_plan_fields(&project, "\"plan\":true,\"plan_outdated\":true");
}

#[test]
fn plan_outdated_false_without_a_plan() {
    let project = spec_with_plan("abc", "approved", None);
    assert_plan_fields(&project, "\"plan\":false,\"plan_outdated\":false");
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

/// Trecho de uma tarefa na resposta, do `id` dela até o fim do objeto.
fn task_json<'a>(response: &'a str, id: &str) -> &'a str {
    let marker = format!("\"id\":\"{id}\"");
    let start = response
        .find(&marker)
        .unwrap_or_else(|| panic!("sem {id}: {response}"));
    let end = start + response[start..].find('}').unwrap();
    &response[start..end]
}

fn planned_spec(project: &Project) {
    write(
        project,
        "vostok/work/specs/refined/SPEC-001-plano.md",
        "---\nid: SPEC-001\ntitle: \"Plano\"\nstatus: planned\ntype: fix\napproved_hash: abc\n---\n\n# Plano\n",
    );
}

fn state_only(project: &Project, id: &str, fields: &str) {
    write(
        project,
        &format!("vostok/work/tasks/SPEC-001/{id}/estado.yml"),
        &format!("id: {id}\ntitle: \"Tarefa {id}\"\n{fields}status: pending\n"),
    );
}

fn assert_task_facts(response: &str, id: &str, wave: &str, profile: &str, persona: &str) {
    let task = task_json(response, id);
    for expected in [
        format!("\"wave\":\"{wave}\""),
        format!("\"profile\":\"{profile}\""),
        format!("\"persona\":\"{persona}\""),
    ] {
        assert!(task.contains(&expected), "{id} sem {expected}: {task}");
    }
}

#[test]
fn api_campos_sem_registro_iguais_ao_task_status() {
    let project = Project::new();
    planned_spec(&project);
    state_only(&project, "TASK-01", "effort: simples\ndepends_on: []\n");
    state_only(
        &project,
        "TASK-02",
        "effort: complexa\ndepends_on: [TASK-01]\n",
    );
    let status = project
        .vostok()
        .args(["task", "status", "SPEC-001"])
        .output()
        .unwrap();
    let table = String::from_utf8_lossy(&status.stdout).to_string();
    let row = |id: &str| -> Vec<String> {
        let line = table
            .lines()
            .find(|line| line.starts_with(&format!("| {id} ")))
            .unwrap_or_else(|| panic!("sem {id}: {table}"));
        line.split('|')
            .map(|cell| cell.trim().to_string())
            .collect()
    };
    assert_eq!(
        (row("TASK-01")[4].as_str(), row("TASK-01")[6].as_str()),
        ("mecanica", "1")
    );
    assert_eq!(
        (row("TASK-02")[4].as_str(), row("TASK-02")[6].as_str()),
        ("julgamento", "2")
    );
    let running = start(project.root());
    for target in ["/api/visao", "/api/specs/SPEC-001"] {
        let response = get(running.address, target);
        assert_status(&response, "200");
        assert_task_facts(&response, "TASK-01", "1", "mecanica", "");
        assert_task_facts(&response, "TASK-02", "2", "julgamento", "");
    }
    let response = get(running.address, "/api/tarefas/SPEC-001/TASK-02");
    assert_status(&response, "200");
    for expected in [
        "\"wave\":\"2\"",
        "\"profile\":\"julgamento\"",
        "\"persona\":\"\"",
    ] {
        assert!(response.contains(expected), "sem {expected}: {response}");
    }
}

#[test]
fn api_campos_registrados_valem_e_ciclo_fica_vazio() {
    let project = Project::new();
    planned_spec(&project);
    state_only(&project, "TASK-01", "depends_on: []\n");
    state_only(
        &project,
        "TASK-02",
        "wave: 7\nprofile: padrao\npersona: qa\neffort: simples\ndepends_on: [TASK-01]\n",
    );
    state_only(&project, "TASK-03", "depends_on: [TASK-04]\n");
    state_only(&project, "TASK-04", "depends_on: [TASK-03]\n");
    state_only(
        &project,
        "TASK-05",
        "wave: null\nprofile: null\npersona: \"-\"\ndepends_on: []\n",
    );
    let running = start(project.root());
    for target in ["/api/visao", "/api/specs/SPEC-001"] {
        let response = get(running.address, target);
        assert_status(&response, "200");
        assert_task_facts(&response, "TASK-02", "7", "padrao", "qa");
        assert_task_facts(&response, "TASK-03", "", "", "");
        assert_task_facts(&response, "TASK-05", "1", "", "");
    }
    for (task, wave, profile, persona) in [
        ("TASK-02", "7", "padrao", "qa"),
        ("TASK-03", "", "", ""),
        ("TASK-05", "1", "", ""),
    ] {
        let response = get(running.address, &format!("/api/tarefas/SPEC-001/{task}"));
        assert_status(&response, "200");
        for expected in [
            format!("\"wave\":\"{wave}\""),
            format!("\"profile\":\"{profile}\""),
            format!("\"persona\":\"{persona}\""),
        ] {
            assert!(
                response.contains(&expected),
                "{task} sem {expected}: {response}"
            );
        }
    }
}

/// As duas rotas de `Layout::file` para o mesmo caminho, já com os escapes da URL.
fn file_routes(encoded_path: &str) -> [String; 2] {
    [
        format!("/arquivos/{encoded_path}"),
        format!("/api/arquivo?caminho={encoded_path}"),
    ]
}

/// Valores do cabeçalho na resposta crua; o nome é comparado sem caixa, como no HTTP.
fn header_values(response: &str, name: &str) -> Vec<String> {
    let (head, _) = response.split_once("\r\n\r\n").unwrap_or((response, ""));
    head.split("\r\n")
        .skip(1)
        .filter_map(|line| line.split_once(':'))
        .filter(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.trim().to_string())
        .collect()
}

fn assert_header(response: &str, name: &str, expected: &str) {
    assert_eq!(
        header_values(response, name),
        [expected],
        "{name}: {response}"
    );
}

fn assert_nosniff(response: &str) {
    assert_header(response, "X-Content-Type-Options", "nosniff");
}

fn assert_only_downloads(response: &str, file_name: &str) {
    assert_status(response, "200");
    assert_header(response, "Content-Type", "application/octet-stream");
    assert_header(
        response,
        "Content-Disposition",
        &format!("attachment; filename=\"{file_name}\""),
    );
    assert_nosniff(response);
}

fn assert_opens_as(response: &str, content_type: &str) {
    assert_status(response, "200");
    assert_header(response, "Content-Type", content_type);
    assert!(
        header_values(response, "Content-Disposition").is_empty(),
        "{response}"
    );
    assert_nosniff(response);
}

#[test]
fn seguranca_html_svg_e_outros_tipos_so_baixam_nas_duas_rotas() {
    let project = Project::new();
    let names = [
        "caso.html",
        "maiusculo.HTML",
        "imagem.svg",
        "misto.SvG",
        "pagina.xhtml",
        "folha.pdf",
        "sem-extensao",
        "antigo.htm",
    ];
    let folders = [
        "vostok/work/tasks/SPEC-001/TASK-01/evidencias",
        "vostok/work/.archive/SPEC-000/tasks/TASK-01/evidencias",
        "vostok/work",
        "vostok/config",
    ];
    for folder in folders {
        for name in names {
            write(
                &project,
                &format!("{folder}/{name}"),
                "<script>ataque()</script>",
            );
        }
    }
    let running = start(project.root());
    for folder in folders {
        for name in names {
            for route in file_routes(&format!("{folder}/{name}")) {
                assert_only_downloads(&get(running.address, &route), name);
            }
        }
    }
}

#[test]
fn seguranca_documentos_finais_abrem_em_sandbox() {
    const DOCS: &str = "vostok/work/.archive/SPEC-000/docs";
    let project = Project::new();
    let documents = ["executivo.html", "tecnico.html", "evidencias.html"];
    for name in documents {
        write(&project, &format!("{DOCS}/{name}"), "<h1>documento</h1>");
    }
    // `EXECUTIVO.HTML` fica em outra spec: num disco sem distinção de caixa, seria o próprio executivo.html.
    let others = [
        format!("{DOCS}/outro.html"),
        format!("{DOCS}/assets/executivo.html"),
        "vostok/work/.archive/SPEC-002/docs/EXECUTIVO.HTML".to_string(),
        "vostok/work/.archive/SPEC-000/executivo.html".to_string(),
        "vostok/work/.archive/SPEC-000/x/docs/executivo.html".to_string(),
        "vostok/work/docs/executivo.html".to_string(),
    ];
    for path in &others {
        write(&project, path, "<script>ataque()</script>");
    }
    #[cfg(unix)]
    let shortcut = {
        write(
            &project,
            "vostok/work/alvo.html",
            "<script>ataque()</script>",
        );
        let shortcut = "vostok/work/.archive/SPEC-003/docs/executivo.html".to_string();
        let link = project.root().join(&shortcut);
        fs::create_dir_all(link.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(project.root().join("vostok/work/alvo.html"), link).unwrap();
        shortcut
    };
    let running = start(project.root());
    for name in documents {
        for route in file_routes(&format!("{DOCS}/{name}")) {
            let response = get(running.address, &route);
            assert_opens_as(&response, "text/html; charset=utf-8");
            assert_header(
                &response,
                "Content-Security-Policy",
                "sandbox allow-scripts allow-downloads",
            );
        }
    }
    for path in &others {
        let name = path.rsplit('/').next().unwrap();
        for route in file_routes(path) {
            let response = get(running.address, &route);
            assert_only_downloads(&response, name);
            assert!(
                header_values(&response, "Content-Security-Policy").is_empty(),
                "{response}"
            );
        }
    }
    #[cfg(unix)]
    for route in file_routes(&shortcut) {
        assert_only_downloads(&get(running.address, &route), "executivo.html");
    }
}

#[test]
fn seguranca_png_com_html_vai_como_imagem_com_nosniff() {
    let project = Project::new();
    let path = format!("{FOLDER}/evidencias/falso.png");
    write(&project, &path, "<html><script>ataque()</script></html>");
    let running = start(project.root());
    for route in file_routes(&path) {
        assert_opens_as(&get(running.address, &route), "image/png");
    }
}

#[test]
fn seguranca_nome_limpo_no_cabecalho() {
    let project = Project::new();
    let cases = [
        ("a\"b\nc.html", "a%22b%0Ac.html", "a_b_c.html"),
        ("a\r\nb.zip", "a%0D%0Ab.zip", "a__b.zip"),
        ("relatório.zip", "relat%C3%B3rio.zip", "relat_rio.zip"),
    ];
    for (real_name, _, _) in cases {
        write(&project, &format!("{FOLDER}/evidencias/{real_name}"), "x");
    }
    let running = start(project.root());
    for (_, encoded, clean) in cases {
        for route in file_routes(&format!("{FOLDER}/evidencias/{encoded}")) {
            let response = get(running.address, &route);
            assert_only_downloads(&response, clean);
            let (head, _) = response.split_once("\r\n\r\n").unwrap();
            assert!(head.is_ascii(), "{head}");
            let header_names: Vec<&str> = head
                .split("\r\n")
                .skip(1)
                .filter_map(|line| line.split_once(':').map(|(key, _)| key))
                .collect();
            assert!(
                header_names
                    .iter()
                    .all(|key| !key.contains(' ') && !key.contains('"')),
                "{head}"
            );
            assert_eq!(head.matches("Content-Disposition").count(), 1, "{head}");
        }
    }
}

#[test]
fn seguranca_videos_com_o_tipo_certo() {
    let project = Project::new();
    let cases = [
        ("video.mp4", "video/mp4"),
        ("video.webm", "video/webm"),
        ("video.mov", "video/quicktime"),
        ("maiusculo.WEBM", "video/webm"),
    ];
    for (name, _) in cases {
        write(&project, &format!("{FOLDER}/evidencias/{name}"), "video");
    }
    let running = start(project.root());
    for (name, content_type) in cases {
        for route in file_routes(&format!("{FOLDER}/evidencias/{name}")) {
            assert_opens_as(&get(running.address, &route), content_type);
        }
    }
}

#[test]
fn seguranca_nosniff_em_toda_resposta() {
    const PLAIN: &str = "text/plain; charset=utf-8";
    let project = Project::new();
    let cases = [
        ("a.png", "image/png"),
        ("a.jpg", "image/jpeg"),
        ("a.md", PLAIN),
        ("a.txt", PLAIN),
        ("a.yml", PLAIN),
        ("a.log", PLAIN),
        ("a.out", PLAIN),
        ("a.diff", PLAIN),
        ("a.csv", PLAIN),
        ("a.json", "application/json; charset=utf-8"),
        ("a.css", "text/css; charset=utf-8"),
        ("a.js", "text/javascript; charset=utf-8"),
    ];
    for (name, _) in cases {
        write(&project, &format!("{FOLDER}/evidencias/{name}"), "conteudo");
    }
    let running = start(project.root());
    for (name, content_type) in cases {
        for route in file_routes(&format!("{FOLDER}/evidencias/{name}")) {
            assert_opens_as(&get(running.address, &route), content_type);
        }
    }
    for route in file_routes("vostok/nao-existe.md") {
        let response = get(running.address, &route);
        assert_status(&response, "404");
        assert_nosniff(&response);
    }
    for route in file_routes("outro/segredo.txt") {
        let response = get(running.address, &route);
        assert_status(&response, "400");
        assert_nosniff(&response);
    }
}

const SPEC_FILE: &str = "vostok/work/specs/refined/SPEC-001-leitura.md";

/// Uma spec refinada com plano, uma tarefa em pasta com evidência e uma spec arquivada com documento.
fn reading_project() -> Project {
    let project = Project::new();
    write(
        &project,
        SPEC_FILE,
        "---\nid: SPEC-001\ntitle: \"Leitura\"\nstatus: approved\ntype: fix\napproved_hash: abc\n---\n\n# Leitura\n",
    );
    write(
        &project,
        "vostok/work/plans/SPEC-001.md",
        "---\nspec: SPEC-001\nspec_hash: abc\n---\n\n# Plano de tarefas\n",
    );
    write(
        &project,
        &format!("{FOLDER}/estado.yml"),
        "id: TASK-01\ntitle: \"Primeira\"\nstatus: approved\ndepends_on: []\n",
    );
    write(&project, &format!("{FOLDER}/execucao.md"), "passos\n");
    write(&project, &format!("{FOLDER}/evidencias/saida.txt"), "ok");
    write(
        &project,
        "vostok/work/.archive/SPEC-000/spec.md",
        "---\nid: SPEC-000\ntitle: \"Antiga\"\nstatus: archived\ntype: fix\n---\n\n# Antiga\n",
    );
    write(
        &project,
        "vostok/work/.archive/SPEC-000/docs/executivo.html",
        "<html></html>",
    );
    project
}

/// As rotas GET do contrato, menos o aviso ao vivo.
fn get_routes() -> Vec<String> {
    vec![
        "/api/visao".to_string(),
        "/api/versao".to_string(),
        "/api/marca".to_string(),
        "/api/documentos".to_string(),
        "/api/specs/SPEC-001".to_string(),
        "/api/planos/SPEC-001".to_string(),
        "/api/tarefas/SPEC-001/TASK-01".to_string(),
        format!("/api/arquivo?caminho={FOLDER}/evidencias/saida.txt"),
        format!("/api/fonte?caminho={SPEC_FILE}"),
        format!("/arquivos/{FOLDER}/evidencias/saida.txt"),
    ]
}

/// Caminho e SHA-256 de cada arquivo de `vostok/`, em ordem.
fn vostok_hashes(project: &Project) -> Vec<(String, String)> {
    fn walk(dir: &Path, out: &mut Vec<(String, String)>) {
        for entry in fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else {
                let hash = sha256_hex(&fs::read(&path).unwrap());
                out.push((path.to_string_lossy().into_owned(), hash));
            }
        }
    }
    let mut out = Vec::new();
    walk(&project.harness(), &mut out);
    out.sort();
    out
}

fn sha256_hex(bytes: &[u8]) -> String {
    vostok::support::sha256_hex(bytes)
}

#[test]
fn leitura_rotas_get_do_contrato_respondem_200() {
    let project = reading_project();
    let running = start(project.root());
    for target in get_routes() {
        let response = get(running.address, &target);
        assert_status(&response, "200");
    }
    assert!(get(running.address, "/api/visao").contains("\"plan_outdated\""));
    assert!(get(running.address, "/api/specs/SPEC-001").contains("\"markdown\""));
    assert!(get(running.address, "/api/tarefas/SPEC-001/TASK-01").contains("\"files\""));
}

#[test]
fn leitura_post_put_e_delete_sao_405_sem_mudar_os_hashes() {
    let project = reading_project();
    let running = start(project.root());
    let before = vostok_hashes(&project);
    assert!(!before.is_empty());
    for target in [
        "/api/visao",
        "/api/arquivo?caminho=vostok/work/specs",
        "/api/specs/SPEC-001",
        "/api/evidencias",
        "/api/pedidos/anexos",
        "/api/notificacoes/lidas",
    ] {
        for method in ["POST", "PUT", "DELETE", "PATCH"] {
            let response = request(running.address, method, target, "x");
            assert_status(&response, "405");
        }
    }
    assert_eq!(vostok_hashes(&project), before);
    for target in get_routes() {
        assert_status(&get(running.address, &target), "200");
    }
    assert_eq!(vostok_hashes(&project), before);
}

#[test]
fn leitura_rotas_de_comunicacao_sao_404() {
    let project = reading_project();
    let running = start(project.root());
    for target in [
        "/api/atividades",
        "/api/notificacoes",
        "/api/pedidos/PED-001",
        "/api/pedidos/anexos/limites",
        "/api/perguntas-livres",
        "/api/commits",
        "/api/configuracoes",
    ] {
        assert_status(&get(running.address, target), "404");
    }
}

#[test]
fn leitura_visao_sem_campos_da_comunicacao() {
    let project = reading_project();
    let running = start(project.root());
    let overview = get(running.address, "/api/visao");
    for field in [
        "\"questions\"",
        "\"requests\"",
        "\"loop\"",
        "\"respostas_nao_lidas\"",
        "\"notificacoes_nao_lidas\"",
        "\"toast_dismiss_seconds\"",
        "\"last_outcome\"",
    ] {
        assert!(!overview.contains(field), "{field}: {overview}");
    }
    let task = get(running.address, "/api/tarefas/SPEC-001/TASK-01");
    assert_status(&task, "200");
    for field in ["\"manual_evidence\"", "\"attach_limit_mb\""] {
        assert!(!task.contains(field), "{field}: {task}");
    }
}

#[test]
fn leitura_ao_vivo_avisa_a_mudanca_de_status_em_ate_1_s() {
    let project = reading_project();
    let running = start(project.root());
    let mut stream = open_live(running.address);
    std::thread::sleep(Duration::from_millis(100));
    let state = format!("{FOLDER}/estado.yml");
    let changed = read(&project, &state).replace("status: approved", "status: in_progress");
    write(&project, &state, &changed);
    let started = Instant::now();
    assert_eq!(next_message(&mut stream), "mudou");
    assert!(started.elapsed() < Duration::from_secs(1));
}

#[test]
fn leitura_mudanca_fora_de_work_nao_avisa() {
    let project = reading_project();
    let running = start(project.root());
    let mut stream = open_live(running.address);
    std::thread::sleep(Duration::from_millis(100));
    write(&project, "vostok/rules/project/nova.md", "regra\n");
    stream
        .set_read_timeout(Some(Duration::from_millis(1500)))
        .unwrap();
    let mut byte = [0u8; 1];
    let outcome = stream.read(&mut byte);
    assert!(
        matches!(&outcome, Err(error) if matches!(error.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut)),
        "chegou mensagem: {outcome:?}"
    );
}
