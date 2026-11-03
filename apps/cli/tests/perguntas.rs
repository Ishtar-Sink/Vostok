//! Testes de integração do `vostok perguntas abrir` (SPEC-033).

mod common;

use std::fs;

use common::{failure_stderr, Project};
use vostok::support::config_set;

const QUESTION: &str =
    "# 1. Qual caminho?\n\n| Opção | Descrição |\n|---|---|\n| A | Seguir (recomendada) |\n";

fn write(project: &Project, relative: &str, content: &str) {
    let path = project.root().join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn read(project: &Project, relative: &str) -> String {
    fs::read_to_string(project.root().join(relative)).unwrap_or_default()
}

fn write_request(project: &Project, state_folder: &str, id: &str, spec: &str, task: &str) {
    write(
        project,
        &format!("vostok/work/pedidos/{state_folder}/{id}.md"),
        &format!("---\nid: {id}\nacao: refinar-spec\nspec: {spec}\ntarefa: {task}\nestado: {state_folder}\norigem: painel\ncriado_em: 2026-11-02T10:00:00Z\natualizado_em: 2026-11-02T10:00:00Z\n---\n\n## Pedido\n\nRefinar.\n\n## Resultado\n\n"),
    );
}

fn open(project: &Project, request: &str, text: &str) -> assert_cmd::Command {
    let mut command = project.vostok();
    command
        .args(["perguntas", "abrir", request])
        .write_stdin(text.to_string());
    command
}

fn stdout_of(command: &mut assert_cmd::Command) -> String {
    let output = command.output().unwrap();
    assert!(output.status.success(), "{output:?}");
    String::from_utf8(output.stdout).unwrap()
}

fn question_files(project: &Project) -> usize {
    let dir = project.root().join("vostok/work/perguntas");
    if !dir.exists() {
        return 0;
    }
    let mut count = 0;
    let mut pending = vec![dir];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else {
                count += 1;
            }
        }
    }
    count
}

#[test]
fn abrir_grava_a_pergunta_aberta_com_o_pedido() {
    let project = Project::new();
    write_request(&project, "em_andamento", "PED-001", "SPEC-001", "TASK-02");
    let out = stdout_of(&mut open(&project, "PED-001", QUESTION));
    assert!(out.contains("P-001"), "{out}");
    let content = read(&project, "vostok/work/perguntas/aberta/P-001.md");
    let (front, body) = content
        .strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---\n"))
        .expect("front-matter");
    let lines: Vec<&str> = front.lines().collect();
    for expected in [
        "id: P-001",
        "pedido: PED-001",
        "spec: SPEC-001",
        "task: TASK-02",
        "status: aberta",
    ] {
        assert!(lines.contains(&expected), "{expected}: {content}");
    }
    assert!(
        lines.iter().any(|line| line.starts_with("created_at: ")),
        "{content}"
    );
    assert!(
        body.starts_with("\n## Pergunta\n\n# 1. Qual caminho?"),
        "{content}"
    );
}

#[test]
fn abrir_numera_depois_da_maior_pergunta() {
    let project = Project::new();
    write_request(&project, "em_andamento", "PED-001", "SPEC-001", "");
    write(
        &project,
        "vostok/work/perguntas/aberta/P-001.md",
        "---\nid: P-001\nstatus: aberta\n---\n",
    );
    write(
        &project,
        "vostok/work/perguntas/respondida/P-004.md",
        "---\nid: P-004\nstatus: respondida\n---\n",
    );
    let out = stdout_of(&mut open(&project, "PED-001", QUESTION));
    assert!(out.contains("P-005"), "{out}");
    assert!(project
        .root()
        .join("vostok/work/perguntas/aberta/P-005.md")
        .is_file());
}

#[test]
fn abrir_acha_o_pedido_em_qualquer_estado() {
    let project = Project::new();
    write_request(&project, "feito", "PED-002", "SPEC-007", "");
    stdout_of(&mut open(&project, "PED-002", QUESTION));
    let content = read(&project, "vostok/work/perguntas/aberta/P-001.md");
    assert!(content.contains("\nspec: SPEC-007\n"), "{content}");
    assert!(content.contains("\npedido: PED-002\n"), "{content}");
}

#[test]
fn abrir_recusa_pedido_inexistente() {
    let project = Project::new();
    let refused = failure_stderr(&mut open(&project, "PED-009", QUESTION));
    assert!(refused.contains("PED-009"), "{refused}");
    assert!(refused.contains("não encontrado"), "{refused}");
    assert_eq!(question_files(&project), 0);
}

#[test]
fn abrir_recusa_id_invalido() {
    let project = Project::new();
    for invalid in ["PED-1", "X-001"] {
        let refused = failure_stderr(&mut open(&project, invalid, QUESTION));
        assert!(refused.contains("ID de pedido inválido"), "{refused}");
        assert!(refused.contains(invalid), "{refused}");
    }
    assert_eq!(question_files(&project), 0);
}

#[test]
fn abrir_recusa_texto_vazio() {
    let project = Project::new();
    write_request(&project, "em_andamento", "PED-001", "SPEC-001", "");
    let refused = failure_stderr(&mut open(&project, "PED-001", "  \n\t \n"));
    assert!(
        refused.contains("texto da pergunta veio vazio"),
        "{refused}"
    );
    assert_eq!(question_files(&project), 0);
}

#[test]
fn abrir_usa_as_pastas_configuradas() {
    let project = Project::new();
    let main = project.harness().join("config/main.yml");
    config_set(&main, "paths.questions", "./work/questions").unwrap();
    config_set(&main, "folders.questions.open", "open").unwrap();
    write_request(&project, "em_andamento", "PED-001", "SPEC-001", "");
    stdout_of(&mut open(&project, "PED-001", QUESTION));
    let content = read(&project, "vostok/work/questions/open/P-001.md");
    assert!(content.contains("\nstatus: aberta\n"), "{content}");
    assert!(!project.root().join("vostok/work/perguntas").exists());
}
