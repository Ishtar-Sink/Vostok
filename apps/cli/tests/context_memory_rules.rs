//! Equivalência dos grupos `context`, `memory` e `rules` com os scripts bash (RF-13, CT-10), pelas
//! fixtures de `tests/fixtures/golden`, e cenários de erro que as fixtures não cobrem.

mod common;

use std::fs;

use common::golden::assert_group;
use common::{failure_stderr, Project};
use vostok::embedded;

const PERSONAS: [&str; 8] = [
    "coordenador",
    "planejador",
    "arquiteto",
    "engenheiro",
    "desenvolvedor",
    "qa-lead",
    "ui-ux-designer",
    "qa",
];

#[test]
fn golden_context() {
    assert_group("context");
}

#[test]
fn golden_memory() {
    assert_group("memory");
}

#[test]
fn golden_rules() {
    assert_group("rules");
}

#[test]
fn context_load_rejects_an_empty_skill_name() {
    let project = Project::new();
    let stderr = failure_stderr(project.vostok().args(["context", "load", "--skill", ""]));
    assert_eq!(stderr, "Uso: vostok context load --skill <nome>\n");
}

#[test]
fn context_load_keeps_loading_after_a_missing_path() {
    let project = Project::new();
    let output = project
        .vostok()
        .args(["context", "load", "nao-existe.md", "config/skills.yml"])
        .assert()
        .failure()
        .code(1)
        .get_output()
        .clone();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("--- arquivo: config/skills.yml ---"));
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "Aviso: 'nao-existe.md' não encontrado\n"
    );
}

#[test]
fn memory_search_without_arguments_is_a_usage_error() {
    let project = Project::new();
    let stderr = failure_stderr(project.vostok().args(["memory", "search"]));
    assert_eq!(
        stderr,
        "Erro: Uso: vostok memory search <tipo|all> [termo...] | --id <ID>\n"
    );
}

#[test]
fn memory_search_with_an_empty_id_is_a_usage_error() {
    let project = Project::new();
    let stderr = failure_stderr(project.vostok().args(["memory", "search", "--id", ""]));
    assert!(stderr.starts_with("Erro: Uso: vostok memory search"));
}

#[test]
fn memory_update_without_the_memory_file_fails() {
    let project = Project::new();
    let stderr = failure_stderr(
        project
            .vostok()
            .args(["memory", "update", "DEC-001", "Status", "ativa"]),
    );
    assert!(
        stderr.starts_with("Erro: arquivo de memória não encontrado: "),
        "{stderr}"
    );
}

#[test]
fn memory_add_rejects_a_corrupted_sequence() {
    let project = Project::new();
    let sequences = project.harness().join("config/sequences.yml");
    let content = fs::read_to_string(&sequences).unwrap();
    fs::write(&sequences, content.replace("issues: 1", "issues: x")).unwrap();
    let stderr = failure_stderr(project.vostok().args(["memory", "add", "issue", "Falha"]));
    assert!(stderr.contains("issues inválido em "), "{stderr}");
    assert!(stderr.ends_with(": 'x'\n"), "{stderr}");
}

#[test]
fn memory_search_omits_entries_beyond_the_limit() {
    let project = Project::new();
    for index in 1..=6 {
        let title = format!("Comum {index}");
        project
            .vostok()
            .args(["memory", "add", "knowledge", &title])
            .assert()
            .success();
        let file = project.harness().join("memory/knowledge.md");
        let content = fs::read_to_string(&file).unwrap();
        let filled = content
            .lines()
            .map(|line| match line.find("#{") {
                Some(start) => format!("{}x", &line[..start]),
                None => line.to_string(),
            })
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&file, filled + "\n").unwrap();
    }
    let output = project
        .vostok()
        .args(["memory", "search", "knowledge", "comum"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();
    assert_eq!(stdout.matches("> Termos encontrados: 1 de 1").count(), 5);
    assert!(stdout
        .ends_with("\nEntradas omitidas: 1. Use termos mais específicos para refinar a busca.\n"));
}

#[test]
fn rules_create_category_rejects_an_invalid_purpose() {
    let project = Project::new();
    for (purpose, message) in [
        ("   ", "Erro: o propósito está vazio\n"),
        ("a\nb", "Erro: o propósito deve ter uma linha só\n"),
        (
            "tem #{X}",
            "Erro: o propósito não pode conter '#{', reservado para variáveis de template\n",
        ),
    ] {
        let stderr = failure_stderr(project.vostok().args([
            "rules",
            "create-category",
            "api",
            "API",
            purpose,
        ]));
        assert_eq!(stderr, message);
    }
    assert!(!project.harness().join("rules/project/api.md").exists());
}

#[test]
fn rules_create_category_rejects_an_empty_title() {
    let project = Project::new();
    let stderr = failure_stderr(project.vostok().args([
        "rules",
        "create-category",
        "api",
        " ",
        "Propósito.",
    ]));
    assert_eq!(stderr, "O título está vazio\n");
}

#[test]
fn rules_load_validates_every_target_before_writing() {
    let project = Project::new();
    let context = project.harness().join("config/context.yml");
    let before = fs::read_to_string(&context).unwrap();
    for (target, message) in [
        (
            "execute-task,Bad",
            "Erro: destino inválido 'Bad': use 'always' ou o nome de uma skill\n",
        ),
        ("", "Erro: informe 'always' ou ao menos uma skill\n"),
    ] {
        let stderr = failure_stderr(project.vostok().args(["rules", "load", "language", target]));
        assert_eq!(stderr, message);
    }
    assert_eq!(fs::read_to_string(&context).unwrap(), before);
}

#[test]
fn rules_write_rejects_a_removal_without_a_number() {
    let project = Project::new();
    let rules = project.root().join("regras.txt");
    fs::write(&rules, "remover: dois\n").unwrap();
    let stderr = failure_stderr(project.vostok().args([
        "rules",
        "write",
        "language",
        rules.to_str().unwrap(),
    ]));
    assert_eq!(
        stderr,
        "Erro: remover: espera o número da regra: 'remover: dois'\n"
    );
}

#[test]
fn rules_validate_reports_a_rule_before_the_purpose() {
    let project = Project::new();
    fs::write(
        project.harness().join("rules/project/extra.md"),
        "# Extra\n\n[project/extra-01]: Regra.\n",
    )
    .unwrap();
    let output = project
        .vostok()
        .args(["rules", "validate"])
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    assert!(String::from_utf8(output).unwrap().contains(
        "rules/project/extra.md:3: falta a frase de propósito antes da primeira regra\n"
    ));
}

#[test]
fn rf09_every_persona_has_the_fixed_structure() {
    for name in PERSONAS {
        let bytes = embedded::file(&format!("personas/{name}.md"))
            .unwrap_or_else(|| panic!("persona {name} ausente"));
        let text = String::from_utf8(bytes.to_vec()).expect("UTF-8");
        assert!(
            text.starts_with(&format!("---\nname: {name}\ntitle: ")),
            "{name}: front-matter"
        );
        assert!(text.contains("\ndescription: \""), "{name}: description");
        assert!(text.contains("\n# Persona: "), "{name}: título");
        let mut last = 0;
        for section in [
            "## Responsabilidade",
            "## Quando atua",
            "## Entradas",
            "## Saídas",
            "## Regras",
            "## Limites",
            "## Ao terminar",
        ] {
            let at = text
                .find(section)
                .unwrap_or_else(|| panic!("{name}: falta {section}"));
            assert!(at > last, "{name}: {section} fora de ordem");
            last = at;
        }
        assert!(
            text.contains(&format!("[persona/{name}-01]:")),
            "{name}: regra 01"
        );
        assert!(
            !text.contains("packages/harness"),
            "{name}: cita packages/harness"
        );
    }
}

#[test]
fn rf09_only_the_eight_personas_are_embedded() {
    let found: Vec<String> = embedded::files()
        .into_iter()
        .filter(|file| file.starts_with("personas/"))
        .collect();
    let mut expected: Vec<String> = PERSONAS
        .iter()
        .map(|n| format!("personas/{n}.md"))
        .collect();
    expected.sort();
    assert_eq!(found, expected);
}

fn position(text: &str, needle: &str) -> usize {
    text.find(needle)
        .unwrap_or_else(|| panic!("falta '{needle}' em:\n{text}"))
}

#[test]
fn ct06_context_load_persona_prints_the_required_context_and_the_persona() {
    let project = Project::new();
    let output = project
        .vostok()
        .args(["context", "load", "--persona", "arquiteto"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).expect("UTF-8");
    let required = position(&text, "# Contexto obrigatório do harness");
    let header = position(&text, "# Persona\n");
    let file = position(&text, "--- arquivo: personas/arquiteto.md ---");
    assert!(required < header && header < file);
    assert!(text.contains("## Responsabilidade"));
}

#[test]
fn ct06_context_load_rejects_an_unknown_persona() {
    let project = Project::new();
    let output = project
        .vostok()
        .args(["context", "load", "--persona", "nao-existe"])
        .assert()
        .failure()
        .code(1)
        .get_output()
        .clone();
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).expect("UTF-8"),
        "Persona 'nao-existe' não existe; as disponíveis são: arquiteto, coordenador, desenvolvedor, engenheiro, planejador, qa, qa-lead, ui-ux-designer\n"
    );
}

#[test]
fn context_load_rejects_an_empty_persona_name() {
    let project = Project::new();
    let stderr = failure_stderr(project.vostok().args(["context", "load", "--persona", ""]));
    assert_eq!(stderr, "Uso: vostok context load --persona <nome>\n");
}

#[test]
fn context_load_without_installed_personas_points_to_update() {
    let project = Project::new();
    fs::remove_dir_all(project.harness().join("personas")).expect("remover personas");
    let stderr =
        failure_stderr(
            project
                .vostok()
                .args(["context", "load", "--persona", "arquiteto"]),
        );
    assert_eq!(
        stderr,
        "Persona 'arquiteto' não existe; nenhuma persona instalada em personas/; execute `vostok update`\n"
    );
}

#[test]
fn context_load_with_skill_and_persona_prints_the_skill_context_and_the_persona() {
    let project = Project::new();
    let output = project
        .vostok()
        .args([
            "context",
            "load",
            "--skill",
            "refine-spec",
            "--persona",
            "planejador",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).expect("UTF-8");
    assert!(text.contains("# Contexto adicional"));
    assert!(!text.contains("# Contexto obrigatório do harness"));
    assert!(
        position(&text, "# Persona\n") < position(&text, "--- arquivo: personas/planejador.md ---")
    );
}

#[test]
fn context_load_prints_several_personas_in_the_given_order() {
    let project = Project::new();
    let output = project
        .vostok()
        .args([
            "context",
            "load",
            "--persona",
            "engenheiro",
            "--persona",
            "arquiteto",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).expect("UTF-8");
    assert!(position(&text, "personas/engenheiro.md") < position(&text, "personas/arquiteto.md"));
}

#[test]
fn context_load_rejects_persona_with_paths() {
    let project = Project::new();
    let output = project
        .vostok()
        .args([
            "context",
            "load",
            "--persona",
            "arquiteto",
            "config/skills.yml",
        ])
        .assert()
        .failure()
        .code(1)
        .get_output()
        .clone();
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).expect("UTF-8"),
        "Uso: vostok context load [--skill <nome>] [--persona <nome>]...; --persona não se combina com caminhos\n"
    );
}

fn tree_contents(
    root: &std::path::Path,
) -> std::collections::BTreeMap<std::path::PathBuf, Vec<u8>> {
    fn walk(
        dir: &std::path::Path,
        out: &mut std::collections::BTreeMap<std::path::PathBuf, Vec<u8>>,
    ) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, out);
            } else {
                out.insert(path.clone(), fs::read(&path).unwrap());
            }
        }
    }
    let mut out = std::collections::BTreeMap::new();
    walk(root, &mut out);
    out
}

#[test]
fn ct15_required_context_includes_the_coordinator() {
    let project = Project::new();
    let output = project
        .vostok()
        .args(["context", "load"])
        .assert()
        .success()
        .get_output()
        .clone();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("--- arquivo: personas/coordenador.md ---"),
        "{stdout}"
    );
}

#[test]
fn ct15_loading_a_skill_with_a_persona_writes_nothing() {
    let project = Project::new();
    let before = tree_contents(project.root());
    project
        .vostok()
        .args([
            "context",
            "load",
            "--skill",
            "refine-spec",
            "--persona",
            "planejador",
        ])
        .assert()
        .success();
    assert_eq!(tree_contents(project.root()), before);
}
