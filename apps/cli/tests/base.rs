//! Testes da base do crate: versão, conteúdo embutido, localização do harness e utilitários.

mod common;

use std::collections::HashMap;
use std::fs;

use common::{failure_stderr, vostok_at, Project};
use tempfile::TempDir;
use vostok::embedded;
use vostok::harness::Harness;
use vostok::support::*;

#[test]
fn ct01_version_is_0_1_0() {
    vostok_at(&std::env::temp_dir())
        .arg("--version")
        .assert()
        .success()
        .stdout("vostok 0.1.0\n");
}

#[test]
fn help_lists_every_group() {
    let output = vostok_at(&std::env::temp_dir())
        .arg("--help")
        .output()
        .unwrap();
    let help = String::from_utf8(output.stdout).unwrap();
    for group in [
        "context", "memory", "rules", "spec", "plan", "task", "commit", "skill", "finalize",
        "init", "update",
    ] {
        assert!(
            help.contains(&format!("  {group} ")),
            "grupo {group} fora da ajuda"
        );
    }
}

#[test]
fn ct02_embedded_content_is_the_distributable_harness() {
    let files = embedded::files();
    for expected in [
        "config/main.yml",
        "config/context.yml",
        "context/skill-catalog.md",
        "rules/agent/boundaries.md",
        "rules/project/language.md",
        "rules/project/naming-convention.md",
        "skills/default/refine-spec/SKILL.md",
        "skills/custom/.gitkeep",
        "templates/responses/question.md",
        "memory/.gitkeep",
    ] {
        assert!(
            files.iter().any(|file| file == expected),
            "falta {expected}"
        );
    }
    for file in &files {
        assert!(!file.starts_with("work/"), "work embutido: {file}");
        assert!(!file.starts_with(".state/"), "estado embutido: {file}");
        assert!(!file.starts_with("tests/"), "testes embutidos: {file}");
        assert!(!file.contains("scripts/"), "script embutido: {file}");
        assert!(
            !file.starts_with("memory/") || file == "memory/.gitkeep",
            "entrada de memória embutida: {file}"
        );
    }
}

#[test]
fn ct02_project_rules_of_this_repository_are_not_embedded() {
    let files = embedded::files();
    assert!(!files
        .iter()
        .any(|file| file == "rules/project/architecture.md"));
    let context =
        String::from_utf8(embedded::file("config/context.yml").unwrap().to_vec()).unwrap();
    assert!(!context.contains("rules/project/architecture.md"));
    assert!(!context.contains("rules/project/security.md"));
    assert!(context.contains("rules/project/language.md"));
}

#[test]
fn ct02_sequences_start_at_one() {
    let sequences =
        String::from_utf8(embedded::file("config/sequences.yml").unwrap().to_vec()).unwrap();
    for line in sequences
        .lines()
        .filter(|line| !line.starts_with('#') && line.contains(':'))
    {
        assert!(line.ends_with(": 1"), "sequência não zerada: {line}");
    }
}

#[test]
fn ct11_harness_is_found_from_a_subfolder() {
    let project = Project::new();
    let nested = project.root().join("a/b");
    fs::create_dir_all(&nested).unwrap();
    assert_eq!(Harness::locate(&nested).unwrap().root, project.harness());
    let stderr = failure_stderr(project.vostok_in("a/b").args(["spec", "list"]));
    assert!(stderr.contains("ainda não foi implementado"), "{stderr}");
}

#[test]
fn ct11_outside_a_project_points_to_init() {
    let empty = TempDir::new().unwrap();
    let error = Harness::locate(empty.path()).unwrap_err().to_string();
    assert!(error.contains("vostok init"), "{error}");
    let stderr = failure_stderr(vostok_at(empty.path()).args(["spec", "list"]));
    assert!(
        stderr.starts_with("Erro: ") && stderr.contains("vostok init"),
        "{stderr}"
    );
}

#[test]
fn paths_are_resolved_from_main_yml() {
    let project = Project::new();
    let harness = Harness::locate(project.root()).unwrap();
    assert_eq!(
        harness.path("specs").unwrap(),
        project.harness().join("work/specs")
    );
    assert_eq!(harness.resolve("/abs"), std::path::PathBuf::from("/abs"));
}

#[test]
fn cb07_missing_key_names_file_and_key() {
    let project = Project::new();
    let main = project.harness().join("config/main.yml");
    let error = config_get(&main, "paths.nao_existe")
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("'paths.nao_existe'") && error.contains("main.yml"),
        "{error}"
    );
    let harness = Harness::locate(project.root()).unwrap();
    assert!(harness.path("nao_existe").is_err());
}

#[test]
fn config_errors_for_missing_file_list_and_block() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("c.yml");
    assert!(config_get(&file, "a")
        .unwrap_err()
        .to_string()
        .contains("não foi possível ler"));
    fs::write(&file, "a: 1\n").unwrap();
    assert!(config_list(&file, "lista")
        .unwrap_err()
        .to_string()
        .contains("Lista 'lista' não encontrada"));
    assert!(config_block(&file, "bloco")
        .unwrap_err()
        .to_string()
        .contains("Bloco 'bloco' não encontrado"));
    assert!(config_set(&file, "b", "2").is_err());
}

#[test]
fn config_reads_scalars_lists_and_blocks() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("c.yml");
    fs::write(
        &file,
        "# comentário\npaths:\n  specs: ./work/specs   # pasta\n  docs: \"../docs\"\nlista:\n  - um  # x\n\n  - 'dois'\noutra: x\nbloco: |\n  linha 1\n    linha 2\nfim: 1\n",
    )
    .unwrap();
    assert_eq!(config_get(&file, "paths.specs").unwrap(), "./work/specs");
    assert_eq!(config_get(&file, "paths.docs").unwrap(), "../docs");
    assert_eq!(config_list(&file, "lista").unwrap(), vec!["um", "dois"]);
    assert_eq!(config_block(&file, "bloco").unwrap(), "linha 1\n  linha 2");
}

#[test]
fn config_set_preserves_the_rest_of_the_file() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("c.yml");
    fs::write(&file, "# topo\nsequences:\n  specs: 1   # c\nkeep: sim\n").unwrap();
    config_set(&file, "sequences.specs", "7").unwrap();
    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        "# topo\nsequences:\n  specs: 7\nkeep: sim\n"
    );
}

#[test]
fn frontmatter_reads_quoted_block_and_sets_values() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("f.md");
    fs::write(
        &file,
        "---\ntitle: \"Diz \\\"oi\\\"\"\nsimple: 'x'\ndescription: >\n  linha um\n  linha dois\nstatus: draft\n---\nstatus: fora\n",
    )
    .unwrap();
    assert_eq!(frontmatter_get(&file, "title").unwrap(), "Diz \"oi\"");
    assert_eq!(frontmatter_get(&file, "simple").unwrap(), "x");
    assert_eq!(
        frontmatter_get(&file, "description").unwrap(),
        "linha um linha dois"
    );
    frontmatter_set(&file, "status", "refined").unwrap();
    let content = fs::read_to_string(&file).unwrap();
    assert!(content.contains("status: refined\n---\nstatus: fora"));
    assert!(frontmatter_get(&file, "nao").is_err());
    assert!(frontmatter_set(&file, "nao", "x").is_err());
    fs::write(&file, "sem front-matter\n").unwrap();
    assert!(frontmatter_get(&file, "title").is_err());
}

#[test]
fn markdown_sections_tables_and_cells() {
    let text = "# T\n## A\n```\n## falso\n```\ntexto\n### X\nx1\n### Y\ny1\n## B\n| ID | V |\n|---|:---:|\n| 1 | um |\n| - | - |\n\nfora\n";
    let section = markdown_section(text, "A").unwrap();
    assert!(section.contains("## falso") && section.contains("texto") && !section.contains("| ID"));
    assert_eq!(markdown_subsection(text, "A", "X").unwrap(), "x1");
    assert!(markdown_section(text, "C").is_none());
    assert!(markdown_subsection(text, "B", "X").is_none());
    let rows = markdown_table_rows(text);
    assert_eq!(rows, vec!["| 1 | um |", "| - | - |"]);
    assert_eq!(markdown_cell(&rows[0], 2), "um");
    assert_eq!(markdown_cell(&rows[0], 9), "");
}

#[test]
fn templates_render_and_report_leftovers() {
    let values = HashMap::from([("NOME", "Vostok".to_string())]);
    assert_eq!(
        render("Olá #{NOME}, #{OUTRO} e #{nome}", &values),
        "Olá Vostok, #{OUTRO} e #{nome}"
    );
    let leftovers = template_leftovers("ok\n#{A}\n`#{B}` citado\n#{ }\n");
    assert_eq!(leftovers, vec![(2, "#{A}".to_string())]);
}

#[test]
fn text_helpers() {
    assert_eq!(slugify("--Olá  mundo--"), "ola-mundo");
    assert_eq!(slugify("Ação & Teste: Ñu"), "acao-teste-nu");
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(yaml_escape(r#"a"b\c"#), r#"a\"b\\c"#);
    assert_eq!(clean_title("  Título  ").unwrap(), "Título");
    assert!(clean_title("   ").is_err());
    assert!(clean_title("a\nb").is_err());
    assert!(clean_title("Tem #{X}").is_err());
}

#[test]
fn extract_writes_every_embedded_file() {
    let project = Project::new();
    for file in embedded::files() {
        assert!(
            project.harness().join(&file).is_file(),
            "não extraído: {file}"
        );
    }
}
