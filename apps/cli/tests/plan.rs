//! Testes de integração dos comandos `plan`: reprodução das fixtures do grupo e cenários de erro.

mod common;

use common::golden::assert_group;
use common::{failure_stderr, Project};

#[test]
fn golden_plan() {
    assert_group("plan");
}

#[test]
fn create_refuses_unknown_spec() {
    let project = Project::new();
    let stderr = failure_stderr(project.vostok().args(["plan", "create", "SPEC-404"]));
    assert_eq!(stderr, "Erro: spec refinada SPEC-404 não encontrada\n");
}

#[test]
fn score_refuses_spec_without_plan() {
    let project = Project::new();
    let stderr = failure_stderr(project.vostok().args(["plan", "score", "SPEC-404"]));
    assert_eq!(
        stderr,
        "Erro: a spec SPEC-404 não tem plano de tarefas. Crie-o com: vostok plan create SPEC-404\n"
    );
}

#[test]
fn adjust_record_without_summary_shows_usage() {
    let project = Project::new();
    let stderr = failure_stderr(
        project
            .vostok()
            .args(["plan", "adjust", "record", "SPEC-001"]),
    );
    assert!(
        stderr.starts_with("Erro: Uso: vostok plan adjust"),
        "{stderr}"
    );
}

#[test]
fn adjust_record_refuses_blank_summary() {
    let project = Project::new();
    let stderr = failure_stderr(
        project
            .vostok()
            .args(["plan", "adjust", "record", "SPEC-001", "  \t "]),
    );
    assert_eq!(stderr, "Erro: o resumo do ajuste está vazio\n");
}

#[test]
fn adjust_record_refuses_multiline_summary() {
    let project = Project::new();
    let stderr = failure_stderr(project.vostok().args([
        "plan",
        "adjust",
        "record",
        "SPEC-001",
        "primeiro\nsegundo",
    ]));
    assert_eq!(
        stderr,
        "Erro: o resumo não pode ter quebra de linha; separe os itens com '; '\n"
    );
}

#[test]
fn approve_validate_refuses_spec_without_plan() {
    let project = Project::new();
    let stderr = failure_stderr(
        project
            .vostok()
            .args(["plan", "approve", "validate", "SPEC-404"]),
    );
    assert_eq!(
        stderr,
        "Erro: a spec SPEC-404 não tem plano de tarefas. Crie-o com a skill plan-tasks\n"
    );
}

#[test]
fn empty_spec_id_shows_usage() {
    let project = Project::new();
    let stderr = failure_stderr(project.vostok().args(["plan", "validate", ""]));
    assert!(
        stderr.starts_with("Erro: Uso: vostok plan create"),
        "{stderr}"
    );
}

#[test]
fn plan_outside_a_harness_fails() {
    let dir = tempfile::TempDir::new().unwrap();
    let stderr =
        failure_stderr(common::vostok_at(dir.path()).args(["plan", "validate", "SPEC-001"]));
    assert!(stderr.contains("nenhum harness encontrado"), "{stderr}");
}

// --- Nome de evidência com caminho no planejamento (SPEC-010, SPEC-011) ---

const SUBITEMS: &str = "  - Prova: ok.\n  - Trecho: ok.\n  - Falha: erro.";

/// Grava a spec aprovada SPEC-001 (com hashes válidos) e o plano dela, com os corpos dados.
fn spec_and_plan(project: &Project, kind: &str, spec_body: &str, plan_body: &str) {
    use vostok::support::sha256_hex;
    let spec_hash = sha256_hex(spec_body.as_bytes());
    write_file(
        project,
        "vostok/work/specs/refined/SPEC-001-exemplo.md",
        &format!("---\nid: SPEC-001\ntitle: \"Exemplo\"\nstatus: approved\ntype: {kind}\nupdated_at: null\ncontent_hash: {spec_hash}\napproved_hash: {spec_hash}\n---\n{spec_body}"),
    );
    let plan_hash = sha256_hex(plan_body.as_bytes());
    write_file(
        project,
        "vostok/work/plans/SPEC-001.md",
        &format!("---\nspec: SPEC-001\nspec_hash: {spec_hash}\ncontent_hash: {plan_hash}\n---\n{plan_body}"),
    );
}

fn write_file(project: &Project, relative: &str, content: &str) {
    let path = project.root().join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

/// Spec aprovada (com hashes válidos), plano e TASK-01 com a subseção "Evidências" dada.
fn plan_with_evidence(project: &Project, evidence: &str) -> std::path::PathBuf {
    let write = |relative: &str, content: &str| write_file(project, relative, content);
    let spec_body = "\n# Exemplo\n\n## Requisitos funcionais\n\n| ID | Requisito |\n|---|---|\n| RF-01 | Entregar. |\n";
    let plan_body = "\n# Plano\n\n## Visão geral\n\nUma tarefa.\n\n## Tarefas\n\n| ID | Título | Descrição | Tipo | Nível | Dependências | Cobre |\n|---|---|---|---|---|---|---|\n| TASK-01 | Tarefa | Entrega. | tarefa | simples | - | RF-01 |\n\n## Fatos de esforço\n\n| ID | Arquivos a criar | Arquivos a alterar | Contrato público | Inferência | Risco alto |\n|---|---|---|---|---|---|\n| TASK-01 | 0 | 1 | não | não | não |\n";
    spec_and_plan(project, "improvement", spec_body, plan_body);
    let task = project.root().join("vostok/work/tasks/SPEC-001/TASK-01.md");
    write(
        "vostok/work/tasks/SPEC-001/TASK-01.md",
        &format!(
            "---\nid: TASK-01\nspec: SPEC-001\ntitle: \"Tarefa\"\neffort: simples\ndepends_on: []\ncovers: [RF-01]\ntype: tarefa\nlabels: [SPEC-001, simples]\nstatus: pending\ncontent_hash: null\nadjustments: 0\nadjusted_at: null\napproved_at: null\napproved_hash: null\nattempts: 0\ntests_result: null\nregression_result: null\nevidence_result: null\n---\n\n# TASK-01: Tarefa\n\n## Execução\n\n### Ler\n\n- `README.md`\n\n### Mudar\n\n- `novo.txt` (criar): arquivo novo.\n\n### Passos\n\n1. Crie o arquivo.\n\n### Baseline\n\n- `true`\n\n### Verificação\n\n```bash\ntrue\n```\n\n### Evidências\n\n{EVIDENCE}\n\n### Concluída quando\n\n- [ ] Pronto (RF-01).\n\n  > Como comprovar: ver o arquivo.\n\n## Descrição\n\n### Resumo executivo\n\nResumo.\n\n### Detalhes técnicos\n\nDetalhes.\n\n### Card\n\n- **Título:** Tarefa\n- **Tipo:** tarefa\n- **Descrição:** Descrição.\n- **Critérios de aceite:**\n  - [ ] Pronto.\n- **Rótulos:** SPEC-001, simples\n- **Estimativa:** simples\n- **Bloqueada por:** -\n\n## Registro de execução\n\n| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n\n## Histórico de ajustes\n\n| # | Data | Origem | Resumo |\n|---|---|---|---|\n",
            EVIDENCE = evidence
        ),
    );
    task
}

#[test]
fn evidence_name_with_a_path_is_accepted_with_a_warning_by_the_plan_validation() {
    let project = Project::new();
    let task = plan_with_evidence(
        &project,
        &format!("- `docs/x/saida.txt` (automática): `echo ok`\n{SUBITEMS}"),
    );
    let before = std::fs::read_to_string(&task).unwrap();
    let output = project
        .vostok()
        .args(["plan", "approve", "validate", "SPEC-001"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();
    assert!(
        stdout.contains("TASK-01: docs/x/saida.txt → saida.txt"),
        "{stdout}"
    );
    assert_eq!(std::fs::read_to_string(&task).unwrap(), before);
}

#[test]
fn evidence_name_with_a_path_is_renamed_by_mark_planned() {
    let project = Project::new();
    let task = plan_with_evidence(
        &project,
        &format!("- `docs/x/saida.txt` (automática): `echo ok`\n{SUBITEMS}"),
    );
    let output = project
        .vostok()
        .args(["plan", "approve", "mark-planned", "SPEC-001"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();
    assert!(
        stdout.contains("TASK-01: docs/x/saida.txt → saida.txt"),
        "{stdout}"
    );
    let content = std::fs::read_to_string(&task).unwrap();
    assert!(!content.contains("docs/x/"), "{content}");
    // O hash gravado vale para o conteúdo já com o nome trocado.
    project
        .vostok()
        .args(["task", "approve", "validate", "SPEC-001", "TASK-01"])
        .assert()
        .success();
}

#[test]
fn evidence_name_rename_keeps_the_kind_command_and_subitems() {
    let project = Project::new();
    let task = plan_with_evidence(
        &project,
        &format!("- `docs/x/saida.txt` (automática): `echo ok`\n{SUBITEMS}\n- `notas.md` (manual): conferir a tela.\n{SUBITEMS}"),
    );
    project
        .vostok()
        .args(["plan", "approve", "mark-planned", "SPEC-001"])
        .assert()
        .success();
    let content = std::fs::read_to_string(&task).unwrap();
    let expected = format!(
        "### Evidências\n\n- `saida.txt` (automática): `echo ok`\n{SUBITEMS}\n- `notas.md` (manual): conferir a tela.\n{SUBITEMS}\n"
    );
    assert!(content.contains(&expected), "{content}");
}

#[test]
fn evidence_name_ambiguous_or_reserved_is_refused_by_the_plan_validation() {
    let cases = [
        (
            format!("- `a/saida.txt` (automática): `echo a`\n{SUBITEMS}\n- `b/saida.txt` (automática): `echo b`\n{SUBITEMS}"),
            "saida.txt",
        ),
        (
            format!("- `docs/baseline` (automática): `echo ok`\n{SUBITEMS}"),
            "docs/baseline",
        ),
        (
            format!("- `docs/evidencias/` (automática): `echo ok`\n{SUBITEMS}"),
            "docs/evidencias/",
        ),
    ];
    for (evidence, name) in cases {
        let project = Project::new();
        let task = plan_with_evidence(&project, &evidence);
        let before = std::fs::read_to_string(&task).unwrap();
        let refused = failure_stderr(
            project
                .vostok()
                .args(["plan", "approve", "validate", "SPEC-001"]),
        );
        assert!(refused.contains(name), "{refused}");
        assert!(refused.contains("TASK-01:"), "{refused}");
        assert_eq!(std::fs::read_to_string(&task).unwrap(), before);
    }
}

// --- Perfis de execução (SPEC-014: RF-17, RF-21, RF-26, CT-12, CT-16) ---

const FACTS_HEADER: &str = "| ID | Arquivos a criar | Arquivos a alterar | Contrato público | Inferência | Risco alto |\n|---|---|---|---|---|---|";

const PROFILE_SPEC: &str = "\n# Exemplo\n\n## Requisitos funcionais\n\n| ID | Requisito |\n|---|---|\n| RF-01 | Um. |\n| RF-02 | Dois. |\n| RF-03 | Três. |\n| RF-04 | Quatro. |\n| RF-05 | Cinco. |\n| RF-06 | Seis. |\n| RF-07 | Sete. |\n| RF-08 | Oito. |\n| RF-09 | Nove. |\n| RF-10 | Dez. |\n| RF-11 | Onze. |\n| RF-12 | Doze. |\n\n## Casos de teste\n\n| ID | Cenário | Passos | Resultado esperado | Requisitos |\n|---|---|---|---|---|\n| CT-01 | Caso. | Rodar. | Ok. | RF-01 |\n";

/// Corpo de plano no formato novo, com as colunas "Persona" e "Perfil".
fn profile_plan(rows: &str, facts: &str) -> String {
    format!("\n# Plano\n\n## Visão geral\n\nUm plano.\n\n## Tarefas\n\n| ID | Título | Descrição | Tipo | Persona | Perfil | Dependências | Cobre |\n|---|---|---|---|---|---|---|---|\n{rows}\n\n## Fatos de esforço\n\n{FACTS_HEADER}\n{facts}\n")
}

/// Corpo de plano no formato antigo, com a coluna "Nível".
fn legacy_plan(rows: &str, facts: &str) -> String {
    format!("\n# Plano\n\n## Visão geral\n\nUm plano.\n\n## Tarefas\n\n| ID | Título | Descrição | Tipo | Nível | Dependências | Cobre |\n|---|---|---|---|---|---|---|\n{rows}\n\n## Fatos de esforço\n\n{FACTS_HEADER}\n{facts}\n")
}

fn profile_row(id: &str, persona: &str, profile: &str, covers: &str) -> String {
    format!("| {id} | Tarefa {id} | Faz. | tarefa | {persona} | {profile} | - | {covers} |")
}

fn plan_text(project: &Project) -> String {
    std::fs::read_to_string(project.root().join("vostok/work/plans/SPEC-001.md")).unwrap()
}

fn score(project: &Project) -> String {
    let output = project
        .vostok()
        .args(["plan", "score", "SPEC-001"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).unwrap()
}

/// Perfil gravado na linha da tarefa de um plano no formato novo.
fn stored_profile(plan: &str, id: &str) -> String {
    let row = plan
        .lines()
        .find(|line| line.starts_with(&format!("| {id} |")))
        .unwrap_or_else(|| panic!("{id} ausente: {plan}"));
    row.split('|').nth(6).unwrap().trim().to_string()
}

fn tasks_section(plan: &str) -> String {
    let start = plan.find("## Tarefas").unwrap();
    let end = plan.find("## Fatos de esforço").unwrap();
    plan[start..end].to_string()
}

/// Plano novo com uma tarefa por caso `(fatos, IDs cobertos)`; devolve o perfil calculado de cada uma.
fn scored_profiles(cases: &[(&str, &str)]) -> Vec<String> {
    let project = Project::new();
    let ids: Vec<String> = (1..=cases.len()).map(|n| format!("TASK-{n:02}")).collect();
    let rows: Vec<String> = cases
        .iter()
        .zip(&ids)
        .map(|((_, covers), id)| profile_row(id, "desenvolvedor", "-", covers))
        .collect();
    let facts: Vec<String> = cases
        .iter()
        .zip(&ids)
        .map(|((facts, _), id)| format!("| {id} | {facts} |"))
        .collect();
    spec_and_plan(
        &project,
        "feature",
        PROFILE_SPEC,
        &profile_plan(&rows.join("\n"), &facts.join("\n")),
    );
    score(&project);
    let plan = plan_text(&project);
    ids.iter().map(|id| stored_profile(&plan, id)).collect()
}

#[test]
fn profile_small_task_with_many_ids_stays_at_most_padrao() {
    let profiles = scored_profiles(&[(
        "0 | 2 | não | não | não",
        "RF-01, RF-02, RF-03, RF-04, RF-05, RF-06, RF-07, RF-08, RF-09, RF-10, RF-11, CT-01",
    )]);
    assert_eq!(profiles, ["padrao"]);
}

#[test]
fn profile_public_contract_alone_stays_padrao() {
    let profiles = scored_profiles(&[("0 | 3 | sim | não | não", "RF-12")]);
    assert_eq!(profiles, ["padrao"]);
}

#[test]
fn profile_judgment_needs_risk_with_contract_or_inference() {
    let profiles = scored_profiles(&[
        ("0 | 1 | sim | não | sim", "RF-01"),
        ("0 | 1 | não | sim | sim", "RF-02"),
        ("0 | 1 | não | não | sim", "RF-03"),
        ("2 | 4 | sim | sim | não", "RF-04, RF-05"),
    ]);
    assert_eq!(profiles, ["julgamento", "julgamento", "padrao", "padrao"]);
}

#[test]
fn profile_limits_follow_the_configuration() {
    let profiles = scored_profiles(&[
        ("0 | 1 | não | não | não", "RF-01"),
        ("1 | 0 | não | não | não", "RF-02"),
        ("1 | 0 | não | não | não", "RF-03, RF-04"),
        ("0 | 2 | não | sim | não", "RF-05, RF-06"),
        ("0 | 3 | sim | não | não", "-"),
        ("6 | 9 | não | não | sim", "RF-07"),
    ]);
    assert_eq!(
        profiles,
        ["mecanica", "mecanica", "padrao", "padrao", "padrao", "padrao"]
    );
}

#[test]
fn profile_score_is_the_same_in_two_runs() {
    let project = Project::new();
    let rows = [
        profile_row("TASK-01", "desenvolvedor", "-", "RF-01"),
        profile_row("TASK-02", "desenvolvedor", "-", "RF-02, RF-03"),
    ];
    let facts = "| TASK-01 | 0 | 1 | sim | não | sim |\n| TASK-02 | 1 | 2 | não | não | não |";
    spec_and_plan(
        &project,
        "feature",
        PROFILE_SPEC,
        &profile_plan(&rows.join("\n"), facts),
    );
    score(&project);
    let first = tasks_section(&plan_text(&project));
    score(&project);
    let second = tasks_section(&plan_text(&project));
    assert_eq!(first, second);
    assert!(first.contains("| julgamento |"), "{first}");
}

#[test]
fn profile_different_from_the_computed_is_refused_by_validate() {
    let project = Project::new();
    spec_and_plan(
        &project,
        "feature",
        PROFILE_SPEC,
        &profile_plan(
            &profile_row("TASK-01", "desenvolvedor", "mecanica", "RF-01"),
            "| TASK-01 | 0 | 3 | sim | não | não |",
        ),
    );
    let stderr = failure_stderr(project.vostok().args(["plan", "validate", "SPEC-001"]));
    assert!(
        stderr.contains(
            "TASK-01: perfil \"mecanica\" diferente do calculado \"padrao\" (execute vostok plan score)"
        ),
        "{stderr}"
    );
}

#[test]
fn profile_score_prints_the_profiles_message() {
    let project = Project::new();
    spec_and_plan(
        &project,
        "feature",
        PROFILE_SPEC,
        &profile_plan(
            &profile_row("TASK-01", "desenvolvedor", "-", "RF-01"),
            "| TASK-01 | 0 | 1 | não | não | não |",
        ),
    );
    let stdout = score(&project);
    assert!(
        stdout.contains("Perfis de execução gravados no plano de SPEC-001:"),
        "{stdout}"
    );
}

#[test]
fn profile_plan_create_uses_the_profile_columns() {
    let project = Project::new();
    spec_and_plan(&project, "feature", PROFILE_SPEC, "");
    std::fs::remove_file(project.root().join("vostok/work/plans/SPEC-001.md")).unwrap();
    project
        .vostok()
        .args(["plan", "create", "SPEC-001"])
        .assert()
        .success();
    let plan = plan_text(&project);
    assert!(
        plan.contains(
            "| ID | Título | Descrição | Tipo | Persona | Perfil | Dependências | Onda | Cobre |\n|---|---|---|---|---|---|---|---|---|"
        ),
        "{plan}"
    );
    assert!(plan.contains("## Perfis de execução"), "{plan}");
    for profile in ["mecanica", "padrao", "julgamento"] {
        assert!(plan.contains(&format!("| {profile} |")), "{plan}");
    }
    assert!(!plan.contains("Nível"), "{plan}");
}

const LEGACY_SPEC_ONE: &str =
    "\n# Exemplo\n\n## Requisitos funcionais\n\n| ID | Requisito |\n|---|---|\n| RF-01 | Um. |\n";

#[test]
fn profile_legacy_plan_with_levels_stays_valid() {
    let project = Project::new();
    let spec = "\n# Exemplo\n\n## Requisitos funcionais\n\n| ID | Requisito |\n|---|---|\n| RF-01 | Um. |\n| RF-02 | Dois. |\n| RF-03 | Três. |\n";
    spec_and_plan(
        &project,
        "improvement",
        spec,
        &legacy_plan(
            "| TASK-01 | Tarefa | Faz. | tarefa | moderada | - | RF-01, RF-02, RF-03 |",
            "| TASK-01 | 0 | 2 | não | não | não |",
        ),
    );
    project
        .vostok()
        .args(["plan", "validate", "SPEC-001"])
        .assert()
        .success();
    let stdout = score(&project);
    assert!(
        stdout.contains("Níveis de esforço gravados no plano de SPEC-001:"),
        "{stdout}"
    );
    assert!(
        plan_text(&project).contains("| tarefa | moderada |"),
        "{}",
        plan_text(&project)
    );
}

#[test]
fn profile_legacy_plan_creates_tasks_with_the_equivalent_profile() {
    let project = Project::new();
    spec_and_plan(
        &project,
        "improvement",
        LEGACY_SPEC_ONE,
        &legacy_plan(
            "| TASK-01 | Tarefa | Faz. | tarefa | complexa | - | RF-01 |",
            "| TASK-01 | 0 | 2 | sim | não | sim |",
        ),
    );
    project
        .vostok()
        .args(["plan", "approve", "create-tasks", "SPEC-001"])
        .assert()
        .success();
    let task =
        std::fs::read_to_string(project.root().join("vostok/work/tasks/SPEC-001/TASK-01.md"))
            .unwrap();
    assert!(task.contains("\nprofile: julgamento\n"), "{task}");
    assert!(task.contains("\npersona: -\n"), "{task}");
    assert!(
        task.contains("\nlabels: [SPEC-001, julgamento]\n"),
        "{task}"
    );
}

#[test]
fn profile_new_plan_creates_tasks_with_profile_and_persona() {
    let project = Project::new();
    spec_and_plan(
        &project,
        "improvement",
        LEGACY_SPEC_ONE,
        &profile_plan(
            &profile_row("TASK-01", "desenvolvedor", "-", "RF-01"),
            "| TASK-01 | 0 | 2 | sim | não | não |",
        ),
    );
    score(&project);
    project
        .vostok()
        .args(["plan", "approve", "create-tasks", "SPEC-001"])
        .assert()
        .success();
    let task =
        std::fs::read_to_string(project.root().join("vostok/work/tasks/SPEC-001/TASK-01.md"))
            .unwrap();
    assert!(task.contains("\nprofile: padrao\n"), "{task}");
    assert!(task.contains("\npersona: desenvolvedor\n"), "{task}");
    assert!(!task.contains("\neffort:"), "{task}");
}

#[test]
fn profile_invalid_persona_is_refused() {
    let project = Project::new();
    spec_and_plan(
        &project,
        "feature",
        PROFILE_SPEC,
        &profile_plan(
            &profile_row("TASK-01", "Dev Ops", "mecanica", "RF-01"),
            "| TASK-01 | 0 | 1 | não | não | não |",
        ),
    );
    let stderr = failure_stderr(project.vostok().args(["plan", "validate", "SPEC-001"]));
    assert!(
        stderr.contains(
            "TASK-01: persona inválida \"Dev Ops\" (use o nome da persona em minúsculas, com hífens, ou -)"
        ),
        "{stderr}"
    );
}

#[test]
fn profile_legacy_task_with_effort_still_loads() {
    let project = Project::new();
    plan_with_evidence(
        &project,
        &format!("- `saida.txt` (automática): `echo ok`\n{SUBITEMS}"),
    );
    project
        .vostok()
        .args(["task", "load", "SPEC-001", "TASK-01"])
        .assert()
        .success();
    project
        .vostok()
        .args(["plan", "approve", "validate", "SPEC-001"])
        .assert()
        .success();
}

#[test]
fn profile_task_profile_maps_the_old_levels() {
    use vostok::commands::plan::task_profile;
    use vostok::harness::Harness;
    let project = Project::new();
    let cases = [
        ("effort: simples\n", Some("mecanica")),
        ("effort: moderada\n", Some("padrao")),
        ("effort: complexa\n", Some("julgamento")),
        ("effort: critica\n", Some("julgamento")),
        ("profile: padrao\n", Some("padrao")),
        ("", None),
    ];
    let harness = Harness::locate(project.root()).unwrap();
    for (index, (line, expected)) in cases.iter().enumerate() {
        let relative = format!("tarefas/TASK-{:02}.md", index + 1);
        write_file(
            &project,
            &relative,
            &format!(
                "---\nid: TASK-{:02}\n{line}status: pending\n---\n\n# Tarefa\n",
                index + 1
            ),
        );
        let file = project.root().join(&relative);
        assert_eq!(
            task_profile(&harness, &file).as_deref(),
            *expected,
            "{line}"
        );
    }
}

// --- Componentes por tarefa e cobertura (SPEC-014: RF-27, RF-28, RF-29, CT-17, CT-18) ---

const COMPONENT_SPEC: &str = "\n# Exemplo\n\n## Requisitos funcionais\n\n| ID | Requisito |\n|---|---|\n| RF-01 | Um. |\n| RF-02 | Dois. |\n\n## Casos de teste\n\n| ID | Cenário | Passos | Resultado esperado | Requisitos |\n|---|---|---|---|---|\n| CT-01 | Caso. | Rodar. | Ok. | RF-01 |\n\n## Componentes afetados\n\n| Caminho | Ação | Motivo |\n|---|---|---|\n| `src/a.rs` | criar | Novo. |\n| `src/b.rs` | alterar | Muda. |\n| `src/c.rs` | remover | Sai. |\n| `docs/d.md` | alterar | Doc. |\n";

/// Acrescenta a seção "Componentes por tarefa" logo depois da tabela de tarefas.
fn with_components(plan: &str, rows: &str) -> String {
    let section =
        format!("## Componentes por tarefa\n\n| ID | Componentes |\n|---|---|\n{rows}\n\n");
    plan.replacen(
        "## Fatos de esforço",
        &format!("{section}## Fatos de esforço"),
        1,
    )
}

const DASH_FACTS: &str =
    "| TASK-01 | - | - | não | não | não |\n| TASK-02 | - | - | não | não | não |";

fn two_tasks() -> String {
    let rows = format!(
        "{}\n{}",
        profile_row("TASK-01", "desenvolvedor", "-", "RF-01, CT-01"),
        profile_row("TASK-02", "desenvolvedor", "-", "RF-02")
    );
    profile_plan(&rows, DASH_FACTS)
}

fn component_project(rows: &str) -> Project {
    let project = Project::new();
    spec_and_plan(
        &project,
        "feature",
        COMPONENT_SPEC,
        &with_components(&two_tasks(), rows),
    );
    project
}

fn fact_line(plan: &str, id: &str) -> String {
    let start = plan.find("## Fatos de esforço").unwrap();
    plan[start..]
        .lines()
        .find(|line| line.starts_with(&format!("| {id} |")))
        .unwrap()
        .to_string()
}

fn validate_stderr(project: &Project) -> String {
    failure_stderr(project.vostok().args(["plan", "validate", "SPEC-001"]))
}

#[test]
fn components_score_computes_files_to_create_and_change() {
    let project = component_project(
        "| TASK-01 | `src/a.rs`, `src/b.rs`, `src/c.rs` |\n| TASK-02 | `docs/d.md` |",
    );
    score(&project);
    let plan = plan_text(&project);
    assert!(
        fact_line(&plan, "TASK-01").starts_with("| TASK-01 | 1 | 2 |"),
        "{plan}"
    );
    assert!(
        fact_line(&plan, "TASK-02").starts_with("| TASK-02 | 0 | 1 |"),
        "{plan}"
    );
    project
        .vostok()
        .args(["plan", "validate", "SPEC-001"])
        .assert()
        .success();
}

#[test]
fn components_repeated_path_in_a_task_counts_once() {
    let project = component_project(
        "| TASK-01 | `src/b.rs`, src/b.rs, `src/b.rs` |\n| TASK-02 | `docs/d.md` |",
    );
    let out = score(&project);
    assert!(out.contains("`src/a.rs`"), "{out}");
    assert!(fact_line(&plan_text(&project), "TASK-01").starts_with("| TASK-01 | 0 | 1 |"));
}

#[test]
fn components_shared_by_two_tasks_count_for_both() {
    let project = component_project("| TASK-01 | `src/b.rs` |\n| TASK-02 | `src/b.rs` |");
    score(&project);
    let plan = plan_text(&project);
    assert!(fact_line(&plan, "TASK-01").starts_with("| TASK-01 | 0 | 1 |"));
    assert!(fact_line(&plan, "TASK-02").starts_with("| TASK-02 | 0 | 1 |"));
}

#[test]
fn components_outside_the_spec_are_refused() {
    let project = component_project("| TASK-01 | `src/z.rs` |\n| TASK-02 | - |");
    let expected = "TASK-01: componente `src/z.rs` não está nos componentes afetados da spec";
    let stderr = failure_stderr(project.vostok().args(["plan", "score", "SPEC-001"]));
    assert!(stderr.contains(expected), "{stderr}");
    assert!(validate_stderr(&project).contains(expected));
}

#[test]
fn components_without_task_are_listed_by_score_and_validate() {
    let project =
        component_project("| TASK-01 | `src/a.rs`, `src/b.rs`, `src/c.rs` |\n| TASK-02 | - |");
    let out = score(&project);
    assert!(
        out.contains("Componentes da spec sem tarefa:\n- `docs/d.md`"),
        "{out}"
    );
    assert!(fact_line(&plan_text(&project), "TASK-02").starts_with("| TASK-02 | 0 | 0 |"));
    assert!(validate_stderr(&project).contains("componente da spec sem tarefa: `docs/d.md`"));
}

#[test]
fn components_task_without_row_is_refused() {
    let project =
        component_project("| TASK-01 | `src/a.rs`, `src/b.rs`, `src/c.rs`, `docs/d.md` |");
    assert!(validate_stderr(&project).contains(
        "TASK-02: sem linha em 'Componentes por tarefa' (use - quando a tarefa não assume componente da spec)"
    ));
    let project = component_project(
        "| TASK-01 | `src/a.rs`, `src/b.rs`, `src/c.rs` |\n| TASK-02 | `docs/d.md` |\n| TASK-09 | - |",
    );
    assert!(validate_stderr(&project).contains("componentes de uma tarefa inexistente: TASK-09"));
}

#[test]
fn components_facts_different_from_computed_are_refused_by_validate() {
    let project = Project::new();
    let facts = "| TASK-01 | 0 | 0 | não | não | não |\n| TASK-02 | 0 | 1 | não | não | não |";
    let rows = format!(
        "{}\n{}",
        profile_row("TASK-01", "desenvolvedor", "mecanica", "RF-01, CT-01"),
        profile_row("TASK-02", "desenvolvedor", "mecanica", "RF-02")
    );
    let plan = with_components(
        &profile_plan(&rows, facts),
        "| TASK-01 | `src/a.rs`, `src/b.rs`, `src/c.rs` |\n| TASK-02 | `docs/d.md` |",
    );
    spec_and_plan(&project, "feature", COMPONENT_SPEC, &plan);
    assert!(validate_stderr(&project).contains(
        "TASK-01: arquivos a criar e a alterar diferentes dos calculados pelos componentes (execute vostok plan score)"
    ));
}

#[test]
fn components_legacy_plan_without_the_section_keeps_typed_facts() {
    for plan in [
        profile_plan(
            &profile_row("TASK-01", "desenvolvedor", "-", "RF-01, CT-01, RF-02"),
            "| TASK-01 | 2 | 3 | não | não | não |",
        ),
        legacy_plan(
            "| TASK-01 | Tarefa | Faz. | tarefa | simples | - | RF-01, CT-01, RF-02 |",
            "| TASK-01 | 2 | 3 | não | não | não |",
        ),
    ] {
        let project = Project::new();
        spec_and_plan(&project, "feature", COMPONENT_SPEC, &plan);
        score(&project);
        let text = plan_text(&project);
        assert!(
            fact_line(&text, "TASK-01").starts_with("| TASK-01 | 2 | 3 |"),
            "{text}"
        );
        assert!(!text.contains("Componentes por tarefa"));
    }
}

const EDGE_SPEC: &str = "\n# Exemplo\n\n## Requisitos funcionais\n\n| ID | Requisito |\n|---|---|\n| RF-01 | Um. |\n\n## Casos de teste\n\n| ID | Cenário | Passos | Resultado esperado | Requisitos |\n|---|---|---|---|---|\n| CT-01 | Caso. | Rodar. | Ok. | RF-01 |\n\n## Casos de borda\n\n| ID | Situação | Comportamento esperado | Requisitos |\n|---|---|---|---|\n| CB-01 | Borda. | Trata. | RF-01 |\n| CB-02 | Outra. | Trata. | RF-01 |\n";

fn coverage_plan(covers: &str) -> String {
    profile_plan(
        &profile_row("TASK-01", "desenvolvedor", "mecanica", covers),
        "| TASK-01 | 0 | 1 | não | não | não |",
    )
}

#[test]
fn coverage_fix_and_feature_accept_cb_as_optional() {
    for kind in ["fix", "feature"] {
        let project = Project::new();
        spec_and_plan(
            &project,
            kind,
            EDGE_SPEC,
            &coverage_plan("RF-01, CT-01, CB-01"),
        );
        project
            .vostok()
            .args(["plan", "validate", "SPEC-001"])
            .assert()
            .success();
    }
}

#[test]
fn coverage_missing_rf_or_ct_is_refused_for_fix_and_feature() {
    for kind in ["fix", "feature"] {
        let project = Project::new();
        spec_and_plan(&project, kind, EDGE_SPEC, &coverage_plan("RF-01, CB-01"));
        assert!(validate_stderr(&project).contains("CT-01 não é coberto por nenhuma tarefa"));
        spec_and_plan(&project, kind, EDGE_SPEC, &coverage_plan("CT-01, CB-01"));
        assert!(validate_stderr(&project).contains("RF-01 não é coberto por nenhuma tarefa"));
    }
}

#[test]
fn coverage_unknown_cb_is_refused() {
    let project = Project::new();
    spec_and_plan(
        &project,
        "feature",
        EDGE_SPEC,
        &coverage_plan("RF-01, CT-01, CB-09"),
    );
    assert!(validate_stderr(&project).contains("a tarefa cobre CB-09, que não existe na spec"));
}

// --- Ondas e arquivos em comum (SPEC-014: RF-14, RF-31, CB-04, CT-09, CT-20, RNF-02) ---

const SHARED_HEADER: &str = "## Arquivos em comum por onda\n\nCalculada pelo score.\n\n| Onda | Arquivo | Tarefas |\n|---|---|---|";

/// Corpo de plano com as colunas "Onda" e os componentes por tarefa, mais a seção de arquivos em comum.
fn wave_plan(rows: &str, facts: &str, components: &str) -> String {
    format!("\n# Plano\n\n## Visão geral\n\nUm plano.\n\n## Tarefas\n\n| ID | Título | Descrição | Tipo | Persona | Perfil | Dependências | Onda | Cobre |\n|---|---|---|---|---|---|---|---|---|\n{rows}\n\n## Componentes por tarefa\n\n| ID | Componentes |\n|---|---|\n{components}\n\n## Fatos de esforço\n\n{FACTS_HEADER}\n{facts}\n\n{SHARED_HEADER}\n| - | - | - |\n")
}

fn wave_row(id: &str, depends: &str, wave: &str, covers: &str) -> String {
    format!("| {id} | Tarefa {id} | Faz. | tarefa | desenvolvedor | - | {depends} | {wave} | {covers} |")
}

fn stored_wave(plan: &str, id: &str) -> String {
    let row = plan
        .lines()
        .find(|line| line.starts_with(&format!("| {id} |")))
        .unwrap_or_else(|| panic!("{id} ausente: {plan}"));
    row.split('|').nth(8).unwrap().trim().to_string()
}

fn shared_section(plan: &str) -> String {
    let start = plan.find("## Arquivos em comum por onda").unwrap();
    let rest = &plan[start..];
    let end = rest[3..].find("\n## ").map_or(rest.len(), |n| n + 3);
    rest[..end].to_string()
}

fn dash_facts(count: usize) -> String {
    (1..=count)
        .map(|n| format!("| TASK-{n:02} | - | - | não | não | não |"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn dash_components(count: usize) -> String {
    (1..=count)
        .map(|n| format!("| TASK-{n:02} | - |"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Projeto com as tarefas dadas `(depende de, onda gravada)`, sem componentes.
fn wave_project(tasks: &[(&str, &str)]) -> Project {
    let rows: Vec<String> = tasks
        .iter()
        .enumerate()
        .map(|(n, (depends, wave))| wave_row(&format!("TASK-{:02}", n + 1), depends, wave, "-"))
        .collect();
    let project = Project::new();
    spec_and_plan(
        &project,
        "feature",
        COMPONENT_SPEC,
        &wave_plan(
            &rows.join("\n"),
            &dash_facts(tasks.len()),
            &dash_components(tasks.len()),
        ),
    );
    project
}

#[test]
fn waves_follow_the_dependencies() {
    let project = wave_project(&[
        ("-", "-"),
        ("-", "-"),
        ("TASK-01", "-"),
        ("TASK-02, TASK-03", "-"),
    ]);
    score(&project);
    let plan = plan_text(&project);
    let waves: Vec<String> = (1..=4)
        .map(|n| stored_wave(&plan, &format!("TASK-{n:02}")))
        .collect();
    assert_eq!(waves, ["1", "1", "2", "3"], "{plan}");
}

#[test]
fn waves_task_without_dependencies_is_in_wave_1() {
    let project = wave_project(&[("-", "-")]);
    score(&project);
    let plan = plan_text(&project);
    assert_eq!(stored_wave(&plan, "TASK-01"), "1", "{plan}");
    assert!(plan.contains("| - | 1 | - |"), "{plan}");
}

#[test]
fn waves_and_profiles_are_the_same_in_two_runs() {
    let project = wave_project(&[("-", "-"), ("TASK-01", "-")]);
    score(&project);
    let first = plan_text(&project);
    score(&project);
    let second = plan_text(&project);
    assert_eq!(tasks_section(&first), tasks_section(&second));
    assert_eq!(shared_section(&first), shared_section(&second));
}

#[test]
fn waves_different_from_the_computed_are_refused_by_validate() {
    let project = wave_project(&[("-", "2")]);
    assert!(validate_stderr(&project)
        .contains("TASK-01: onda \"2\" diferente da calculada \"1\" (execute vostok plan score)"));
}

#[test]
fn waves_shared_files_are_listed_by_score() {
    let project = Project::new();
    let rows = format!(
        "{}\n{}",
        wave_row("TASK-01", "-", "-", "RF-01, CT-01"),
        wave_row("TASK-02", "-", "-", "RF-02")
    );
    spec_and_plan(
        &project,
        "feature",
        COMPONENT_SPEC,
        &wave_plan(
            &rows,
            &dash_facts(2),
            "| TASK-01 | `src/a.rs` |\n| TASK-02 | `src/a.rs`, `src/b.rs` |",
        ),
    );
    let out = score(&project);
    let section = shared_section(&plan_text(&project));
    assert!(
        section.contains("| 1 | `src/a.rs` | TASK-01, TASK-02 |"),
        "{section}"
    );
    assert!(!section.contains("src/b.rs"), "{section}");
    assert!(!section.contains("| - | - | - |"), "{section}");
    assert!(
        out.contains("Arquivos em comum por onda (executar em sequência):"),
        "{out}"
    );
    assert!(
        out.contains("- onda 1: `src/a.rs` (TASK-01, TASK-02)"),
        "{out}"
    );
}

#[test]
fn waves_files_shared_across_waves_are_not_listed() {
    let project = Project::new();
    let rows = format!(
        "{}\n{}",
        wave_row("TASK-01", "-", "-", "RF-01, CT-01"),
        wave_row("TASK-02", "TASK-01", "-", "RF-02")
    );
    spec_and_plan(
        &project,
        "feature",
        COMPONENT_SPEC,
        &wave_plan(
            &rows,
            &dash_facts(2),
            "| TASK-01 | `src/a.rs` |\n| TASK-02 | `src/a.rs` |",
        ),
    );
    let out = score(&project);
    let section = shared_section(&plan_text(&project));
    assert!(section.contains("| - | - | - |"), "{section}");
    assert!(
        out.contains("Nenhum arquivo em comum entre tarefas da mesma onda."),
        "{out}"
    );
}

#[test]
fn waves_outdated_shared_files_section_is_refused_by_validate() {
    let project = wave_project(&[("-", "-")]);
    score(&project);
    let edited = plan_text(&project).replace(
        "|---|---|---|\n| - | - | - |",
        "|---|---|---|\n| 1 | `src/a.rs` | TASK-01, TASK-02 |",
    );
    assert_ne!(edited, plan_text(&project));
    write_file(&project, "vostok/work/plans/SPEC-001.md", &edited);
    assert!(validate_stderr(&project).contains(
        "a seção 'Arquivos em comum por onda' está desatualizada (execute vostok plan score)"
    ));
}

#[test]
fn waves_create_tasks_writes_the_wave_field() {
    let project = Project::new();
    let rows = format!(
        "{}\n{}",
        wave_row("TASK-01", "-", "-", "RF-01, CT-01"),
        wave_row("TASK-02", "TASK-01", "-", "RF-02")
    );
    spec_and_plan(
        &project,
        "feature",
        COMPONENT_SPEC,
        &wave_plan(
            &rows,
            &dash_facts(2),
            "| TASK-01 | `src/a.rs`, `src/b.rs` |\n| TASK-02 | `src/c.rs`, `docs/d.md` |",
        ),
    );
    score(&project);
    project
        .vostok()
        .args(["plan", "approve", "create-tasks", "SPEC-001"])
        .assert()
        .success();
    let read = |id: &str| {
        std::fs::read_to_string(
            project
                .root()
                .join(format!("vostok/work/tasks/SPEC-001/{id}.md")),
        )
        .unwrap()
    };
    assert!(
        read("TASK-01").contains("\nwave: 1\n"),
        "{}",
        read("TASK-01")
    );
    assert!(
        read("TASK-02").contains("\nwave: 2\n"),
        "{}",
        read("TASK-02")
    );
}

#[test]
fn waves_plan_without_the_column_stays_valid() {
    let project = Project::new();
    spec_and_plan(
        &project,
        "improvement",
        LEGACY_SPEC_ONE,
        &profile_plan(
            &profile_row("TASK-01", "desenvolvedor", "-", "RF-01"),
            "| TASK-01 | 0 | 1 | não | não | não |",
        ),
    );
    score(&project);
    let plan = plan_text(&project);
    assert!(!plan.contains("Onda"), "{plan}");
    project
        .vostok()
        .args(["plan", "validate", "SPEC-001"])
        .assert()
        .success();

    let project = Project::new();
    spec_and_plan(
        &project,
        "improvement",
        LEGACY_SPEC_ONE,
        &legacy_plan(
            "| TASK-01 | Tarefa | Faz. | tarefa | simples | - | RF-01 |\n| TASK-02 | Tarefa 2 | Faz. | tarefa | simples | TASK-01 | RF-01 |",
            "| TASK-01 | 0 | 1 | não | não | não |\n| TASK-02 | 0 | 1 | não | não | não |",
        ),
    );
    project
        .vostok()
        .args(["plan", "validate", "SPEC-001"])
        .assert()
        .success();
    project
        .vostok()
        .args(["plan", "approve", "create-tasks", "SPEC-001"])
        .assert()
        .success();
    let task = |id: &str| {
        std::fs::read_to_string(
            project
                .root()
                .join(format!("vostok/work/tasks/SPEC-001/{id}.md")),
        )
        .unwrap()
    };
    assert!(task("TASK-01").contains("\nwave: 1\n"));
    assert!(task("TASK-02").contains("\nwave: 2\n"));
}

#[test]
fn waves_public_function_computes_the_waves() {
    let tasks = vec![
        ("TASK-01".to_string(), vec![]),
        ("TASK-02".to_string(), vec!["TASK-01".to_string()]),
        ("TASK-03".to_string(), vec![]),
    ];
    let waves = vostok::commands::plan::waves(&tasks);
    assert_eq!(waves["TASK-01"], 1);
    assert_eq!(waves["TASK-02"], 2);
    assert_eq!(waves["TASK-03"], 1);
}

// --- Baseline sem a entrega (SPEC-017) ---

const DELIVERY_MESSAGE: &str = "'Baseline' com comando que cita 'novo.txt', que a tarefa cria";

fn set_baseline(task: &std::path::Path, command: &str) {
    let text = std::fs::read_to_string(task).unwrap();
    std::fs::write(
        task,
        text.replace(
            "### Baseline\n\n- `true`",
            &format!("### Baseline\n\n- `{command}`"),
        ),
    )
    .unwrap();
}

#[test]
fn baseline_delivery_is_refused_by_the_plan_validation() {
    let project = Project::new();
    let task = plan_with_evidence(&project, "Não se aplica.");
    set_baseline(&task, "test -e novo.txt && echo existe");
    let output = project
        .vostok()
        .args(["plan", "approve", "validate", "SPEC-001"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let stderr = String::from_utf8(output).unwrap();
    assert!(stderr.contains(DELIVERY_MESSAGE), "{stderr}");
}

#[test]
fn baseline_delivery_other_commands_stay_valid_in_the_plan() {
    let project = Project::new();
    let task = plan_with_evidence(&project, "Não se aplica.");
    set_baseline(&task, "cargo --version");
    project
        .vostok()
        .args(["plan", "approve", "validate", "SPEC-001"])
        .assert()
        .success();
}

// --- Comando de baseline que mede a entrega (SPEC-018) ---

#[test]
fn delivery_command_new_folder_is_refused_by_the_plan_validation() {
    let project = Project::new();
    let task = plan_with_evidence(&project, "Não se aplica.");
    let text = std::fs::read_to_string(&task).unwrap();
    std::fs::write(
        &task,
        text.replace("- `novo.txt` (criar)", "- `src/modulo/a.rs` (criar)")
            .replace(
                "### Baseline\n\n- `true`",
                "### Baseline\n\n- `test -e src/modulo && echo existe`",
            ),
    )
    .unwrap();
    let output = project
        .vostok()
        .args(["plan", "approve", "validate", "SPEC-001"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let stderr = String::from_utf8(output).unwrap();
    assert!(
        stderr.contains("'Baseline' com comando que cita 'src/modulo', pasta que a tarefa cria"),
        "{stderr}"
    );
}

// --- Aviso de `--` e regras novas de evidência no plano (SPEC-020) ---

fn plan_task_with(project: &Project, verification: &str, evidence: &str) -> std::path::PathBuf {
    let task = plan_with_evidence(project, evidence);
    let content = std::fs::read_to_string(&task).unwrap().replace(
        "```bash\ntrue\n```",
        &format!("```bash\n{verification}\n```"),
    );
    std::fs::write(&task, content).unwrap();
    task
}

fn approve_validate(project: &Project) -> (bool, String) {
    let output = project
        .vostok()
        .args(["plan", "approve", "validate", "SPEC-001"])
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (output.status.success(), text)
}

fn simple_evidence() -> String {
    format!("- `saida.txt` (automática): `echo ok`\n{SUBITEMS}")
}

#[test]
fn dashdash_warns_for_pnpm_npm_and_yarn() {
    for command in [
        "pnpm --filter web test -- src/x",
        "npm run test -- src/x",
        "yarn test -- src/x",
    ] {
        let project = Project::new();
        plan_task_with(&project, command, &simple_evidence());
        let (success, text) = approve_validate(&project);
        assert!(success, "{command}: {text}");
        assert!(
            text.contains(&format!(
                "Aviso: TASK-01: o comando '{command}' passa argumentos depois de '--'"
            )),
            "{command}: {text}"
        );
    }
}

#[test]
fn dashdash_does_not_warn_for_cargo() {
    let project = Project::new();
    plan_task_with(&project, "cargo test -- brief_", &simple_evidence());
    let (success, text) = approve_validate(&project);
    assert!(success, "{text}");
    assert!(!text.contains("Aviso: TASK-01: o comando"), "{text}");
}

#[test]
fn plan_evidence_rules_accept_the_new_kinds() {
    let project = Project::new();
    let evidence = format!(
        "- `saida.txt` (automática, da verificação)\n{SUBITEMS}\n- `novo.diff` (diff): `novo.txt`\n{SUBITEMS}\n- `tela.png` (print): `npx playwright screenshot`\n{SUBITEMS}\n  - Espera: `ok`"
    );
    plan_task_with(&project, "# evidência: saida.txt\necho ok", &evidence);
    let (success, text) = approve_validate(&project);
    assert!(success, "{text}");
}

#[test]
fn plan_evidence_rules_refuse_a_label_without_evidence() {
    let project = Project::new();
    plan_task_with(
        &project,
        "# evidência: outra.txt\necho ok",
        &simple_evidence(),
    );
    let (success, text) = approve_validate(&project);
    assert!(!success, "{text}");
    assert!(
        text.contains("rótulo `# evidência: outra.txt` sem a evidência"),
        "{text}"
    );
}

#[test]
fn plan_evidence_rules_refuse_reading_a_changed_path() {
    let project = Project::new();
    plan_task_with(
        &project,
        "true",
        &format!("- `leitura.txt` (automática): `grep -n x novo.txt`\n{SUBITEMS}"),
    );
    let (success, text) = approve_validate(&project);
    assert!(!success, "{text}");
    assert!(
        text.contains("a evidência leitura.txt só lê 'novo.txt', que a tarefa muda"),
        "{text}"
    );
}

#[test]
fn plan_evidence_rules_require_a_print_for_an_interface_task() {
    let project = Project::new();
    let task = plan_task_with(&project, "true", &simple_evidence());
    let content = std::fs::read_to_string(&task).unwrap().replace(
        "- `novo.txt` (criar): arquivo novo.",
        "- `tela.tsx` (criar): tela nova.",
    );
    std::fs::write(&task, content).unwrap();
    let (success, text) = approve_validate(&project);
    assert!(!success, "{text}");
    assert!(text.contains("tarefa com interface sem print"), "{text}");
}
