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

#[allow(clippy::too_many_arguments)]
fn write_request(
    project: &Project,
    folder: &str,
    id: &str,
    action: &str,
    task: &str,
    updated: &str,
    extra: &str,
    result: &str,
) {
    write(
        project,
        &format!("vostok/work/pedidos/{folder}/{id}.md"),
        &format!("---\nid: {id}\nacao: {action}\nspec: SPEC-001\ntarefa: {task}\nestado: {folder}\norigem: painel\ncriado_em: 2026-01-01T00:00:00Z\natualizado_em: {updated}\n{extra}---\n\n## Pedido\n\nPedido {id}.\n\n## Resultado\n\n{result}\n"),
    );
}

fn overview(project: &Project) -> String {
    let running = start(project.root());
    let response = get(running.address, "/api/visao");
    assert_status(&response, "200");
    response
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
    let avais = read(&project, "vostok/work/avais/SPEC-001.log");
    assert!(
        avais.contains("SPEC-001/TASK-01\t") && avais.trim_end().ends_with("\tpainel\taprovar"),
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
    let order = read(&project, "vostok/work/pedidos/pendente/PED-001.md");
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

#[test]
fn requests_in_the_overview_come_from_pending_and_in_progress() {
    let project = Project::new();
    let day = |n: u32| format!("2026-01-0{n}T00:00:00Z");
    write_request(
        &project,
        "pendente",
        "PED-002",
        "planejar-spec",
        "",
        &day(2),
        "",
        "",
    );
    write_request(
        &project,
        "em_andamento",
        "PED-001",
        "executar-tarefa",
        "TASK-01",
        &day(1),
        "",
        "",
    );
    write_request(
        &project,
        "feito",
        "PED-003",
        "ajustar-spec",
        "",
        &day(3),
        "",
        "",
    );
    write_request(
        &project,
        "recusado",
        "PED-004",
        "ajustar-spec",
        "",
        &day(3),
        "",
        "",
    );
    let response = overview(&project);
    let expected = "\"requests\":[{\"id\":\"PED-001\",\"acao\":\"executar-tarefa\",\"spec\":\"SPEC-001\",\"tarefa\":\"TASK-01\",\"onda\":\"\",\"estado\":\"em_andamento\",\"atualizado_em\":\"2026-01-01T00:00:00Z\"},{\"id\":\"PED-002\",\"acao\":\"planejar-spec\",\"spec\":\"SPEC-001\",\"tarefa\":\"\",\"onda\":\"\",\"estado\":\"pendente\",\"atualizado_em\":\"2026-01-02T00:00:00Z\"}]";
    assert!(response.contains(expected), "{response}");
}

#[test]
fn requests_empty_without_open_requests() {
    let response = overview(&Project::new());
    assert!(response.contains("\"requests\":[]"), "{response}");
}

#[test]
fn requests_carry_the_wave_of_executar_onda() {
    let project = Project::new();
    let at = "2026-01-01T00:00:00Z";
    write_request(
        &project,
        "pendente",
        "PED-001",
        "executar-onda",
        "",
        at,
        "",
        "",
    );
    write_request(
        &project,
        "pendente",
        "PED-002",
        "executar-onda",
        "",
        at,
        "",
        "",
    );
    write_request(
        &project,
        "pendente",
        "PED-003",
        "executar-tarefa",
        "TASK-01",
        at,
        "",
        "",
    );
    for (id, from, to) in [
        ("PED-001", "Pedido PED-001.", "Onda 2.\nCom cuidado."),
        ("PED-002", "Pedido PED-002.", "Sem onda."),
        ("PED-003", "Pedido PED-003.", "Onda 3."),
    ] {
        let path = project
            .root()
            .join(format!("vostok/work/pedidos/pendente/{id}.md"));
        let content = fs::read_to_string(&path).unwrap().replace(from, to);
        fs::write(path, content).unwrap();
    }
    let response = overview(&project);
    for expected in [
        "\"id\":\"PED-001\",\"acao\":\"executar-onda\",\"spec\":\"SPEC-001\",\"tarefa\":\"\",\"onda\":\"2\"",
        "\"id\":\"PED-002\",\"acao\":\"executar-onda\",\"spec\":\"SPEC-001\",\"tarefa\":\"\",\"onda\":\"\"",
        "\"id\":\"PED-003\",\"acao\":\"executar-tarefa\",\"spec\":\"SPEC-001\",\"tarefa\":\"TASK-01\",\"onda\":\"\"",
    ] {
        assert!(response.contains(expected), "{expected}\n{response}");
    }
}

#[test]
fn loop_in_the_overview_comes_from_the_round_file() {
    let project = Project::new();
    write(
        &project,
        "vostok/work/pedidos/.loop",
        "ultima_rodada: 2026-11-03T10:00:00Z\nintervalo: 90\nagente: codex\n",
    );
    let response = overview(&project);
    let expected = "\"loop\":{\"ultima_rodada\":\"2026-11-03T10:00:00Z\",\"intervalo\":90,\"agente\":\"codex\"}";
    assert!(response.contains(expected), "{response}");
}

#[test]
fn loop_after_vostok_pedidos_rodada() {
    let project = Project::new();
    project
        .vostok()
        .args(["pedidos", "rodada", "--intervalo", "60"])
        .assert()
        .success();
    let response = overview(&project);
    assert!(
        response.contains("\"loop\":{\"ultima_rodada\":\""),
        "{response}"
    );
    assert!(
        response.contains("\",\"intervalo\":60,\"agente\":\"claude\"}"),
        "{response}"
    );
}

#[test]
fn loop_null_without_the_round_file() {
    let response = overview(&Project::new());
    assert!(
        response.contains("\"requests\":[],\"loop\":null"),
        "{response}"
    );
}

#[test]
fn ct23_atividades_por_paginas_sem_repetir() {
    let project = pending_task();
    planned_spec(&project);
    let mut registro = String::from(
        "## Registro de execução\n\n| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n",
    );
    for number in 1..=120 {
        registro.push_str(&format!(
            "| {number} | 2026-11-03T10:00:00Z | testes | passed | linha {number} |\n"
        ));
    }
    write(&project, &format!("{FOLDER}/registro.md"), &registro);
    let running = start(project.root());
    let page = |offset: &str| {
        let response = get(
            running.address,
            &format!("/api/atividades?tipo=tarefa&spec=SPEC-001&limite=50&deslocamento={offset}"),
        );
        assert_status(&response, "200");
        response
    };
    let details = |response: &str| -> Vec<String> {
        response
            .split("\"detail\":\"")
            .skip(1)
            .map(|rest| rest.split('"').next().unwrap().to_string())
            .collect()
    };
    let first = page("0");
    let second = page("50");
    let third = page("100");
    let counts =
        [&first, &second, &third].map(|response| response.matches("\"detail\":\"linha ").count());
    assert_eq!(counts, [50, 50, 20]);
    assert_eq!(details(&first)[0], "linha 120");
    assert_eq!(details(&third).last().unwrap(), "linha 1");
    let all: std::collections::BTreeSet<String> = [&first, &second, &third]
        .iter()
        .flat_map(|response| details(response))
        .collect();
    assert_eq!(all.len(), 120);
    let body_of = |response: &str| {
        response
            .split("\r\n\r\n")
            .nth(1)
            .unwrap_or_default()
            .to_string()
    };
    assert_eq!(body_of(&page("abc")), body_of(&first));
    assert_eq!(body_of(&page("-1")), body_of(&first));
    assert_eq!(body_of(&page("500")), "[]");
}

/// Trecho de uma tarefa na resposta, entre o `id` dela e o `last_outcome` seguinte.
fn task_json<'a>(response: &'a str, id: &str) -> &'a str {
    let marker = format!("\"id\":\"{id}\"");
    let start = response
        .find(&marker)
        .unwrap_or_else(|| panic!("sem {id}: {response}"));
    let end = start + response[start..].find("\"last_outcome\"").unwrap();
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

#[test]
fn esperando_api_resumo_e_detalhe() {
    let project = Project::new();
    planned_spec(&project);
    state_only(
        &project,
        "TASK-01",
        "waiting_on: [TASK-02]\nwaiting_reason: \"espera \\\"o\\\" candidato\"\n",
    );
    state_only(&project, "TASK-02", "");
    let running = start(project.root());
    let waiting = "\"waiting_on\":\"[TASK-02]\"";
    let reason = "\"waiting_reason\":\"espera \\\"o\\\" candidato\"";
    let overview = overview(&project);
    assert!(
        task_json(&overview, "TASK-01").contains(waiting),
        "{overview}"
    );
    assert!(
        task_json(&overview, "TASK-01").contains(reason),
        "{overview}"
    );
    for expected in ["\"waiting_on\":\"\"", "\"waiting_reason\":\"\""] {
        assert!(
            task_json(&overview, "TASK-02").contains(expected),
            "{overview}"
        );
    }
    let detail = get(running.address, "/api/tarefas/SPEC-001/TASK-01");
    assert_status(&detail, "200");
    assert!(
        detail.contains(waiting) && detail.contains(reason),
        "{detail}"
    );
    let detail = get(running.address, "/api/tarefas/SPEC-001/TASK-02");
    assert_status(&detail, "200");
    for expected in ["\"waiting_on\":\"\"", "\"waiting_reason\":\"\""] {
        assert!(detail.contains(expected), "{detail}");
    }
}

#[test]
fn last_outcome_of_the_spec_and_of_the_task() {
    let project = pending_task();
    write(
        &project,
        "vostok/work/specs/refined/SPEC-001-plano.md",
        "---\nid: SPEC-001\ntitle: \"Plano\"\nstatus: planned\ntype: fix\napproved_hash: abc\n---\n\n# Plano\n",
    );
    write_request(
        &project,
        "feito",
        "PED-001",
        "ajustar-spec",
        "",
        "2026-01-01T00:00:00Z",
        "desfecho: concluido\nsaidas: []\n",
        "Ajustada.",
    );
    write_request(
        &project,
        "feito",
        "PED-002",
        "ajustar-spec",
        "",
        "2026-01-02T00:00:00Z",
        "desfecho: pausado\nsaidas: [criar-spec]\n",
        "Falta a spec de login.",
    );
    write_request(
        &project,
        "feito",
        "PED-003",
        "ajustar-tarefa",
        "TASK-01",
        "2026-01-03T00:00:00Z",
        "",
        "Tarefa ajustada.",
    );
    write_request(
        &project,
        "pendente",
        "PED-004",
        "ajustar-tarefa",
        "TASK-01",
        "2026-01-04T00:00:00Z",
        "",
        "",
    );
    let running = start(project.root());
    let spec_outcome = "\"last_outcome\":{\"id\":\"PED-002\",\"acao\":\"ajustar-spec\",\"desfecho\":\"pausado\",\"pergunta\":\"\",\"saidas\":[\"criar-spec\"],\"resultado\":\"Falta a spec de login.\"}";
    let task_outcome = "\"last_outcome\":{\"id\":\"PED-003\",\"acao\":\"ajustar-tarefa\",\"desfecho\":\"concluido\",\"pergunta\":\"\",\"saidas\":[],\"resultado\":\"Tarefa ajustada.\"}";
    for target in ["/api/visao", "/api/specs/SPEC-001"] {
        let response = get(running.address, target);
        assert_status(&response, "200");
        assert!(response.contains(spec_outcome), "{response}");
        assert!(response.contains(task_outcome), "{response}");
    }
    let response = get(running.address, "/api/tarefas/SPEC-001/TASK-01");
    assert_status(&response, "200");
    assert!(response.contains(task_outcome), "{response}");
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

#[test]
fn last_outcome_null_without_done_requests() {
    let project = spec_with_plan("abc", "approved", None);
    let running = start(project.root());
    for target in ["/api/visao", "/api/specs/SPEC-001"] {
        let response = get(running.address, target);
        assert_status(&response, "200");
        assert!(
            response.contains("\"plan_outdated\":false,\"last_outcome\":null"),
            "{response}"
        );
    }
}

fn panel_token(address: SocketAddr) -> String {
    let page = get(address, "/");
    let marker = "<meta name=\"painel-token\" content=\"";
    let start_at = page.find(marker).expect("token na página") + marker.len();
    page[start_at..start_at + page[start_at..].find('"').unwrap()].to_string()
}

fn free_question_file(project: &Project, folder: &str, id: &str, extra: &str, body: &str) {
    write(
        project,
        &format!("vostok/work/pedidos/{folder}/{id}.md"),
        &format!("---\nid: {id}\nacao: perguntar-agente\nspec: SPEC-001\ntarefa: \nestado: {folder}\norigem: painel\ncriado_em: 2026-01-01T00:00:00Z\natualizado_em: 2026-01-01T00:00:00Z\n{extra}---\n\n{body}"),
    );
}

fn answered_question(project: &Project, folder: &str, id: &str, extra: &str) {
    free_question_file(
        project,
        folder,
        id,
        extra,
        &format!("## Pedido\n\nPergunta {id}.\n\n## Resultado\n\nResposta {id}.\n"),
    );
}

fn pending_files(project: &Project) -> Vec<String> {
    let dir = project.root().join("vostok/work/pedidos/pendente");
    fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn pergunta_livre_criada_sem_aval_com_e_sem_spec() {
    let project = Project::new();
    let running = start(project.root());
    let token = panel_token(running.address);
    let without_spec = request(
        running.address,
        "POST",
        "/api/pedidos?acao=perguntar-agente",
        &token,
        "Qual o formato?",
    );
    assert_status(&without_spec, "200");
    assert_eq!(json_field(&without_spec, "id"), "PED-001");
    let with_spec = request(
        running.address,
        "POST",
        "/api/pedidos?acao=perguntar-agente&spec=SPEC-001",
        &token,
        "E o filtro?",
    );
    assert_status(&with_spec, "200");
    let first = read(&project, "vostok/work/pedidos/pendente/PED-001.md");
    assert!(
        first.contains("acao: perguntar-agente\nspec: \n"),
        "{first}"
    );
    assert!(first.contains("## Pedido\n\nQual o formato?\n"), "{first}");
    let second = read(&project, "vostok/work/pedidos/pendente/PED-002.md");
    assert!(
        second.contains("acao: perguntar-agente\nspec: SPEC-001\n"),
        "{second}"
    );
    assert!(!project.root().join("vostok/work/avais").exists());
}

#[test]
fn pergunta_livre_ignora_a_tarefa() {
    let project = Project::new();
    let running = start(project.root());
    let token = panel_token(running.address);
    let response = request(
        running.address,
        "POST",
        "/api/pedidos?acao=perguntar-agente&spec=SPEC-001&tarefa=TASK-01",
        &token,
        "Qual o formato?",
    );
    assert_status(&response, "200");
    let content = read(&project, "vostok/work/pedidos/pendente/PED-001.md");
    assert!(content.contains("spec: SPEC-001\ntarefa: \n"), "{content}");
}

#[test]
fn pergunta_livre_sem_texto_e_400() {
    let project = Project::new();
    let running = start(project.root());
    let token = panel_token(running.address);
    for body in ["", "   \n  "] {
        let response = request(
            running.address,
            "POST",
            "/api/pedidos?acao=perguntar-agente",
            &token,
            body,
        );
        assert_status(&response, "400");
    }
    assert!(pending_files(&project).is_empty());
}

#[test]
fn pergunta_livre_lida_grava_uma_vez() {
    let project = Project::new();
    answered_question(&project, "feito", "PED-001", "");
    answered_question(&project, "recusado", "PED-002", "");
    answered_question(
        &project,
        "feito",
        "PED-003",
        "lida_em: 2026-01-01T00:00:00Z\n",
    );
    let running = start(project.root());
    let token = panel_token(running.address);
    for (id, folder) in [("PED-001", "feito"), ("PED-002", "recusado")] {
        let path = format!("vostok/work/pedidos/{folder}/{id}.md");
        let before = read(&project, &path);
        let first = request(
            running.address,
            "POST",
            &format!("/api/pedidos/{id}/lida"),
            &token,
            "",
        );
        assert_status(&first, "200");
        let read_at = json_field(&first, "lida_em");
        assert!(!read_at.is_empty());
        let after = read(&project, &path);
        assert_eq!(after.replace(&format!("lida_em: {read_at}\n"), ""), before);
        assert!(after.contains(&format!("estado: {folder}\n")));
        assert!(after.contains("atualizado_em: 2026-01-01T00:00:00Z\n"));
        let second = request(
            running.address,
            "POST",
            &format!("/api/pedidos/{id}/lida"),
            &token,
            "",
        );
        assert_status(&second, "200");
        assert_eq!(json_field(&second, "lida_em"), read_at);
        assert_eq!(read(&project, &path), after);
    }
    let path = "vostok/work/pedidos/feito/PED-003.md";
    let before = read(&project, path);
    let response = request(
        running.address,
        "POST",
        "/api/pedidos/PED-003/lida",
        &token,
        "",
    );
    assert_status(&response, "200");
    assert_eq!(json_field(&response, "lida_em"), "2026-01-01T00:00:00Z");
    assert_eq!(read(&project, path), before);
}

#[test]
fn pergunta_livre_lida_inexistente_e_404() {
    let project = Project::new();
    let running = start(project.root());
    let token = panel_token(running.address);
    for id in ["PED-999", "PED-abc"] {
        let response = request(
            running.address,
            "POST",
            &format!("/api/pedidos/{id}/lida"),
            &token,
            "",
        );
        assert_status(&response, "404");
    }
}

#[test]
fn pergunta_livre_lida_sem_token_e_403() {
    let project = Project::new();
    answered_question(&project, "feito", "PED-001", "");
    let path = "vostok/work/pedidos/feito/PED-001.md";
    let before = read(&project, path);
    let running = start(project.root());
    let response = request(running.address, "POST", "/api/pedidos/PED-001/lida", "", "");
    assert_status(&response, "403");
    assert_eq!(read(&project, path), before);
}

#[test]
fn pergunta_livre_lida_fora_de_resposta_e_409() {
    let project = Project::new();
    free_question_file(
        &project,
        "pendente",
        "PED-001",
        "",
        "## Pedido\n\nPergunta.\n\n## Resultado\n\n",
    );
    write_request(
        &project,
        "feito",
        "PED-002",
        "planejar-spec",
        "",
        "2026-01-01T00:00:00Z",
        "",
        "Planejada.",
    );
    let running = start(project.root());
    let token = panel_token(running.address);
    for (id, folder) in [("PED-001", "pendente"), ("PED-002", "feito")] {
        let path = format!("vostok/work/pedidos/{folder}/{id}.md");
        let before = read(&project, &path);
        let response = request(
            running.address,
            "POST",
            &format!("/api/pedidos/{id}/lida"),
            &token,
            "",
        );
        assert_status(&response, "409");
        assert_eq!(read(&project, &path), before);
    }
}

#[test]
fn pergunta_livre_lista_os_quatro_estados() {
    let project = Project::new();
    free_question_file(
        &project,
        "pendente",
        "PED-1000",
        "",
        "## Pedido\n\nPrimeira.\n\n## Seção própria\n\nMais texto.\n\n## Histórico\n\n- x\n\n## Resultado\n\n",
    );
    free_question_file(
        &project,
        "em_andamento",
        "PED-999",
        "",
        "## Pedido\n\nSegunda.\n\n## Resultado\n\n",
    );
    free_question_file(
        &project,
        "feito",
        "PED-050",
        "lida_em: 2026-02-02T00:00:00Z\n",
        "## Pedido\n\nTerceira.\n\n## Resultado\n\nCurta.\n\n## Detalhe\n\nLonga.\n",
    );
    answered_question(&project, "recusado", "PED-010", "");
    write_request(
        &project,
        "feito",
        "PED-200",
        "planejar-spec",
        "",
        "2026-01-01T00:00:00Z",
        "",
        "Planejada.",
    );
    let running = start(project.root());
    let response = get(running.address, "/api/perguntas-livres");
    assert_status(&response, "200");
    assert!(!response.contains("PED-200"), "{response}");
    let positions: Vec<usize> = ["PED-1000", "PED-999", "PED-050", "PED-010"]
        .iter()
        .map(|id| {
            response
                .find(&format!("\"id\":\"{id}\""))
                .unwrap_or_else(|| panic!("sem {id}: {response}"))
        })
        .collect();
    assert!(
        positions.windows(2).all(|pair| pair[0] < pair[1]),
        "{response}"
    );
    assert!(
        response.contains("\"texto\":\"Primeira.\\n\\n## Seção própria\\n\\nMais texto.\""),
        "{response}"
    );
    assert!(!response.contains("- x"), "{response}");
    assert!(
        response.contains("\"resultado\":\"Curta.\\n\\n## Detalhe\\n\\nLonga.\""),
        "{response}"
    );
    for field in ["spec", "estado", "atualizado_em", "lida_em"] {
        assert!(response.contains(&format!("\"{field}\":")), "{field}");
    }
    assert!(response.contains("\"lida_em\":\"2026-02-02T00:00:00Z\""));
}

#[test]
fn pergunta_livre_nao_lida_na_visao() {
    let empty = overview(&Project::new());
    assert!(empty.contains("\"respostas_nao_lidas\":[]"), "{empty}");

    let project = Project::new();
    answered_question(&project, "feito", "PED-001", "");
    answered_question(&project, "recusado", "PED-002", "");
    answered_question(
        &project,
        "feito",
        "PED-003",
        "lida_em: 2026-01-01T00:00:00Z\n",
    );
    free_question_file(
        &project,
        "pendente",
        "PED-004",
        "",
        "## Pedido\n\nPergunta.\n\n## Resultado\n\n",
    );
    write_request(
        &project,
        "feito",
        "PED-005",
        "planejar-spec",
        "",
        "2026-01-01T00:00:00Z",
        "",
        "Planejada.",
    );
    let response = overview(&project);
    let start_at = response.find("\"respostas_nao_lidas\":[").expect(&response);
    let unread = &response[start_at..];
    assert!(
        unread
            .contains("{\"id\":\"PED-001\",\"spec\":\"SPEC-001\",\"texto\":\"Pergunta PED-001.\"}"),
        "{unread}"
    );
    assert!(
        unread
            .contains("{\"id\":\"PED-002\",\"spec\":\"SPEC-001\",\"texto\":\"Pergunta PED-002.\"}"),
        "{unread}"
    );
    for id in ["PED-003", "PED-004", "PED-005"] {
        assert!(!unread.contains(id), "{id}: {unread}");
    }
}

#[test]
fn pergunta_livre_nao_troca_o_desfecho_da_spec() {
    let project = pending_task();
    write(
        &project,
        "vostok/work/specs/refined/SPEC-001-plano.md",
        "---\nid: SPEC-001\ntitle: \"Plano\"\nstatus: planned\ntype: fix\napproved_hash: abc\n---\n\n# Plano\n",
    );
    write_request(
        &project,
        "feito",
        "PED-001",
        "ajustar-spec",
        "",
        "2026-01-01T00:00:00Z",
        "desfecho: pausado\nsaidas: [criar-spec]\n",
        "Falta a spec de login.",
    );
    answered_question(&project, "feito", "PED-002", "");
    let response = overview(&project);
    assert!(
        response.contains("\"last_outcome\":{\"id\":\"PED-001\""),
        "{response}"
    );
    assert!(
        !response.contains("\"last_outcome\":{\"id\":\"PED-002\""),
        "{response}"
    );
}

fn start_request(project: &Project, id: &str) {
    write_request(
        project,
        "pendente",
        id,
        "executar-tarefa",
        "TASK-01",
        "2026-01-01T00:00:00Z",
        "",
        "",
    );
    project
        .vostok()
        .args(["pedidos", "iniciar", id])
        .assert()
        .success();
}

fn note(project: &Project, id: &str, text: &str, detail: Option<&str>) {
    let mut command = project.vostok();
    command.args(["pedidos", "nota", id, text]);
    if let Some(detail) = detail {
        command.args(["--detalhe", detail]);
    }
    command.assert().success();
}

fn request_detail(project: &Project, id: &str) -> String {
    let running = start(project.root());
    get(running.address, &format!("/api/pedidos/{id}"))
}

#[test]
fn ct05_pedido_traz_as_notas_em_ordem_com_o_detalhe() {
    let project = Project::new();
    start_request(&project, "PED-001");
    note(&project, "PED-001", "lendo a spec", None);
    note(
        &project,
        "PED-001",
        "rodando os testes",
        Some("Faltam dois testes.\n## Resultado\nEste título fica no detalhe."),
    );
    let response = request_detail(&project, "PED-001");
    assert_status(&response, "200");
    for field in [
        "\"id\":\"PED-001\"",
        "\"acao\":\"executar-tarefa\"",
        "\"spec\":\"SPEC-001\"",
        "\"tarefa\":\"TASK-01\"",
        "\"estado\":\"em_andamento\"",
        "\"texto\":\"Pedido PED-001.\"",
        "\"resultado\":\"\"",
        "\"lida_em\":\"\"",
    ] {
        assert!(response.contains(field), "{field}: {response}");
    }
    let first = response
        .find("\"texto\":\"lendo a spec\",\"detalhe\":\"\"")
        .expect(&response);
    let second = response
        .find("\"texto\":\"rodando os testes\",\"detalhe\":\"Faltam dois testes.\\n## Resultado\\nEste título fica no detalhe.\"")
        .expect(&response);
    assert!(first < second, "{response}");
}

#[test]
fn ct05_resultado_fica_fora_das_notas_mesmo_com_titulo_e_cerca_no_detalhe() {
    let project = Project::new();
    start_request(&project, "PED-001");
    note(
        &project,
        "PED-001",
        "com cerca",
        Some("antes\n## Resultado\n```rust\nlet x = 1;\n```\ndepois"),
    );
    project
        .vostok()
        .args(["pedidos", "concluir", "PED-001"])
        .write_stdin("Feito.")
        .assert()
        .success();
    let response = request_detail(&project, "PED-001");
    assert_status(&response, "200");
    assert!(response.contains("\"resultado\":\"Feito.\""), "{response}");
    assert!(
        response
            .contains("\"detalhe\":\"antes\\n## Resultado\\n```rust\\nlet x = 1;\\n```\\ndepois\""),
        "{response}"
    );
    assert!(!response.contains("\\```"), "{response}");
}

#[test]
fn pedido_antigo_sem_nota_nem_lida_em_e_lido_sem_falhar() {
    let project = Project::new();
    write_request(
        &project,
        "em_andamento",
        "PED-001",
        "executar-tarefa",
        "TASK-01",
        "2026-01-01T00:00:00Z",
        "",
        "",
    );
    let running = start(project.root());
    let response = get(running.address, "/api/pedidos/PED-001");
    assert_status(&response, "200");
    assert!(response.contains("\"notas\":[]"), "{response}");
    assert!(response.contains("\"lida_em\":\"\""), "{response}");
    let overview = get(running.address, "/api/visao");
    assert!(overview.contains("\"id\":\"PED-001\""), "{overview}");
    assert!(!overview.contains("\"nota\":"), "{overview}");
}

#[test]
fn pedido_inexistente_ou_com_id_invalido_e_404() {
    let project = Project::new();
    start_request(&project, "PED-001");
    let running = start(project.root());
    for id in ["PED-999", "..", "PED-001%2F.."] {
        let response = get(running.address, &format!("/api/pedidos/{id}"));
        assert_status(&response, "404");
    }
}

#[test]
fn visao_traz_a_nota_e_a_hora_do_pedido() {
    let project = Project::new();
    start_request(&project, "PED-001");
    note(&project, "PED-001", "lendo a spec", None);
    let response = overview(&project);
    assert!(response.contains("\"nota\":\"lendo a spec\""), "{response}");
    assert!(response.contains("\"nota_em\":\"20"), "{response}");
}

#[test]
fn loop_traz_o_aviso_de_falta_de_nota() {
    let project = Project::new();
    write(
        &project,
        "vostok/work/pedidos/.loop",
        "ultima_rodada: 2026-11-03T10:00:00Z\nintervalo: 90\nagente: codex\nsem_nota_pedido: PED-001\nsem_nota_desde: 2026-11-03T09:00:00Z\n",
    );
    let response = overview(&project);
    let expected = "\"agente\":\"codex\",\"sem_nota_pedido\":\"PED-001\",\"sem_nota_desde\":\"2026-11-03T09:00:00Z\"}";
    assert!(response.contains(expected), "{response}");
}

const MANUAL_EVIDENCE: &str = "### Evidências\n\n- `video.mov` (manual): o vídeo\n";

/// Troca o status da tarefa em pasta e faz dela uma tarefa que pede o anexo manual `video.mov`.
fn manual_evidence_task(status: &str) -> Project {
    let project = pending_task();
    let state = format!("{FOLDER}/estado.yml");
    let changed = read(&project, &state).replace("status: pending", &format!("status: {status}"));
    write(&project, &state, &changed);
    let verification = format!("{FOLDER}/verificacao.md");
    let changed = read(&project, &verification)
        .replace("### Evidências\n\nNão se aplica.\n", MANUAL_EVIDENCE);
    write(&project, &verification, &changed);
    project
}

fn set_attach_limit(project: &Project, limit: Option<&str>) {
    let config = "vostok/config/tasks.yml";
    let kept: String = read(project, config)
        .lines()
        .filter(|line| !line.starts_with("manual_attachment_max_mb"))
        .map(|line| format!("{line}\n"))
        .collect();
    let line = limit
        .map(|value| format!("manual_attachment_max_mb: {value}\n"))
        .unwrap_or_default();
    write(project, config, &format!("{kept}{line}"));
}

#[test]
fn evidence_upload_ct39_live_version_ignores_the_temp_folder() {
    let project = manual_evidence_task("validating");
    let leftover = format!("{FOLDER}/evidencias/.anexo-tmp/velho");
    write(&project, &leftover, "sobra");
    let painel = start(project.root());
    let token = panel_token(painel.address);
    let mut live = open_live(painel.address);

    let mut upload = TcpStream::connect(painel.address).unwrap();
    upload
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    upload
        .set_write_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let head = format!(
        "PUT /api/evidencias?spec=SPEC-001&tarefa=TASK-01&evidencia=video.mov&extensao=mov HTTP/1.1\r\nHost: x\r\nX-Painel-Token: {token}\r\nContent-Length: 200000\r\nConnection: close\r\n\r\n"
    );
    upload.write_all(head.as_bytes()).unwrap();
    upload.write_all(&[7u8; 100_000]).unwrap();

    let temp = project.root().join(&leftover);
    let deadline = Instant::now() + Duration::from_secs(5);
    while temp.exists() {
        assert!(Instant::now() < deadline, "a sobra não sumiu");
        std::thread::sleep(Duration::from_millis(20));
    }
    live.set_read_timeout(Some(Duration::from_millis(1500)))
        .unwrap();
    let mut frame = [0u8; 2];
    assert!(
        live.read_exact(&mut frame).is_err(),
        "a escrita no temporário avisou a mudança"
    );

    upload.write_all(&[7u8; 100_000]).unwrap();
    let mut response = String::new();
    let mut buffer = [0u8; 1024];
    while !response.contains("\"sha256\"") {
        let count = upload.read(&mut buffer).unwrap();
        assert!(count > 0, "resposta incompleta: {response}");
        response.push_str(&String::from_utf8_lossy(&buffer[..count]));
    }
    assert_status(&response, "200");

    live.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    assert_eq!(next_message(&mut live), "mudou");
}

#[test]
fn task_detail_has_the_manual_evidence_and_the_limit() {
    let project = manual_evidence_task("in_progress");
    set_attach_limit(&project, None);
    let painel = start(project.root());
    let target = "/api/tarefas/SPEC-001/TASK-01";

    let missing = get(painel.address, target);
    assert!(missing.contains("\"attach_limit_mb\":500"), "{missing}");
    for field in [
        "\"name\":\"video.mov\"",
        "\"description\":\"o vídeo\"",
        "\"extension\":\"mov\"",
        "\"attached\":false",
        "\"size\":0",
        &format!("\"path\":\"{FOLDER}/evidencias/video.mov\""),
    ] {
        assert!(missing.contains(field), "{field}: {missing}");
    }

    write(&project, &format!("{FOLDER}/evidencias/video.mov"), "12345");
    let attached = get(painel.address, target);
    assert!(attached.contains("\"attached\":true"), "{attached}");
    assert!(attached.contains("\"size\":5"), "{attached}");

    for (value, expected) in [("1", 1), ("0", 500), ("muito", 500)] {
        set_attach_limit(&project, Some(value));
        let response = get(painel.address, target);
        assert!(
            response.contains(&format!("\"attach_limit_mb\":{expected}")),
            "{value}: {response}"
        );
    }
}

#[test]
fn evidence_upload_archived_task_has_no_manual_evidence() {
    let project = manual_evidence_task("done");
    let archived = project.root().join("vostok/work/.archive/SPEC-001/tasks");
    fs::create_dir_all(&archived).unwrap();
    fs::rename(project.root().join(FOLDER), archived.join("TASK-01")).unwrap();
    let painel = start(project.root());

    let response = get(painel.address, "/api/tarefas/SPEC-001/TASK-01");

    assert_status(&response, "200");
    assert!(response.contains("\"archived\":true"), "{response}");
    assert!(response.contains("\"manual_evidence\":[]"), "{response}");
}

fn notification_request(project: &Project, folder: &str, id: &str, updated: &str, extra: &str) {
    write_request(
        project,
        folder,
        id,
        "executar-tarefa",
        "TASK-01",
        updated,
        extra,
        "ok",
    );
}

fn count_of(response: &str, needle: &str) -> usize {
    response.matches(needle).count()
}

fn token_post(address: SocketAddr, target: &str) -> String {
    request(address, "POST", target, &panel_token(address), "")
}

fn unread_notifications(project: &Project, count: usize) {
    for index in 0..count {
        notification_request(
            project,
            "feito",
            &format!("PED-{}", 100 + index),
            &format!("2026-03-01T10:{index:02}:00Z"),
            "notificacao: nao_lida\n",
        );
    }
}

fn request_field(project: &Project, id: &str, field: &str) -> String {
    let content = read(project, &format!("vostok/work/pedidos/feito/{id}.md"));
    content
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{field}: ")))
        .unwrap_or_default()
        .to_string()
}

#[test]
fn notificacoes_lista_e_contagem() {
    let project = Project::new();
    unread_notifications(&project, 60);
    notification_request(&project, "feito", "PED-001", "2026-01-01T00:00:00Z", "");
    notification_request(&project, "pendente", "PED-002", "2026-04-01T00:00:00Z", "");
    let running = start(project.root());

    let list = get(running.address, "/api/notificacoes");

    assert_status(&list, "200");
    assert!(list.contains("\"nao_lidas\":60"), "{list}");
    assert_eq!(count_of(&list, "\"acao\":"), 50, "{list}");
    let first = list.find("\"id\":\"PED-159\"").expect("o mais recente");
    let second = list.find("\"id\":\"PED-158\"").expect("o seguinte");
    assert!(first < second, "{list}");
    assert!(list.contains("\"id\":\"PED-110\""), "{list}");
    assert!(!list.contains("\"id\":\"PED-109\""), "{list}");
    assert!(!list.contains("\"id\":\"PED-001\""), "{list}");
    assert!(!list.contains("\"id\":\"PED-002\""), "{list}");
    assert!(list.contains("\"lida\":false"), "{list}");
    let overview = get(running.address, "/api/visao");
    assert!(
        overview.contains("\"notificacoes_nao_lidas\":60"),
        "{overview}"
    );
}

#[test]
fn notificacoes_vazia_sem_pedidos() {
    let project = Project::new();
    let running = start(project.root());

    let list = get(running.address, "/api/notificacoes");

    assert_status(&list, "200");
    assert!(list.ends_with("{\"nao_lidas\":0,\"itens\":[]}"), "{list}");
}

#[test]
fn notificacoes_marcar_por_ids() {
    let project = Project::new();
    unread_notifications(&project, 3);
    let running = start(project.root());

    let reply = token_post(
        running.address,
        "/api/notificacoes/lidas?ids=PED-101,%20PED-102",
    );

    assert_status(&reply, "200");
    assert!(reply.ends_with("{\"marcadas\":2}"), "{reply}");
    assert_eq!(request_field(&project, "PED-101", "notificacao"), "lida");
    assert_eq!(request_field(&project, "PED-102", "notificacao"), "lida");
    assert!(!request_field(&project, "PED-101", "notificacao_lida_em").is_empty());
    assert_eq!(
        request_field(&project, "PED-100", "notificacao"),
        "nao_lida"
    );
    let list = get(running.address, "/api/notificacoes");
    assert!(list.contains("\"nao_lidas\":1"), "{list}");
}

#[test]
fn notificacoes_marcar_todas_alem_das_50() {
    let project = Project::new();
    unread_notifications(&project, 60);
    let running = start(project.root());

    let reply = token_post(running.address, "/api/notificacoes/lidas?todas=sim");

    assert_status(&reply, "200");
    assert!(reply.ends_with("{\"marcadas\":60}"), "{reply}");
    assert_eq!(request_field(&project, "PED-100", "notificacao"), "lida");
    let overview = get(running.address, "/api/visao");
    assert!(
        overview.contains("\"notificacoes_nao_lidas\":0"),
        "{overview}"
    );
}

#[test]
fn notificacoes_marcar_ignora_e_nao_conta() {
    let project = Project::new();
    notification_request(
        &project,
        "feito",
        "PED-001",
        "2026-03-01T10:00:00Z",
        "notificacao: lida\nnotificacao_lida_em: 2026-03-02T00:00:00Z\n",
    );
    notification_request(
        &project,
        "pendente",
        "PED-002",
        "2026-03-01T10:00:00Z",
        "notificacao: nao_lida\n",
    );
    notification_request(
        &project,
        "em_andamento",
        "PED-003",
        "2026-03-01T10:00:00Z",
        "notificacao: nao_lida\n",
    );
    notification_request(
        &project,
        "feito",
        "PED-005",
        "2026-03-01T10:00:00Z",
        "notificacao: nao_lida\n",
    );
    let running = start(project.root());

    let reply = token_post(
        running.address,
        "/api/notificacoes/lidas?ids=PED-001,PED-002,PED-003,PED-004,PED-005",
    );

    assert_status(&reply, "200");
    assert!(reply.ends_with("{\"marcadas\":1}"), "{reply}");
    assert_eq!(
        request_field(&project, "PED-001", "notificacao_lida_em"),
        "2026-03-02T00:00:00Z"
    );
    assert!(
        read(&project, "vostok/work/pedidos/pendente/PED-002.md").contains("notificacao: nao_lida")
    );
    assert!(
        read(&project, "vostok/work/pedidos/em_andamento/PED-003.md")
            .contains("notificacao: nao_lida")
    );
    assert_eq!(request_field(&project, "PED-005", "notificacao"), "lida");
}

#[test]
fn notificacoes_marcar_nao_muda_estado_nem_atualizado_em() {
    let project = Project::new();
    unread_notifications(&project, 1);
    let running = start(project.root());

    token_post(running.address, "/api/notificacoes/lidas?ids=PED-100");

    assert_eq!(request_field(&project, "PED-100", "estado"), "feito");
    assert_eq!(
        request_field(&project, "PED-100", "atualizado_em"),
        "2026-03-01T10:00:00Z"
    );
}

#[test]
fn notificacoes_marcar_id_invalido_e_sem_parametro_400() {
    let project = Project::new();
    unread_notifications(&project, 1);
    let running = start(project.root());

    for target in [
        "/api/notificacoes/lidas?ids=../x",
        "/api/notificacoes/lidas?ids=PED-100,x",
        "/api/notificacoes/lidas",
        "/api/notificacoes/lidas?todas=nao",
    ] {
        let reply = token_post(running.address, target);
        assert_status(&reply, "400");
        assert_eq!(
            request_field(&project, "PED-100", "notificacao"),
            "nao_lida",
            "{target}"
        );
    }
}

#[test]
fn notificacoes_marcar_sem_token_403() {
    let project = Project::new();
    unread_notifications(&project, 1);
    let running = start(project.root());
    let target = "/api/notificacoes/lidas?todas=sim";

    let without = request(running.address, "POST", target, "", "");
    let wrong = request(running.address, "POST", target, "errado", "");

    assert_status(&without, "403");
    assert_status(&wrong, "403");
    assert_eq!(
        request_field(&project, "PED-100", "notificacao"),
        "nao_lida"
    );
}

fn toast_seconds_with(line: Option<&str>) -> String {
    let project = Project::new();
    let main = project.root().join("vostok/config/main.yml");
    let kept: Vec<String> = fs::read_to_string(&main)
        .unwrap()
        .lines()
        .filter(|row| !row.starts_with("toast_dismiss_seconds"))
        .map(str::to_string)
        .chain(line.map(str::to_string))
        .collect();
    fs::write(&main, kept.join("\n") + "\n").unwrap();
    let overview = overview(&project);
    let marker = "\"toast_dismiss_seconds\":";
    let start_at = overview.find(marker).expect("tempo do aviso") + marker.len();
    overview[start_at..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect()
}

#[test]
fn toast_dismiss_seconds_na_visao() {
    assert_eq!(toast_seconds_with(Some("toast_dismiss_seconds: 10")), "10");
    assert_eq!(toast_seconds_with(Some("toast_dismiss_seconds: 0")), "0");
    assert_eq!(toast_seconds_with(None), "5");
    assert_eq!(toast_seconds_with(Some("toast_dismiss_seconds: -1")), "5");
    assert_eq!(toast_seconds_with(Some("toast_dismiss_seconds: abc")), "5");
}

#[test]
fn toast_dismiss_seconds_gravado_pela_configuracao() {
    let project = Project::new();
    let running = start(project.root());
    let path = "vostok/config/main.yml";
    let source = get(running.address, &format!("/api/fonte?caminho={path}"));
    let hash = json_field(&source, "hash");
    let edited =
        read(&project, path).replace("toast_dismiss_seconds: 5", "toast_dismiss_seconds: 10");

    let saved = request(
        running.address,
        "PUT",
        &format!("/api/configuracao?caminho={path}&hash={hash}"),
        &panel_token(running.address),
        &edited,
    );

    assert_status(&saved, "200");
    let overview = get(running.address, "/api/visao");
    assert!(
        overview.contains("\"toast_dismiss_seconds\":10"),
        "{overview}"
    );
}

fn put_draft(project: &Project, draft: &str, name: &str, content: &str) {
    write(
        project,
        &format!("vostok/work/pedidos/anexos/.rascunho/{draft}/{name}"),
        content,
    );
}

#[test]
fn ct63_limites_trazem_a_lista_de_nomes() {
    let project = Project::new();
    let running = start(project.root());
    let response = get(running.address, "/api/pedidos/anexos/limites");
    assert_status(&response, "200");
    assert!(response.contains("\"max_mb\":500"), "{response}");
    assert!(response.contains("\"max_arquivos\":10"), "{response}");
    let (refused, accepted) = response
        .split_once("\"nomes_aceitos\"")
        .unwrap_or_else(|| panic!("sem nomes_aceitos: {response}"));
    assert!(refused.contains("\"nomes_recusados\""), "{response}");
    assert!(refused.contains("\".env\""), "{response}");
    assert!(accepted.contains("\".env.example\""), "{response}");
    assert!(!refused.contains("\".env.example\""), "{response}");
}

#[test]
fn ct64_lista_de_perguntas_traz_so_o_que_esta_na_pasta() {
    let project = Project::new();
    put_draft(&project, "r1", "01-a.txt", "abc");
    put_draft(&project, "r1", "02-b.txt", "abcde");
    let running = start(project.root());
    let token = panel_token(running.address);
    let created = request(
        running.address,
        "POST",
        "/api/pedidos?acao=perguntar-agente&rascunho=r1",
        &token,
        "Qual o formato?",
    );
    assert_status(&created, "200");
    let listed = get(running.address, "/api/perguntas-livres");
    assert!(listed.contains("\"nome\":\"01-a.txt\""), "{listed}");
    assert!(
        listed.contains("\"caminho\":\"vostok/work/pedidos/anexos/PED-001/02-b.txt\""),
        "{listed}"
    );
    assert!(listed.contains("\"tamanho\":5"), "{listed}");
    assert!(listed.contains("\"texto\":\"Qual o formato?\""), "{listed}");

    fs::remove_file(
        project
            .root()
            .join("vostok/work/pedidos/anexos/PED-001/01-a.txt"),
    )
    .unwrap();
    let again = get(running.address, "/api/perguntas-livres");
    assert!(!again.contains("01-a.txt"), "{again}");
    assert!(again.contains("02-b.txt"), "{again}");
}

#[test]
fn ct64_listar_funciona_com_a_pasta_de_anexos() {
    let project = Project::new();
    write_request(
        &project,
        "pendente",
        "PED-001",
        "ajustar-tarefa",
        "TASK-01",
        "2026-01-01T00:00:00Z",
        "",
        "",
    );
    let list = || {
        let output = project
            .vostok()
            .args(["pedidos", "listar"])
            .output()
            .unwrap();
        assert!(output.status.success());
        String::from_utf8_lossy(&output.stdout).into_owned()
    };
    let without = list();
    put_draft(&project, "r1", "01-a.txt", "abc");
    write(
        &project,
        "vostok/work/pedidos/anexos/PED-001/01-a.txt",
        "abc",
    );
    assert_eq!(list(), without);
    assert!(without.contains("PED-001"), "{without}");
}

#[test]
fn lida_marca_tambem_a_notificacao() {
    let project = Project::new();
    answered_question(&project, "feito", "PED-001", "notificacao: nao_lida\n");
    let running = start(project.root());

    let reply = token_post(running.address, "/api/pedidos/PED-001/lida");

    assert_status(&reply, "200");
    assert!(!request_field(&project, "PED-001", "lida_em").is_empty());
    assert_eq!(request_field(&project, "PED-001", "notificacao"), "lida");
    assert!(!request_field(&project, "PED-001", "notificacao_lida_em").is_empty());
}

#[test]
fn lida_sem_notificacao_nao_cria_o_campo() {
    let project = Project::new();
    answered_question(&project, "feito", "PED-001", "");
    let running = start(project.root());

    let reply = token_post(running.address, "/api/pedidos/PED-001/lida");

    assert_status(&reply, "200");
    let content = read(&project, "vostok/work/pedidos/feito/PED-001.md");
    assert!(!content.contains("notificacao"), "{content}");
}

#[test]
fn marcar_notificacao_nao_grava_lida_em() {
    let project = Project::new();
    answered_question(&project, "feito", "PED-001", "notificacao: nao_lida\n");
    let running = start(project.root());

    let reply = token_post(running.address, "/api/notificacoes/lidas?ids=PED-001");

    assert_status(&reply, "200");
    assert_eq!(request_field(&project, "PED-001", "notificacao"), "lida");
    assert!(request_field(&project, "PED-001", "lida_em").is_empty());
}
