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
    for task in ["TASK-01", "TASK-02", "TASK-03"] {
        for part in ["execucao", "verificacao", "criterios", "tecnico"] {
            write(
                &format!("work/tasks/SPEC-001/{task}/{part}.md"),
                &format!("Texto de {part}.\n"),
            );
        }
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
        "| # | Tarefa | Local | Problema | Correção | Muda o escopo |",
        "Nenhum achado.",
    ] {
        assert!(out.contains(expected), "falta '{expected}' em:\n{out}");
    }
}

#[test]
fn revisao_roteiro_da_spec_e_do_plano() {
    let project = Project::new();
    artifacts(&project);
    set_review(&project, "review:\n  spec: true\n  plan: true");

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
    set_review(
        &project,
        "review:\n  spec: true\n  plan: true\n  task: false",
    );

    let off = brief(&project, &["task", "SPEC-001", "TASK-01"]);

    assert_eq!(off.trim(), "Revisão desligada para task.");
    assert_eq!(
        brief(&project, &["task", "SPEC-001"]).trim(),
        "Revisão desligada para task."
    );
    assert!(brief(&project, &["spec", "SPEC-001"]).contains("# Roteiro"));
    assert!(brief(&project, &["plan", "SPEC-001"]).contains("# Roteiro"));
}

fn assert_only_task_on(project: &Project) {
    assert_eq!(
        brief(project, &["spec", "SPEC-001"]).trim(),
        "Revisão desligada para spec."
    );
    assert_eq!(
        brief(project, &["plan", "SPEC-001"]).trim(),
        "Revisão desligada para plan."
    );
    assert!(brief(project, &["task", "SPEC-001"]).contains("# Roteiro"));
}

#[test]
fn revisao_sem_chave_so_a_tarefa_fica_ligada() {
    let project = Project::new();
    artifacts(&project);
    set_review(&project, "");

    assert_only_task_on(&project);
}

#[test]
fn revisao_chave_vazia_segue_a_ausente() {
    let project = Project::new();
    artifacts(&project);
    set_review(&project, "review:\n  spec:\n  plan:\n  task:");

    assert_only_task_on(&project);
}

#[test]
fn revisao_padrao_distribuido_so_tarefas() {
    let project = Project::new();
    artifacts(&project);

    let config = fs::read_to_string(project.harness().join("config/main.yml")).unwrap();

    for line in ["  spec: false", "  plan: false", "  task: true"] {
        assert!(config.lines().any(|l| l == line), "falta '{line}'");
    }
    assert_eq!(
        brief(&project, &["spec", "SPEC-001"]).trim(),
        "Revisão desligada para spec."
    );
}

#[test]
fn revisao_todas_as_tarefas() {
    let project = Project::new();
    artifacts(&project);

    let out = brief(&project, &["task", "SPEC-001"]);

    assert!(
        out.contains("# Revisão de task SPEC-001 (todas as tarefas)"),
        "{out}"
    );
    let files = out.split("# Arquivos").nth(1).unwrap();
    for task in ["TASK-01", "TASK-02", "TASK-03"] {
        for part in ["execucao", "verificacao", "criterios", "tecnico"] {
            let path = format!("work/tasks/SPEC-001/{task}/{part}.md");
            assert!(files.contains(&path), "falta {path}:\n{out}");
        }
    }
    assert!(files.contains("SPEC-001-exemplo.md"), "{out}");
    assert!(!files.contains("## Contexto"), "{out}");
}

#[test]
fn revisao_tarefas_indicadas_com_contexto() {
    let project = Project::new();
    artifacts(&project);

    let out = brief(&project, &["task", "SPEC-001", "TASK-02", "TASK-03"]);

    assert!(
        out.contains("# Revisão de task SPEC-001 TASK-02 TASK-03"),
        "{out}"
    );
    let files = out.split("# Arquivos").nth(1).unwrap();
    let mut parts = files.split("## Contexto");
    let reviewed = parts.next().unwrap();
    let context = parts.next().expect("falta ## Contexto");
    assert!(
        reviewed.contains("TASK-02") && reviewed.contains("TASK-03"),
        "{out}"
    );
    assert!(!reviewed.contains("TASK-01"), "{out}");
    let context = context.split("# Resposta").next().unwrap();
    assert!(
        context.trim_start().starts_with("Leia estas tarefas só para achar inconsistências com as revisadas; todo achado cita uma tarefa revisada."),
        "{out}"
    );
    assert!(context.contains("TASK-01"), "{out}");
    assert!(
        !context.contains("TASK-02") && !context.contains("TASK-03"),
        "{out}"
    );
}

#[test]
fn revisao_todas_indicadas_sem_contexto() {
    let project = Project::new();
    artifacts(&project);

    let out = brief(
        &project,
        &["task", "SPEC-001", "TASK-01", "TASK-02", "TASK-03"],
    );

    let files = out.split("# Arquivos").nth(1).unwrap();
    assert!(!files.contains("## Contexto"), "{out}");
}

#[test]
fn revisao_roteiro_entre_tarefas() {
    let project = Project::new();
    artifacts(&project);

    let out = brief(&project, &["task", "SPEC-001"]);

    let route = out
        .split("# Roteiro")
        .nth(1)
        .unwrap()
        .split("# Arquivos")
        .next()
        .unwrap();
    for expected in [
        "Entre tarefas, procure:",
        "mesmo arquivo criado por duas tarefas",
        "passo que usa o resultado de outra tarefa sem depender dela",
        "duas tarefas da mesma onda mudando o mesmo arquivo",
        "nome ou contrato diferente para a mesma coisa",
        "ID da spec que nenhuma tarefa cobre",
    ] {
        assert!(route.contains(expected), "falta '{expected}' em:\n{route}");
    }
}

#[test]
fn revisao_spec_sem_tarefas_recusada() {
    let project = Project::new();
    artifacts(&project);
    fs::remove_dir_all(project.harness().join("work/tasks/SPEC-001")).unwrap();

    let err = failure_stderr(
        project
            .vostok()
            .args(["review", "brief", "task", "SPEC-001"]),
    );

    assert!(err.contains("a SPEC-001 não tem tarefas"), "{err}");
}

#[test]
fn revisao_tarefa_repetida_uma_vez() {
    let project = Project::new();
    artifacts(&project);

    let out = brief(&project, &["task", "SPEC-001", "TASK-01", "TASK-01"]);

    assert_eq!(
        out.matches("work/tasks/SPEC-001/TASK-01/execucao.md")
            .count(),
        1,
        "{out}"
    );
}

#[test]
fn revisao_tarefa_em_arquivo_unico() {
    let project = Project::new();
    artifacts(&project);
    let tasks = project.harness().join("work/tasks/SPEC-001");
    fs::remove_dir_all(tasks.join("TASK-01")).unwrap();
    fs::write(tasks.join("TASK-01.md"), "Tarefa única.\n").unwrap();

    let out = brief(&project, &["task", "SPEC-001"]);

    assert!(out.contains("work/tasks/SPEC-001/TASK-01.md"), "{out}");
    assert!(
        out.contains("work/tasks/SPEC-001/TASK-02/execucao.md"),
        "{out}"
    );
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
    assert!(err.contains("não foi encontrada"), "{err}");
}
