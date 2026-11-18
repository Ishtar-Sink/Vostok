//! `vostok review brief`: roteiro do revisor sem contexto para spec, plano e tarefa.

mod common;

use std::fs;

use common::{failure_stderr, Project};

fn artifacts(project: &Project) {
    let harness = project.harness();
    let write = |relative: &str, content: &str| {
        let path = harness.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    };
    write(
        "work/specs/refined/SPEC-001-exemplo.md",
        "---\nid: SPEC-001\nstatus: approved\n---\n\n# Exemplo\n",
    );
    write("work/plans/SPEC-001.md", "# Plano\n\nTexto do plano.\n");
    for part in ["execucao", "verificacao", "criterios", "tecnico"] {
        write(
            &format!("work/tasks/SPEC-001/TASK-01/{part}.md"),
            &format!("Texto de {part}.\n"),
        );
    }
}

fn set_review(project: &Project, block: &str) {
    let path = project.harness().join("config/main.yml");
    let current = fs::read_to_string(&path).unwrap();
    let kept = current.split("\nreview:").next().unwrap().to_string();
    let kept = if block.is_empty() {
        kept
    } else {
        format!("{kept}\n{block}\n")
    };
    fs::write(path, kept).unwrap();
}

fn brief(project: &Project, args: &[&str]) -> String {
    let output = project
        .vostok()
        .arg("review")
        .arg("brief")
        .args(args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).unwrap()
}

#[test]
fn revisao_roteiro_da_tarefa() {
    let project = Project::new();
    artifacts(&project);

    let out = brief(&project, &["task", "SPEC-001", "TASK-01"]);

    for expected in [
        "# Revisão de task SPEC-001 TASK-01",
        "- Perfil: julgamento",
        "# Persona",
        "Persona: Revisor",
        "# Roteiro",
        "# Arquivos",
        "work/tasks/SPEC-001/TASK-01/execucao.md",
        "work/tasks/SPEC-001/TASK-01/verificacao.md",
        "work/tasks/SPEC-001/TASK-01/criterios.md",
        "work/tasks/SPEC-001/TASK-01/tecnico.md",
        "work/specs/refined/SPEC-001-exemplo.md",
        "| # | Local | Problema | Correção | Muda o escopo |",
        "Nenhum achado.",
    ] {
        assert!(out.contains(expected), "falta '{expected}' em:\n{out}");
    }
}

#[test]
fn revisao_roteiro_da_spec_e_do_plano() {
    let project = Project::new();
    artifacts(&project);

    let spec = brief(&project, &["spec", "SPEC-001"]);
    let plan = brief(&project, &["plan", "SPEC-001"]);

    let spec_files = spec.split("# Arquivos").nth(1).unwrap();
    assert!(spec_files.contains("SPEC-001-exemplo.md"), "{spec}");
    assert!(!spec_files.contains("work/plans"), "{spec}");
    let plan_files = plan.split("# Arquivos").nth(1).unwrap();
    assert!(plan_files.contains("work/plans/SPEC-001.md"), "{plan}");
    assert!(plan_files.contains("SPEC-001-exemplo.md"), "{plan}");
}

#[test]
fn revisao_desligada() {
    let project = Project::new();
    artifacts(&project);
    set_review(&project, "review:\n  task: false");

    let off = brief(&project, &["task", "SPEC-001", "TASK-01"]);

    assert_eq!(off.trim(), "Revisão desligada para task.");
    assert!(brief(&project, &["spec", "SPEC-001"]).contains("# Roteiro"));
    assert!(brief(&project, &["plan", "SPEC-001"]).contains("# Roteiro"));
}

#[test]
fn revisao_sem_chave_fica_ligada() {
    let project = Project::new();
    artifacts(&project);
    set_review(&project, "");

    assert!(brief(&project, &["spec", "SPEC-001"]).contains("# Roteiro"));
    assert!(brief(&project, &["plan", "SPEC-001"]).contains("# Roteiro"));
    assert!(brief(&project, &["task", "SPEC-001", "TASK-01"]).contains("# Roteiro"));
}

#[test]
fn revisao_valor_invalido_recusado() {
    let project = Project::new();
    artifacts(&project);
    set_review(&project, "review:\n  plan: talvez");

    let err = failure_stderr(
        project
            .vostok()
            .args(["review", "brief", "plan", "SPEC-001"]),
    );

    assert!(err.contains("review.plan"), "{err}");
    assert!(err.contains("use true ou false"), "{err}");
}

#[test]
fn revisao_artefato_inexistente_recusado() {
    let project = Project::new();
    artifacts(&project);

    let err = failure_stderr(
        project
            .vostok()
            .args(["review", "brief", "task", "SPEC-001", "TASK-09"]),
    );

    assert!(err.contains("TASK-09"), "{err}");
}
