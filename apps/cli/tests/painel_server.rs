//! Testes do `vostok painel` com o binário real: escuta só em `127.0.0.1`, token de sessão na página,
//! escrita só nas três rotas de conferência e na de configurações das evidências, 405 para o resto e
//! entrega de arquivos (SPEC-042, SPEC-051, SPEC-052).
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

fn project_with_port(value: &str) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("vostok/config");
    fs::create_dir_all(&config).unwrap();
    fs::write(
        config.join("main.yml"),
        format!("paths:\n  work: ./work\npanel_port: {value}\n"),
    )
    .unwrap();
    dir
}

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn start_from_config(folder: &Path) -> Running {
    let child = vostok()
        .args(["painel", "--sem-navegador", "--pasta"])
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
fn pagina_serve_o_app_ou_explica_o_bundle_ausente() {
    let project = project();
    let painel = start(project.path());
    let response = request(
        painel.address,
        "GET /specs/SPEC-001 HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n",
    );
    let served = status_line(&response).contains(" 200 ");
    let explained = status_line(&response).contains(" 503 ")
        && response.contains("pnpm --dir apps/painel build");
    assert!(served || explained, "{response}");
}

#[test]
fn leitura_post_sem_token_e_405() {
    let project = project();
    let painel = start(project.path());
    for raw in [
        "POST /api/visao HTTP/1.1\r\nHost: x\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        "PUT /api/arquivo HTTP/1.1\r\nHost: x\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
    ] {
        let response = request(painel.address, raw);
        assert!(status_line(&response).contains(" 405 "), "{response}");
        assert!(!status_line(&response).contains(" 403 "), "{response}");
    }
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
fn panel_port_ct01_uses_the_configured_port() {
    let port = free_port();
    let project = project_with_port(&port.to_string());
    let painel = start_from_config(project.path());
    assert_eq!(painel.address.ip().to_string(), "127.0.0.1");
    assert_eq!(painel.address.port(), port);
}

#[test]
fn panel_port_ct02_missing_zero_or_invalid_uses_a_free_port() {
    let mut projects = vec![project()];
    for value in ["0", "abc", "70000"] {
        projects.push(project_with_port(value));
    }
    for project in &projects {
        let painel = start_from_config(project.path());
        assert_eq!(painel.address.ip().to_string(), "127.0.0.1");
        assert_ne!(painel.address.port(), 0);
    }
}

#[test]
fn panel_port_ct03_explicit_port_wins_even_with_zero() {
    let busy = TcpListener::bind("127.0.0.1:0").unwrap();
    let busy_port = busy.local_addr().unwrap().port();
    let project = project_with_port(&busy_port.to_string());
    let painel = start(project.path());
    assert_ne!(painel.address.port(), busy_port);
}

#[test]
fn panel_port_cb01_busy_configured_port_cites_the_key() {
    let busy = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = busy.local_addr().unwrap().port().to_string();
    let project = project_with_port(&port);
    let output = run_painel(&[
        "--pasta",
        project.path().to_str().unwrap(),
        "--sem-navegador",
    ]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(&port) && stderr.contains("ocupada") && stderr.contains("panel_port"),
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

// --- Estado da conferência no JSON da tarefa (SPEC-043) ---

const REVIEW_TASKS_CONFIG: &str = "status:
  pending: pending
  approved: approved
  in_progress: in_progress
  validating: validating
  blocked: blocked
  done: done
execution_section: Execução
evidence_subsection: Evidências
record_section: Registro de execução
evidence_from_verify: \"automática, da verificação\"
evidence_review_kinds: [automática]
";

fn review_hash(bytes: &[u8]) -> String {
    vostok::support::sha256_hex(bytes)[..8].to_string()
}

fn review_project() -> TempDir {
    let project = project();
    let root = project.path().join("vostok");
    fs::write(
        root.join("config/main.yml"),
        "paths:\n  work: ./work\n  tasks: ./work/tasks\n",
    )
    .unwrap();
    fs::write(root.join("config/tasks.yml"), REVIEW_TASKS_CONFIG).unwrap();
    let task = root.join("work/tasks/SPEC-001/TASK-01");
    let evidence = task.join("evidencias");
    fs::create_dir_all(&evidence).unwrap();
    fs::write(task.join("estado.yml"), "status: in_progress\n").unwrap();
    fs::write(
        task.join("verificacao.md"),
        "### Evidências\n\n- `conferida.txt` (automática): `cat conferida.txt`\n- `rejeitada.txt` (automática): `cat rejeitada.txt`\n- `pendente.txt` (automática): `cat pendente.txt`\n- `tela.png` (print): tela\n",
    )
    .unwrap();
    for name in ["conferida.txt", "rejeitada.txt", "pendente.txt", "tela.png"] {
        fs::write(evidence.join(name), format!("conteúdo de {name}")).unwrap();
    }
    let hash = |name: &str| review_hash(&fs::read(evidence.join(name)).unwrap());
    fs::write(
        task.join("registro.md"),
        format!(
            "## Registro de execução\n\n| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n| 1 | 2026-11-09 | conferência | usuário | evidência conferida.txt conferida pelo usuário (sha256 {}) |\n| 2 | 2026-11-09 | rejeição | blocked | evidência rejeitada.txt rejeitada pelo usuário (sha256 {}): falta o total |\n",
            hash("conferida.txt"),
            hash("rejeitada.txt")
        ),
    )
    .unwrap();
    project
}

#[test]
fn reviews_ct22_returns_the_four_states() {
    let project = review_project();
    let painel = start(project.path());
    let response = get_raw(painel.address, "/api/tarefas/SPEC-001/TASK-01", None);
    assert!(response.status().contains(" 200 "), "{}", response.head);
    let body = String::from_utf8(response.body).unwrap();
    for expected in [
        r#"{"id":"EV-01","name":"conferida.txt","kind":"automática","status":"conferida","reason":"","reviewed_at":"2026-11-09""#,
        r#""status":"recusada","reason":"falta o total","reviewed_at":"2026-11-09""#,
        r#""name":"pendente.txt","kind":"automática","status":"a conferir""#,
        r#""name":"tela.png","kind":"print","status":"sem aval""#,
        r#""id":"EV-04","name":"tela.png""#,
    ] {
        assert!(body.contains(expected), "faltou {expected} em {body}");
    }
}

#[test]
fn reviews_carry_the_date_and_the_user_words() {
    let project = review_project();
    let task = project.path().join("vostok/work/tasks/SPEC-001/TASK-01");
    let hash = |name: &str| review_hash(&fs::read(task.join("evidencias").join(name)).unwrap());
    fs::write(
        task.join("registro.md"),
        format!(
            "## Registro de execução\n\n| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n| 1 | 2026-11-10 | conferência | usuário | EV-01 conferida.txt conferida pelo usuário (sha256 {}): \"a tela bate com o layout\" |\n| 2 | 2026-11-10 | conferência | usuário | EV-02 rejeitada.txt conferida pelo usuário (sha256 {}) |\n| 3 | 2026-11-10 | conferência | usuário | EV-03 pendente.txt conferida pelo usuário (sha256 {}) |\n| 4 | 2026-11-11 | conferência retirada | usuário | EV-03 pendente.txt: \"engano\" |\n",
            hash("conferida.txt"),
            hash("rejeitada.txt"),
            hash("pendente.txt")
        ),
    )
    .unwrap();
    let painel = start(project.path());
    let response = get_raw(painel.address, "/api/tarefas/SPEC-001/TASK-01", None);
    assert!(response.status().contains(" 200 "), "{}", response.head);
    let body = String::from_utf8(response.body).unwrap();
    for expected in [
        r#""name":"conferida.txt","kind":"automática","status":"conferida","reason":"a tela bate com o layout","reviewed_at":"2026-11-10""#,
        r#""name":"rejeitada.txt","kind":"automática","status":"conferida","reason":"","reviewed_at":"2026-11-10""#,
        r#""name":"pendente.txt","kind":"automática","status":"a conferir","reason":"","reviewed_at":"""#,
    ] {
        assert!(body.contains(expected), "faltou {expected} em {body}");
    }
}

#[test]
fn reviews_only_get_post_is_405() {
    let project = review_project();
    let painel = start(project.path());
    for method in ["POST", "PUT", "DELETE"] {
        let raw = format!(
            "{method} /api/tarefas/SPEC-001/TASK-01 HTTP/1.1\r\nHost: x\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        );
        let response = request(painel.address, &raw);
        assert!(status_line(&response).contains(" 405 "), "{response}");
    }
}

// --- Conferência pelo painel (SPEC-051) ---

const WRITABLE_TASK: &str = "vostok/work/tasks/SPEC-001/TASK-01";
const REVIEW_ROUTE: &str = "/api/tarefas/SPEC-001/TASK-01/evidencias/EV-01";

/// Projeto com o harness extraído e a TASK-01 em pasta, em execução, com `saida.txt` a conferir.
fn writable_project() -> common::Project {
    let project = common::Project::new();
    let root = project.root();
    fs::write(root.join("valor.txt"), "a\n").unwrap();
    let task = root.join(WRITABLE_TASK);
    fs::create_dir_all(&task).unwrap();
    let files = [
        ("estado.yml", "id: TASK-01\nspec: SPEC-001\ntitle: \"Tarefa\"\neffort: simples\ndepends_on: []\ncovers: [RF-01]\ntype: tarefa\nlabels: [SPEC-001, simples]\nstatus: pending\ncontent_hash: null\nadjustments: 0\nadjusted_at: null\napproved_at: null\napproved_hash: null\nattempts: 0\ntests_result: null\nregression_result: null\nevidence_result: null\n"),
        ("execucao.md", "### Ler\n\n- `README.md`\n\n### Mudar\n\n- `novo.txt` (criar): arquivo novo.\n\n### Passos\n\n1. Crie o arquivo.\n"),
        ("verificacao.md", "### Baseline\n\n- `cat valor.txt`\n\n### Verificação\n\n```bash\ntrue\n```\n\n### Evidências\n\n- `saida.txt` (automática): `cat valor.txt`\n  - Prova: o valor é lido.\n  - Trecho: a linha.\n  - Falha: arquivo ausente.\n"),
        ("criterios.md", "### Concluída quando\n\n- [ ] Pronto (RF-01).\n\n  > Como comprovar: ver o arquivo.\n"),
        ("executivo.md", "Resumo.\n"),
        ("tecnico.md", "Detalhes.\n"),
        ("registro.md", "## Registro de execução\n\n| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n\n## Histórico de ajustes\n\n| # | Data | Origem | Resumo |\n|---|---|---|---|\n"),
    ];
    for (name, content) in files {
        fs::write(task.join(name), content).unwrap();
    }
    for args in [
        ["task", "adjust", "record-manual", "SPEC-001", "TASK-01"].as_slice(),
        ["task", "approve", "approve", "SPEC-001", "TASK-01"].as_slice(),
        ["task", "start", "SPEC-001", "TASK-01"].as_slice(),
        ["task", "evidence", "SPEC-001", "TASK-01"].as_slice(),
    ] {
        project.vostok().args(args).assert().success();
    }
    project
}

fn page_token(address: SocketAddr) -> String {
    let body = String::from_utf8(get_raw(address, "/", None).body).unwrap();
    let marker = "<meta name=\"vostok-token\" content=\"";
    let start = body
        .find(marker)
        .unwrap_or_else(|| panic!("página sem token: {body}"))
        + marker.len();
    body[start..]
        .split('"')
        .next()
        .unwrap_or_default()
        .to_string()
}

fn trusted(address: SocketAddr, host: &str, token: &str) -> Vec<(String, String)> {
    let port = address.port();
    vec![
        ("Host".to_string(), format!("{host}:{port}")),
        ("Origin".to_string(), format!("http://{host}:{port}")),
        ("X-Vostok-Token".to_string(), token.to_string()),
    ]
}

fn send(
    address: SocketAddr,
    method: &str,
    target: &str,
    headers: &[(String, String)],
    body: &str,
) -> Raw {
    let mut raw = format!("{method} {target} HTTP/1.1\r\n");
    if !headers.iter().any(|(name, _)| name == "Host") {
        raw.push_str("Host: x\r\n");
    }
    for (name, value) in headers {
        raw.push_str(&format!("{name}: {value}\r\n"));
    }
    raw.push_str(&format!(
        "Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    ));
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

fn post(address: SocketAddr, target: &str, headers: &[(String, String)], body: &str) -> Raw {
    send(address, "POST", target, headers, body)
}

fn task_file(project: &common::Project, name: &str) -> String {
    fs::read_to_string(project.root().join(WRITABLE_TASK).join(name)).unwrap_or_default()
}

fn body_text(raw: &Raw) -> String {
    String::from_utf8_lossy(&raw.body).into_owned()
}

#[cfg(painel_bundle)]
#[test]
fn conferencia_post_grava_aceite_com_origem_painel() {
    let project = writable_project();
    let painel = start(project.root());
    let token = page_token(painel.address);
    let response = post(
        painel.address,
        &format!("{REVIEW_ROUTE}/aceitar"),
        &trusted(painel.address, "127.0.0.1", &token),
        r#"{"texto": "bate com o log"}"#,
    );
    let body = body_text(&response);
    assert!(
        response.status().contains(" 200 "),
        "{} {body}",
        response.head
    );
    for expected in [
        r#""id":"EV-01""#,
        r#""status":"conferida""#,
        r#""origin":"painel""#,
    ] {
        assert!(body.contains(expected), "faltou {expected} em {body}");
    }
    let reviews = task_file(&project, "conferencias.md");
    assert!(
        reviews.contains("| EV-01 | saida.txt | conferida |"),
        "{reviews}"
    );
    assert!(reviews.contains("| painel | bate com o log |"), "{reviews}");
    let record = task_file(&project, "registro.md");
    assert!(
        record.contains("| conferência | usuário | EV-01 conferida no painel |"),
        "{record}"
    );
}

#[cfg(painel_bundle)]
#[test]
fn conferencia_post_grava_recusa_e_retirada_com_texto() {
    let project = writable_project();
    let painel = start(project.root());
    let token = page_token(painel.address);
    let headers = trusted(painel.address, "localhost", &token);
    let rejected = post(
        painel.address,
        &format!("{REVIEW_ROUTE}/recusar"),
        &headers,
        r#"{"texto": "falta o total"}"#,
    );
    let body = body_text(&rejected);
    assert!(
        rejected.status().contains(" 200 "),
        "{} {body}",
        rejected.head
    );
    assert!(body.contains(r#""status":"recusada""#), "{body}");
    assert!(body.contains(r#""task_status":"in_progress""#), "{body}");
    let reviews = task_file(&project, "conferencias.md");
    assert!(
        reviews.contains("| EV-01 | saida.txt | recusada |"),
        "{reviews}"
    );
    let withdrawn = post(
        painel.address,
        &format!("{REVIEW_ROUTE}/retirar"),
        &headers,
        r#"{"texto": "engano"}"#,
    );
    let body = body_text(&withdrawn);
    assert!(
        withdrawn.status().contains(" 200 "),
        "{} {body}",
        withdrawn.head
    );
    assert!(body.contains(r#""status":"a conferir""#), "{body}");
    let reviews = task_file(&project, "conferencias.md");
    assert!(
        reviews.contains("| EV-01 | saida.txt | retirada |"),
        "{reviews}"
    );
}

#[cfg(painel_bundle)]
#[test]
fn conferencia_post_recusa_sem_token_ou_de_outra_origem() {
    let project = writable_project();
    let painel = start(project.root());
    let token = page_token(painel.address);
    let record = task_file(&project, "registro.md");
    let port = painel.address.port();
    let good = trusted(painel.address, "127.0.0.1", &token);
    let without_token: Vec<_> = good
        .iter()
        .filter(|(name, _)| name != "X-Vostok-Token")
        .cloned()
        .collect();
    let wrong_token = trusted(painel.address, "127.0.0.1", &"0".repeat(64));
    let mut evil_origin = good.clone();
    evil_origin[1].1 = "http://evil.example".to_string();
    let without_origin: Vec<_> = good
        .iter()
        .filter(|(name, _)| name != "Origin")
        .cloned()
        .collect();
    let mut evil_host = good.clone();
    evil_host[0].1 = format!("evil.example:{port}");
    for headers in [
        without_token,
        wrong_token,
        evil_origin,
        without_origin,
        evil_host,
    ] {
        let response = post(
            painel.address,
            &format!("{REVIEW_ROUTE}/aceitar"),
            &headers,
            r#"{"texto": "ok"}"#,
        );
        assert!(
            response.status().contains(" 403 "),
            "{:?} {}",
            headers,
            response.head
        );
        assert!(task_file(&project, "conferencias.md").is_empty());
        assert_eq!(task_file(&project, "registro.md"), record);
    }
}

#[cfg(painel_bundle)]
#[test]
fn conferencia_post_recusa_texto_ausente() {
    let project = writable_project();
    let painel = start(project.root());
    let headers = trusted(painel.address, "127.0.0.1", &page_token(painel.address));
    for (action, body) in [
        ("recusar", "{}"),
        ("retirar", r#"{"texto": "   "}"#),
        ("aceitar", "não é json"),
        ("aceitar", r#"{"texto": 5}"#),
    ] {
        let response = post(
            painel.address,
            &format!("{REVIEW_ROUTE}/{action}"),
            &headers,
            body,
        );
        assert!(
            response.status().contains(" 400 "),
            "{action} {body} {}",
            response.head
        );
        assert!(task_file(&project, "conferencias.md").is_empty());
    }
}

#[cfg(painel_bundle)]
#[test]
fn conferencia_post_recusa_evidencia_ou_tarefa_inexistente() {
    let project = writable_project();
    let painel = start(project.root());
    let headers = trusted(painel.address, "127.0.0.1", &page_token(painel.address));
    for route in [
        "/api/tarefas/SPEC-001/TASK-01/evidencias/EV-09",
        "/api/tarefas/SPEC-001/TASK-09/evidencias/EV-01",
        "/api/tarefas/SPEC-999/TASK-01/evidencias/EV-01",
        "/api/tarefas/SPEC-001/TASK-01/evidencias/saida.txt",
    ] {
        let response = post(
            painel.address,
            &format!("{route}/aceitar"),
            &headers,
            r#"{"texto": "ok"}"#,
        );
        assert!(
            response.status().contains(" 404 "),
            "{route} {}",
            response.head
        );
    }
}

#[cfg(painel_bundle)]
#[test]
fn conferencia_post_recusa_tarefa_fora_de_conferencia() {
    let project = writable_project();
    let painel = start(project.root());
    let headers = trusted(painel.address, "127.0.0.1", &page_token(painel.address));
    let changed = post(
        painel.address,
        &format!("{REVIEW_ROUTE}/aceitar"),
        &headers,
        r#"{"texto": "", "sha256": "00000000"}"#,
    );
    assert!(changed.status().contains(" 409 "), "{}", changed.head);
    assert!(task_file(&project, "conferencias.md").is_empty());
    let state = project.root().join(WRITABLE_TASK).join("estado.yml");
    let text = fs::read_to_string(&state).unwrap();
    fs::write(&state, text.replace("status: in_progress", "status: done")).unwrap();
    let closed = post(
        painel.address,
        &format!("{REVIEW_ROUTE}/aceitar"),
        &headers,
        r#"{"texto": "ok"}"#,
    );
    assert!(closed.status().contains(" 409 "), "{}", closed.head);
    assert!(task_file(&project, "conferencias.md").is_empty());
}

#[test]
fn conferencia_post_somente_tres_rotas_aceitam_post() {
    let project = project();
    let painel = start(project.path());
    let headers = trusted(painel.address, "127.0.0.1", "x");
    for method in ["PUT", "DELETE", "PATCH"] {
        let response = send(
            painel.address,
            method,
            &format!("{REVIEW_ROUTE}/aceitar"),
            &headers,
            "{}",
        );
        assert!(
            response.status().contains(" 405 "),
            "{method} {}",
            response.head
        );
    }
    for route in [
        "/api/visao".to_string(),
        "/api/tarefas/SPEC-001/TASK-01".to_string(),
        REVIEW_ROUTE.to_string(),
        format!("{REVIEW_ROUTE}/apagar"),
        "/".to_string(),
    ] {
        let response = post(painel.address, &route, &headers, "{}");
        assert!(
            response.status().contains(" 405 "),
            "POST {route} {}",
            response.head
        );
    }
}

#[cfg(painel_bundle)]
#[test]
fn conferencia_post_pagina_entrega_o_token_da_sessao() {
    let project = project();
    let painel = start(project.path());
    let token = page_token(painel.address);
    assert_eq!(token.len(), 64, "{token}");
    assert!(token.chars().all(|c| c.is_ascii_hexdigit()), "{token}");
    let page = body_text(&get_raw(painel.address, "/specs/SPEC-001", None));
    assert!(page.contains(&format!("content=\"{token}\"")), "{page}");
    let other = start(project.path());
    assert_ne!(page_token(other.address), token);
}

// --- Configurações das evidências (SPEC-052) ---

const SETTINGS_ROUTE: &str = "/api/configuracoes/evidencias";
const SETTINGS_KEYS: [&str; 12] = [
    "documents.embed_max_mb",
    "documents.image_extensions",
    "documents.text_extensions",
    "documents.video_extensions",
    "tasks.evidence_baseline_timeout",
    "tasks.evidence_review_kinds",
    "tasks.evidence_title_max",
    "tasks.persist_dir",
    "tasks.persist_max_mb",
    "tasks.persist_readme",
    "tasks.regression_alert",
    "tasks.regression_ignore",
];

fn config_text(project: &common::Project, name: &str) -> String {
    fs::read_to_string(project.harness().join("config").join(name)).unwrap()
}

fn settings_get(address: SocketAddr) -> serde_json::Value {
    let raw = get_raw(address, SETTINGS_ROUTE, None);
    assert!(raw.status().contains(" 200 "), "{}", raw.head);
    serde_json::from_slice(&raw.body).unwrap()
}

fn settings_json_of(raw: &Raw) -> serde_json::Value {
    serde_json::from_slice(&raw.body)
        .unwrap_or_else(|_| panic!("corpo sem JSON: {}", body_text(raw)))
}

fn settings_request(values: serde_json::Value, files: &serde_json::Value) -> String {
    serde_json::json!({"valores": values, "arquivos": files}).to_string()
}

#[test]
fn configuracoes_get_le_as_chaves_e_os_sha256() {
    use serde_json::json;
    let project = common::Project::new();
    let painel = start(project.root());
    let raw = get_raw(painel.address, SETTINGS_ROUTE, None);
    assert!(raw.status().contains(" 200 "), "{}", raw.head);
    assert!(
        raw.header("Content-Type")
            .unwrap_or_default()
            .contains("application/json"),
        "{}",
        raw.head
    );
    let settings = settings_json_of(&raw);
    let values = &settings["valores"];
    let mut keys: Vec<&str> = values
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(keys, SETTINGS_KEYS);
    assert_eq!(values["tasks.evidence_baseline_timeout"], 120);
    assert_eq!(values["tasks.evidence_title_max"], 60);
    assert_eq!(values["tasks.persist_dir"], "");
    assert_eq!(
        values["tasks.evidence_review_kinds"],
        json!([
            "automática",
            "automática, da verificação",
            "diff",
            "manual",
            "print"
        ])
    );
    assert_eq!(values["documents.video_extensions"], json!(["mp4", "webm"]));
    let files = settings["arquivos"].as_object().unwrap();
    let mut names: Vec<&str> = files.keys().map(String::as_str).collect();
    names.sort_unstable();
    assert_eq!(names, ["documents.yml", "tasks.yml"]);
    for name in names {
        assert_eq!(
            files[name],
            vostok::support::sha256_hex(config_text(&project, name).as_bytes()),
            "{name}"
        );
    }
}

#[cfg(painel_bundle)]
#[test]
fn configuracoes_post_grava_so_as_chaves_enviadas() {
    use serde_json::json;
    let project = common::Project::new();
    let painel = start(project.root());
    let before = settings_get(painel.address);
    let tasks_before = config_text(&project, "tasks.yml");
    let response = post(
        painel.address,
        SETTINGS_ROUTE,
        &trusted(painel.address, "127.0.0.1", &page_token(painel.address)),
        &settings_request(
            json!({"tasks.evidence_title_max": 80, "documents.video_extensions": ["mp4", "webm", "mov"]}),
            &before["arquivos"],
        ),
    );
    assert!(
        response.status().contains(" 200 "),
        "{} {}",
        response.head,
        body_text(&response)
    );
    let saved = settings_json_of(&response);
    assert_eq!(saved["valores"]["tasks.evidence_title_max"], 80);
    assert_eq!(
        saved["valores"]["documents.video_extensions"],
        json!(["mp4", "webm", "mov"])
    );
    assert_eq!(saved["valores"]["tasks.evidence_baseline_timeout"], 120);
    let tasks_after = config_text(&project, "tasks.yml");
    assert_eq!(
        saved["arquivos"]["tasks.yml"],
        vostok::support::sha256_hex(tasks_after.as_bytes())
    );
    assert_ne!(
        saved["arquivos"]["tasks.yml"],
        before["arquivos"]["tasks.yml"]
    );
    assert!(
        tasks_after.contains("evidence_title_max: 80"),
        "{tasks_after}"
    );
    assert!(
        !tasks_after.contains("evidence_title_max: 60"),
        "{tasks_after}"
    );
    assert_eq!(tasks_after.lines().count(), tasks_before.lines().count());
    assert_eq!(
        settings_get(painel.address)["valores"]["tasks.evidence_title_max"],
        80
    );
}

#[cfg(painel_bundle)]
#[test]
fn configuracoes_post_recusa_sem_token_ou_de_outra_origem() {
    use serde_json::json;
    let project = common::Project::new();
    let painel = start(project.root());
    let token = page_token(painel.address);
    let tasks = config_text(&project, "tasks.yml");
    let documents = config_text(&project, "documents.yml");
    let body = settings_request(
        json!({"tasks.evidence_title_max": 80}),
        &settings_get(painel.address)["arquivos"],
    );
    let port = painel.address.port();
    let good = trusted(painel.address, "localhost", &token);
    let without_token: Vec<_> = good
        .iter()
        .filter(|(name, _)| name != "X-Vostok-Token")
        .cloned()
        .collect();
    let wrong_token = trusted(painel.address, "localhost", &"0".repeat(64));
    let mut evil_origin = good.clone();
    evil_origin[1].1 = "http://evil.example".to_string();
    let without_origin: Vec<_> = good
        .iter()
        .filter(|(name, _)| name != "Origin")
        .cloned()
        .collect();
    let mut evil_host = good.clone();
    evil_host[0].1 = format!("evil.example:{port}");
    for headers in [
        without_token,
        wrong_token,
        evil_origin,
        without_origin,
        evil_host,
    ] {
        let response = post(painel.address, SETTINGS_ROUTE, &headers, &body);
        assert!(
            response.status().contains(" 403 "),
            "{:?} {}",
            headers,
            response.head
        );
        assert_eq!(config_text(&project, "tasks.yml"), tasks);
        assert_eq!(config_text(&project, "documents.yml"), documents);
    }
}

#[cfg(painel_bundle)]
#[test]
fn configuracoes_post_recusa_chave_fora_da_lista_ou_valor_invalido() {
    use serde_json::json;
    let project = common::Project::new();
    let painel = start(project.root());
    let headers = trusted(painel.address, "127.0.0.1", &page_token(painel.address));
    let tasks = config_text(&project, "tasks.yml");
    let documents = config_text(&project, "documents.yml");
    let files = settings_get(painel.address)["arquivos"].clone();
    let cases = [
        (
            settings_request(json!({"tasks.max_attempts": 5}), &files),
            "tasks.max_attempts",
        ),
        (
            settings_request(
                json!({"tasks.evidence_title_max": 80, "tasks.evidence_baseline_timeout": -1}),
                &files,
            ),
            "tasks.evidence_baseline_timeout",
        ),
        (
            settings_request(json!({"tasks.evidence_title_max": true}), &files),
            "tasks.evidence_title_max",
        ),
        ("não é json".to_string(), "corpo"),
        (r#"{"valores": 5, "arquivos": {}}"#.to_string(), "valores"),
    ];
    for (body, key) in cases {
        let response = post(painel.address, SETTINGS_ROUTE, &headers, &body);
        assert!(
            response.status().contains(" 400 "),
            "{body} {}",
            response.head
        );
        assert!(
            response
                .header("Content-Type")
                .unwrap_or_default()
                .contains("application/json"),
            "{body} {}",
            response.head
        );
        let errors = &settings_json_of(&response)["erros"];
        assert!(errors.get(key).is_some(), "{body} {errors}");
        assert_eq!(config_text(&project, "tasks.yml"), tasks, "{body}");
        assert_eq!(config_text(&project, "documents.yml"), documents, "{body}");
    }
}

#[cfg(painel_bundle)]
#[test]
fn configuracoes_post_recusa_arquivo_alterado_depois_da_leitura() {
    use serde_json::json;
    let project = common::Project::new();
    let painel = start(project.root());
    let headers = trusted(painel.address, "127.0.0.1", &page_token(painel.address));
    let before = settings_get(painel.address);
    let tasks_path = project.harness().join("config").join("tasks.yml");
    let edited = format!("{}# editado à mão\n", config_text(&project, "tasks.yml"));
    fs::write(&tasks_path, &edited).unwrap();
    let stale = post(
        painel.address,
        SETTINGS_ROUTE,
        &headers,
        &settings_request(json!({"tasks.evidence_title_max": 80}), &before["arquivos"]),
    );
    assert!(stale.status().contains(" 409 "), "{}", stale.head);
    assert_eq!(settings_json_of(&stale), json!({"arquivo": "tasks.yml"}));
    assert_eq!(config_text(&project, "tasks.yml"), edited);
    let reread = post(
        painel.address,
        SETTINGS_ROUTE,
        &headers,
        &settings_request(
            json!({"tasks.evidence_title_max": 80}),
            &settings_get(painel.address)["arquivos"],
        ),
    );
    assert!(
        reread.status().contains(" 200 "),
        "{} {}",
        reread.head,
        body_text(&reread)
    );
}

#[test]
fn configuracoes_somente_get_e_post_na_rota() {
    let project = project();
    let painel = start(project.path());
    let headers = trusted(painel.address, "127.0.0.1", "x");
    for method in ["PUT", "DELETE", "PATCH"] {
        let response = send(painel.address, method, SETTINGS_ROUTE, &headers, "{}");
        assert!(
            response.status().contains(" 405 "),
            "{method} {}",
            response.head
        );
    }
    for route in [
        "/api/configuracoes",
        "/api/configuracoes/geral",
        "/api/configuracoes/evidencias/extra",
    ] {
        let response = post(painel.address, route, &headers, "{}");
        assert!(
            response.status().contains(" 405 "),
            "POST {route} {}",
            response.head
        );
    }
    let protected = post(painel.address, SETTINGS_ROUTE, &headers, "{}");
    assert!(protected.status().contains(" 403 "), "{}", protected.head);
    let unknown = get_raw(painel.address, "/api/configuracoes/geral", None);
    assert!(unknown.status().contains(" 404 "), "{}", unknown.head);
}
