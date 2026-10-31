//! Testes do grupo `finalize`: equivalência com `finalize-spec.sh` pelas fixtures (CT-10, CT-12) e
//! recusas fora delas.

mod common;

use std::fs;

use common::golden::assert_group;
use common::{failure_stderr, Project};

#[test]
fn golden_finalize() {
    assert_group("finalize");
}

fn write_spec(project: &Project, status: &str) {
    let refined = project.harness().join("work/specs/refined");
    fs::create_dir_all(&refined).unwrap();
    fs::write(
        refined.join("SPEC-001-exemplo.md"),
        format!("---\nid: SPEC-001\ntitle: Exemplo\nstatus: {status}\n---\n\n# Exemplo\n"),
    )
    .unwrap();
}

fn write_task(project: &Project, id: &str, status: &str) {
    let tasks = project.harness().join("work/tasks/SPEC-001");
    fs::create_dir_all(&tasks).unwrap();
    fs::write(
        tasks.join(format!("{id}.md")),
        format!("---\nid: {id}\ntitle: Tarefa\nstatus: {status}\n---\n"),
    )
    .unwrap();
}

#[test]
fn every_subcommand_requires_a_spec_id() {
    let project = Project::new();
    for subcommand in ["sources", "create", "validate", "build", "archive"] {
        project
            .vostok()
            .args(["finalize", subcommand])
            .assert()
            .failure();
    }
}

#[test]
fn invalid_spec_id_is_refused() {
    let project = Project::new();
    let stderr = failure_stderr(project.vostok().args(["finalize", "sources", "spec-1"]));
    assert_eq!(stderr, "Erro: ID de spec inválido: 'spec-1'\n");
}

#[test]
fn unknown_spec_is_refused() {
    let project = Project::new();
    let stderr = failure_stderr(project.vostok().args(["finalize", "build", "SPEC-999"]));
    assert_eq!(
        stderr,
        "Erro: spec SPEC-999 não encontrada entre as refinadas\n"
    );
}

#[test]
fn spec_without_tasks_is_refused() {
    let project = Project::new();
    write_spec(&project, "implemented");
    let stderr = failure_stderr(project.vostok().args(["finalize", "create", "SPEC-001"]));
    assert!(stderr.starts_with("Erro: a spec SPEC-001 não pode ser finalizada:\n"));
    assert!(stderr.contains("  - nenhuma tarefa encontrada em "));
    assert!(!project.harness().join("work/finalize").exists());
}

#[test]
fn every_pending_task_is_listed() {
    let project = Project::new();
    write_spec(&project, "approved");
    write_task(&project, "TASK-01", "done");
    write_task(&project, "TASK-02", "in_progress");
    let stderr = failure_stderr(project.vostok().args(["finalize", "archive", "SPEC-001"]));
    assert_eq!(
        stderr,
        "Erro: a spec SPEC-001 não pode ser finalizada:\n  \
         - a spec está com status 'approved'; só specs com status 'implemented' são finalizadas\n  \
         - TASK-02: status 'in_progress', não 'done'\n"
    );
}

#[test]
fn validate_without_content_points_to_create() {
    let project = Project::new();
    write_spec(&project, "implemented");
    write_task(&project, "TASK-01", "done");
    let stderr = failure_stderr(project.vostok().args(["finalize", "validate", "SPEC-001"]));
    assert!(stderr.starts_with("Erro: o conteúdo de SPEC-001 tem problemas:\n"));
    assert!(stderr.contains("conteúdo não encontrado: "));
    assert!(stderr
        .trim_end()
        .ends_with("; crie com vostok finalize create"));
}

#[test]
fn evidence_document_lists_the_accepted_regression_differences() {
    let project = Project::new();
    write_spec(&project, "implemented");
    let tasks = project.harness().join("work/tasks/SPEC-001");
    fs::create_dir_all(&tasks).unwrap();
    fs::write(
        tasks.join("TASK-01.md"),
        "---\nid: TASK-01\ntitle: Tarefa\nstatus: done\n---\n\n# TASK-01: Tarefa\n\n## Execução\n\n### Evidências\n\nNão se aplica.\n\n## Registro de execução\n\n| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n| 1 | 2026-10-14 | conclusão | done | ok |\n",
    )
    .unwrap();
    let regression = tasks.join("evidence/TASK-01/validation/regression");
    fs::create_dir_all(&regression).unwrap();
    fs::write(
        regression.join("accepted"),
        format!(
            "1\t{}\t2026-10-14T10:00:00Z\tgrep -n '\"e2e:cart:' package.json\tvírgula exigida pelo passo 2\n",
            "a".repeat(64)
        ),
    )
    .unwrap();
    fs::write(
        regression.join("accepted-01.diff"),
        "3c3\n< um\n---\n> um,\n",
    )
    .unwrap();

    project
        .vostok()
        .args(["finalize", "create", "SPEC-001"])
        .assert()
        .success();
    let content_path = project.harness().join("work/finalize/SPEC-001/content.md");
    let content = fs::read_to_string(&content_path).unwrap();
    let filled = fill_variables(&content);
    fs::write(&content_path, filled).unwrap();
    let output = project
        .vostok()
        .args(["finalize", "build", "SPEC-001"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let html = fs::read_to_string(content_path.with_file_name("evidencias.html")).unwrap();
    assert!(
        html.contains("Diferença de regressão aceita: grep -n"),
        "{html}"
    );
    assert!(html.contains("e2e:cart:"));
    assert!(html.contains("&gt; um,"));
    assert!(html.contains("vírgula exigida pelo passo 2"));
}

/// Monta uma tarefa concluída com a evidência `saida.txt` e, se pedido, o baseline dela, e devolve o
/// `evidencias.html` gerado.
fn evidence_html_with_baseline(baseline: Option<&str>) -> String {
    let project = Project::new();
    write_spec(&project, "implemented");
    let tasks = project.harness().join("work/tasks/SPEC-001");
    fs::create_dir_all(&tasks).unwrap();
    fs::write(
        tasks.join("TASK-01.md"),
        "---\nid: TASK-01\ntitle: Tarefa\nstatus: done\n---\n\n# TASK-01: Tarefa\n\n## Execução\n\n### Evidências\n\n- `saida.txt` (automática): `echo depois`\n\n## Registro de execução\n\n| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n| 1 | 2026-10-14 | conclusão | done | ok |\n",
    )
    .unwrap();
    let evidence = tasks.join("evidence/TASK-01");
    fs::create_dir_all(&evidence).unwrap();
    fs::write(evidence.join("saida.txt"), "saida depois <ok>\n").unwrap();
    if let Some(output) = baseline {
        let dir = evidence.join("baseline/evidence");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("saida.txt"), output).unwrap();
    }
    project
        .vostok()
        .args(["finalize", "create", "SPEC-001"])
        .assert()
        .success();
    let content_path = project.harness().join("work/finalize/SPEC-001/content.md");
    let content = fs::read_to_string(&content_path).unwrap();
    let content = content
        .replace("#{EVIDENCE_01_HOW}", "rodar.")
        .replace("#{EVIDENCE_01_READ}", "ler.");
    fs::write(&content_path, fill_variables(&content)).unwrap();
    let output = project
        .vostok()
        .args(["finalize", "build", "SPEC-001"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    fs::read_to_string(content_path.with_file_name("evidencias.html")).unwrap()
}

#[test]
fn evidence_before_after_shows_the_baseline_output_before_the_final_one() {
    let html = evidence_html_with_baseline(Some("$ echo antes\nexit: 0\nsaida antes <x>\n"));
    let before = html.find("Antes da tarefa").expect(&html);
    let after = html.find("Depois da tarefa").expect(&html);
    assert!(before < after);
    assert!(html.contains("saida antes &lt;x&gt;"), "{html}");
    assert!(!html.contains("$ echo antes"), "{html}");
    assert!(html.find("saida antes").unwrap() < after);
    assert!(html.contains("saida depois &lt;ok&gt;"), "{html}");
    assert!(html.rfind("saida depois").unwrap() > after);
}

#[test]
fn evidence_before_after_shows_only_the_final_output_without_baseline() {
    let html = evidence_html_with_baseline(None);
    assert!(!html.contains("Antes da tarefa"), "{html}");
    assert!(!html.contains("Depois da tarefa"), "{html}");
    assert!(html.contains("saida depois &lt;ok&gt;"), "{html}");
}

/// Troca cada variável `#{...}` por um texto válido para o tipo da seção.
fn fill_variables(content: &str) -> String {
    let mut out = String::new();
    for line in content.lines() {
        if let Some(start) = line.find("#{") {
            let name = &line[start + 2..line.find('}').unwrap()];
            out.push_str(match name {
                "EXEC_DELIVERIES" => "### Aceite\n- **Impacto:** alto\n- **O que mudou:** aceite.\n- **Valor:** conclusão.\n- **Como perceber:** tarefa concluída.",
                _ => "Texto.",
            });
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}
