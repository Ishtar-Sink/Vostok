//! Testes do índice de evidências reaproveitáveis (SPEC-075).

mod common;

use std::fs;
use std::path::PathBuf;

use common::{failure_stderr, Project};

use tempfile::TempDir;
use vostok::evidence_reuse::{command_hash, reuse_key, Entry, ReuseIndex, INDEX_DIR};
use vostok::support::sha256_hex;

fn entry(key: &str, command: &str, content: &[u8], kind: &str) -> Entry {
    Entry {
        key: key.into(),
        command_hash: command_hash(command),
        depends: vec!["assets/fase1/".into()],
        inputs: "entradas".into(),
        content_sha256: sha256_hex(content),
        kind: kind.into(),
        exit_code: 0,
        spec: "SPEC-001".into(),
        task: "TASK-01".into(),
        evidence_id: "EV-01".into(),
        date: "2026-11-19T05:00:00Z".into(),
    }
}

fn index_text(work: &TempDir) -> String {
    fs::read_to_string(work.path().join(INDEX_DIR).join("index.yml")).unwrap()
}

fn index_json(work: &TempDir) -> serde_json::Value {
    serde_json::from_str::<serde_json::Value>(&index_text(work)).unwrap()
}

fn object_path(work: &TempDir, content: &[u8]) -> std::path::PathBuf {
    work.path()
        .join(INDEX_DIR)
        .join("objects")
        .join(sha256_hex(content))
}

fn object_count(work: &TempDir) -> usize {
    fs::read_dir(work.path().join(INDEX_DIR).join("objects"))
        .unwrap()
        .count()
}

#[test]
fn indice_hash_do_comando() {
    assert_eq!(
        command_hash("abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        command_hash(""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn indice_chave_deterministica() {
    assert_eq!(reuse_key("cmd", "x"), reuse_key("cmd", "x"));
    assert_eq!(reuse_key("cmd", "x"), sha256_hex(b"cmd\0x"));
    assert_ne!(reuse_key("cmd", "x"), reuse_key("cmd", "y"));
    assert_ne!(reuse_key("cmd", "x"), reuse_key("cmd2", "x"));
    assert_ne!(reuse_key("ab", ""), reuse_key("a", "b"));
}

#[test]
fn indice_grava_e_le() {
    let work = TempDir::new().unwrap();
    let index = ReuseIndex::at(work.path());
    let gravada = entry("k", "cargo test", b"saida\n", "log");
    index.record(gravada.clone(), b"saida\n").unwrap();

    let achada = index.lookup("k").unwrap();
    assert_eq!(achada, Some((gravada, b"saida\n".to_vec())));
    assert_eq!(INDEX_DIR, "evidence-index");
    assert!(work.path().join("evidence-index/index.yml").is_file());
    assert!(object_path(&work, b"saida\n").is_file());
}

#[test]
fn indice_sem_indice_nao_acha() {
    let work = TempDir::new().unwrap();
    let index = ReuseIndex::at(work.path());
    assert_eq!(index.lookup("x").unwrap(), None);
    assert_eq!(index.by_command("x").unwrap(), vec![]);
    assert!(!work.path().join(INDEX_DIR).exists());
}

#[test]
fn indice_chave_ausente_nao_acha() {
    let work = TempDir::new().unwrap();
    let index = ReuseIndex::at(work.path());
    index
        .record(entry("k", "cargo test", b"um", "log"), b"um")
        .unwrap();
    assert_eq!(index.lookup("outra").unwrap(), None);
}

#[test]
fn indice_objeto_alterado_nao_vale() {
    let work = TempDir::new().unwrap();
    let index = ReuseIndex::at(work.path());
    index
        .record(entry("k", "cargo test", b"um", "log"), b"um")
        .unwrap();
    fs::write(object_path(&work, b"um"), b"outro").unwrap();
    assert_eq!(index.lookup("k").unwrap(), None);
}

#[test]
fn indice_objeto_apagado_nao_vale() {
    let work = TempDir::new().unwrap();
    let index = ReuseIndex::at(work.path());
    index
        .record(entry("k", "cargo test", b"um", "log"), b"um")
        .unwrap();
    fs::remove_file(object_path(&work, b"um")).unwrap();
    assert_eq!(index.lookup("k").unwrap(), None);
}

#[test]
fn indice_record_regrava_objeto_corrompido() {
    let work = TempDir::new().unwrap();
    let index = ReuseIndex::at(work.path());
    let gravada = entry("k", "cargo test", b"um", "log");
    index.record(gravada.clone(), b"um").unwrap();
    fs::write(object_path(&work, b"um"), b"outro").unwrap();
    index.record(gravada.clone(), b"um").unwrap();
    assert_eq!(index.lookup("k").unwrap(), Some((gravada, b"um".to_vec())));
}

#[test]
fn indice_substitui_entrada_da_chave() {
    let work = TempDir::new().unwrap();
    let index = ReuseIndex::at(work.path());
    index
        .record(entry("k", "cargo test", b"um", "log"), b"um")
        .unwrap();
    let mut segunda = entry("k", "cargo test", b"dois", "log");
    segunda.evidence_id = "EV-02".into();
    index.record(segunda, b"dois").unwrap();

    let (achada, conteudo) = index.lookup("k").unwrap().unwrap();
    assert_eq!(achada.evidence_id, "EV-02");
    assert_eq!(conteudo, b"dois".to_vec());
    assert_eq!(index_json(&work)["entries"].as_array().unwrap().len(), 1);
}

#[test]
fn indice_conteudo_igual_guardado_uma_vez() {
    let work = TempDir::new().unwrap();
    let index = ReuseIndex::at(work.path());
    index
        .record(entry("a", "cargo test", b"igual", "log"), b"igual")
        .unwrap();
    index
        .record(entry("b", "cargo build", b"igual", "log"), b"igual")
        .unwrap();
    assert_eq!(object_count(&work), 1);
}

#[test]
fn indice_por_comando() {
    let work = TempDir::new().unwrap();
    let index = ReuseIndex::at(work.path());
    index
        .record(entry("b", "print.sh fase1", b"b", "print"), b"b")
        .unwrap();
    index
        .record(entry("a", "print.sh fase1", b"a", "print"), b"a")
        .unwrap();
    index
        .record(entry("c", "print.sh fase2", b"c", "print"), b"c")
        .unwrap();

    let chaves: Vec<String> = index
        .by_command(&command_hash("print.sh fase1"))
        .unwrap()
        .into_iter()
        .map(|achada| achada.key)
        .collect();
    assert_eq!(chaves, vec!["a".to_string(), "b".to_string()]);
}

#[test]
fn indice_ordem_pela_chave() {
    let primeira = TempDir::new().unwrap();
    let segunda = TempDir::new().unwrap();
    let um = ReuseIndex::at(primeira.path());
    um.record(entry("b", "cmd", b"b", "log"), b"b").unwrap();
    um.record(entry("a", "cmd", b"a", "log"), b"a").unwrap();
    let dois = ReuseIndex::at(segunda.path());
    dois.record(entry("a", "cmd", b"a", "log"), b"a").unwrap();
    dois.record(entry("b", "cmd", b"b", "log"), b"b").unwrap();

    assert_eq!(index_text(&primeira), index_text(&segunda));
    assert_eq!(index_json(&primeira)["entries"][0]["key"], "a");
}

#[test]
fn indice_recusa_sha_diferente() {
    let work = TempDir::new().unwrap();
    let index = ReuseIndex::at(work.path());
    let erro = index
        .record(entry("k", "cargo test", b"um", "log"), b"dois")
        .unwrap_err();
    assert!(erro.to_string().contains("não confere"), "{erro}");
    assert!(!work.path().join(INDEX_DIR).exists());
}

#[test]
fn indice_recusa_tipo_invalido() {
    let work = TempDir::new().unwrap();
    let index = ReuseIndex::at(work.path());
    let erro = index
        .record(entry("k", "git diff", b"um", "diff"), b"um")
        .unwrap_err();
    assert!(
        erro.to_string()
            .contains("tipo de evidência inválido: diff"),
        "{erro}"
    );
}

#[test]
fn indice_invalido_e_erro() {
    let work = TempDir::new().unwrap();
    let dir = work.path().join(INDEX_DIR);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("index.yml"), "não é json").unwrap();

    let erro = ReuseIndex::at(work.path()).lookup("x").unwrap_err();
    let mensagem = format!("{erro:#}");
    assert!(
        mensagem.contains("índice de reaproveitamento inválido"),
        "{mensagem}"
    );
    assert!(mensagem.contains("index.yml"), "{mensagem}");
}

#[test]
fn indice_formato_do_arquivo() {
    let work = TempDir::new().unwrap();
    let index = ReuseIndex::at(work.path());
    let print_command = "scripts/print.sh fase1 fase1.png";
    index
        .record(
            entry(
                &reuse_key(print_command, "entradas-a"),
                print_command,
                b"png",
                "print",
            ),
            b"png",
        )
        .unwrap();
    let mut log = entry(
        &reuse_key("cargo test", "entradas-b"),
        "cargo test",
        b"ok\n",
        "log",
    );
    log.evidence_id = "EV-02".into();
    index.record(log, b"ok\n").unwrap();

    println!("{}", index_text(&work));
    let objetos = object_count(&work);
    println!("objetos={objetos}");
    assert_eq!(objetos, 2);
}

// --- Reaproveitamento no fluxo das tarefas (SPEC-075 TASK-03) ---

const REAPROVEITA_LOG: &str = "- `saida.txt` (automática): `echo x >> rodadas.txt; cat valor.txt`\n  - Título: O valor é lido\n  - Prova: o valor é lido.\n  - Depende: `valor.txt`\n  - Trecho: a linha `a`.\n  - Falha: se aparecer `ausente`, o arquivo sumiu.";

const REAPROVEITA_SEM_DEPENDE: &str = "- `saida.txt` (automática): `cat valor.txt`\n  - Título: O valor é lido\n  - Prova: o valor é lido.\n  - Trecho: a linha `a`.\n  - Falha: se aparecer `ausente`, o arquivo sumiu.";

const REAPROVEITA_PRINT: &str = "- `tela.png` (print): `echo x >> rodadas.txt; printf 'PNG1' > \"$VOSTOK_EVIDENCE/tela.png\"`\n  - Título: A tela da fase\n  - Prova: a tela aparece.\n  - Depende: `assets/fase1/`\n  - Trecho: a imagem.\n  - Falha: se aparecer `tela vazia`, quebrou.\n- `fase.webm` (print): `echo x >> rodadas.txt; printf 'WEBM1' > \"$VOSTOK_EVIDENCE/fase.webm\"`\n  - Título: O vídeo da fase\n  - Prova: o vídeo roda.\n  - Depende: `assets/fase1/`\n  - Trecho: o vídeo.\n  - Falha: se aparecer `vídeo vazio`, quebrou.";

fn reaproveita_ler(project: &Project, relative: &str) -> String {
    fs::read_to_string(project.root().join(relative)).unwrap_or_default()
}

fn reaproveita_gravar(project: &Project, relative: &str, content: &str) {
    let path = project.root().join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn reaproveita_projeto() -> Project {
    let project = Project::new();
    reaproveita_gravar(&project, "valor.txt", "a\n");
    reaproveita_gravar(&project, "assets/fase1/cor.txt", "azul\n");
    reaproveita_gravar(&project, "fora.txt", "1\n");
    let config: String = reaproveita_ler(&project, "vostok/config/tasks.yml")
        .lines()
        .filter(|line| !line.starts_with("evidence_baseline_dir:"))
        .map(|line| format!("{line}\n"))
        .collect();
    reaproveita_gravar(&project, "vostok/config/tasks.yml", &config);
    project
}

fn reaproveita_config(project: &Project, key: &str, value: &str) {
    let path = "vostok/config/tasks.yml";
    let current = reaproveita_ler(project, path);
    let prefix = format!("{key}:");
    let mut found = false;
    let mut content: String = current
        .lines()
        .map(|line| {
            if line.starts_with(&prefix) {
                found = true;
                format!("{key}: {value}\n")
            } else {
                format!("{line}\n")
            }
        })
        .collect();
    if !found {
        content.push_str(&format!("{key}: {value}\n"));
    }
    reaproveita_gravar(project, path, &content);
}

fn reaproveita_tarefa(project: &Project, spec: &str, task: &str, evidencias: &str, status: &str) {
    let folder = format!("vostok/work/tasks/{spec}/{task}");
    let files = [
        ("estado.yml", format!("id: {task}\nspec: {spec}\ntitle: \"Tarefa\"\neffort: simples\ndepends_on: []\ncovers: [RF-01]\ntype: tarefa\nlabels: [{spec}, simples]\nstatus: {status}\ncontent_hash: null\nadjustments: 0\nadjusted_at: null\napproved_at: null\napproved_hash: null\nattempts: 0\ntests_result: null\nregression_result: null\nevidence_result: null\n")),
        ("execucao.md", "### Ler\n\n- `README.md`\n\n### Mudar\n\n- `novo.txt` (criar): arquivo novo.\n\n### Passos\n\n1. Crie o arquivo.\n".to_string()),
        ("verificacao.md", format!("### Baseline\n\n- `cat valor.txt`\n\n### Verificação\n\n```bash\ntrue\n```\n\n### Evidências\n\n{evidencias}\n")),
        ("criterios.md", "### Concluída quando\n\n- [ ] Pronto (RF-01).\n\n  > Como comprovar: ver o arquivo.\n".to_string()),
        ("executivo.md", "Resumo.\n".to_string()),
        ("tecnico.md", "Detalhes.\n".to_string()),
        ("registro.md", "## Registro de execução\n\n| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n\n## Histórico de ajustes\n\n| # | Data | Origem | Resumo |\n|---|---|---|---|\n".to_string()),
    ];
    for (name, content) in files {
        reaproveita_gravar(project, &format!("{folder}/{name}"), &content);
    }
}

fn reaproveita_comando(project: &Project, args: &[&str]) -> assert_cmd::Command {
    let mut command = project.vostok();
    command.args(args);
    command
}

fn reaproveita_saida(command: &mut assert_cmd::Command) -> String {
    let output = command.assert().success().get_output().stdout.clone();
    String::from_utf8(output).unwrap()
}

fn reaproveita_preparar(project: &Project, spec: &str, task: &str) {
    reaproveita_comando(project, &["task", "adjust", "record-manual", spec, task])
        .assert()
        .success();
    reaproveita_comando(project, &["task", "approve", "approve", spec, task])
        .assert()
        .success();
}

fn reaproveita_iniciar(project: &Project, spec: &str, task: &str) -> String {
    reaproveita_preparar(project, spec, task);
    reaproveita_saida(&mut reaproveita_comando(
        project,
        &["task", "start", spec, task],
    ))
}

fn reaproveita_antecipada(project: &Project, spec: &str, task: &str, extra: &[&str]) -> String {
    let mut args = vec!["task", "evidence", spec, task, "--antecipada"];
    args.extend_from_slice(extra);
    reaproveita_saida(&mut reaproveita_comando(project, &args))
}

fn reaproveita_indice(project: &Project) -> String {
    reaproveita_ler(project, "vostok/work/evidence-index/index.yml")
}

fn reaproveita_objetos(project: &Project) -> Vec<PathBuf> {
    let Ok(listing) = fs::read_dir(project.root().join("vostok/work/evidence-index/objects"))
    else {
        return Vec::new();
    };
    let mut paths: Vec<PathBuf> = listing.flatten().map(|entry| entry.path()).collect();
    paths.sort();
    paths
}

fn reaproveita_rodadas(project: &Project) -> usize {
    reaproveita_ler(project, "rodadas.txt").lines().count()
}

/// Projeto com a SPEC-001 TASK-01 em execução, com as evidências dadas.
fn reaproveita_uma(evidencias: &str) -> Project {
    let project = reaproveita_projeto();
    reaproveita_tarefa(&project, "SPEC-001", "TASK-01", evidencias, "pending");
    reaproveita_iniciar(&project, "SPEC-001", "TASK-01");
    project
}

fn reaproveita_outra(project: &Project, spec: &str, evidencias: &str) {
    reaproveita_tarefa(project, spec, "TASK-01", evidencias, "pending");
    reaproveita_iniciar(project, spec, "TASK-01");
}

#[test]
fn reaproveita_gera_copia_com_depende_quando_muda_arquivo_fora() {
    let project = reaproveita_uma(REAPROVEITA_LOG);
    reaproveita_antecipada(&project, "SPEC-001", "TASK-01", &[]);
    reaproveita_gravar(&project, "fora.txt", "2\n");
    let output = reaproveita_antecipada(&project, "SPEC-001", "TASK-01", &[]);
    assert!(
        output.contains("- [evidência saida.txt] reaproveitada de SPEC-001 TASK-01 EV-01 (chave "),
        "{output}"
    );
    assert_eq!(reaproveita_rodadas(&project), 1);
    let record = reaproveita_ler(&project, "vostok/work/tasks/SPEC-001/TASK-01/registro.md");
    assert!(
        record.lines().any(|line| line.contains("| reaproveitada |")
            && line.contains("saida.txt de SPEC-001 TASK-01 EV-01 (chave ")),
        "{record}"
    );
    let evidence = reaproveita_ler(
        &project,
        "vostok/work/tasks/SPEC-001/TASK-01/evidencias/saida.txt",
    );
    let body = evidence.split_once("\n\n").map(|(_, body)| body);
    assert_eq!(body, Some("a\n"), "{evidence}");
}

#[test]
fn reaproveita_gera_roda_de_novo_quando_muda_um_byte_em_depende() {
    let project = reaproveita_uma(REAPROVEITA_LOG);
    reaproveita_antecipada(&project, "SPEC-001", "TASK-01", &[]);
    reaproveita_gravar(&project, "valor.txt", "b\n");
    let output = reaproveita_antecipada(&project, "SPEC-001", "TASK-01", &[]);
    assert!(!output.contains("reaproveitada"), "{output}");
    assert_eq!(reaproveita_rodadas(&project), 2);
}

#[test]
fn reaproveita_gera_pasta_em_depende_entra_com_os_arquivos() {
    let project = reaproveita_uma(REAPROVEITA_PRINT);
    reaproveita_antecipada(&project, "SPEC-001", "TASK-01", &[]);
    reaproveita_gravar(&project, "assets/fase1/nova.txt", "nova\n");
    reaproveita_antecipada(&project, "SPEC-001", "TASK-01", &[]);
    assert_eq!(reaproveita_rodadas(&project), 4);
}

#[test]
fn reaproveita_gera_sem_depende_usa_a_arvore() {
    let project = reaproveita_uma(REAPROVEITA_SEM_DEPENDE);
    reaproveita_antecipada(&project, "SPEC-001", "TASK-01", &[]);
    let second = reaproveita_antecipada(&project, "SPEC-001", "TASK-01", &[]);
    assert!(second.contains("reaproveitada"), "{second}");
    reaproveita_gravar(&project, "fora.txt", "2\n");
    let third = reaproveita_antecipada(&project, "SPEC-001", "TASK-01", &[]);
    assert!(!third.contains("reaproveitada"), "{third}");
}

#[test]
fn reaproveita_gera_copia_entre_specs_com_a_origem() {
    let project = reaproveita_uma(REAPROVEITA_LOG);
    reaproveita_outra(&project, "SPEC-002", REAPROVEITA_LOG);
    reaproveita_antecipada(&project, "SPEC-001", "TASK-01", &[]);
    let output = reaproveita_antecipada(&project, "SPEC-002", "TASK-01", &[]);
    assert!(
        output.contains("reaproveitada de SPEC-001 TASK-01 EV-01"),
        "{output}"
    );
    assert_eq!(reaproveita_rodadas(&project), 1);
}

#[test]
fn reaproveita_gera_objeto_alterado_roda_e_substitui() {
    let project = reaproveita_uma(REAPROVEITA_LOG);
    reaproveita_antecipada(&project, "SPEC-001", "TASK-01", &[]);
    let objects = reaproveita_objetos(&project);
    assert_eq!(objects.len(), 1);
    fs::write(&objects[0], "x").unwrap();
    let output = reaproveita_antecipada(&project, "SPEC-001", "TASK-01", &[]);
    assert!(!output.contains("reaproveitada"), "{output}");
    assert_eq!(reaproveita_rodadas(&project), 2);
    let objects = reaproveita_objetos(&project);
    assert_eq!(objects.len(), 1);
    assert_eq!(fs::read(&objects[0]).unwrap(), b"a\n");
    assert!(objects[0].ends_with(sha256_hex(b"a\n")));
}

#[test]
fn reaproveita_gera_objeto_apagado_roda_e_substitui() {
    let project = reaproveita_uma(REAPROVEITA_LOG);
    reaproveita_antecipada(&project, "SPEC-001", "TASK-01", &[]);
    let objects = reaproveita_objetos(&project);
    fs::remove_file(&objects[0]).unwrap();
    let output = reaproveita_antecipada(&project, "SPEC-001", "TASK-01", &[]);
    assert!(!output.contains("reaproveitada"), "{output}");
    assert_eq!(reaproveita_rodadas(&project), 2);
    let objects = reaproveita_objetos(&project);
    assert_eq!(objects.len(), 1);
    assert!(objects[0].ends_with(sha256_hex(b"a\n")));
}

#[test]
fn reaproveita_gera_falha_nao_entra_no_indice() {
    let item = |name: &str, command: &str, extra: &str| {
        format!("- `{name}` (automática): `{command}`\n  - Título: Saída de {name}\n  - Prova: a saída.\n  - Trecho: a saída.\n  - Falha: se aparecer `quebrou-{name}`, quebrou.{extra}")
    };
    let evidencias = [
        item("codigo.txt", "exit 1", ""),
        item("espera.txt", "echo outro", "\n  - Espera: `nao-sai`"),
        "- `falha.txt` (automática): `echo erro-x`\n  - Título: Saída de falha.txt\n  - Prova: a saída.\n  - Trecho: a saída.\n  - Falha: se aparecer `erro-x`, quebrou.".to_string(),
        item("boa.txt", "echo ok", ""),
    ]
    .join("\n");
    let project = reaproveita_uma(&evidencias);
    reaproveita_comando(
        &project,
        &["task", "evidence", "SPEC-001", "TASK-01", "--antecipada"],
    )
    .assert()
    .failure();
    let index = reaproveita_indice(&project);
    assert!(index.contains("\"EV-04\""), "{index}");
    for id in ["\"EV-01\"", "\"EV-02\"", "\"EV-03\""] {
        assert!(!index.contains(id), "{index}");
    }
}

#[test]
fn reaproveita_gera_espera_do_destino_e_conferido_na_copia() {
    let project = reaproveita_uma(REAPROVEITA_LOG);
    reaproveita_outra(
        &project,
        "SPEC-002",
        &format!("{REAPROVEITA_LOG}\n  - Espera: `nao-sai`"),
    );
    reaproveita_antecipada(&project, "SPEC-001", "TASK-01", &[]);
    let stderr = failure_stderr(&mut reaproveita_comando(
        &project,
        &["task", "evidence", "SPEC-002", "TASK-01", "--antecipada"],
    ));
    assert!(
        stderr.contains("texto esperado não encontrado: nao-sai"),
        "{stderr}"
    );
    assert_eq!(reaproveita_rodadas(&project), 1);
}

#[test]
fn reaproveita_gera_print_e_video_copiados_com_bytes_iguais() {
    let project = reaproveita_uma(REAPROVEITA_PRINT);
    reaproveita_outra(&project, "SPEC-002", REAPROVEITA_PRINT);
    reaproveita_antecipada(&project, "SPEC-001", "TASK-01", &[]);
    reaproveita_antecipada(&project, "SPEC-002", "TASK-01", &[]);
    for name in ["tela.png", "fase.webm"] {
        let origem = fs::read(project.root().join(format!(
            "vostok/work/tasks/SPEC-001/TASK-01/evidencias/{name}"
        )))
        .unwrap();
        let destino = fs::read(project.root().join(format!(
            "vostok/work/tasks/SPEC-002/TASK-01/evidencias/{name}"
        )))
        .unwrap();
        assert_eq!(origem, destino, "{name}");
    }
    assert_eq!(reaproveita_rodadas(&project), 2);
    let index = reaproveita_indice(&project);
    assert!(index.contains("\"print\""), "{index}");
    assert!(index.contains("\"video\""), "{index}");
}

#[test]
fn reaproveita_gera_sem_reaproveitar_roda_e_atualiza() {
    let project = reaproveita_uma(REAPROVEITA_LOG);
    reaproveita_antecipada(&project, "SPEC-001", "TASK-01", &[]);
    let output = reaproveita_antecipada(&project, "SPEC-001", "TASK-01", &["--sem-reaproveitar"]);
    assert!(!output.contains("reaproveitada"), "{output}");
    assert_eq!(reaproveita_rodadas(&project), 2);
    let index: serde_json::Value = serde_json::from_str(&reaproveita_indice(&project)).unwrap();
    assert_eq!(index["entries"].as_array().unwrap().len(), 1);
}

#[test]
fn reaproveita_gera_evidence_sem_antecipada_indexa_o_print() {
    let project = reaproveita_uma(REAPROVEITA_PRINT);
    reaproveita_comando(&project, &["task", "evidence", "SPEC-001", "TASK-01"])
        .assert()
        .success();
    let index = reaproveita_indice(&project);
    assert!(index.contains("\"EV-01\""), "{index}");
    assert!(index.contains("\"EV-02\""), "{index}");
}

#[test]
fn reaproveita_baseline_start_copia_de_spec_arquivada() {
    let project = reaproveita_projeto();
    reaproveita_config(&project, "evidence_baseline_dir", "evidence");
    reaproveita_tarefa(&project, "SPEC-001", "TASK-01", REAPROVEITA_LOG, "pending");
    reaproveita_iniciar(&project, "SPEC-001", "TASK-01");
    reaproveita_antecipada(&project, "SPEC-001", "TASK-01", &[]);
    fs::remove_dir_all(project.root().join("vostok/work/tasks/SPEC-001")).unwrap();
    reaproveita_tarefa(&project, "SPEC-002", "TASK-01", REAPROVEITA_LOG, "pending");
    let output = reaproveita_iniciar(&project, "SPEC-002", "TASK-01");
    assert!(
        output.contains("reaproveitada de SPEC-001 TASK-01 EV-01"),
        "{output}"
    );
    assert_eq!(
        reaproveita_ler(
            &project,
            "vostok/work/tasks/SPEC-002/TASK-01/evidencias/baseline/evidence/saida.txt"
        ),
        "$ echo x >> rodadas.txt; cat valor.txt\nexit: 0\na\n"
    );
    assert_eq!(reaproveita_rodadas(&project), 1);
}

#[test]
fn reaproveita_baseline_start_sem_reaproveitar_roda() {
    let project = reaproveita_projeto();
    reaproveita_config(&project, "evidence_baseline_dir", "evidence");
    reaproveita_tarefa(&project, "SPEC-001", "TASK-01", REAPROVEITA_LOG, "pending");
    reaproveita_iniciar(&project, "SPEC-001", "TASK-01");
    reaproveita_antecipada(&project, "SPEC-001", "TASK-01", &[]);
    reaproveita_tarefa(&project, "SPEC-002", "TASK-01", REAPROVEITA_LOG, "pending");
    reaproveita_preparar(&project, "SPEC-002", "TASK-01");
    let output = reaproveita_saida(&mut reaproveita_comando(
        &project,
        &["task", "start", "SPEC-002", "TASK-01", "--sem-reaproveitar"],
    ));
    assert!(!output.contains("reaproveitada"), "{output}");
    assert_eq!(reaproveita_rodadas(&project), 2);
}

#[test]
fn reaproveita_baseline_grava_no_indice() {
    let project = reaproveita_projeto();
    reaproveita_config(&project, "evidence_baseline_dir", "evidence");
    reaproveita_tarefa(&project, "SPEC-001", "TASK-01", REAPROVEITA_LOG, "pending");
    reaproveita_iniciar(&project, "SPEC-001", "TASK-01");
    let index = reaproveita_indice(&project);
    assert!(index.contains("\"EV-01\""), "{index}");
}

#[test]
fn reaproveita_confere_conferencia_levada_da_origem() {
    let project = reaproveita_uma(REAPROVEITA_LOG);
    reaproveita_outra(&project, "SPEC-002", REAPROVEITA_LOG);
    reaproveita_antecipada(&project, "SPEC-001", "TASK-01", &[]);
    reaproveita_comando(
        &project,
        &[
            "task",
            "evidence",
            "SPEC-001",
            "TASK-01",
            "--reviewed",
            "EV-01",
        ],
    )
    .assert()
    .success();
    let output = reaproveita_antecipada(&project, "SPEC-002", "TASK-01", &[]);
    assert!(!output.contains("Evidências a conferir"), "{output}");
    let reviews = reaproveita_ler(
        &project,
        "vostok/work/tasks/SPEC-002/TASK-01/conferencias.md",
    );
    assert!(
        reviews
            .lines()
            .any(|line| line.starts_with("| EV-01 | saida.txt | conferida |")
                && line.ends_with("| reaproveitada | reaproveitada de SPEC-001 TASK-01 EV-01 |")),
        "{reviews}"
    );
    let record = reaproveita_ler(&project, "vostok/work/tasks/SPEC-002/TASK-01/registro.md");
    assert!(
        record.lines().any(|line| line.contains("| conferência |")
            && line.contains("EV-01 reaproveitada de SPEC-001 TASK-01 EV-01")),
        "{record}"
    );
}

#[test]
fn reaproveita_confere_origem_sem_conferencia_pede_conferencia() {
    let project = reaproveita_uma(REAPROVEITA_LOG);
    reaproveita_outra(&project, "SPEC-002", REAPROVEITA_LOG);
    reaproveita_antecipada(&project, "SPEC-001", "TASK-01", &[]);
    let output = reaproveita_antecipada(&project, "SPEC-002", "TASK-01", &[]);
    assert!(output.contains("reaproveitada de"), "{output}");
    assert!(output.contains("Evidências a conferir"), "{output}");
    assert!(!project
        .root()
        .join("vostok/work/tasks/SPEC-002/TASK-01/conferencias.md")
        .exists());
}

#[test]
fn reaproveita_config_desligado_nao_le_nem_grava() {
    let project = reaproveita_projeto();
    reaproveita_config(&project, "evidence_reuse", "desligado");
    reaproveita_tarefa(&project, "SPEC-001", "TASK-01", REAPROVEITA_LOG, "pending");
    reaproveita_iniciar(&project, "SPEC-001", "TASK-01");
    reaproveita_antecipada(&project, "SPEC-001", "TASK-01", &[]);
    let output = reaproveita_antecipada(&project, "SPEC-001", "TASK-01", &[]);
    assert!(!output.contains("reaproveitada"), "{output}");
    assert_eq!(reaproveita_rodadas(&project), 2);
    assert!(!project.root().join("vostok/work/evidence-index").exists());
}

#[test]
fn reaproveita_config_invalido_avisa_e_vale_ligado() {
    let project = reaproveita_projeto();
    reaproveita_config(&project, "evidence_reuse", "talvez");
    reaproveita_tarefa(&project, "SPEC-001", "TASK-01", REAPROVEITA_LOG, "pending");
    reaproveita_iniciar(&project, "SPEC-001", "TASK-01");
    reaproveita_antecipada(&project, "SPEC-001", "TASK-01", &[]);
    let output = reaproveita_comando(
        &project,
        &["task", "evidence", "SPEC-001", "TASK-01", "--antecipada"],
    )
    .assert()
    .success()
    .get_output()
    .clone();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stderr.contains("Aviso: evidence_reuse 'talvez' inválido em config/tasks.yml; use ligado ou desligado. Vale ligado."),
        "{stderr}"
    );
    assert!(stdout.contains("reaproveitada"), "{stdout}");
}

const REAPROVEITA_DEPENDE_INVALIDO: [&str; 3] =
    ["`nao-existe.txt`", "`../fora.txt`", "`vostok/work/tasks`"];

fn reaproveita_tres_depende(subitems: &str) -> String {
    REAPROVEITA_DEPENDE_INVALIDO
        .iter()
        .enumerate()
        .map(|(index, depends)| {
            format!(
                "- `ok{}.txt` (automática): `echo ok`\n{subitems}\n  - Depende: {depends}",
                index + 1
            )
        })
        .collect::<Vec<String>>()
        .join("\n")
}

fn reaproveita_mensagens_depende(stderr: &str) {
    for expected in [
        "'Evidências': a evidência ok1.txt depende de 'nao-existe.txt', que não existe no projeto",
        "'Evidências': a evidência ok2.txt depende de '../fora.txt', que fica fora do projeto",
        "'Evidências': a evidência ok3.txt depende de 'vostok/work/tasks', que fica na pasta de trabalho do harness",
    ] {
        assert!(stderr.contains(expected), "{expected}\n{stderr}");
    }
}

const REAPROVEITA_SUBITENS: &str =
    "  - Título: Saída do comando.\n  - Prova: ok.\n  - Trecho: ok.\n  - Falha: erro.";

#[test]
fn reaproveita_valida_task_adjust_recusa_depende_invalido() {
    let project = reaproveita_projeto();
    reaproveita_tarefa(
        &project,
        "SPEC-001",
        "TASK-01",
        &reaproveita_tres_depende(REAPROVEITA_SUBITENS),
        "pending",
    );
    let stderr = failure_stderr(&mut reaproveita_comando(
        &project,
        &["task", "adjust", "validate", "SPEC-001", "TASK-01"],
    ));
    reaproveita_mensagens_depende(&stderr);
}

fn reaproveita_spec_e_plano(project: &Project, kind: &str, spec_body: &str, plan_body: &str) {
    let spec_hash = sha256_hex(spec_body.as_bytes());
    reaproveita_gravar(
        project,
        "vostok/work/specs/refined/SPEC-001-exemplo.md",
        &format!("---\nid: SPEC-001\ntitle: \"Exemplo\"\nstatus: approved\ntype: {kind}\nupdated_at: null\ncontent_hash: {spec_hash}\napproved_hash: {spec_hash}\n---\n{spec_body}"),
    );
    let plan_hash = sha256_hex(plan_body.as_bytes());
    reaproveita_gravar(
        project,
        "vostok/work/plans/SPEC-001.md",
        &format!("---\nspec: SPEC-001\nspec_hash: {spec_hash}\ncontent_hash: {plan_hash}\n---\n{plan_body}"),
    );
}

fn reaproveita_plano_com_evidencia(project: &Project, evidence: &str) {
    let spec_body = "\n# Exemplo\n\n## Requisitos funcionais\n\n| ID | Requisito |\n|---|---|\n| RF-01 | Entregar. |\n";
    let plan_body = "\n# Plano\n\n## Visão geral\n\nUma tarefa.\n\n## Tarefas\n\n| ID | Título | Descrição | Tipo | Nível | Dependências | Cobre |\n|---|---|---|---|---|---|---|\n| TASK-01 | Tarefa | Entrega. | tarefa | simples | - | RF-01 |\n\n## Fatos de esforço\n\n| ID | Arquivos a criar | Arquivos a alterar | Contrato público | Inferência | Risco alto |\n|---|---|---|---|---|---|\n| TASK-01 | 0 | 1 | não | não | não |\n";
    reaproveita_spec_e_plano(project, "improvement", spec_body, plan_body);
    reaproveita_gravar(
        project,
        "vostok/work/tasks/SPEC-001/TASK-01.md",
        &format!(
            "---\nid: TASK-01\nspec: SPEC-001\ntitle: \"Tarefa\"\neffort: simples\ndepends_on: []\ncovers: [RF-01]\ntype: tarefa\nlabels: [SPEC-001, simples]\nstatus: pending\ncontent_hash: null\nadjustments: 0\nadjusted_at: null\napproved_at: null\napproved_hash: null\nattempts: 0\ntests_result: null\nregression_result: null\nevidence_result: null\n---\n\n# TASK-01: Tarefa\n\n## Execução\n\n### Ler\n\n- `README.md`\n\n### Mudar\n\n- `novo.txt` (criar): arquivo novo.\n\n### Passos\n\n1. Crie o arquivo.\n\n### Baseline\n\n- `true`\n\n### Verificação\n\n```bash\ntrue\n```\n\n### Evidências\n\n{evidence}\n\n### Concluída quando\n\n- [ ] Pronto (RF-01).\n\n  > Como comprovar: ver o arquivo.\n\n## Descrição\n\n### Resumo executivo\n\nResumo.\n\n### Detalhes técnicos\n\nDetalhes.\n\n### Card\n\n- **Título:** Tarefa\n- **Tipo:** tarefa\n- **Descrição:** Descrição.\n- **Critérios de aceite:**\n  - [ ] Pronto.\n- **Rótulos:** SPEC-001, simples\n- **Estimativa:** simples\n- **Bloqueada por:** -\n\n## Registro de execução\n\n| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n\n## Histórico de ajustes\n\n| # | Data | Origem | Resumo |\n|---|---|---|---|\n"
        ),
    );
}

#[test]
fn reaproveita_valida_plan_recusa_depende_invalido() {
    let project = reaproveita_projeto();
    reaproveita_plano_com_evidencia(&project, &reaproveita_tres_depende(REAPROVEITA_SUBITENS));
    let stderr = failure_stderr(&mut reaproveita_comando(
        &project,
        &["plan", "approve", "validate", "SPEC-001"],
    ));
    reaproveita_mensagens_depende(&stderr);
}

#[test]
fn reaproveita_valida_depende_existente_passa() {
    let project = reaproveita_projeto();
    let evidence = format!(
        "- `ok1.txt` (automática): `echo ok`\n{REAPROVEITA_SUBITENS}\n  - Depende: `valor.txt`\n- `ok2.txt` (automática): `echo ok`\n{REAPROVEITA_SUBITENS}\n  - Depende: `assets/fase1/`"
    );
    reaproveita_plano_com_evidencia(&project, &evidence);
    reaproveita_comando(&project, &["plan", "approve", "validate", "SPEC-001"])
        .assert()
        .success();
}

// --- Candidatas de baseline de imagem (SPEC-075, TASK-05) ---

const CANDIDATAS_PRINT: &str = "- `fase1.png` (print): `cp assets/fase1.png \"$VOSTOK_EVIDENCE/fase1.png\"`\n  - Título: A tela da fase 1\n  - Prova: a fase 1.\n  - Depende: `assets/fase1.png`\n  - Trecho: o cenário.\n  - Falha: tela vazia.";

const CANDIDATAS_DESTINO: &str = "vostok/work/tasks/SPEC-002/evidence/TASK-01";

fn candidatas_tarefa(spec: &str, status: &str, evidence: &str) -> String {
    format!(
        "---\nid: TASK-01\nspec: {spec}\ntitle: \"Tarefa\"\neffort: simples\ndepends_on: []\ncovers: [RF-01]\ntype: tarefa\nlabels: [{spec}, simples]\nstatus: {status}\ncontent_hash: null\nadjustments: 0\nadjusted_at: null\napproved_at: null\napproved_hash: null\nattempts: 0\ntests_result: null\nregression_result: null\nevidence_result: null\n---\n\n# TASK-01: Tarefa\n\n## Execução\n\n### Ler\n\n- `README.md`\n\n### Mudar\n\n- `novo.txt` (criar): arquivo novo.\n\n### Passos\n\n1. Crie o arquivo.\n\n### Baseline\n\n- `true`\n\n### Verificação\n\n```bash\ntrue\n```\n\n### Evidências\n\n{evidence}\n\n### Concluída quando\n\n- [ ] Pronto (RF-01).\n\n  > Como comprovar: ver o arquivo.\n\n## Descrição\n\n### Resumo executivo\n\nResumo.\n\n### Detalhes técnicos\n\nDetalhes.\n\n### Card\n\n- **Título:** Tarefa\n- **Tipo:** tarefa\n- **Descrição:** Descrição.\n- **Critérios de aceite:**\n  - [ ] Pronto.\n- **Rótulos:** {spec}, simples\n- **Estimativa:** simples\n- **Bloqueada por:** -\n\n## Registro de execução\n\n| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n\n## Histórico de ajustes\n\n| # | Data | Origem | Resumo |\n|---|---|---|---|\n"
    )
}

fn candidatas_rodar(project: &Project, args: &[&str]) -> assert_cmd::Command {
    let mut command = project.vostok();
    command.args(args);
    command
}

fn candidatas_saida(project: &Project, args: &[&str]) -> String {
    let output = candidatas_rodar(project, args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).unwrap()
}

fn candidatas_gravar(project: &Project, relative: &str, content: &str) {
    let path = project.root().join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn candidatas_ler(project: &Project, relative: &str) -> String {
    fs::read_to_string(project.root().join(relative)).unwrap_or_default()
}

fn candidatas_existe(project: &Project, relative: &str) -> bool {
    project.root().join(relative).exists()
}

fn candidatas_projeto() -> Project {
    let project = Project::new();
    candidatas_gravar(&project, "assets/fase1.png", "PNG1");
    candidatas_gravar(
        &project,
        "vostok/work/tasks/SPEC-001/TASK-01.md",
        &candidatas_tarefa("SPEC-001", "in_progress", CANDIDATAS_PRINT),
    );
    candidatas_saida(&project, &["task", "baseline", "SPEC-001", "TASK-01"]);
    candidatas_saida(&project, &["task", "evidence", "SPEC-001", "TASK-01"]);
    fs::remove_dir_all(project.root().join("vostok/work/tasks/SPEC-001")).unwrap();
    candidatas_gravar(
        &project,
        "vostok/work/tasks/SPEC-002/TASK-01.md",
        &candidatas_tarefa("SPEC-002", "pending", CANDIDATAS_PRINT),
    );
    candidatas_saida(
        &project,
        &["task", "adjust", "record-manual", "SPEC-002", "TASK-01"],
    );
    candidatas_saida(
        &project,
        &["task", "approve", "approve", "SPEC-002", "TASK-01"],
    );
    project
}

fn candidatas_aplicar(project: &Project, de: &str) -> assert_cmd::Command {
    candidatas_rodar(
        project,
        &[
            "task",
            "reuse",
            "apply",
            "SPEC-002",
            "TASK-01",
            "fase1.png",
            "--de",
            "SPEC-001",
            "TASK-01",
            de,
            "--motivo",
            "usuário aceitou",
        ],
    )
}

fn candidatas_linha(saida: &str) -> String {
    saida
        .lines()
        .find(|line| line.starts_with("| TASK-01 | fase1.png | SPEC-001 TASK-01 EV-01 | "))
        .unwrap_or_else(|| panic!("sem a linha da candidata: {saida}"))
        .to_string()
}

fn candidatas_iniciar(project: &Project) -> String {
    candidatas_saida(
        project,
        &["task", "start", "SPEC-002", "TASK-01", "--model", "teste"],
    )
}

#[test]
fn candidatas_lista_a_origem_com_entradas_iguais() {
    let project = candidatas_projeto();
    let saida = candidatas_saida(&project, &["task", "reuse", "candidates", "SPEC-002"]);
    assert!(
        saida.contains("### Candidatas de baseline da SPEC-002"),
        "{saida}"
    );
    assert!(candidatas_linha(&saida).ends_with("| iguais |"), "{saida}");
    assert!(
        saida
            .lines()
            .any(|line| line.starts_with("Reaproveitar: vostok task reuse apply SPEC-002")),
        "{saida}"
    );
}

#[test]
fn candidatas_entradas_mudaram_depois_da_gravacao() {
    let project = candidatas_projeto();
    candidatas_gravar(&project, "assets/fase1.png", "PNG2");
    let saida = candidatas_saida(&project, &["task", "reuse", "candidates", "SPEC-002"]);
    assert!(candidatas_linha(&saida).ends_with("| mudaram |"), "{saida}");
}

#[test]
fn candidatas_spec_sem_candidatas() {
    let project = candidatas_projeto();
    let outra = CANDIDATAS_PRINT
        .replace("$VOSTOK_EVIDENCE/fase1.png", "$VOSTOK_EVIDENCE/outra.png")
        .replace("`fase1.png` (print)", "`outra.png` (print)");
    candidatas_gravar(
        &project,
        "vostok/work/tasks/SPEC-003/TASK-01.md",
        &candidatas_tarefa("SPEC-003", "pending", &outra),
    );
    let saida = candidatas_saida(&project, &["task", "reuse", "candidates", "SPEC-003"]);
    assert_eq!(saida, "Nenhuma evidência para reaproveitar.\n");
}

#[test]
fn candidatas_aplica_copia_e_registra_o_motivo() {
    let project = candidatas_projeto();
    let output = candidatas_aplicar(&project, "EV-01")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let saida = String::from_utf8(output).unwrap();
    assert!(
        saida.contains("- [evidência fase1.png] reaproveitada de SPEC-001 TASK-01 EV-01 (chave "),
        "{saida}"
    );
    assert_eq!(
        candidatas_ler(
            &project,
            &format!("{CANDIDATAS_DESTINO}/baseline-reaproveitado/fase1.png")
        ),
        "PNG1"
    );
    assert!(!candidatas_existe(
        &project,
        &format!("{CANDIDATAS_DESTINO}/baseline")
    ));
    let tarefa = candidatas_ler(&project, "vostok/work/tasks/SPEC-002/TASK-01.md");
    let linha = tarefa
        .lines()
        .find(|line| line.contains("| reaproveitada |"))
        .unwrap_or_else(|| panic!("sem a linha reaproveitada: {tarefa}"));
    assert!(
        linha.contains("fase1.png de SPEC-001 TASK-01 EV-01 (chave "),
        "{linha}"
    );
    assert!(linha.contains("motivo: usuário aceitou"), "{linha}");
}

#[test]
fn candidatas_aplicar_e_iniciar_move_para_o_baseline() {
    let project = candidatas_projeto();
    println!(
        "{}",
        candidatas_saida(&project, &["task", "reuse", "candidates", "SPEC-002"])
    );
    let output = candidatas_aplicar(&project, "EV-01")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    println!("{}", String::from_utf8(output).unwrap());
    let inicio = candidatas_iniciar(&project);
    println!("{inicio}");
    assert!(
        inicio.contains("1 evidência(s) reaproveitada(s) movida(s) para o baseline"),
        "{inicio}"
    );
    assert_eq!(
        candidatas_ler(
            &project,
            &format!("{CANDIDATAS_DESTINO}/baseline/evidence/fase1.png")
        ),
        "PNG1"
    );
    assert!(!candidatas_existe(
        &project,
        &format!("{CANDIDATAS_DESTINO}/baseline-reaproveitado")
    ));
}

#[test]
fn candidatas_recusa_depois_do_baseline() {
    let project = candidatas_projeto();
    candidatas_iniciar(&project);
    let saida = candidatas_saida(&project, &["task", "reuse", "candidates", "SPEC-002"]);
    assert!(
        saida.contains("Nenhuma evidência para reaproveitar."),
        "{saida}"
    );
    let stderr = failure_stderr(&mut candidatas_aplicar(&project, "EV-01"));
    assert!(stderr.contains("já foi coletado"), "{stderr}");
}

#[test]
fn candidatas_aplicar_recusa_entrada_invalida() {
    let project = candidatas_projeto();
    let destino = format!("{CANDIDATAS_DESTINO}/baseline-reaproveitado");
    let base = [
        "task",
        "reuse",
        "apply",
        "SPEC-002",
        "TASK-01",
        "fase1.png",
        "--de",
        "SPEC-001",
        "TASK-01",
        "EV-01",
    ];
    let stderr = failure_stderr(&mut candidatas_rodar(&project, &base));
    assert!(stderr.contains("informe em --motivo"), "{stderr}");
    let stderr = failure_stderr(&mut candidatas_aplicar(&project, "EV-02"));
    assert!(stderr.contains("não é candidata"), "{stderr}");
    let mut outra = base.to_vec();
    outra[5] = "outra.png";
    outra.extend_from_slice(&["--motivo", "usuário aceitou"]);
    let stderr = failure_stderr(&mut candidatas_rodar(&project, &outra));
    assert!(stderr.contains("não é uma evidência (print)"), "{stderr}");
    candidatas_rodar(
        &project,
        &[
            "task",
            "reuse",
            "apply",
            "SPEC-002",
            "TASK-01",
            "fase1.png",
            "--de",
            "SPEC-001",
            "TASK-01",
            "--motivo",
            "usuário aceitou",
        ],
    )
    .assert()
    .failure();
    assert!(!candidatas_existe(&project, &destino));
}

#[test]
fn candidatas_aplicar_recusa_copia_alterada() {
    let project = candidatas_projeto();
    let objetos = project.root().join("vostok/work/evidence-index/objects");
    for objeto in fs::read_dir(&objetos).unwrap() {
        fs::write(objeto.unwrap().path(), "ALTERADO").unwrap();
    }
    let stderr = failure_stderr(&mut candidatas_aplicar(&project, "EV-01"));
    assert!(stderr.contains("não está íntegra"), "{stderr}");
    let criados: Vec<_> = fs::read_dir(project.root().join(CANDIDATAS_DESTINO))
        .map(|entries| entries.collect())
        .unwrap_or_default();
    assert!(criados.is_empty(), "{criados:?}");
}

#[test]
fn candidatas_desligado() {
    let project = candidatas_projeto();
    let path = "vostok/config/tasks.yml";
    let mut config: String = candidatas_ler(&project, path)
        .lines()
        .filter(|line| !line.starts_with("evidence_reuse:"))
        .map(|line| format!("{line}\n"))
        .collect();
    config.push_str("evidence_reuse: desligado\n");
    candidatas_gravar(&project, path, &config);
    let saida = candidatas_saida(&project, &["task", "reuse", "candidates", "SPEC-002"]);
    assert!(
        saida.contains("Reaproveitamento desligado em config/tasks.yml (evidence_reuse)."),
        "{saida}"
    );
    let stderr = failure_stderr(&mut candidatas_aplicar(&project, "EV-01"));
    assert!(
        stderr.contains("o reaproveitamento está desligado"),
        "{stderr}"
    );
}
