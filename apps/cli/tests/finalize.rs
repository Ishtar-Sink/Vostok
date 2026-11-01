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

/// Troca cada variável `#{...}` por um texto válido para o tipo da seção.
fn fill_variables(content: &str) -> String {
    let mut out = String::new();
    for line in content.lines() {
        if let Some(start) = line.find("#{") {
            let name = &line[start + 2..line.find('}').unwrap()];
            out.push_str(match name {
                "EXEC_NUMBERS" => "- **2** tarefas concluídas",
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
    assert!(technical.contains("<h4>O que foi feito</h4>"), "{technical}");
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
            assert!(allowed.contains(&color), "{name}: #{color} fora de documents.yml");
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
        stderr.contains("código no executivo (caminho, comando ou função): Ficou em `src/novo.rs`."),
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
    assert!(stderr.contains("Mapa de cobertura (Evidências): falta RF-01"), "{stderr}");
    assert!(stderr.contains("Mapa de cobertura (Evidências): falta RNF-01"), "{stderr}");
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
    assert!(technical.contains("<td><code>src/novo.rs</code></td>"), "{technical}");
    assert!(technical.contains("<li>Rodar <code>vostok finalize build</code>.</li>"), "{technical}");
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
    assert!(evidence.contains("<b>Prova</b>a saída de CT-01."), "{evidence}");
    assert!(evidence.contains("id=\"p-cobertura\""), "{evidence}");
}
