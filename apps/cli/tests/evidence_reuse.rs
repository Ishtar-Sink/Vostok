//! Testes do índice de evidências reaproveitáveis (SPEC-075).

mod common;

use std::fs;

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
