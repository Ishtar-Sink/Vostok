//! Testes do grupo `finalize`: equivalência com `finalize-spec.sh` pelas fixtures (CT-10, CT-12) e
//! recusas fora delas.

mod common;

use std::fs;
use std::path::PathBuf;

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
        .replace("#{EVIDENCE_01_PROOF}", "a saída de CT-01.")
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

/// Projeto com a SPEC-001 implementada, a TASK-01 com duas evidências (`saida.txt`, automática, e
/// `tela.txt`, manual) gravadas e o conteúdo criado por `finalize create`.
fn two_evidence_project() -> Project {
    let project = Project::new();
    write_spec(&project, "implemented");
    let tasks = project.harness().join("work/tasks/SPEC-001");
    fs::create_dir_all(&tasks).unwrap();
    fs::write(
        tasks.join("TASK-01.md"),
        "---\nid: TASK-01\ntitle: Tarefa\nstatus: done\n---\n\n# TASK-01: Tarefa\n\n## Execução\n\n### Evidências\n\n- `saida.txt` (automática): `echo depois`\n- `tela.txt` (manual): anotação\n\n## Registro de execução\n\n| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n| 1 | 2026-10-14 | conclusão | done | ok |\n",
    )
    .unwrap();
    let evidence = tasks.join("evidence/TASK-01");
    fs::create_dir_all(&evidence).unwrap();
    fs::write(evidence.join("saida.txt"), "depois\n").unwrap();
    fs::write(evidence.join("tela.txt"), "anotação\n").unwrap();
    project
        .vostok()
        .args(["finalize", "create", "SPEC-001"])
        .assert()
        .success();
    project
}

fn finalize_content_path(project: &Project) -> PathBuf {
    project.harness().join("work/finalize/SPEC-001/content.md")
}

fn fill_two_evidences(content: &str) -> String {
    let content = content
        .replace("#{EVIDENCE_01_PROOF}", "a saída de CT-01.")
        .replace("#{EVIDENCE_01_HOW}", "rodar o comando.")
        .replace("#{EVIDENCE_01_READ}", "ler a saída.")
        .replace("#{EVIDENCE_02_PROOF}", "a anotação de CT-02.")
        .replace("#{EVIDENCE_02_HOW}", "abrir a tela.")
        .replace("#{EVIDENCE_02_READ}", "ler a anotação.");
    fill_variables(&content)
}

fn build_html(project: &Project) -> String {
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
    fs::read_to_string(finalize_content_path(project).with_file_name("evidencias.html")).unwrap()
}

fn remove_block(content: &str, heading: &str) -> String {
    let mut inside = false;
    let mut out = String::new();
    for line in content.lines() {
        if line.starts_with("# ") || line.starts_with("## ") {
            inside = line == heading;
        }
        if !inside {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

#[test]
fn evidence_id_blocks_cite_the_id() {
    let project = two_evidence_project();
    let path = finalize_content_path(&project);
    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content
            .lines()
            .any(|line| line == "## TASK-01 / EV-01 / saida.txt"),
        "{content}"
    );
    assert!(
        content
            .lines()
            .any(|line| line == "## TASK-01 / EV-02 / tela.txt"),
        "{content}"
    );
    fs::write(&path, fill_two_evidences(&content)).unwrap();
    let html = build_html(&project);
    assert!(html.contains("EV-01 saida.txt"), "{html}");
    assert!(html.contains("EV-02 tela.txt"), "{html}");
}

#[test]
fn evidence_id_legacy_block_still_validates() {
    let project = two_evidence_project();
    let path = finalize_content_path(&project);
    let content = fill_two_evidences(&fs::read_to_string(&path).unwrap())
        .replace("## TASK-01 / EV-01 / saida.txt", "## TASK-01 / saida.txt");
    fs::write(&path, content).unwrap();
    let html = build_html(&project);
    assert!(html.contains("rodar o comando."), "{html}");
}

#[test]
fn evidence_id_missing_block_names_the_new_heading() {
    let project = two_evidence_project();
    let path = finalize_content_path(&project);
    let content = fill_two_evidences(&fs::read_to_string(&path).unwrap());
    fs::write(
        &path,
        remove_block(&content, "## TASK-01 / EV-02 / tela.txt"),
    )
    .unwrap();
    let stderr = failure_stderr(project.vostok().args(["finalize", "validate", "SPEC-001"]));
    assert!(
        stderr.contains("evidência sem bloco: ## TASK-01 / EV-02 / tela.txt"),
        "{stderr}"
    );
}

/// Troca cada variável `#{...}` por um texto válido para o tipo da seção.
fn fill_variables(content: &str) -> String {
    let mut out = String::new();
    for line in content.lines() {
        if let Some(start) = line.find("#{") {
            let name = &line[start + 2..line.find('}').unwrap()];
            out.push_str(match name {
                "EXEC_NUMBERS" => "- **2** tarefas concluídas",
                "EXEC_VALUE_INDICATOR" => "Não se aplica.",
                "TECH_NUMBERS" => "Não se aplica.",
                "TECH_OPERATION" => "1. Rodar `vostok finalize build`.",
                "TECH_CODE_MAP" => "| Caminho | Papel | Testes |\n|---|---|---|\n| `src/novo.rs` | novo | `novo_` |",
                "EVIDENCE_COVERAGE" => "| ID | O que foi verificado | Evidência |\n|---|---|---|\n| CT-01 | saída | TASK-01 / saida.txt |",
                _ => "Texto.",
            });
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

// --- Antes de evidência não executada ou interrompida (SPEC-022) ---

#[test]
fn before_skipped_shows_the_not_executed_line() {
    let html = evidence_html_with_baseline(Some(
        "$ echo antes\nexit: 1\nnão executada: novo.txt ainda não existe\n",
    ));
    let before = html.find("Antes da tarefa").expect(&html);
    let line = html
        .find("não executada: novo.txt ainda não existe")
        .expect(&html);
    assert!(before < line && line < html.find("Depois da tarefa").unwrap());
}

#[test]
fn before_skipped_shows_the_interrupted_line() {
    let html = evidence_html_with_baseline(Some(
        "$ echo antes\nexit: 124\nparcial <x>\ninterrompida após 120 s\n",
    ));
    let line = html.find("interrompida após 120 s").expect(&html);
    assert!(html.contains("parcial &lt;x&gt;"), "{html}");
    assert!(line < html.find("Depois da tarefa").unwrap());
}

// --- Tarefas em pasta (SPEC-023) ---

fn write_folder_task(project: &Project, id: &str) -> std::path::PathBuf {
    let folder = project.harness().join("work/tasks/SPEC-001").join(id);
    fs::create_dir_all(folder.join("evidencias")).unwrap();
    let files = [
        (
            "estado.yml",
            format!("id: {id}\ntitle: Tarefa em pasta\nstatus: done\n"),
        ),
        (
            "execucao.md",
            "### Ler\n\n- `a.rs`\n\n### Mudar\n\n- `src/novo.rs` (criar): novo\n\n### Passos\n\n1. Fazer.\n".to_string(),
        ),
        (
            "verificacao.md",
            "### Baseline\n\n- `true`\n\n### Verificação\n\n```bash\ntrue\n```\n\n### Evidências\n\n- `saida.txt` (automática): `echo depois`\n".to_string(),
        ),
        ("criterios.md", "### Concluída quando\n\n- [x] Feito.\n".to_string()),
        ("executivo.md", "Resumo <executivo> da pasta.\n".to_string()),
        ("tecnico.md", "Detalhe **técnico** com <tag> e `x`.\n".to_string()),
        (
            "registro.md",
            "## Registro de execução\n\n| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n| 1 | 2026-10-14 | conclusão | done | ok |\n".to_string(),
        ),
        ("evidencias/saida.txt", "saida da pasta <ok>\n".to_string()),
    ];
    for (name, text) in files {
        fs::write(folder.join(name), text).unwrap();
    }
    folder
}

/// Monta uma spec com uma tarefa em pasta e roda `create` e `build`.
fn built_folder_project() -> Project {
    built_folder_project_with(|_| {})
}

/// Como `built_folder_project`, com `prepare` mexendo na pasta da tarefa antes do `create`.
fn built_folder_project_with(prepare: impl FnOnce(&std::path::Path)) -> Project {
    let project = filled_folder_project(prepare);
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
    project
}

/// Como `built_folder_project_with`, parando antes do `build`, com o conteúdo criado e preenchido.
fn filled_folder_project(prepare: impl FnOnce(&std::path::Path)) -> Project {
    let project = Project::new();
    write_spec(&project, "implemented");
    prepare(&write_folder_task(&project, "TASK-01"));
    project
        .vostok()
        .args(["finalize", "create", "SPEC-001"])
        .assert()
        .success();
    let content_path = project.harness().join("work/finalize/SPEC-001/content.md");
    let content = fs::read_to_string(&content_path)
        .unwrap()
        .replace("#{EVIDENCE_01_PROOF}", "a saída de CT-01.")
        .replace("#{EVIDENCE_01_HOW}", "rodar.")
        .replace("#{EVIDENCE_01_READ}", "ler.");
    fs::write(&content_path, fill_variables(&content)).unwrap();
    project
}

#[test]
fn finalize_folder_sources_lists_the_executive_and_the_technical_of_each_task() {
    let project = Project::new();
    write_spec(&project, "implemented");
    write_folder_task(&project, "TASK-01");
    let output = project
        .vostok()
        .args(["finalize", "sources", "SPEC-001"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("### TASK-01: Tarefa em pasta"), "{stdout}");
    assert!(stdout.contains("- **Status:** done"), "{stdout}");
    assert!(
        stdout.contains("**Resumo executivo**\n\nResumo <executivo> da pasta."),
        "{stdout}"
    );
    assert!(
        stdout.contains("**Detalhes técnicos**\n\nDetalhe **técnico** com <tag> e `x`."),
        "{stdout}"
    );
    assert!(
        stdout.contains("| 1 | 2026-10-14 | conclusão | done | ok |"),
        "{stdout}"
    );
    assert!(
        stdout.contains("evidencias/saida.txt` (automática): existe"),
        "{stdout}"
    );
}

#[test]
fn finalize_folder_build_puts_the_technical_of_each_task_in_the_technical_document() {
    let project = built_folder_project();
    let dir = project.harness().join("work/finalize/SPEC-001");
    let technical = fs::read_to_string(dir.join("tecnico.html")).unwrap();
    assert!(
        technical.contains("<h3>TASK-01: Tarefa em pasta</h3>"),
        "{technical}"
    );
    assert!(
        technical.contains("<strong>técnico</strong>"),
        "{technical}"
    );
    assert!(technical.contains("&lt;tag&gt;"), "{technical}");
    assert!(!technical.contains("<tag>"), "{technical}");
    let evidence = fs::read_to_string(dir.join("evidencias.html")).unwrap();
    assert!(evidence.contains("saida da pasta &lt;ok&gt;"), "{evidence}");
}

#[test]
fn finalize_folder_archive_moves_the_task_folders_with_their_evidence() {
    let project = built_folder_project();
    project
        .vostok()
        .args(["finalize", "archive", "SPEC-001"])
        .assert()
        .success();
    let tasks = project.harness().join("work/tasks/SPEC-001");
    assert!(!tasks.exists());
    let archived = project
        .harness()
        .join("work/.archive/SPEC-001/tasks/TASK-01");
    assert!(archived.join("estado.yml").is_file());
    assert!(archived.join("tecnico.md").is_file());
    assert!(archived.join("evidencias/saida.txt").is_file());
}

#[test]
fn finalize_folder_mixed_formats_are_refused() {
    let project = Project::new();
    write_spec(&project, "implemented");
    write_folder_task(&project, "TASK-01");
    write_task(&project, "TASK-02", "done");
    let stderr = failure_stderr(project.vostok().args(["finalize", "sources", "SPEC-001"]));
    assert!(stderr.contains("tarefas nos dois formatos"), "{stderr}");
}

/// Os três documentos gerados de uma spec com uma tarefa em pasta.
fn built_documents(project: &Project) -> Vec<(&'static str, String)> {
    let dir = project.harness().join("work/finalize/SPEC-001");
    ["executivo.html", "tecnico.html", "evidencias.html"]
        .into_iter()
        .map(|name| (name, fs::read_to_string(dir.join(name)).unwrap()))
        .collect()
}

/// Cores hexadecimais (`#abc`, `#aabbcc` ou `#aabbccdd`) de um texto, em minúsculas.
fn hex_colors(text: &str) -> Vec<String> {
    let mut colors = Vec::new();
    for (index, _) in text.match_indices('#') {
        let rest = &text[index + 1..];
        let digits = rest.bytes().take_while(u8::is_ascii_hexdigit).count();
        let next = rest[digits..].chars().next();
        let ends = next.is_none_or(|c| !(c.is_alphanumeric() || c == '-' || c == '_'));
        if [3, 6, 8].contains(&digits) && ends {
            colors.push(rest[..digits].to_ascii_lowercase());
        }
    }
    colors
}

#[test]
fn visual_documents_default_to_the_light_theme() {
    let project = built_folder_project();
    for (name, html) in built_documents(&project) {
        assert!(html.contains("<html lang=\"pt-BR\">"), "{name}");
        assert!(html.contains(":root {\n  color-scheme: light;"), "{name}");
        assert!(
            html.contains(":root[data-theme=\"dark\"] {\n  color-scheme: dark;"),
            "{name}"
        );
        assert!(html.contains("prefers-color-scheme: dark"), "{name}");
    }
}

#[test]
fn visual_documents_have_no_external_requests() {
    let project = built_folder_project();
    for (name, html) in built_documents(&project) {
        assert!(!html.contains("http://"), "{name}");
        assert!(!html.contains("https://"), "{name}");
        assert!(!html.contains("<link"), "{name}");
        assert_eq!(
            html.matches("src: url(data:font/woff2;base64,").count(),
            2,
            "{name}: Inter e Outfit embutidas"
        );
    }
}

#[test]
fn visual_task_technical_headings_render_as_headings() {
    let project = built_folder_project_with(|folder| {
        fs::write(
            folder.join("tecnico.md"),
            "Detalhe inicial.\n\n## O que foi feito\n\nTexto.\n\n### Parte\n\n```rust\nlet x = 1 < 2;\n```\n\n| A | B |\n|---|---|\n| 1 | 2 |\n",
        )
        .unwrap();
    });
    let technical = &built_documents(&project)[1].1;
    assert!(
        technical.contains("<h4>O que foi feito</h4>"),
        "{technical}"
    );
    assert!(technical.contains("<h5>Parte</h5>"), "{technical}");
    assert!(
        technical.contains("<pre><code>let x = 1 &lt; 2;</code></pre>"),
        "{technical}"
    );
    assert!(technical.contains("<th>A</th>"), "{technical}");
    assert!(!technical.contains("## O que foi feito"), "{technical}");
    assert!(!technical.contains("```"), "{technical}");
}

#[test]
fn visual_colors_come_only_from_documents_yml() {
    let project = built_folder_project();
    let config = fs::read_to_string(project.harness().join("config/documents.yml")).unwrap();
    let allowed = hex_colors(&config);
    assert!(!allowed.is_empty());
    for (name, html) in built_documents(&project) {
        for color in hex_colors(&html) {
            assert!(
                allowed.contains(&color),
                "{name}: #{color} fora de documents.yml"
            );
        }
    }
}

/// Spec com uma tarefa em pasta e o conteúdo criado e preenchido, sem `build`; devolve o caminho do conteúdo.
fn filled_content(project: &Project) -> std::path::PathBuf {
    write_folder_task(project, "TASK-01");
    project
        .vostok()
        .args(["finalize", "create", "SPEC-001"])
        .assert()
        .success();
    let path = project.harness().join("work/finalize/SPEC-001/content.md");
    let content = fs::read_to_string(&path)
        .unwrap()
        .replace("#{EVIDENCE_01_PROOF}", "a saída de CT-01.")
        .replace("#{EVIDENCE_01_HOW}", "rodar.")
        .replace("#{EVIDENCE_01_READ}", "ler.");
    fs::write(&path, fill_variables(&content)).unwrap();
    path
}

/// Troca o texto da seção `## <title>` (a primeira com esse título) por `text`.
fn set_section(path: &std::path::Path, title: &str, text: &str) {
    let content = fs::read_to_string(path).unwrap();
    let heading = format!("## {title}\n");
    let start = content.find(&heading).unwrap() + heading.len();
    let end = content[start..]
        .find("\n#")
        .map_or(content.len(), |offset| start + offset + 1);
    let updated = format!("{}\n{text}\n\n{}", &content[..start], &content[end..]);
    fs::write(path, updated).unwrap();
}

fn validate_stderr(project: &Project) -> String {
    failure_stderr(project.vostok().args(["finalize", "validate", "SPEC-001"]))
}

#[test]
fn content_validate_refuses_an_empty_new_section() {
    let project = Project::new();
    write_spec(&project, "implemented");
    let path = filled_content(&project);
    set_section(&path, "Objetivo", "<!-- vazio -->");
    let stderr = validate_stderr(&project);
    assert!(stderr.contains("seção vazia: ## Objetivo"), "{stderr}");
}

#[test]
fn content_validate_refuses_code_in_the_executive() {
    let project = Project::new();
    write_spec(&project, "implemented");
    let path = filled_content(&project);
    set_section(&path, "Valor e impacto", "Ficou em `src/novo.rs`.");
    let stderr = validate_stderr(&project);
    assert!(
        stderr
            .contains("código no executivo (caminho, comando ou função): Ficou em `src/novo.rs`."),
        "{stderr}"
    );
}

#[test]
fn content_validate_requires_every_requirement_in_the_coverage_map() {
    let project = Project::new();
    let refined = project.harness().join("work/specs/refined");
    fs::create_dir_all(&refined).unwrap();
    fs::write(
        refined.join("SPEC-001-exemplo.md"),
        "---\nid: SPEC-001\ntitle: Exemplo\nstatus: implemented\n---\n\n# Exemplo\n\n| ID | Requisito |\n|---|---|\n| RF-01 | Um. |\n| RF-10 | Dez. |\n\n| ID | Requisito | Verificação |\n|---|---|---|\n| RNF-01 | Rápido. | Medir. |\n",
    )
    .unwrap();
    let path = filled_content(&project);
    set_section(
        &path,
        "Mapa de cobertura",
        "| ID | O que foi verificado | Evidência |\n|---|---|---|\n| RF-10 | dez | TASK-01 / saida.txt |",
    );
    let stderr = validate_stderr(&project);
    assert!(
        stderr.contains("Mapa de cobertura (Evidências): falta RF-01"),
        "{stderr}"
    );
    assert!(
        stderr.contains("Mapa de cobertura (Evidências): falta RNF-01"),
        "{stderr}"
    );
    assert!(!stderr.contains("falta RF-10"), "{stderr}");
}

#[test]
fn content_executive_has_the_new_sections() {
    let project = built_folder_project();
    let executive = &built_documents(&project)[0].1;
    for title in [
        "Objetivo",
        "O que foi entregue",
        "Valor e impacto",
        "Ganhos secundários",
        "Riscos e limites",
        "Próximos passos",
    ] {
        assert!(executive.contains(&format!("<h2>{title}</h2>")), "{title}");
    }
    assert!(!executive.contains("<h2>Entregas</h2>"), "{executive}");
}

#[test]
fn content_technical_has_the_support_sections() {
    let project = built_folder_project();
    let technical = &built_documents(&project)[1].1;
    for title in [
        "Contexto",
        "Como funciona",
        "Decisões",
        "Operação e diagnóstico",
        "Mapa do código",
        "Limites e dívidas",
    ] {
        assert!(technical.contains(&format!("<h2>{title}</h2>")), "{title}");
    }
    assert!(
        technical.contains("<td><code>src/novo.rs</code></td>"),
        "{technical}"
    );
    assert!(
        technical.contains("<li>Rodar <code>vostok finalize build</code>.</li>"),
        "{technical}"
    );
}

#[test]
fn content_numbers_render_in_the_summary_band() {
    let project = built_folder_project();
    let documents = built_documents(&project);
    let executive = &documents[0].1;
    assert!(
        executive.contains("<div class=\"fact\"><b>2</b><span>tarefas concluídas</span></div>"),
        "{executive}"
    );
    assert!(!executive.contains("<h2>Em números</h2>"), "{executive}");
    let technical = &documents[1].1;
    assert!(!technical.contains("class=\"facts\""), "{technical}");
    let evidence = &documents[2].1;
    assert!(
        evidence.contains("<b>Prova</b>a saída de CT-01."),
        "{evidence}"
    );
    assert!(evidence.contains("id=\"p-cobertura\""), "{evidence}");
}

/// Projeto pronto para o `build`, com o grupo `brand` acrescentado ao `documents.yml` e os arquivos
/// dados gravados na raiz do projeto.
fn project_with_brand(brand: &str, files: &[(&str, Vec<u8>)]) -> Project {
    let project = filled_folder_project(|_| {});
    let config = project.harness().join("config/documents.yml");
    let text = fs::read_to_string(&config).unwrap();
    let mut in_brand = false;
    let mut text: String = text
        .lines()
        .filter(|line| {
            if !line.starts_with([' ', '\t']) && !line.trim().is_empty() {
                in_brand = line.starts_with("brand:");
            }
            !in_brand
        })
        .map(|line| format!("{line}\n"))
        .collect();
    text.push_str(&format!("\nbrand:\n{brand}"));
    fs::write(&config, text).unwrap();
    for (name, bytes) in files {
        fs::write(project.root().join(name), bytes).unwrap();
    }
    project
}

fn build_succeeds(project: &Project) -> Vec<(&'static str, String)> {
    project
        .vostok()
        .args(["finalize", "build", "SPEC-001"])
        .assert()
        .success();
    built_documents(project)
}

fn small_svg() -> Vec<u8> {
    let mut svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 16 16\"><circle cx=\"8\" cy=\"8\" r=\"7\"/>".to_vec();
    svg.extend(std::iter::repeat_n(b' ', 900));
    svg.extend(b"</svg>");
    svg
}

#[test]
fn brand_name_replaces_vostok_in_the_header() {
    let project = project_with_brand("  name: \"Atria & Co\"\n", &[]);
    for (name, html) in build_succeeds(&project) {
        assert!(
            html.contains("<span class=\"brand\"><i></i>Atria &amp; Co</span>"),
            "{name}"
        );
        assert!(!html.contains("<i></i>Vostok</span>"), "{name}");
        assert!(html.contains("Vostok"), "{name}: o rodapé cita o Vostok");
    }
}

#[test]
fn brand_icon_is_embedded_in_the_header() {
    let project = project_with_brand("  icon: \"logo.svg\"\n", &[("logo.svg", small_svg())]);
    for (name, html) in build_succeeds(&project) {
        assert!(
            html.contains("<img class=\"brand-mark\" alt=\"\" src=\"data:image/svg+xml;base64,"),
            "{name}"
        );
        assert!(!html.contains("<span class=\"brand\"><i></i>"), "{name}");
        assert!(!html.contains("https://"), "{name}");
        assert!(!html.contains("http://www.w3.org/2000/svg\""), "{name}");
    }
}

#[test]
fn brand_missing_icon_fails_the_build() {
    let project = project_with_brand("  icon: \"sumiu.svg\"\n", &[]);
    let stderr = failure_stderr(project.vostok().args(["finalize", "build", "SPEC-001"]));
    assert!(stderr.contains("sumiu.svg"), "{stderr}");
}

#[test]
fn brand_large_or_unknown_icon_fails_the_build() {
    let project = project_with_brand(
        "  icon: \"grande.png\"\n",
        &[("grande.png", vec![0u8; 300 * 1024])],
    );
    let stderr = failure_stderr(project.vostok().args(["finalize", "build", "SPEC-001"]));
    assert!(stderr.contains("grande.png"), "{stderr}");
    assert!(stderr.contains("256 KB"), "{stderr}");

    let project = project_with_brand("  icon: \"logo.gif\"\n", &[("logo.gif", vec![0u8; 100])]);
    let stderr = failure_stderr(project.vostok().args(["finalize", "build", "SPEC-001"]));
    assert!(stderr.contains("logo.gif"), "{stderr}");
    assert!(stderr.contains("svg, png, webp"), "{stderr}");
}

#[test]
fn brand_empty_keeps_the_vostok_identity() {
    let without_group = built_folder_project();
    let empty_group = project_with_brand("  name: \"\"\n  icon: \"\"\n", &[]);
    let documents = built_documents(&without_group)
        .into_iter()
        .chain(build_succeeds(&empty_group));
    for (name, html) in documents {
        assert!(
            html.contains("<span class=\"brand\"><i></i>Vostok</span>"),
            "{name}"
        );
    }
}

#[test]
fn coverage_ids_do_not_wrap() {
    let project = built_folder_project();
    let evidence = &built_documents(&project)[2].1;
    assert!(
        evidence.contains(
            "#p-cobertura th:first-child, #p-cobertura td:first-child { white-space: nowrap; }"
        ),
        "{evidence}"
    );
}

#[test]
fn archive_records_archived_in_the_status_history() {
    let project = built_folder_project();
    project
        .vostok()
        .args(["finalize", "archive", "SPEC-001"])
        .assert()
        .success();
    let content =
        fs::read_to_string(project.harness().join("work/.archive/SPEC-001/spec.md")).unwrap();
    let history = content
        .lines()
        .find(|line| line.starts_with("status_history: "))
        .unwrap_or_else(|| panic!("sem status_history: {content}"));
    assert!(history.ends_with(" archived"), "{history}");
}

#[test]
fn chat_aval_archive_without_aval_file() {
    let project = built_folder_project();
    let mut config = fs::OpenOptions::new()
        .append(true)
        .open(project.harness().join("config/main.yml"))
        .unwrap();
    std::io::Write::write_all(&mut config, b"approval_lock: true\n").unwrap();
    project
        .vostok()
        .args(["finalize", "archive", "SPEC-001"])
        .assert()
        .success();
    let archived = project.harness().join("work/.archive/SPEC-001");
    assert!(archived.join("spec.md").exists());
    assert!(!archived.join("pedidos").exists());
    assert!(!project.harness().join("work/avais").exists());
}

// --- Métricas da finalização (SPEC-055) ---

const CYCLE: &str = "created_at: 2026-11-13T22:29:56Z\napproved_at: 2026-11-13T22:37:19Z\nstatus_history: 2026-11-13T22:37:19Z approved; 2026-11-13T22:49:59Z planned; 2026-11-14T00:26:44Z implemented\n";

fn write_metrics_spec(project: &Project, frontmatter: &str) {
    let refined = project.harness().join("work/specs/refined");
    fs::create_dir_all(&refined).unwrap();
    fs::write(
        refined.join("SPEC-001-exemplo.md"),
        format!("---\nid: SPEC-001\ntitle: Exemplo\nstatus: implemented\n{frontmatter}---\n\n# Exemplo\n\n## Critérios de aceite\n\n- [ ] CA-01: Um.\n- [ ] CA-02: Dois.\n"),
    )
    .unwrap();
}

fn metrics_task(
    project: &Project,
    id: &str,
    wave: u32,
    evidences: &[&str],
    rows: &str,
    reviews: &str,
) -> PathBuf {
    let folder = write_folder_task(project, id);
    fs::write(
        folder.join("estado.yml"),
        format!("id: {id}\ntitle: Tarefa em pasta\nstatus: done\nwave: {wave}\nattempts: 0\n"),
    )
    .unwrap();
    let mut verification = String::from(
        "### Baseline\n\n- `true`\n\n### Verificação\n\n```bash\ntrue\n```\n\n### Evidências\n\n",
    );
    for name in evidences {
        verification.push_str(&format!("- `{name}` (automática): `echo x`\n"));
        fs::write(folder.join("evidencias").join(name), "x\n").unwrap();
    }
    fs::write(folder.join("verificacao.md"), verification).unwrap();
    if !evidences.contains(&"saida.txt") {
        fs::remove_file(folder.join("evidencias/saida.txt")).unwrap();
    }
    fs::write(
        folder.join("registro.md"),
        format!("## Registro de execução\n\n| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n| 1 | 2026-11-13 | conclusão | done | ok |\n{rows}"),
    )
    .unwrap();
    if !reviews.is_empty() {
        fs::write(
            folder.join("conferencias.md"),
            format!("| ID | Arquivo | Estado | sha256 | Data | Origem | Texto |\n|---|---|---|---|---|---|---|\n{reviews}"),
        )
        .unwrap();
    }
    folder
}

fn coverage(project: &Project, rows: &str) {
    project
        .vostok()
        .args(["finalize", "create", "SPEC-001"])
        .assert()
        .success();
    set_section(
        &finalize_content_path(project),
        "Mapa de cobertura",
        &format!("| ID | O que foi verificado | Evidência |\n|---|---|---|\n{rows}"),
    );
}

fn metrics_output(project: &Project) -> String {
    let output = project
        .vostok()
        .args(["finalize", "sources", "SPEC-001"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    String::from_utf8(output.stdout).unwrap()
}

fn metrics_block(project: &Project) -> String {
    let out = metrics_output(project);
    let mut lines = out.lines().skip_while(|line| *line != "## Métricas");
    let mut block = vec![lines.next().expect(&out).to_string()];
    block.push(lines.next().unwrap_or("").to_string());
    block.extend(
        lines
            .take_while(|line| !line.is_empty())
            .map(str::to_string),
    );
    block.join("\n")
}

const GREEN_REVIEWS_1: &str = "| EV-01 | saida.txt | conferida | aaaa | 2026-11-13T23:00:00Z | painel |  |\n| EV-02 | tela.png | conferida | bbbb | 2026-11-13T23:01:00Z | painel |  |\n";
const GREEN_REVIEWS_2: &str =
    "| EV-01 | fluxo.webm | conferida | cccc | 2026-11-13T23:02:00Z | painel |  |\n";
const MET_ALL: &str =
    "| CA-01 | Atendido: um | TASK-01 / saida.txt |\n| CA-02 | Atendido: dois | TASK-02 / fluxo.webm |\n";

fn green_project_with(task1_rows: &str, task1_attempts: u32, map: &str) -> Project {
    let project = Project::new();
    write_metrics_spec(&project, CYCLE);
    let folder = metrics_task(
        &project,
        "TASK-01",
        1,
        &["saida.txt", "tela.png"],
        &format!("| 2 | 2026-11-13 | ajuste | in_progress | passo 2 refeito |\n{task1_rows}"),
        GREEN_REVIEWS_1,
    );
    fs::write(
        folder.join("estado.yml"),
        format!("id: TASK-01\ntitle: Tarefa em pasta\nstatus: done\nwave: 1\nattempts: {task1_attempts}\n"),
    )
    .unwrap();
    metrics_task(&project, "TASK-02", 2, &["fluxo.webm"], "", GREEN_REVIEWS_2);
    coverage(&project, map);
    project
}

fn green_project() -> Project {
    green_project_with("", 0, MET_ALL)
}

#[test]
fn metrics_sources_prints_the_block_in_the_contract_format() {
    let project = green_project();
    let expected = [
        "## Métricas",
        "",
        "- Ciclo: criada 2026-11-13 22:29, aprovada 22:37, implementada 2026-11-14 00:26",
        "- Da aprovação à implementação: 1 h 49 min",
        "- Tarefas: 2 em 2 ondas",
        "- Retrabalho: 1 ajuste em execução, 0 tentativas extras, 0 bloqueios, 0 recusas",
        "- Critérios de aceite: 2 de 2 atendidos",
        "- Evidências: 3 (1 print, 1 texto, 1 vídeo); 3 conferidas, 0 recusadas",
        "- Volume: fora de um repositório git",
        "- Situação: verde",
    ]
    .join("\n");
    assert_eq!(metrics_block(&project), expected);
    let out = metrics_output(&project);
    let spec = out.find("## Spec").unwrap();
    let metrics = out.find("## Métricas").unwrap();
    let tasks = out.find("## Tarefas").unwrap();
    assert!(spec < metrics && metrics < tasks, "{out}");
}

#[test]
fn metrics_blocked_and_resumed_task_is_amber() {
    let project = green_project_with(
        "| 3 | 2026-11-13 | testes | blocked | 2 falhas |\n| 4 | 2026-11-13 | retomada | validating | decisão do usuário |\n",
        2,
        MET_ALL,
    );
    let block = metrics_block(&project);
    assert!(
        block.contains(
            "- Retrabalho: 1 ajuste em execução, 2 tentativas extras, 1 bloqueio, 0 recusas"
        ),
        "{block}"
    );
    assert!(block.contains("- Situação: âmbar"), "{block}");
}

#[test]
fn metrics_refused_evidence_is_amber() {
    let project = green_project_with(
        "| 3 | 2026-11-13 | conferência | usuário | EV-02 recusada no chat |\n",
        0,
        MET_ALL,
    );
    let folder = project.harness().join("work/tasks/SPEC-001/TASK-02");
    let record = folder.join("registro.md");
    let mut text = fs::read_to_string(&record).unwrap();
    text.push_str(
        "| 2 | 2026-11-12 | rejeição | blocked | evidência fluxo.webm rejeitada pelo usuário |\n",
    );
    fs::write(&record, text).unwrap();
    let block = metrics_block(&project);
    assert!(
        block.contains("0 tentativas extras, 1 bloqueio, 2 recusas"),
        "{block}"
    );
    assert!(block.contains("- Situação: âmbar"), "{block}");
}

#[test]
fn metrics_unmet_criterion_is_red() {
    let project = green_project_with(
        "| 3 | 2026-11-13 | testes | blocked | 2 falhas |\n",
        0,
        "| CA-01 | Atendido: um | x |\n| CA-02 | Não atendido: faltou o vídeo | x |\n",
    );
    let block = metrics_block(&project);
    assert!(
        block.contains("- Critérios de aceite: 1 de 2 atendidos, 1 não atendido"),
        "{block}"
    );
    assert!(block.contains("- Situação: vermelho"), "{block}");
}

#[test]
fn metrics_partial_criterion_is_amber() {
    let project = green_project_with(
        "",
        0,
        "| CA-01 | Atendido: um | x |\n| CA-02 | Parcial: só no tema claro | x |\n",
    );
    let block = metrics_block(&project);
    assert!(
        block.contains("- Critérios de aceite: 1 de 2 atendidos, 1 parcial"),
        "{block}"
    );
    assert!(block.contains("- Situação: âmbar"), "{block}");
}

#[test]
fn metrics_without_coverage_map_counts_criteria_to_check() {
    let project = Project::new();
    write_metrics_spec(&project, CYCLE);
    metrics_task(&project, "TASK-01", 1, &["saida.txt"], "", "");
    let block = metrics_block(&project);
    assert!(
        block.contains("- Critérios de aceite: 2 a conferir no mapa de cobertura"),
        "{block}"
    );
    assert!(block.contains("- Situação: âmbar"), "{block}");

    coverage(&project, "| CA-01 | Atendido: um | x |\n");
    let block = metrics_block(&project);
    assert!(
        block.contains("- Critérios de aceite: 1 de 2 atendidos, 1 a conferir"),
        "{block}"
    );
}

#[test]
fn metrics_cycle_without_history_shows_only_existing_dates() {
    let project = Project::new();
    write_metrics_spec(&project, "approved_at: 2026-11-13T22:37:19Z\n");
    metrics_task(&project, "TASK-01", 1, &["saida.txt"], "", "");
    let block = metrics_block(&project);
    assert!(
        block.contains("- Ciclo: aprovada 2026-11-13 22:37\n"),
        "{block}"
    );
    assert!(
        block.contains("- Da aprovação à implementação: sem registro de hora"),
        "{block}"
    );

    let project = Project::new();
    write_metrics_spec(&project, "");
    metrics_task(&project, "TASK-01", 1, &["saida.txt"], "", "");
    let block = metrics_block(&project);
    assert!(block.contains("- Ciclo: sem datas registradas"), "{block}");
}

#[test]
fn metrics_tasks_without_wave_and_evidence_to_check() {
    let project = Project::new();
    write_metrics_spec(&project, CYCLE);
    let folder = metrics_task(&project, "TASK-01", 1, &["saida.txt"], "", "");
    fs::write(
        folder.join("estado.yml"),
        "id: TASK-01\ntitle: Tarefa em pasta\nstatus: done\n",
    )
    .unwrap();
    let block = metrics_block(&project);
    assert!(block.contains("- Tarefas: 1\n"), "{block}");
    assert!(
        block.contains("- Evidências: 1 (1 texto); 0 conferidas, 0 recusadas, 1 a conferir"),
        "{block}"
    );
}

#[test]
fn metrics_volume_counts_commits_files_and_lines() {
    let project = green_project();
    let git = |dates: &str, args: &[&str]| {
        let status = std::process::Command::new("git")
            .arg("-C")
            .arg(project.root())
            .args(["-c", "user.name=v", "-c", "user.email=v@v"])
            .args(args)
            .env("GIT_AUTHOR_DATE", dates)
            .env("GIT_COMMITTER_DATE", dates)
            .status()
            .unwrap();
        assert!(status.success());
    };
    git("2026-11-13T22:00:00Z", &["init", "-q"]);
    git("2026-11-13T22:00:00Z", &["add", "-A"]);
    git("2026-11-13T22:00:00Z", &["commit", "-q", "-m", "base"]);
    fs::create_dir_all(project.root().join("src")).unwrap();
    fs::write(project.root().join("src/novo.rs"), "a\nb\nc\n").unwrap();
    git("2026-11-13T23:00:00Z", &["add", "-A"]);
    git("2026-11-13T23:00:00Z", &["commit", "-q", "-m", "novo"]);
    let block = metrics_block(&project);
    assert!(
        block.contains("- Volume: 1 commit, 1 arquivo, +3 / -0 linhas"),
        "{block}"
    );
}

#[test]
fn metrics_legacy_content_template_keeps_sources_without_the_block() {
    let project = green_project();
    fs::write(
        project.harness().join("templates/finalize/content.md"),
        "# Executivo\n\n## Em resumo\n\nTexto.\n",
    )
    .unwrap();
    assert!(!metrics_output(&project).contains("## Métricas"));
}

// --- Executivo como documento de decisão (SPEC-055) ---

// --- Evidências sem conferência exigida (SPEC-073) ---

/// Troca a chave `evidence_review_kinds` do `config/tasks.yml` do projeto; `None` a remove.
fn set_review_kinds(project: &Project, line: Option<&str>) {
    let path = project.harness().join("config/tasks.yml");
    let mut content: String = fs::read_to_string(&path)
        .unwrap()
        .lines()
        .filter(|row| !row.starts_with("evidence_review_kinds:"))
        .map(|row| format!("{row}\n"))
        .collect();
    if let Some(line) = line {
        content.push_str(line);
        content.push('\n');
    }
    fs::write(path, content).unwrap();
}

/// Spec com a TASK-01 de uma evidência `saida.txt` do tipo dado, a lista de tipos dada e o mapa preenchido.
fn review_free_project(kinds: Option<&str>, kind: &str, reviews: &str) -> Project {
    let project = Project::new();
    write_metrics_spec(&project, CYCLE);
    let folder = metrics_task(&project, "TASK-01", 1, &["saida.txt"], "", reviews);
    if kind != "automática" {
        let path = folder.join("verificacao.md");
        let text = fs::read_to_string(&path).unwrap().replace(
            "- `saida.txt` (automática): `echo x`",
            &format!("- `saida.txt` ({kind})"),
        );
        fs::write(path, text).unwrap();
    }
    set_review_kinds(&project, kinds);
    coverage(&project, MET_ALL);
    project
}

const AUTOMATIC_AND_PRINT: &str = "evidence_review_kinds: [automática, print]";
const FROM_VERIFY: &str = "automática, da verificação";

#[test]
fn review_free_verify_output_is_not_to_check() {
    let project = review_free_project(Some(AUTOMATIC_AND_PRINT), FROM_VERIFY, "");
    let block = metrics_block(&project);
    assert!(
        block.contains(
            "- Evidências: 1 (1 texto); 0 conferidas, 0 recusadas, 1 sem conferência exigida"
        ),
        "{block}"
    );
    assert!(!block.contains("a conferir"), "{block}");
}

#[test]
fn review_free_verify_output_in_the_executive_bar() {
    let project = review_free_project(Some(AUTOMATIC_AND_PRINT), FROM_VERIFY, "");
    let html = executive_html(&project, &DECISION_SECTIONS);
    for expected in [
        "<figcaption>0 de 0 evidências conferidas</figcaption>",
        "<li>1 sem conferência exigida</li>",
        "<rect class=\"seg skip\"",
        "aria-label=\"Evidências: 0 conferidas, 0 recusadas, 0 a conferir, 1 sem conferência exigida\"",
    ] {
        assert!(html.contains(expected), "{expected}\n{html}");
    }
}

#[test]
fn review_free_listed_kind_stays_to_check() {
    let project = review_free_project(Some(AUTOMATIC_AND_PRINT), "automática", "");
    let block = metrics_block(&project);
    assert!(
        block.contains("0 conferidas, 0 recusadas, 1 a conferir"),
        "{block}"
    );
    let html = executive_html(&project, &DECISION_SECTIONS);
    for expected in [
        "<li>0 sem conferência exigida</li>",
        "aria-label=\"Evidências: 0 conferidas, 0 recusadas, 1 a conferir, 0 sem conferência exigida\"",
    ] {
        assert!(html.contains(expected), "{expected}\n{html}");
    }
}

#[test]
fn review_free_missing_list_asks_for_every_kind() {
    let project = review_free_project(None, FROM_VERIFY, "");
    let block = metrics_block(&project);
    assert!(block.contains("1 a conferir"), "{block}");
    assert!(!block.contains("sem conferência exigida"), "{block}");
}

#[test]
fn review_free_empty_list_asks_for_nothing() {
    let project = review_free_project(Some("evidence_review_kinds: []"), "automática", "");
    let block = metrics_block(&project);
    assert!(block.contains("1 sem conferência exigida"), "{block}");
    assert!(!block.contains("a conferir"), "{block}");
}

#[test]
fn review_free_reviewed_anyway_counts_as_reviewed() {
    let project = review_free_project(
        Some(AUTOMATIC_AND_PRINT),
        FROM_VERIFY,
        "| EV-01 | saida.txt | conferida | 0 | 2026-11-13T22:00:00Z | painel |  |\n",
    );
    let block = metrics_block(&project);
    assert!(block.contains("1 conferidas, 0 recusadas"), "{block}");
    assert!(!block.contains("sem conferência exigida"), "{block}");
}

#[test]
fn review_free_quality_card_counts_only_the_reviewed_kinds() {
    let project = review_free_project(Some(AUTOMATIC_AND_PRINT), FROM_VERIFY, "");
    let html = executive_html(&project, &DECISION_SECTIONS);
    assert!(
        html.contains("0 de 0 evidências conferidas, 1 sem conferência exigida, 0 recusas"),
        "{html}"
    );
}

/// Preenche o conteúdo criado, aplica as seções dadas, gera os documentos e devolve o `executivo.html`.
fn executive_html(project: &Project, sections: &[(&str, &str)]) -> String {
    let path = finalize_content_path(project);
    let mut content = fs::read_to_string(&path).unwrap();
    for number in 1..=9 {
        for field in ["PROOF", "HOW", "READ"] {
            content = content.replace(&format!("#{{EVIDENCE_{number:02}_{field}}}"), "texto.");
        }
    }
    fs::write(&path, fill_variables(&content)).unwrap();
    for (title, text) in sections {
        set_section(&path, title, text);
    }
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
    fs::read_to_string(
        project
            .harness()
            .join("work/finalize/SPEC-001/executivo.html"),
    )
    .unwrap()
}

const DECISION_SECTIONS: [(&str, &str); 2] = [
    (
        "Resultado",
        "Quem usa o painel ajusta tudo em poucos cliques.",
    ),
    ("Decisão pedida", "Escolher o próximo grupo."),
];

#[test]
fn executive_green_spec_shows_badge_result_decision_and_cards() {
    let html = executive_html(&green_project(), &DECISION_SECTIONS);
    assert!(
        html.contains("<p class=\"badge green\" role=\"status\"><span aria-hidden=\"true\"></span>Situação: verde · Concluída sem pendências</p>"),
        "{html}"
    );
    assert!(
        html.contains("<p class=\"result\">Quem usa o painel ajusta tudo em poucos cliques.</p>"),
        "{html}"
    );
    assert!(
        html.contains("<p class=\"ask\"><b>Decisão pedida</b>Escolher o próximo grupo.</p>"),
        "{html}"
    );
    let titles: Vec<usize> = ["Resultado", "Tempo", "Qualidade", "Esforço"]
        .iter()
        .map(|title| {
            html.find(&format!("<h2 class=\"card-title\">{title}</h2>"))
                .unwrap_or_else(|| panic!("cartão {title}"))
        })
        .collect();
    assert!(
        titles.windows(2).all(|pair| pair[0] < pair[1]),
        "{titles:?}"
    );
    for value in ["1 h 49 min", "2 de 2", "2 tarefas"] {
        assert!(
            html.contains(&format!("<b class=\"card-value\">{value}</b>")),
            "{value}"
        );
    }
    assert!(html.find("class=\"badge green\"").unwrap() < html.find("class=\"summary\"").unwrap());
}

#[test]
fn executive_amber_and_red_badges_follow_the_rule() {
    let amber = green_project_with(
        "| 3 | 2026-11-13 | testes | blocked | 2 falhas |\n",
        0,
        MET_ALL,
    );
    let html = executive_html(&amber, &DECISION_SECTIONS);
    assert!(html.contains("class=\"badge amber\""), "{html}");
    assert!(
        html.contains("Situação: âmbar · Concluída com ressalvas"),
        "{html}"
    );
    assert!(html.contains("1 bloqueio"), "{html}");

    let red = green_project_with(
        "",
        0,
        "| CA-01 | Atendido: um | x |\n| CA-02 | Não atendido: faltou o vídeo | x |\n",
    );
    let html = executive_html(&red, &DECISION_SECTIONS);
    assert!(html.contains("class=\"badge red\""), "{html}");
    assert!(
        html.contains("Situação: vermelho · Critério de aceite não atendido"),
        "{html}"
    );
}

#[test]
fn executive_value_indicator_fills_the_result_card() {
    let table = "| Valor | Antes | Depois | Rótulo |\n|---|---|---|---|\n| 12 | 0 | 12 | configurações editáveis no painel |";
    let html = executive_html(&green_project(), &[("Indicador de valor", table)]);
    assert!(html.contains("<b class=\"card-value\">12</b>"), "{html}");
    assert!(html.contains("configurações editáveis no painel"), "{html}");
    assert!(html.contains("antes: 0 · depois: 12"), "{html}");

    let html = executive_html(
        &green_project(),
        &[
            ("Indicador de valor", "Não se aplica."),
            ("O que foi entregue", "**Página nova.** Abre pelo botão."),
        ],
    );
    assert!(
        html.contains("<b class=\"card-value\">Página nova</b>"),
        "{html}"
    );
    assert!(html.contains("sem métrica de valor nas fontes"), "{html}");
}

#[test]
fn executive_bars_are_inline_svg_with_text() {
    let html = executive_html(&green_project(), &DECISION_SECTIONS);
    for id in ["barra-ciclo", "barra-evidencias"] {
        let figure = html
            .split(&format!("<figure class=\"bar\" id=\"{id}\">"))
            .nth(1)
            .unwrap_or_else(|| panic!("{id}"));
        let figure = &figure[..figure.find("</figure>").unwrap()];
        assert!(figure.contains("<svg role=\"img\""), "{figure}");
        assert!(figure.contains("aria-label="), "{figure}");
    }
    assert!(
        html.contains("<figcaption>1 h 49 min da aprovação à implementação</figcaption>"),
        "{html}"
    );
    for mark in [
        "<li>criada 22:29</li>",
        "<li>aprovada 22:37</li>",
        "<li>planejada 22:49</li>",
        "<li>implementada 2026-11-14 00:26</li>",
    ] {
        assert!(html.contains(mark), "{mark}");
    }
    assert!(
        html.contains("aria-label=\"Evidências: 3 conferidas, 0 recusadas, 0 a conferir, 0 sem conferência exigida\""),
        "{html}"
    );
    assert!(!html.contains("http://") && !html.contains("https://"));
    assert!(!html.contains("<link") && !html.contains("<script src"));
}

const HISTORY_HEADER: &str =
    "\n## Histórico de ajustes\n\n| # | Data | Origem | Resumo |\n|---|---|---|---|\n";

fn append_to(path: &std::path::Path, text: &str) {
    let mut content = fs::read_to_string(path).unwrap();
    content.push_str(text);
    fs::write(path, content).unwrap();
}

fn spec_path(project: &Project) -> PathBuf {
    project
        .harness()
        .join("work/specs/refined/SPEC-001-exemplo.md")
}

#[test]
fn revisao_ajustes_contados_a_parte() {
    let project = green_project();
    let before = metrics_block(&project);
    let rework = before
        .lines()
        .find(|line| line.starts_with("- Retrabalho:"))
        .unwrap()
        .to_string();
    append_to(
        &spec_path(&project),
        &format!("{HISTORY_HEADER}| 1 | 2026-11-13 | revisor | requisito esclarecido |\n"),
    );
    let registro = project
        .harness()
        .join("work/tasks/SPEC-001/TASK-01/registro.md");
    append_to(
        &registro,
        &format!(
            "{HISTORY_HEADER}| 1 | 2026-11-13 | revisor | passo esclarecido |\n| 2 | 2026-11-13 | agente | ajuste pedido |\n"
        ),
    );
    let block = metrics_block(&project);
    assert!(block.contains("- Revisão: 2 ajustes do revisor"), "{block}");
    assert!(block.lines().any(|line| line == rework), "{block}");
    let html = executive_html(&project, &DECISION_SECTIONS);
    assert!(html.contains("<li>2 ajustes do revisor</li>"), "{html}");
}

#[test]
fn revisao_um_ajuste_no_singular() {
    let project = green_project();
    append_to(
        &spec_path(&project),
        &format!("{HISTORY_HEADER}| 1 | 2026-11-13 | revisor | requisito esclarecido |\n"),
    );
    let block = metrics_block(&project);
    assert!(block.contains("- Revisão: 1 ajuste do revisor"), "{block}");
}

#[test]
fn revisao_sem_ajustes_sem_linha() {
    let project = green_project();
    let block = metrics_block(&project);
    assert!(!block.contains("- Revisão:"), "{block}");
    let html = executive_html(&project, &DECISION_SECTIONS);
    assert!(!html.contains("do revisor"), "{html}");
}

#[test]
fn executive_cycle_bar_shows_only_existing_marks() {
    let project = Project::new();
    write_metrics_spec(&project, "approved_at: 2026-11-13T22:37:19Z\n");
    metrics_task(&project, "TASK-01", 1, &["saida.txt"], "", "");
    coverage(&project, MET_ALL);
    let html = executive_html(&project, &DECISION_SECTIONS);
    assert!(
        html.contains("<p class=\"na\">Sem horas suficientes para desenhar o ciclo.</p>"),
        "{html}"
    );
    assert!(
        !html.contains("id=\"barra-ciclo\"><figcaption>1 h"),
        "{html}"
    );
    assert!(html.contains("sem registro de hora"), "{html}");
}

#[test]
fn executive_execution_block_has_no_redundant_ratios() {
    let project = green_project();
    let sources = metrics_block(&project);
    let html = executive_html(&project, &DECISION_SECTIONS);
    let block = html.split("id=\"execucao\"").nth(1).unwrap();
    let block = &block[..block.find("</section>").unwrap()];
    assert!(!block.contains("<small> / "), "{block}");
    assert!(block.contains("<li>2 tarefas em 2 ondas</li>"), "{block}");
    let rework = sources
        .lines()
        .find_map(|line| line.strip_prefix("- Retrabalho: "))
        .unwrap();
    assert!(block.contains(&format!("<li>{rework}</li>")), "{block}");
}

#[test]
fn executive_validate_refuses_long_result_and_bad_indicator() {
    let project = green_project();
    let path = finalize_content_path(&project);
    let content = fs::read_to_string(&path).unwrap();
    fs::write(&path, fill_variables(&content)).unwrap();
    let long = vec!["palavra"; 31].join(" ");
    set_section(&path, "Resultado", &long);
    set_section(
        &path,
        "Indicador de valor",
        "| Valor | Antes |\n|---|---|\n| 1 | 2 |",
    );
    let stderr = validate_stderr(&project);
    assert!(
        stderr.contains("Resultado (Executivo): 31 palavras, o máximo é 30"),
        "{stderr}"
    );
    assert!(
        stderr.contains("Indicador de valor (Executivo): use a tabela \"| Valor | Antes | Depois | Rótulo |\" com uma linha, ou \"Não se aplica.\""),
        "{stderr}"
    );
}

// --- Documento de evidências com os campos da tarefa (SPEC-055) ---

const NEW_FIELDS: &str = "  - Título: Valor inválido é recusado\n  - Mostra: A CLI recusa o valor e não grava nada.\n  - Cobre: RF-01, CT-01\n  - Onde olhar: No fim do log, `1 passed`.\n  - Trecho: `1 passed`\n  - Falha: Se aparecer `FAILED`, o valor passou.\n  - Reproduzir: `cargo test -- recusa`\n";
const LOG: &str =
    "$ cargo test -- recusa\n# data: 2026-11-16T10:00:00Z\n# commit: abc1234\n1 passed\n";

/// Reescreve as evidências da tarefa em pasta com os subitens dados e grava o log da evidência.
fn evidence_task(subitems: &str, log: &str) -> impl FnOnce(&std::path::Path) {
    let subitems = subitems.to_string();
    let log = log.to_string();
    move |folder: &std::path::Path| {
        fs::write(
            folder.join("verificacao.md"),
            format!("### Baseline\n\n- `true`\n\n### Verificação\n\n```bash\ntrue\n```\n\n### Evidências\n\n- `saida.txt` (automática): `echo depois`\n{subitems}"),
        )
        .unwrap();
        fs::write(folder.join("evidencias/saida.txt"), log).unwrap();
    }
}

fn evidence_doc(subitems: &str, log: &str) -> (Project, String) {
    let project = built_folder_project_with(evidence_task(subitems, log));
    let html = built_documents(&project).remove(2).1;
    (project, html)
}

fn assert_in_order(html: &str, parts: &[&str]) {
    let mut from = 0;
    for part in parts {
        let at = html[from..]
            .find(part)
            .unwrap_or_else(|| panic!("faltou, na ordem, {part}:\n{html}"));
        from += at + part.len();
    }
}

#[test]
fn evidence_doc_uses_the_title_and_the_task_fields() {
    let (_project, html) = evidence_doc(NEW_FIELDS, LOG);
    assert_in_order(
        &html,
        &[
            "<b class=\"piece-title\">Valor inválido é recusado</b>",
            "<small class=\"piece-file\">EV-01 saida.txt</small>",
            "<p><b>Mostra</b>A CLI recusa o valor e não grava nada.</p>",
            "<p><b>Onde olhar</b>No fim do log, <code>1 passed</code>.</p>",
            "<p><b>Falha</b>Se aparecer <code>FAILED</code>, o valor passou.</p>",
            "<p><b>Como chegar</b><code>cargo test -- recusa</code></p>",
        ],
    );
    assert!(!html.contains("<b>Prova</b>a saída de CT-01."), "{html}");
}

#[test]
fn evidence_doc_create_prefills_the_real_command() {
    let project = filled_folder_project(evidence_task(NEW_FIELDS, LOG));
    let content = fs::read_to_string(finalize_content_path(&project)).unwrap();
    assert!(
        content.contains("- **Como chegar:** `cargo test -- recusa`"),
        "{content}"
    );
    assert!(!content.contains("#{EVIDENCE_01_HOW}"), "{content}");
}

#[test]
fn evidence_doc_mostra_wins_over_prova() {
    let subitems = format!("{NEW_FIELDS}  - Prova: texto antigo da prova (RF-01).\n");
    let (_project, html) = evidence_doc(&subitems, LOG);
    assert!(
        html.contains("<b>Mostra</b>A CLI recusa o valor e não grava nada."),
        "{html}"
    );
    assert!(!html.contains("texto antigo da prova"), "{html}");
}

#[test]
fn evidence_doc_current_format_keeps_prova_and_trecho() {
    let subitems = "  - Título: Saída aparece\n  - Prova: a saída aparece (CT-01).\n  - Trecho: `depois`\n  - Falha: `FAILED`\n";
    let (_project, html) = evidence_doc(subitems, LOG);
    assert!(
        html.contains("<p><b>Mostra</b>a saída aparece (CT-01).</p>"),
        "{html}"
    );
    assert!(
        html.contains("<p><b>O que olhar</b><code>depois</code></p>"),
        "{html}"
    );
    assert!(
        html.contains("<p><b>Falha</b><code>FAILED</code></p>"),
        "{html}"
    );
}

#[test]
fn evidence_doc_without_log_command_uses_the_written_how() {
    let (_project, html) = evidence_doc(NEW_FIELDS, "depois\n");
    assert!(html.contains("<p><b>Como chegar</b>rodar.</p>"), "{html}");
    let project = Project::new();
    write_spec(&project, "implemented");
    evidence_task(NEW_FIELDS, "depois\n")(&write_folder_task(&project, "TASK-01"));
    project
        .vostok()
        .args(["finalize", "create", "SPEC-001"])
        .assert()
        .success();
    let content = fs::read_to_string(finalize_content_path(&project)).unwrap();
    assert!(content.contains("#{EVIDENCE_01_HOW}"), "{content}");
}

#[test]
fn evidence_doc_without_title_falls_back_to_the_file() {
    let (_project, html) = evidence_doc("", "depois\n");
    assert!(
        html.contains("<b class=\"piece-title\">EV-01 saida.txt</b>"),
        "{html}"
    );
    assert!(
        html.contains("<p><b>Prova</b>a saída de CT-01.</p>"),
        "{html}"
    );
    for label in ["<b>Mostra</b>", "<b>Onde olhar</b>", "<b>Falha</b>"] {
        assert!(!html.contains(label), "{label}: {html}");
    }
}
