//! Testes do `vostok painel` com o binário real: localização do harness, escuta só em
//! `127.0.0.1`, porta ocupada e recusa das ações sem o token de sessão (RF-01, RF-08, CT-02, CT-05).
#![cfg(feature = "painel")]

mod common;

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
    let child = vostok()
        .args(["painel", "--sem-navegador", "--porta", "0", "--pasta"])
        .arg(folder)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    running(child)
}

fn running(mut child: Child) -> Running {
    let mut line = String::new();
    let mut output = BufReader::new(child.stdout.take().unwrap());
    output.read_line(&mut line).unwrap();
    // O painel segue escrevendo na saída; um pipe fechado derrubaria o envio.
    std::thread::spawn(move || std::io::copy(&mut output, &mut std::io::sink()));
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

const FILE_SIZE: usize = 70_000;

fn sample_bytes(size: usize) -> Vec<u8> {
    (0..size).map(|index| (index % 251) as u8).collect()
}

fn write_work_file(project: &TempDir, name: &str, content: &[u8]) -> String {
    let path = project.path().join("vostok/work").join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, content).unwrap();
    format!("vostok/work/{name}")
}

/// Resposta crua: cabeçalhos em texto e corpo em bytes.
struct Raw {
    head: String,
    body: Vec<u8>,
}

impl Raw {
    fn status(&self) -> &str {
        status_line(&self.head)
    }

    fn header(&self, name: &str) -> Option<String> {
        self.head.lines().skip(1).find_map(|line| {
            let (key, value) = line.split_once(':')?;
            key.eq_ignore_ascii_case(name)
                .then(|| value.trim().to_string())
        })
    }
}

fn get_raw(address: SocketAddr, target: &str, range: Option<&str>) -> Raw {
    let range = range
        .map(|value| format!("Range: bytes={value}\r\n"))
        .unwrap_or_default();
    let raw = format!("GET {target} HTTP/1.1\r\nHost: x\r\n{range}Connection: close\r\n\r\n");
    let mut stream = TcpStream::connect(address).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    stream.write_all(raw.as_bytes()).unwrap();
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).unwrap();
    let split = bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .unwrap_or_else(|| panic!("resposta sem cabeçalhos: {bytes:?}"));
    Raw {
        head: String::from_utf8_lossy(&bytes[..split]).into_owned(),
        body: bytes[split + 4..].to_vec(),
    }
}

fn file_target(relative: &str) -> String {
    format!("/arquivos/{relative}")
}

#[test]
fn entrega_range_devolve_206_com_o_trecho() {
    let project = project();
    let content = sample_bytes(FILE_SIZE);
    let relative = write_work_file(&project, "dados.txt", &content);
    let painel = start(project.path());
    let response = get_raw(painel.address, &file_target(&relative), Some("0-99"));
    assert!(response.status().contains(" 206 "), "{}", response.head);
    assert_eq!(
        response.header("Content-Range").as_deref(),
        Some(format!("bytes 0-99/{FILE_SIZE}").as_str())
    );
    assert_eq!(response.header("Content-Length").as_deref(), Some("100"));
    assert_eq!(response.header("Accept-Ranges").as_deref(), Some("bytes"));
    assert_eq!(response.body, content[..100]);
}

#[test]
fn entrega_range_aberto_e_final() {
    let project = project();
    let content = sample_bytes(FILE_SIZE);
    let relative = write_work_file(&project, "dados.txt", &content);
    let painel = start(project.path());
    let target = file_target(&relative);
    let cases = [
        ("50-", 50, FILE_SIZE - 1),
        ("-10", FILE_SIZE - 10, FILE_SIZE - 1),
        ("0-999999", 0, FILE_SIZE - 1),
    ];
    for (range, start_at, end_at) in cases {
        let response = get_raw(painel.address, &target, Some(range));
        assert!(
            response.status().contains(" 206 "),
            "{range}: {}",
            response.head
        );
        assert_eq!(
            response.header("Content-Range").as_deref(),
            Some(format!("bytes {start_at}-{end_at}/{FILE_SIZE}").as_str()),
            "{range}"
        );
        assert_eq!(response.body, content[start_at..=end_at], "{range}");
    }
}

#[test]
fn entrega_range_invalido_ou_alem_do_fim_da_416() {
    let project = project();
    let relative = write_work_file(&project, "dados.txt", &sample_bytes(FILE_SIZE));
    let empty = write_work_file(&project, "vazio.txt", b"");
    let painel = start(project.path());
    let size_cases = [
        (&relative, FILE_SIZE, "abc"),
        (&relative, FILE_SIZE, "-"),
        (&relative, FILE_SIZE, "-0"),
        (&relative, FILE_SIZE, "x-y"),
        (&relative, FILE_SIZE, "5-2"),
        (&relative, FILE_SIZE, "70000-"),
        (&relative, FILE_SIZE, "0-1,3-4"),
        (&empty, 0, "0-0"),
    ];
    for (path, size, range) in size_cases {
        let response = get_raw(painel.address, &file_target(path), Some(range));
        assert!(
            response.status().contains(" 416 "),
            "{range}: {}",
            response.head
        );
        assert_eq!(
            response.header("Content-Range").as_deref(),
            Some(format!("bytes */{size}").as_str()),
            "{range}"
        );
        assert_eq!(
            response.header("X-Content-Type-Options").as_deref(),
            Some("nosniff"),
            "{range}"
        );
    }
}

#[test]
fn entrega_sem_range_da_200_com_o_tamanho() {
    let project = project();
    let content = sample_bytes(FILE_SIZE);
    let relative = write_work_file(&project, "dados.txt", &content);
    let painel = start(project.path());
    let response = get_raw(painel.address, &file_target(&relative), None);
    assert!(response.status().contains(" 200 "), "{}", response.head);
    assert_eq!(
        response.header("Content-Length").as_deref(),
        Some(FILE_SIZE.to_string().as_str())
    );
    assert_eq!(response.header("Accept-Ranges").as_deref(), Some("bytes"));
    assert_eq!(
        response.header("Transfer-Encoding"),
        None,
        "{}",
        response.head
    );
    assert_eq!(response.body, content);
}

#[test]
fn entrega_tipos_de_video() {
    let project = project();
    let content = sample_bytes(FILE_SIZE);
    let painel = start(project.path());
    for (name, content_type) in [
        ("video.mp4", "video/mp4"),
        ("video.webm", "video/webm"),
        ("video.mov", "video/quicktime"),
    ] {
        let relative = write_work_file(&project, name, &content);
        let target = file_target(&relative);
        let whole = get_raw(painel.address, &target, None);
        assert!(whole.status().contains(" 200 "), "{name}: {}", whole.head);
        assert_eq!(whole.header("Content-Type").as_deref(), Some(content_type));
        let part = get_raw(painel.address, &target, Some("100-199"));
        assert!(part.status().contains(" 206 "), "{name}: {}", part.head);
        assert_eq!(part.header("Content-Type").as_deref(), Some(content_type));
        assert_eq!(part.body, content[100..200], "{name}");
    }
}

#[test]
fn entrega_nas_duas_rotas() {
    let project = project();
    let content = sample_bytes(FILE_SIZE);
    let relative = write_work_file(&project, "dados.txt", &content);
    let painel = start(project.path());
    let target = format!("/api/arquivo?caminho={relative}");
    let response = get_raw(painel.address, &target, Some("10-19"));
    assert!(response.status().contains(" 206 "), "{}", response.head);
    assert_eq!(
        response.header("Content-Range").as_deref(),
        Some(format!("bytes 10-19/{FILE_SIZE}").as_str())
    );
    assert_eq!(response.body, content[10..20]);
}

#[test]
fn entrega_mantem_a_seguranca() {
    let project = project();
    let page = write_work_file(&project, "pagina.html", b"<script>1</script>");
    let final_document = write_work_file(
        &project,
        ".archive/SPEC-000/docs/executivo.html",
        b"<p>executivo</p>",
    );
    let outside = project.path().join("outro/segredo.txt");
    fs::create_dir_all(outside.parent().unwrap()).unwrap();
    fs::write(&outside, "segredo").unwrap();
    let painel = start(project.path());

    let page = get_raw(painel.address, &file_target(&page), Some("0-3"));
    assert!(page.status().contains(" 206 "), "{}", page.head);
    assert!(page
        .header("Content-Disposition")
        .is_some_and(|value| value.starts_with("attachment")));
    assert_eq!(
        page.header("X-Content-Type-Options").as_deref(),
        Some("nosniff")
    );

    let document = get_raw(painel.address, &file_target(&final_document), Some("0-3"));
    assert!(document.status().contains(" 206 "), "{}", document.head);
    assert_eq!(
        document.header("Content-Security-Policy").as_deref(),
        Some("sandbox allow-scripts allow-downloads")
    );
    assert_eq!(document.header("Content-Disposition"), None);

    for (target, status) in [
        (file_target("outro/segredo.txt"), " 400 "),
        (file_target("vostok/work/nao-existe.txt"), " 404 "),
    ] {
        let response = get_raw(painel.address, &target, Some("0-3"));
        assert!(
            response.status().contains(status),
            "{target}: {}",
            response.head
        );
        assert_eq!(
            response.header("X-Content-Type-Options").as_deref(),
            Some("nosniff"),
            "{target}"
        );
    }
}

fn peak_memory_kb(pid: u32) -> u64 {
    let status = fs::read_to_string(format!("/proc/{pid}/status")).unwrap();
    status
        .lines()
        .find_map(|line| line.strip_prefix("VmHWM:"))
        .and_then(|value| value.trim().trim_end_matches("kB").trim().parse().ok())
        .expect("VmHWM no status do painel")
}

#[test]
#[ignore = "arquivo de 500 MB: rode com --ignored"]
#[cfg(target_os = "linux")]
fn rnf01_ler_500_mb_nao_sobe_a_memoria() {
    const SIZE: u64 = 500 * 1024 * 1024;
    let project = project();
    let work = project.path().join("vostok/work");
    fs::create_dir_all(&work).unwrap();
    fs::File::create(work.join("grande.bin"))
        .unwrap()
        .set_len(SIZE)
        .unwrap();
    let painel = start(project.path());
    let pid = painel.child.id();
    let before = peak_memory_kb(pid);

    let mut stream = TcpStream::connect(painel.address).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(60)))
        .unwrap();
    stream
        .write_all(
            b"GET /arquivos/vostok/work/grande.bin HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n",
        )
        .unwrap();
    let received = std::io::copy(&mut stream, &mut std::io::sink()).unwrap();
    let after = peak_memory_kb(pid);
    let growth_mb = after.saturating_sub(before) / 1024;
    eprintln!("recebidos {received} bytes; alta de VmHWM: {growth_mb} MB");
    assert!(received >= SIZE, "recebeu só {received} bytes");
    assert!(
        growth_mb <= 100,
        "o painel subiu {growth_mb} MB (limite 100)"
    );

    let started = std::time::Instant::now();
    let part = get_raw(
        painel.address,
        "/arquivos/vostok/work/grande.bin",
        Some("1048576-2097151"),
    );
    let elapsed = started.elapsed();
    eprintln!("Range de 1 MB em {elapsed:?}");
    assert!(part.status().contains(" 206 "), "{}", part.head);
    assert_eq!(part.body.len(), 1024 * 1024);
    assert!(
        elapsed <= Duration::from_millis(500),
        "levou {elapsed:?} (limite 500 ms)"
    );
}

const UPLOAD_TASK: &str = "vostok/work/tasks/SPEC-001/TASK-01";
const UPLOAD_TARGET: &str =
    "/api/evidencias?spec=SPEC-001&tarefa=TASK-01&evidencia=video.mov&extensao=mov";
const UPLOAD_TIMEOUT: Duration = Duration::from_secs(10);

/// Tarefa em pasta, em execução, que pede o anexo manual `video.mov`; `limit` (em MB) troca o
/// limite de envio do projeto.
fn upload_project(limit: Option<u64>) -> common::Project {
    let project = common::Project::new();
    let task = project.root().join(UPLOAD_TASK);
    fs::create_dir_all(&task).unwrap();
    let files = [
        ("estado.yml", "id: TASK-01\nspec: SPEC-001\ntitle: \"Tarefa\"\neffort: simples\ndepends_on: []\ncovers: [RF-01]\ntype: tarefa\nlabels: [SPEC-001, simples]\nstatus: in_progress\ncontent_hash: null\nadjustments: 0\nadjusted_at: null\napproved_at: null\napproved_hash: null\nattempts: 0\ntests_result: null\nregression_result: null\nevidence_result: null\n"),
        ("execucao.md", "### Ler\n\n- `README.md`\n\n### Mudar\n\n- `novo.txt` (criar): arquivo novo.\n\n### Passos\n\n1. Crie o arquivo.\n"),
        ("verificacao.md", "### Baseline\n\n- `true`\n\n### Verificação\n\n```bash\ntrue\n```\n\n### Evidências\n\n- `video.mov` (manual): o vídeo\n"),
        ("criterios.md", "### Concluída quando\n\n- [ ] Pronto (RF-01).\n\n  > Como comprovar: ver o arquivo.\n"),
        ("executivo.md", "Resumo.\n"),
        ("tecnico.md", "Detalhes.\n"),
        ("registro.md", "## Registro de execução\n\n| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n\n## Histórico de ajustes\n\n| # | Data | Origem | Resumo |\n|---|---|---|---|\n"),
    ];
    for (name, content) in files {
        fs::write(task.join(name), content).unwrap();
    }
    if let Some(limit) = limit {
        let config = project.harness().join("config/tasks.yml");
        let kept: String = fs::read_to_string(&config)
            .unwrap()
            .lines()
            .filter(|line| !line.starts_with("manual_attachment_max_mb"))
            .map(|line| format!("{line}\n"))
            .collect();
        fs::write(config, format!("{kept}manual_attachment_max_mb: {limit}\n")).unwrap();
    }
    project
}

fn evidence_dir(project: &common::Project) -> std::path::PathBuf {
    project.root().join(UPLOAD_TASK).join("evidencias")
}

/// Quantas entradas a pasta temporária dos anexos tem; zero quando ela não existe.
fn temp_entries(project: &common::Project) -> usize {
    fs::read_dir(evidence_dir(project).join(".anexo-tmp")).map_or(0, |entries| entries.count())
}

fn wait_until(what: &str, condition: impl Fn() -> bool) {
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while !condition() {
        assert!(std::time::Instant::now() < deadline, "{what}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn session_token(address: SocketAddr) -> String {
    let page = request(
        address,
        "GET / HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n",
    );
    let marker = "<meta name=\"painel-token\" content=\"";
    let start_at = page
        .find(marker)
        .expect("token na página: compile com o bundle")
        + marker.len();
    page[start_at..start_at + page[start_at..].find('"').unwrap()].to_string()
}

fn connect(address: SocketAddr, read_timeout: Duration) -> TcpStream {
    let stream = TcpStream::connect_timeout(&address, UPLOAD_TIMEOUT).unwrap();
    stream.set_read_timeout(Some(read_timeout)).unwrap();
    stream.set_write_timeout(Some(UPLOAD_TIMEOUT)).unwrap();
    stream
}

fn upload_head(target: &str, token: Option<&str>, framing: &str) -> String {
    let token = token
        .map(|value| format!("X-Painel-Token: {value}\r\n"))
        .unwrap_or_default();
    format!("PUT {target} HTTP/1.1\r\nHost: x\r\n{token}{framing}\r\nConnection: close\r\n\r\n")
}

/// Lê cabeçalhos e corpo pelo `Content-Length`, sem esperar o fim da conexão: o servidor pode
/// seguir lendo o corpo que recusou. O tempo limite da leitura reprova em vez de travar.
fn read_response(stream: &mut TcpStream) -> String {
    let mut data = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        let text = String::from_utf8_lossy(&data).into_owned();
        if let Some((head, body)) = text.split_once("\r\n\r\n") {
            let length = head
                .lines()
                .find_map(|line| {
                    line.to_lowercase()
                        .strip_prefix("content-length:")
                        .and_then(|value| value.trim().parse::<usize>().ok())
                })
                .unwrap_or(0);
            if body.len() >= length {
                return text;
            }
        }
        match stream.read(&mut chunk) {
            Ok(0) => return text,
            Ok(count) => data.extend_from_slice(&chunk[..count]),
            Err(error) => panic!("sem resposta completa ({error}); recebido: {text:?}"),
        }
    }
}

fn put_to(address: SocketAddr, target: &str, token: &str, body: &[u8]) -> String {
    let mut stream = connect(address, UPLOAD_TIMEOUT);
    let head = upload_head(
        target,
        Some(token),
        &format!("Content-Length: {}", body.len()),
    );
    // A recusa antes de ler o corpo fecha a escrita: a resposta é o que vale.
    let _ = stream
        .write_all(head.as_bytes())
        .and_then(|_| stream.write_all(body));
    read_response(&mut stream)
}

fn put_evidence(address: SocketAddr, token: &str, body: &[u8]) -> String {
    put_to(address, UPLOAD_TARGET, token, body)
}

#[test]
fn evidence_upload_ct28_without_or_with_a_wrong_token_is_403_and_writes_nothing() {
    let project = upload_project(None);
    let painel = start(project.root());
    for token in [None, Some("errado")] {
        let mut stream = connect(painel.address, Duration::from_secs(5));
        let head = upload_head(UPLOAD_TARGET, token, "Content-Length: 3145728");
        stream.write_all(head.as_bytes()).unwrap();
        let response = read_response(&mut stream);
        assert_eq!(
            status_line(&response),
            "HTTP/1.1 403 Forbidden",
            "{token:?}"
        );
        assert!(!evidence_dir(&project).join("video.mov").exists());
        assert_eq!(temp_entries(&project), 0);
    }
}

#[test]
fn evidence_upload_ct28_five_megabytes_arrive_equal() {
    let project = upload_project(None);
    let painel = start(project.root());
    let token = session_token(painel.address);
    let sent = sample_bytes(5 * 1024 * 1024);

    let response = put_evidence(painel.address, &token, &sent);

    assert_eq!(status_line(&response), "HTTP/1.1 200 OK", "{response}");
    assert!(
        response.contains("\"evidencia\":\"video.mov\""),
        "{response}"
    );
    assert!(
        response.contains(&format!("\"tamanho\":{}", sent.len())),
        "{response}"
    );
    assert!(response.contains("\"sha256\":\""), "{response}");
    assert_eq!(
        fs::read(evidence_dir(&project).join("video.mov")).unwrap(),
        sent
    );
}

#[test]
fn evidence_upload_ct30_above_the_limit_is_413_without_the_body() {
    let project = upload_project(Some(1));
    let painel = start(project.root());
    let token = session_token(painel.address);
    let mut stream = connect(painel.address, Duration::from_secs(5));
    let head = upload_head(
        UPLOAD_TARGET,
        Some(&token),
        &format!("Content-Length: {}", 2 * 1024 * 1024),
    );
    stream.write_all(head.as_bytes()).unwrap();

    let response = read_response(&mut stream);

    assert_eq!(
        status_line(&response),
        "HTTP/1.1 413 Payload Too Large",
        "{response}"
    );
    assert!(response.contains("limite de 1 MB"), "{response}");
    assert!(!evidence_dir(&project).join("video.mov").exists());
    assert_eq!(temp_entries(&project), 0);
}

#[test]
fn evidence_upload_ct30_without_content_length_is_411() {
    let project = upload_project(None);
    let painel = start(project.root());
    let token = session_token(painel.address);
    let mut stream = connect(painel.address, Duration::from_secs(5));
    let head = upload_head(UPLOAD_TARGET, Some(&token), "Transfer-Encoding: chunked");
    stream.write_all(head.as_bytes()).unwrap();
    stream.write_all(b"3\r\nabc\r\n0\r\n\r\n").unwrap();

    let response = read_response(&mut stream);

    assert_eq!(
        status_line(&response),
        "HTTP/1.1 411 Length Required",
        "{response}"
    );
    assert!(!evidence_dir(&project).join("video.mov").exists());
    assert_eq!(temp_entries(&project), 0);
}

#[test]
fn evidence_upload_status_and_destination_map_to_409_and_400() {
    let project = upload_project(None);
    let painel = start(project.root());
    let token = session_token(painel.address);
    let other = |extra: &str| UPLOAD_TARGET.replace("evidencia=video.mov&extensao=mov", extra);

    let not_manual = put_to(
        painel.address,
        &other("evidencia=app-base.png&extensao=png"),
        &token,
        b"x",
    );
    assert_eq!(
        status_line(&not_manual),
        "HTTP/1.1 400 Bad Request",
        "{not_manual}"
    );
    assert!(
        not_manual.contains("não é uma evidência manual"),
        "{not_manual}"
    );

    let wrong_extension = put_to(
        painel.address,
        &other("evidencia=video.mov&extensao=mp4"),
        &token,
        b"x",
    );
    assert_eq!(
        status_line(&wrong_extension),
        "HTTP/1.1 400 Bad Request",
        "{wrong_extension}"
    );
    assert!(wrong_extension.contains(".mov"), "{wrong_extension}");

    let empty = put_evidence(painel.address, &token, b"");
    assert_eq!(status_line(&empty), "HTTP/1.1 400 Bad Request", "{empty}");
    assert!(empty.contains("vazio"), "{empty}");

    let state = project.root().join(UPLOAD_TASK).join("estado.yml");
    let blocked = fs::read_to_string(&state)
        .unwrap()
        .replace("status: in_progress", "status: blocked");
    fs::write(&state, blocked).unwrap();
    let wrong_status = put_evidence(painel.address, &token, b"x");
    assert_eq!(
        status_line(&wrong_status),
        "HTTP/1.1 409 Conflict",
        "{wrong_status}"
    );

    assert!(!evidence_dir(&project).join("video.mov").exists());
    assert_eq!(temp_entries(&project), 0);
}

#[test]
fn evidence_upload_cut_connection_keeps_the_previous_file() {
    let project = upload_project(None);
    let painel = start(project.root());
    let token = session_token(painel.address);
    fs::create_dir_all(evidence_dir(&project)).unwrap();
    fs::write(evidence_dir(&project).join("video.mov"), "anterior").unwrap();

    // Mais de 1024 bytes de propósito: até esse tamanho o `tiny_http` lê o corpo antes de chamar o servidor.
    let mut stream = connect(painel.address, Duration::from_secs(5));
    let head = upload_head(UPLOAD_TARGET, Some(&token), "Content-Length: 100000");
    stream.write_all(head.as_bytes()).unwrap();
    stream.write_all(&sample_bytes(5000)).unwrap();
    wait_until("o temporário não apareceu", || {
        temp_entries(&project) == 1
    });
    drop(stream);

    wait_until("o temporário não sumiu", || temp_entries(&project) == 0);
    assert_eq!(
        fs::read_to_string(evidence_dir(&project).join("video.mov")).unwrap(),
        "anterior"
    );
}

#[cfg(unix)]
#[test]
fn evidence_upload_cb33_write_failure_is_500_and_leaves_no_temp() {
    let project = upload_project(None);
    let script = format!(
        "trap '' XFSZ; ulimit -f 8; exec '{}' painel --sem-navegador --porta 0 --pasta '{}'",
        env!("CARGO_BIN_EXE_vostok"),
        project.root().display()
    );
    let child = Command::new("sh")
        .args(["-c", &script])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let painel = running(child);
    let token = session_token(painel.address);

    let response = put_evidence(painel.address, &token, &sample_bytes(1024 * 1024));

    assert_eq!(
        status_line(&response),
        "HTTP/1.1 500 Internal Server Error",
        "{response}"
    );
    assert!(
        response.contains("não foi possível gravar o anexo"),
        "{response}"
    );
    assert!(!evidence_dir(&project).join("video.mov").exists());
    assert!(evidence_dir(&project).join(".anexo-tmp").is_dir());
    assert_eq!(temp_entries(&project), 0);
}

#[test]
fn evidence_upload_ct39_next_send_removes_a_leftover() {
    let project = upload_project(None);
    let painel = start(project.root());
    let token = session_token(painel.address);
    let leftover = evidence_dir(&project).join(".anexo-tmp/velho");
    fs::create_dir_all(leftover.parent().unwrap()).unwrap();
    fs::write(&leftover, "sobra").unwrap();

    let response = put_evidence(painel.address, &token, b"0123456789");

    assert_eq!(status_line(&response), "HTTP/1.1 200 OK", "{response}");
    assert!(!leftover.exists());
    assert_eq!(temp_entries(&project), 0);
}

// --- anexos do pedido (SPEC-038) ---

const DRAFTS: &str = "vostok/work/pedidos/anexos/.rascunho";

fn attachment_target(draft: &str, name: &str) -> String {
    format!("/api/pedidos/anexos?rascunho={draft}&nome={name}")
}

fn draft_dir(project: &common::Project, draft: &str) -> std::path::PathBuf {
    project.root().join(DRAFTS).join(draft)
}

fn draft_names(project: &common::Project, draft: &str) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(draft_dir(project, draft))
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

fn set_attachment_limit(project: &common::Project, megabytes: u64) {
    let config = project.harness().join("config/tasks.yml");
    let kept: String = fs::read_to_string(&config)
        .unwrap()
        .lines()
        .filter(|line| !line.starts_with("manual_attachment_max_mb"))
        .map(|line| format!("{line}\n"))
        .collect();
    fs::write(
        config,
        format!("{kept}manual_attachment_max_mb: {megabytes}\n"),
    )
    .unwrap();
}

/// Envia o cabeçalho e o corpo como estão e lê a resposta com tempo limite; a recusa antes de ler
/// o corpo pode fechar a escrita, e a resposta é o que vale.
fn raw_request(address: SocketAddr, head: &str, body: &[u8]) -> String {
    let mut stream = connect(address, UPLOAD_TIMEOUT);
    let _ = stream
        .write_all(head.as_bytes())
        .and_then(|_| stream.write_all(body));
    read_response(&mut stream)
}

fn json_text(response: &str, field: &str) -> String {
    let marker = format!("\"{field}\":\"");
    let start_at = response
        .find(&marker)
        .unwrap_or_else(|| panic!("sem {field}: {response}"))
        + marker.len();
    response[start_at..start_at + response[start_at..].find('"').unwrap()].to_string()
}

#[test]
fn ct62_sem_token_ou_com_token_errado_e_403_sem_gravar() {
    let project = common::Project::new();
    let painel = start(project.root());
    for token in [None, Some("errado")] {
        let mut stream = connect(painel.address, Duration::from_secs(5));
        let head = upload_head(
            &attachment_target("r1", "a.txt"),
            token,
            "Content-Length: 3145728",
        );
        stream.write_all(head.as_bytes()).unwrap();
        let response = read_response(&mut stream);
        assert_eq!(
            status_line(&response),
            "HTTP/1.1 403 Forbidden",
            "{token:?}"
        );
        assert!(!draft_dir(&project, "r1").exists());
    }
}

#[test]
fn ct62_cinco_mb_chegam_iguais_com_nome_limpo() {
    let project = common::Project::new();
    let painel = start(project.root());
    let token = session_token(painel.address);
    let sent = sample_bytes(5 * 1024 * 1024);

    let response = put_to(
        painel.address,
        &attachment_target("r1", "relat%C3%B3rio.pdf"),
        &token,
        &sent,
    );

    assert_eq!(status_line(&response), "HTTP/1.1 200 OK", "{response}");
    assert_eq!(json_text(&response, "anexo"), "01-relat_rio.pdf");
    assert!(
        response.contains(&format!("\"tamanho\":{}", sent.len())),
        "{response}"
    );
    let stored = draft_dir(&project, "r1").join("01-relat_rio.pdf");
    assert_eq!(fs::read(&stored).unwrap(), sent);
    let expected = Command::new("sha256sum").arg(&stored).output().unwrap();
    let expected = String::from_utf8_lossy(&expected.stdout);
    assert_eq!(
        json_text(&response, "sha256"),
        expected.split_whitespace().next().unwrap()
    );

    let traversal = put_to(
        painel.address,
        &attachment_target("r1", "..%2F..%2Fx.png"),
        &token,
        b"png",
    );
    assert_eq!(status_line(&traversal), "HTTP/1.1 200 OK", "{traversal}");
    assert_eq!(
        draft_names(&project, "r1"),
        ["01-relat_rio.pdf", "02-x.png"]
    );
    assert!(!project.root().join("vostok/work/x.png").exists());
}

#[test]
fn ct62_vazio_e_400_sem_gravar() {
    let project = common::Project::new();
    let painel = start(project.root());
    let token = session_token(painel.address);
    let response = put_to(
        painel.address,
        &attachment_target("r1", "a.txt"),
        &token,
        b"",
    );
    assert_eq!(
        status_line(&response),
        "HTTP/1.1 400 Bad Request",
        "{response}"
    );
    assert!(draft_names(&project, "r1").is_empty());
}

#[test]
fn ct62_cortado_nao_deixa_sobra() {
    let project = common::Project::new();
    let painel = start(project.root());
    let token = session_token(painel.address);
    let mut stream = connect(painel.address, Duration::from_secs(5));
    let head = upload_head(
        &attachment_target("r1", "a.txt"),
        Some(&token),
        "Content-Length: 1000",
    );
    stream.write_all(head.as_bytes()).unwrap();
    stream.write_all(&[7u8; 500]).unwrap();
    stream.shutdown(std::net::Shutdown::Write).unwrap();

    // Com a escrita fechada, o servidor pode encerrar sem resposta: a conexão fechada marca o fim
    // do tratamento, e o tempo limite da leitura reprova em vez de travar.
    let mut ignored = Vec::new();
    if let Err(error) = stream.read_to_end(&mut ignored) {
        assert!(
            !matches!(
                error.kind(),
                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
            ),
            "o servidor não encerrou o envio cortado: {error}"
        );
    }

    wait_until("o envio cortado deixou arquivo", || {
        draft_names(&project, "r1").is_empty()
    });
}

#[test]
fn ct62_sem_content_length_e_411() {
    let project = common::Project::new();
    let painel = start(project.root());
    let token = session_token(painel.address);
    let head = upload_head(
        &attachment_target("r1", "a.txt"),
        Some(&token),
        "Transfer-Encoding: chunked",
    );
    let response = raw_request(painel.address, &head, b"3\r\nabc\r\n0\r\n\r\n");
    assert_eq!(
        status_line(&response),
        "HTTP/1.1 411 Length Required",
        "{response}"
    );
    assert!(draft_names(&project, "r1").is_empty());
}

#[test]
fn ct62_decimo_primeiro_e_400_e_soma_acima_e_413() {
    let project = common::Project::new();
    set_attachment_limit(&project, 1);
    let painel = start(project.root());
    let token = session_token(painel.address);
    for number in 1..=10 {
        let response = put_to(
            painel.address,
            &attachment_target("r1", &format!("f{number}.txt")),
            &token,
            b"x",
        );
        assert_eq!(status_line(&response), "HTTP/1.1 200 OK", "{response}");
    }
    let eleventh = put_to(
        painel.address,
        &attachment_target("r1", "f11.txt"),
        &token,
        b"x",
    );
    assert_eq!(
        status_line(&eleventh),
        "HTTP/1.1 400 Bad Request",
        "{eleventh}"
    );
    assert_eq!(draft_names(&project, "r1").len(), 10);

    let big = sample_bytes(700 * 1024);
    let first = put_to(
        painel.address,
        &attachment_target("r2", "a.bin"),
        &token,
        &big,
    );
    assert_eq!(status_line(&first), "HTTP/1.1 200 OK", "{first}");
    let over = put_to(
        painel.address,
        &attachment_target("r2", "b.bin"),
        &token,
        &sample_bytes(400 * 1024),
    );
    assert_eq!(
        status_line(&over),
        "HTTP/1.1 413 Payload Too Large",
        "{over}"
    );
    assert_eq!(draft_names(&project, "r2"), ["01-a.bin"]);
}

#[test]
fn ct62_duas_image_png_viram_01_e_02() {
    let project = common::Project::new();
    let painel = start(project.root());
    let token = session_token(painel.address);
    for _ in 0..2 {
        let response = put_to(
            painel.address,
            &attachment_target("r1", "image.png"),
            &token,
            b"png",
        );
        assert_eq!(status_line(&response), "HTTP/1.1 200 OK", "{response}");
    }
    assert_eq!(
        draft_names(&project, "r1"),
        ["01-image.png", "02-image.png"]
    );
}

#[test]
fn ct63_cada_nome_recusado_nao_grava() {
    use vostok::commands::pedidos::REFUSED_ATTACHMENT_NAMES;
    let project = common::Project::new();
    let painel = start(project.root());
    let token = session_token(painel.address);
    for pattern in REFUSED_ATTACHMENT_NAMES {
        let name = pattern.replace(
            '*',
            if pattern.starts_with('*') {
                "chave"
            } else {
                "local"
            },
        );
        let response = put_to(
            painel.address,
            &attachment_target("r1", &name),
            &token,
            b"x",
        );
        assert_eq!(
            status_line(&response),
            "HTTP/1.1 400 Bad Request",
            "{name}: {response}"
        );
    }
    assert!(!draft_dir(&project, "r1").exists());
    let accepted = put_to(
        painel.address,
        &attachment_target("r1", ".env.example"),
        &token,
        b"x",
    );
    assert_eq!(status_line(&accepted), "HTTP/1.1 200 OK", "{accepted}");
}

#[test]
fn ct64_pedido_aparece_com_a_pasta_no_lugar() {
    let project = common::Project::new();
    let painel = start(project.root());
    let token = session_token(painel.address);
    let sent = put_to(
        painel.address,
        &attachment_target("r1", "a.txt"),
        &token,
        b"abc",
    );
    assert_eq!(status_line(&sent), "HTTP/1.1 200 OK", "{sent}");

    let mut stream = connect(painel.address, UPLOAD_TIMEOUT);
    let text = "Qual o problema?";
    let raw = format!(
        "POST /api/pedidos?acao=perguntar-agente&rascunho=r1 HTTP/1.1\r\nHost: x\r\nX-Painel-Token: {token}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{text}",
        text.len()
    );
    stream.write_all(raw.as_bytes()).unwrap();
    let response = read_response(&mut stream);

    assert_eq!(status_line(&response), "HTTP/1.1 200 OK", "{response}");
    let work = project.root().join("vostok/work/pedidos");
    let content = fs::read_to_string(work.join("pendente/PED-001.md")).unwrap();
    assert!(content.contains("## Anexos"), "{content}");
    assert!(work.join("anexos/PED-001/01-a.txt").is_file());
    assert!(!draft_dir(&project, "r1").exists());
}

#[test]
#[ignore = "500 MB: rode com --ignored"]
#[cfg(target_os = "linux")]
fn ct73_quinhentos_mb_nao_sobem_a_memoria() {
    const BLOCKS: usize = 500;
    let project = common::Project::new();
    let painel = start(project.root());
    let token = session_token(painel.address);
    let pid = painel.child.id();
    let before = peak_memory_kb(pid);

    let mut stream = connect(painel.address, Duration::from_secs(120));
    let head = upload_head(
        &attachment_target("r1", "grande.bin"),
        Some(&token),
        &format!("Content-Length: {}", BLOCKS * 1024 * 1024),
    );
    stream.write_all(head.as_bytes()).unwrap();
    let block = sample_bytes(1024 * 1024);
    for _ in 0..BLOCKS {
        stream.write_all(&block).unwrap();
    }
    let response = read_response(&mut stream);

    assert_eq!(status_line(&response), "HTTP/1.1 200 OK", "{response}");
    let growth_mb = peak_memory_kb(pid).saturating_sub(before) / 1024;
    eprintln!("alta de VmHWM: {growth_mb} MB");
    assert!(
        growth_mb <= 100,
        "o painel subiu {growth_mb} MB (limite 100)"
    );
}
