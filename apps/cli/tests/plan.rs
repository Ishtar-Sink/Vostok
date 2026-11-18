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

const SUBITEMS: &str =
    "  - Título: Saída do comando.\n  - Prova: ok.\n  - Trecho: ok.\n  - Falha: erro.";

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

// --- Título das evidências (SPEC-043) ---

#[test]
fn evidence_title_missing_is_refused_by_the_plan_approval() {
    let project = Project::new();
    plan_with_evidence(
        &project,
        "- `saida.txt` (automática): `echo ok`\n  - Prova: ok.\n  - Trecho: ok.\n  - Falha: erro.",
    );
    let refused = failure_stderr(
        project
            .vostok()
            .args(["plan", "approve", "validate", "SPEC-001"]),
    );
    assert!(refused.contains("sem o subitem 'Título'"), "{refused}");
    assert!(refused.contains("saida.txt"), "{refused}");
}

#[test]
fn evidence_title_longer_than_60_is_refused_by_the_plan_approval() {
    let project = Project::new();
    plan_with_evidence(
        &project,
        &format!(
            "- `saida.txt` (automática): `echo ok`\n  - Título: {}\n  - Prova: ok.\n  - Trecho: ok.\n  - Falha: erro.",
            "á".repeat(61)
        ),
    );
    let refused = failure_stderr(
        project
            .vostok()
            .args(["plan", "approve", "validate", "SPEC-001"]),
    );
    assert!(refused.contains("mais de 60 caracteres"), "{refused}");
    assert!(refused.contains("saida.txt"), "{refused}");
}

// --- Campos novos das evidências (SPEC-055) ---

const NEW_FIELDS_EVIDENCE: &str = "- `saida.txt` (automática): `echo ok`\n  - Título: Saída aparece\n  - Mostra: o comando escreve ok.\n  - Cobre: RF-01\n  - Onde olhar: a primeira linha.\n  - Trecho: a linha `ok`.\n  - Falha: se aparecer erro, então o comando quebrou.\n  - Reproduzir: `echo ok`";

fn plan_validation_passes(evidence: &str) {
    let project = Project::new();
    plan_with_evidence(&project, evidence);
    project
        .vostok()
        .args(["plan", "approve", "validate", "SPEC-001"])
        .assert()
        .success();
}

fn plan_validation_refusal(evidence: &str) -> String {
    let project = Project::new();
    plan_with_evidence(&project, evidence);
    failure_stderr(
        project
            .vostok()
            .args(["plan", "approve", "validate", "SPEC-001"]),
    )
}

#[test]
fn new_evidence_fields_mostra_without_prova_is_approved_by_the_plan() {
    plan_validation_passes(NEW_FIELDS_EVIDENCE);
}

#[test]
fn new_evidence_fields_prova_and_mostra_together_are_approved_by_the_plan() {
    plan_validation_passes(&format!("{NEW_FIELDS_EVIDENCE}\n  - Prova: ok."));
}

#[test]
fn new_evidence_fields_without_prova_and_mostra_are_refused_by_the_plan() {
    let evidence = NEW_FIELDS_EVIDENCE.replace("\n  - Mostra: o comando escreve ok.", "");
    let refused = plan_validation_refusal(&evidence);
    assert!(
        refused.contains("sem o subitem 'Prova' ou 'Mostra'"),
        "{refused}"
    );
    assert!(refused.contains("saida.txt"), "{refused}");
}

#[test]
fn new_evidence_fields_unknown_multi_word_field_is_refused_by_the_plan() {
    let refused = plan_validation_refusal(&format!("{NEW_FIELDS_EVIDENCE}\n  - Onde ver: x"));
    assert!(refused.contains("desconhecido 'Onde ver'"), "{refused}");
}

#[test]
fn current_evidence_format_is_approved_by_the_plan() {
    plan_validation_passes(
        "- `saida.txt` (automática): `echo ok`\n  - Título: Saída aparece\n  - Prova: ok.\n  - Trecho: ok.\n  - Falha: erro.",
    );
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
            "| ID | Título | Descrição | Tipo | Persona | Perfil | Dependências | Fecha depois de | Onda | Cobre |\n|---|---|---|---|---|---|---|---|---|---|"
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
    let task = std::fs::read_to_string(
        project
            .root()
            .join("vostok/work/tasks/SPEC-001/TASK-01/estado.yml"),
    )
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
    let task = std::fs::read_to_string(
        project
            .root()
            .join("vostok/work/tasks/SPEC-001/TASK-01/estado.yml"),
    )
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

// --- criar só na primeira tarefa da cadeia (SPEC-072) ---

fn chain_row(id: &str, depends: &str, covers: &str) -> String {
    format!("| {id} | Tarefa {id} | Faz. | tarefa | desenvolvedor | - | {depends} | {covers} |")
}

fn chain_facts(rows: &[(&str, &str)]) -> String {
    rows.iter()
        .map(|(id, files)| format!("| {id} | {files} | não | não | não |"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn chain_project(rows: &str, facts: &str, components: &str) -> Project {
    let project = Project::new();
    spec_and_plan(
        &project,
        "feature",
        COMPONENT_SPEC,
        &with_components(&profile_plan(rows, facts), components),
    );
    project
}

/// TASK-01 cria `src/a.rs`; TASK-02 depende dela (ou de `dependency`) e também o lista.
fn three_task_chain(dependency: &str, facts: &[(&str, &str)]) -> Project {
    let rows = [
        chain_row("TASK-01", "-", "RF-01, CT-01"),
        chain_row("TASK-02", dependency, "-"),
        chain_row("TASK-03", "-", "RF-02"),
    ]
    .join("\n");
    chain_project(
        &rows,
        &chain_facts(facts),
        "| TASK-01 | `src/a.rs` |\n| TASK-02 | `src/a.rs`, `src/b.rs` |\n| TASK-03 | `src/c.rs`, `docs/d.md` |",
    )
}

const DASH_THREE: [(&str, &str); 3] = [
    ("TASK-01", "- | -"),
    ("TASK-02", "- | -"),
    ("TASK-03", "- | -"),
];

#[test]
fn creates_once_dependent_task_counts_the_new_file_as_change() {
    let project = three_task_chain("TASK-01", &DASH_THREE);
    score(&project);
    let plan = plan_text(&project);
    assert!(
        fact_line(&plan, "TASK-01").starts_with("| TASK-01 | 1 | 0 |"),
        "{plan}"
    );
    assert!(
        fact_line(&plan, "TASK-02").starts_with("| TASK-02 | 0 | 2 |"),
        "{plan}"
    );
    assert!(
        fact_line(&plan, "TASK-03").starts_with("| TASK-03 | 0 | 2 |"),
        "{plan}"
    );
}

#[test]
fn creates_once_validate_accepts_the_corrected_facts() {
    let project = three_task_chain("TASK-01", &DASH_THREE);
    score(&project);
    project
        .vostok()
        .args(["plan", "validate", "SPEC-001"])
        .assert()
        .success();
}

#[test]
fn creates_once_validate_refuses_the_old_facts() {
    let project = three_task_chain(
        "TASK-01",
        &[
            ("TASK-01", "1 | 0"),
            ("TASK-02", "1 | 1"),
            ("TASK-03", "0 | 2"),
        ],
    );
    let stderr = validate_stderr(&project);
    assert!(
        stderr.contains("TASK-02: arquivos a criar e a alterar diferentes dos calculados pelos componentes (execute vostok plan score)"),
        "{stderr}"
    );
}

#[test]
fn creates_once_indirect_dependency_counts_as_change() {
    let rows = [
        chain_row("TASK-01", "-", "RF-01, CT-01"),
        chain_row("TASK-02", "TASK-01", "RF-02"),
        chain_row("TASK-03", "TASK-02", "-"),
    ]
    .join("\n");
    let project = chain_project(
        &rows,
        &chain_facts(&DASH_THREE),
        "| TASK-01 | `src/a.rs` |\n| TASK-02 | `src/b.rs` |\n| TASK-03 | `src/a.rs` |",
    );
    score(&project);
    let plan = plan_text(&project);
    assert!(
        fact_line(&plan, "TASK-03").starts_with("| TASK-03 | 0 | 1 |"),
        "{plan}"
    );
}

#[test]
fn creates_once_independent_tasks_both_count_create() {
    let rows = [
        chain_row("TASK-01", "-", "RF-01, CT-01"),
        chain_row("TASK-02", "-", "RF-02"),
    ]
    .join("\n");
    let project = chain_project(
        &rows,
        &chain_facts(&[("TASK-01", "- | -"), ("TASK-02", "- | -")]),
        "| TASK-01 | `src/a.rs` |\n| TASK-02 | `src/a.rs` |",
    );
    score(&project);
    let plan = plan_text(&project);
    assert!(
        fact_line(&plan, "TASK-01").starts_with("| TASK-01 | 1 | 0 |"),
        "{plan}"
    );
    assert!(
        fact_line(&plan, "TASK-02").starts_with("| TASK-02 | 1 | 0 |"),
        "{plan}"
    );
}

#[test]
fn creates_once_missing_dependency_keeps_the_spec_action() {
    let project = three_task_chain(
        "TASK-09",
        &[
            ("TASK-01", "1 | 0"),
            ("TASK-02", "1 | 1"),
            ("TASK-03", "0 | 2"),
        ],
    );
    let stderr = validate_stderr(&project);
    assert!(
        stderr.contains("TASK-02: dependência inexistente TASK-09"),
        "{stderr}"
    );
    assert!(
        !stderr.contains("TASK-02: arquivos a criar e a alterar diferentes"),
        "{stderr}"
    );
}

#[test]
fn creates_once_cycle_keeps_the_spec_action() {
    let rows = [
        chain_row("TASK-01", "TASK-02", "RF-01, CT-01"),
        chain_row("TASK-02", "TASK-01", "RF-02"),
        chain_row("TASK-03", "TASK-02", "-"),
    ]
    .join("\n");
    let project = chain_project(
        &rows,
        &chain_facts(&[
            ("TASK-01", "1 | 0"),
            ("TASK-02", "1 | 0"),
            ("TASK-03", "1 | 0"),
        ]),
        "| TASK-01 | `src/a.rs` |\n| TASK-02 | `src/a.rs` |\n| TASK-03 | `src/a.rs` |",
    );
    let stderr = validate_stderr(&project);
    assert!(
        stderr.contains("dependência em ciclo envolvendo"),
        "{stderr}"
    );
    assert!(
        !stderr.contains("arquivos a criar e a alterar diferentes"),
        "{stderr}"
    );
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
                .join(format!("vostok/work/tasks/SPEC-001/{id}/estado.yml")),
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
                .join(format!("vostok/work/tasks/SPEC-001/{id}/estado.yml")),
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

// --- Fecha depois de (SPEC-044) ---

fn closes_plan(rows: &str, count: usize) -> String {
    let components = if count == 2 {
        "| TASK-01 | `src/a.rs`, `src/b.rs` |\n| TASK-02 | `src/c.rs`, `docs/d.md` |".to_string()
    } else {
        dash_components(count)
    };
    format!("\n# Plano\n\n## Visão geral\n\nUm plano.\n\n## Tarefas\n\n| ID | Título | Descrição | Tipo | Persona | Perfil | Dependências | Fecha depois de | Onda | Cobre |\n|---|---|---|---|---|---|---|---|---|---|\n{rows}\n\n## Componentes por tarefa\n\n| ID | Componentes |\n|---|---|\n{components}\n\n## Fatos de esforço\n\n{FACTS_HEADER}\n{}\n\n{SHARED_HEADER}\n| - | - | - |\n", dash_facts(count))
}

fn closes_row(id: &str, depends: &str, closes: &str, wave: &str, covers: &str) -> String {
    format!("| {id} | Tarefa {id} | Faz. | tarefa | desenvolvedor | - | {depends} | {closes} | {wave} | {covers} |")
}

fn closes_project(rows: &[String]) -> Project {
    let project = Project::new();
    spec_and_plan(
        &project,
        "feature",
        COMPONENT_SPEC,
        &closes_plan(&rows.join("\n"), rows.len()),
    );
    project
}

fn closes_pair() -> Project {
    closes_project(&[
        closes_row("TASK-01", "-", "-", "-", "RF-01, CT-01"),
        closes_row("TASK-02", "-", "TASK-01", "-", "RF-02"),
    ])
}

fn task_state(project: &Project, id: &str) -> String {
    std::fs::read_to_string(
        project
            .root()
            .join(format!("vostok/work/tasks/SPEC-001/{id}/estado.yml")),
    )
    .unwrap()
}

#[test]
fn closes_after_plan_is_valid_and_keeps_the_waves() {
    let project = closes_pair();
    score(&project);
    let plan = plan_text(&project);
    for id in ["TASK-01", "TASK-02"] {
        let row = plan
            .lines()
            .find(|line| line.starts_with(&format!("| {id} |")))
            .unwrap();
        assert_eq!(row.split('|').nth(9).unwrap().trim(), "1", "{plan}");
    }
    project
        .vostok()
        .args(["plan", "validate", "SPEC-001"])
        .assert()
        .success();
}

#[test]
fn closes_after_plan_create_tasks_writes_the_field() {
    let project = closes_pair();
    score(&project);
    project
        .vostok()
        .args(["plan", "approve", "create-tasks", "SPEC-001"])
        .assert()
        .success();
    let second = task_state(&project, "TASK-02");
    assert!(second.contains("\ncloses_after: [TASK-01]\n"), "{second}");
    let first = task_state(&project, "TASK-01");
    assert!(first.contains("\ncloses_after: []\n"), "{first}");
    assert!(first.contains("\nwaiting_on: []\n"), "{first}");
    assert!(first.contains("\nwaiting_reason: null\n"), "{first}");
}

#[test]
fn closes_after_plan_without_the_column_writes_empty_lists() {
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
    let first = task_state(&project, "TASK-01");
    assert!(first.contains("\ncloses_after: []\n"), "{first}");
}

#[test]
fn closes_after_plan_refuses_unknown_self_and_cycle() {
    let unknown = closes_project(&[closes_row("TASK-01", "-", "TASK-09", "-", "-")]);
    assert!(validate_stderr(&unknown)
        .contains("TASK-01: \"Fecha depois de\" cita tarefa inexistente TASK-09"));
    let itself = closes_project(&[closes_row("TASK-01", "-", "TASK-01", "-", "-")]);
    assert!(validate_stderr(&itself).contains("TASK-01: fecha depois de si mesma"));
    let cycle = closes_project(&[
        closes_row("TASK-01", "-", "TASK-02", "-", "-"),
        closes_row("TASK-02", "-", "TASK-01", "-", "-"),
    ]);
    assert!(validate_stderr(&cycle).contains("\"Fecha depois de\" em ciclo envolvendo"));
}

#[test]
fn closes_after_plan_accepts_a_task_also_in_the_dependencies() {
    let project = closes_project(&[
        closes_row("TASK-01", "-", "-", "-", "RF-01, CT-01"),
        closes_row("TASK-02", "TASK-01", "TASK-01", "-", "RF-02"),
    ]);
    score(&project);
    project
        .vostok()
        .args(["plan", "validate", "SPEC-001"])
        .assert()
        .success();
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
fn grep_aviso_para_substituicao_e_wc() {
    for command in ["a=$(grep x valor.txt | wc -l)", "grep x valor.txt | wc -l"] {
        let project = Project::new();
        plan_task_with(&project, command, &simple_evidence());
        let (success, text) = approve_validate(&project);
        assert!(success, "{command}: {text}");
        assert!(
            text.contains(&format!(
                "Aviso: TASK-01: o comando '{command}' sai com erro sob pipefail quando o grep não acha nada"
            )),
            "{command}: {text}"
        );
    }
}

#[test]
fn grep_aviso_nao_para_contagem_ou_true() {
    for command in [
        "n=$(grep -c x valor.txt)",
        "a=$(grep x valor.txt | wc -l || true)",
        "cargo test 2>&1 | grep -E ok | sort",
    ] {
        let project = Project::new();
        plan_task_with(&project, command, &simple_evidence());
        let (success, text) = approve_validate(&project);
        assert!(success, "{command}: {text}");
        assert!(
            !text.contains("sai com erro sob pipefail"),
            "{command}: {text}"
        );
    }
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

// --- Mudança esperada no Baseline (SPEC-021) ---

fn expected_change_plan(item: &str) -> (Project, bool, String) {
    let project = Project::new();
    let task = plan_with_evidence(&project, "Não se aplica.");
    let text = std::fs::read_to_string(&task).unwrap();
    std::fs::write(&task, text.replace("### Baseline\n\n- `true`", item)).unwrap();
    let output = project
        .vostok()
        .args(["plan", "approve", "validate", "SPEC-001"])
        .output()
        .unwrap();
    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (project, output.status.success(), all)
}

#[test]
fn expected_change_with_a_reason_is_accepted_by_the_plan_validation() {
    let (_, ok, all) =
        expected_change_plan("### Baseline\n\n- `cargo --version` (muda): a versão muda");
    assert!(ok, "{all}");
}

#[test]
fn expected_change_without_a_reason_is_refused_by_the_plan_validation() {
    let (_, ok, all) = expected_change_plan("### Baseline\n\n- `cargo --version` (muda)");
    assert!(!ok, "{all}");
    assert!(
        all.contains("'Baseline' com item (muda) sem motivo: - `cargo --version` (muda)"),
        "{all}"
    );
}

// --- Requer nas evidências (SPEC-022) ---

fn requires_plan(requires: &str) -> (bool, String) {
    let project = Project::new();
    plan_with_evidence(
        &project,
        &format!("- `saida.txt` (automática): `test -s novo.txt && echo existe`\n{SUBITEMS}\n  - Requer: {requires}"),
    );
    let output = project
        .vostok()
        .args(["plan", "approve", "validate", "SPEC-001"])
        .output()
        .unwrap();
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn requires_a_created_path_is_accepted_by_the_plan_validation() {
    let (ok, stderr) = requires_plan("`novo.txt`");
    assert!(ok, "{stderr}");
}

#[test]
fn requires_a_path_the_task_does_not_create_is_refused_by_the_plan_validation() {
    let (ok, stderr) = requires_plan("`README.md`");
    assert!(!ok);
    assert!(
        stderr.contains("a evidência saida.txt requer 'README.md', que não é criado pela tarefa"),
        "{stderr}"
    );
}

// --- Tempo-limite das evidências (SPEC-022) ---

fn evidence_timeout_plan(value: &str) -> (bool, String) {
    let project = Project::new();
    plan_with_evidence(
        &project,
        &format!("- `saida.txt` (automática): `echo ok`\n{SUBITEMS}\n  - Tempo-limite: {value}"),
    );
    let output = project
        .vostok()
        .args(["plan", "approve", "validate", "SPEC-001"])
        .output()
        .unwrap();
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn evidence_timeout_number_is_accepted_by_the_plan_validation() {
    let (ok, stderr) = evidence_timeout_plan("900");
    assert!(ok, "{stderr}");
}

#[test]
fn evidence_timeout_text_is_refused_by_the_plan_validation() {
    let (ok, stderr) = evidence_timeout_plan("abc");
    assert!(!ok);
    assert!(
        stderr.contains("a evidência saida.txt tem Tempo-limite que não é um número de segundos"),
        "{stderr}"
    );
}

// --- Tarefas criadas em pastas (SPEC-023) ---

const TASK_FOLDER: &str = "vostok/work/tasks/SPEC-001/TASK-01";

fn read_in(project: &Project, relative: &str) -> String {
    std::fs::read_to_string(project.root().join(relative)).unwrap_or_default()
}

fn created_folder_task() -> Project {
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
    project
}

/// Preenche os arquivos de execução da TASK-01 como o detalhamento faria.
fn fill_folder_task(project: &Project) {
    write_file(
        project,
        &format!("{TASK_FOLDER}/execucao.md"),
        "### Ler\n\n- `README.md`\n\n### Mudar\n\n- `novo.txt` (criar): arquivo novo.\n\n### Passos\n\n1. Crie o arquivo.\n",
    );
    write_file(
        project,
        &format!("{TASK_FOLDER}/verificacao.md"),
        "### Baseline\n\n- `true`\n\n### Verificação\n\n```bash\ntrue\n```\n\n### Evidências\n\nNão se aplica.\n",
    );
    write_file(
        project,
        &format!("{TASK_FOLDER}/criterios.md"),
        "### Concluída quando\n\n- [ ] Pronto (RF-01).\n\n  > Como comprovar: ver o arquivo.\n",
    );
    write_file(project, &format!("{TASK_FOLDER}/executivo.md"), "Resumo.\n");
    write_file(project, &format!("{TASK_FOLDER}/tecnico.md"), "Detalhes.\n");
}

#[test]
fn create_folder_creates_each_task_as_a_folder_with_the_files_of_the_contract() {
    let project = created_folder_task();
    for name in [
        "estado.yml",
        "execucao.md",
        "verificacao.md",
        "criterios.md",
        "executivo.md",
        "tecnico.md",
        "registro.md",
        "README.md",
    ] {
        assert!(
            project.root().join(TASK_FOLDER).join(name).is_file(),
            "{name}"
        );
    }
    assert!(!project
        .root()
        .join("vostok/work/tasks/SPEC-001/TASK-01.md")
        .exists());
    let state = read_in(&project, &format!("{TASK_FOLDER}/estado.yml"));
    assert!(
        state.starts_with("id: TASK-01\nspec: SPEC-001\n"),
        "{state}"
    );
    assert!(state.contains("\nstatus: pending\n"), "{state}");
    assert!(!state.contains("---"), "{state}");
    let execution = read_in(&project, &format!("{TASK_FOLDER}/execucao.md"));
    assert!(
        execution.starts_with("### Ler\n\n#{TASK_READ}"),
        "{execution}"
    );
    assert!(read_in(&project, &format!("{TASK_FOLDER}/executivo.md")).contains("#{TASK_EXECUTIVE}"));
    assert!(read_in(&project, &format!("{TASK_FOLDER}/tecnico.md")).contains("#{TASK_TECHNICAL}"));
    let record = read_in(&project, &format!("{TASK_FOLDER}/registro.md"));
    assert!(record.starts_with("## Registro de execução\n"), "{record}");
    assert!(record.contains("## Histórico de ajustes\n"), "{record}");
}

#[test]
fn create_folder_keeps_the_plan_description_in_the_state_for_the_card() {
    let project = created_folder_task();
    let state = read_in(&project, &format!("{TASK_FOLDER}/estado.yml"));
    assert!(state.contains("\ndescription: \""), "{state}");
    let readme = read_in(&project, &format!("{TASK_FOLDER}/README.md"));
    assert!(readme.starts_with("# TASK-01: "), "{readme}");
    assert!(readme.contains("- **Status:** pending\n"), "{readme}");
    assert!(readme.contains("- **Onda:** 1\n"), "{readme}");
    assert!(readme.contains("- **Bloqueada por:** -\n"), "{readme}");
    assert!(!readme.contains("#{"), "{readme}");
}

#[test]
fn create_folder_validation_asks_for_the_execution_files_but_not_for_the_description() {
    let project = created_folder_task();
    let stderr = String::from_utf8(
        project
            .vostok()
            .args(["plan", "approve", "validate", "SPEC-001"])
            .output()
            .unwrap()
            .stderr,
    )
    .unwrap();
    assert!(
        stderr.contains("TASK-01: variável sem preencher"),
        "{stderr}"
    );
    assert!(!stderr.contains("TASK_EXECUTIVE"), "{stderr}");
    fill_folder_task(&project);
    project
        .vostok()
        .args(["plan", "approve", "validate", "SPEC-001"])
        .assert()
        .success();
}

#[test]
fn create_folder_mark_planned_stores_the_hashes_of_the_execution_files() {
    let project = created_folder_task();
    fill_folder_task(&project);
    project
        .vostok()
        .args(["plan", "approve", "mark-planned", "SPEC-001"])
        .assert()
        .success();
    let state = read_in(&project, &format!("{TASK_FOLDER}/estado.yml"));
    for key in ["content_hash", "execution_hash", "state_hash"] {
        let value = state
            .lines()
            .find_map(|line| line.strip_prefix(&format!("{key}: ")))
            .unwrap_or_default();
        assert_eq!(value.len(), 64, "{key}: {state}");
    }
}

#[test]
fn create_folder_readme_follows_the_status_and_the_criteria() {
    let project = created_folder_task();
    fill_folder_task(&project);
    project
        .vostok()
        .args(["plan", "approve", "mark-planned", "SPEC-001"])
        .assert()
        .success();
    project
        .vostok()
        .args(["task", "approve", "approve", "SPEC-001", "TASK-01"])
        .assert()
        .success();
    let readme = read_in(&project, &format!("{TASK_FOLDER}/README.md"));
    assert!(readme.contains("- **Status:** approved\n"), "{readme}");
    assert!(
        readme.contains("- **Critérios de aceite:**\n  - [ ] Pronto (RF-01).\n"),
        "{readme}"
    );
    assert!(!readme.contains("Como comprovar"), "{readme}");
}

#[test]
fn create_folder_refuses_the_spec_when_a_task_exists_in_both_formats() {
    let project = created_folder_task();
    write_file(
        &project,
        "vostok/work/tasks/SPEC-001/TASK-01.md",
        "---\nid: TASK-01\n---\n",
    );
    let output = project
        .vostok()
        .args(["plan", "approve", "validate", "SPEC-001"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("nos dois formatos"), "{stderr}");
}

#[test]
fn mark_planned_records_the_status_history() {
    let project = Project::new();
    plan_with_evidence(
        &project,
        &format!("- `saida.txt` (automática): `echo ok`\n{SUBITEMS}"),
    );
    let spec = project
        .root()
        .join("vostok/work/specs/refined/SPEC-001-exemplo.md");
    let field = |key: &str| {
        std::fs::read_to_string(&spec)
            .unwrap()
            .lines()
            .find_map(|line| line.strip_prefix(&format!("{key}: ")).map(String::from))
            .unwrap_or_default()
    };
    let hash = field("content_hash");
    project
        .vostok()
        .args(["plan", "approve", "mark-planned", "SPEC-001"])
        .assert()
        .success();
    assert!(
        field("status_history").ends_with(" planned"),
        "{}",
        field("status_history")
    );
    assert_eq!(field("content_hash"), hash);
}

#[test]
fn chat_aval_mark_planned_without_aval_file() {
    let project = Project::new();
    plan_with_evidence(
        &project,
        &format!("- `saida.txt` (automática): `echo ok`\n{SUBITEMS}"),
    );
    let mut config = std::fs::OpenOptions::new()
        .append(true)
        .open(project.harness().join("config/main.yml"))
        .unwrap();
    std::io::Write::write_all(&mut config, b"approval_lock: true\n").unwrap();
    project
        .vostok()
        .args(["plan", "approve", "mark-planned", "SPEC-001"])
        .assert()
        .success();
    let spec = project
        .root()
        .join("vostok/work/specs/refined/SPEC-001-exemplo.md");
    let content = std::fs::read_to_string(spec).unwrap();
    assert!(content.contains("\nstatus: planned\n"), "{content}");
    assert!(!project.harness().join("work/avais").exists());
}

// --- Busca da Falha no baseline (SPEC-057) ---

fn folder_task_with_failure_evidence() -> Project {
    let project = created_folder_task();
    fill_folder_task(&project);
    write_file(
        &project,
        &format!("{TASK_FOLDER}/verificacao.md"),
        "### Baseline\n\n- `true`\n\n### Verificação\n\n```bash\ntrue\n```\n\n### Evidências\n\n- `saida.txt` (automática): `echo rc=1`\n  - Título: Saída do comando.\n  - Prova: o comando roda.\n  - Trecho: a saída.\n  - Falha: `rc`\n",
    );
    project
}

#[test]
fn approve_validate_busca_avisa_falha_no_baseline() {
    let project = folder_task_with_failure_evidence();
    write_file(
        &project,
        &format!("{TASK_FOLDER}/evidencias/baseline/evidence/saida.txt"),
        "$ echo rc=1\nexit: 0\nrc=1\n",
    );
    let (ok, text) = approve_validate(&project);
    assert!(ok, "{text}");
    assert!(
        text.contains("Falha que já aparece no baseline de EV-01 saida.txt:"),
        "{text}"
    );
    assert!(text.contains("- `rc`"), "{text}");
}

#[test]
fn approve_validate_busca_sem_baseline_nao_avisa() {
    let project = folder_task_with_failure_evidence();
    let (ok, text) = approve_validate(&project);
    assert!(ok, "{text}");
    assert!(!text.contains("Falha que já aparece"), "{text}");
}

// --- Falha contraditória e com condição (SPEC-059) ---

fn folder_task_with_failure(trecho: &str, falha: &str) -> Project {
    let project = created_folder_task();
    fill_folder_task(&project);
    write_file(
        &project,
        &format!("{TASK_FOLDER}/verificacao.md"),
        &format!("### Baseline\n\n- `true`\n\n### Verificação\n\n```bash\ntrue\n```\n\n### Evidências\n\n- `saida.txt` (automática): `echo ok`\n  - Título: Saída do comando.\n  - Prova: o comando roda.\n  - Trecho: {trecho}\n  - Falha: {falha}\n"),
    );
    project
}

#[test]
fn approve_validate_falha_recusa_contradicao() {
    let project = folder_task_with_failure(
        "`mp4_referencia=152`",
        "se `mp4_referencia` diferente de 152",
    );
    let (ok, text) = approve_validate(&project);
    assert!(!ok, "{text}");
    assert!(text.contains("que está no Trecho"), "{text}");
}

#[test]
fn approve_validate_falha_avisa_condicao() {
    let project = folder_task_with_failure("`ok`", "se `rm_na_referencia` for maior que 0");
    let (ok, text) = approve_validate(&project);
    assert!(ok, "{text}");
    assert!(
        text.contains("TASK-01: EV-01 saida.txt: a Falha tem condição junto de `rm_na_referencia` (\"maior que\")"),
        "{text}"
    );
}

// --- plan extend (SPEC-058) ---

const EXTEND_SPEC: &str = "\n# Exemplo\n\n## Requisitos funcionais\n\n| ID | Requisito |\n|---|---|\n| RF-01 | Um. |\n| RF-02 | Dois. |\n| RF-03 | Três. |\n\n## Casos de teste\n\n| ID | Cenário | Passos | Resultado esperado | Requisitos |\n|---|---|---|---|---|\n| CT-01 | Caso. | Rodar. | Ok. | RF-01 |\n| CT-02 | Outro. | Rodar. | Ok. | RF-02 |\n";

/// Grava a spec SPEC-001 com o status dado (hashes válidos) e uma pasta por tarefa
/// `(id, status, covers, wave)`, sem plano.
fn spec_with_status(
    project: &Project,
    status: &str,
    extra_front: &str,
    spec_body: &str,
    tasks: &[(&str, &str, &str, &str)],
) {
    use vostok::support::sha256_hex;
    let hash = sha256_hex(spec_body.as_bytes());
    write_file(
        project,
        "vostok/work/specs/refined/SPEC-001-exemplo.md",
        &format!("---\nid: SPEC-001\ntitle: \"Exemplo\"\nstatus: {status}\ntype: improvement\n{extra_front}updated_at: null\ncontent_hash: {hash}\napproved_hash: {hash}\n---\n{spec_body}"),
    );
    for (id, task_status, covers, wave) in tasks {
        write_file(
            project,
            &format!("vostok/work/tasks/SPEC-001/{id}/estado.yml"),
            &format!("id: {id}\nspec: SPEC-001\ntitle: \"Tarefa\"\ndepends_on: []\nwave: {wave}\ncovers: [{covers}]\nstatus: {task_status}\n"),
        );
    }
}

fn planned_with_tasks(project: &Project, spec_body: &str, tasks: &[(&str, &str, &str, &str)]) {
    spec_with_status(project, "planned", "", spec_body, tasks);
}

fn thirteen_tasks() -> Vec<(String, &'static str, &'static str, &'static str)> {
    (1..=13)
        .map(|n| {
            let covers = if n == 1 {
                "RF-01, RF-02, CT-01, CT-02"
            } else {
                "RF-01"
            };
            let wave = if n == 13 { "3" } else { "1" };
            (format!("TASK-{n:02}"), "done", covers, wave)
        })
        .collect()
}

#[test]
fn extend_create_covers_the_ids_without_task() {
    let project = Project::new();
    let owned = thirteen_tasks();
    let tasks: Vec<(&str, &str, &str, &str)> = owned
        .iter()
        .map(|(id, status, covers, wave)| (id.as_str(), *status, *covers, *wave))
        .collect();
    planned_with_tasks(&project, EXTEND_SPEC, &tasks);
    let output = project
        .vostok()
        .args(["plan", "extend", "SPEC-001"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();
    let mut lines = stdout.lines();
    assert_eq!(
        lines.next().unwrap(),
        "Plano de extensão de SPEC-001 criado: cobre RF-03; próxima tarefa: TASK-14; primeira onda: 4."
    );
    assert!(lines.next().unwrap().ends_with("SPEC-001.md"), "{stdout}");
    let plan =
        std::fs::read_to_string(project.root().join("vostok/work/plans/SPEC-001.md")).unwrap();
    assert!(plan.contains("\nextends: true\n"), "{plan}");
    assert!(plan.contains("\nfirst_task: TASK-14\n"), "{plan}");
    assert!(plan.contains("\nfirst_wave: 4\n"), "{plan}");
    assert!(plan.contains("\ncover_ids: [RF-03]\n"), "{plan}");
}

#[test]
fn extend_create_adds_the_extend_ids() {
    let project = Project::new();
    spec_with_status(
        &project,
        "planned",
        "extend_ids: [RF-02]\n",
        EXTEND_SPEC,
        &[("TASK-01", "done", "RF-01, RF-02, RF-03, CT-01, CT-02", "1")],
    );
    project
        .vostok()
        .args(["plan", "extend", "SPEC-001"])
        .assert()
        .success();
    let plan =
        std::fs::read_to_string(project.root().join("vostok/work/plans/SPEC-001.md")).unwrap();
    assert!(plan.contains("\ncover_ids: [RF-02]\n"), "{plan}");
}

#[test]
fn extend_create_counts_single_file_tasks() {
    let project = Project::new();
    planned_with_tasks(&project, EXTEND_SPEC, &[]);
    for (id, covers) in [("TASK-01", "RF-01, CT-01"), ("TASK-02", "RF-02, CT-02")] {
        write_file(
            &project,
            &format!("vostok/work/tasks/SPEC-001/{id}.md"),
            &format!("---\nid: {id}\nspec: SPEC-001\ncovers: [{covers}]\nstatus: done\n---\n"),
        );
    }
    project
        .vostok()
        .args(["plan", "extend", "SPEC-001"])
        .assert()
        .success();
    let plan =
        std::fs::read_to_string(project.root().join("vostok/work/plans/SPEC-001.md")).unwrap();
    assert!(plan.contains("\nfirst_task: TASK-03\n"), "{plan}");
    assert!(plan.contains("\nfirst_wave: 2\n"), "{plan}");
}

#[test]
fn extend_create_refuses_without_ids() {
    let project = Project::new();
    planned_with_tasks(
        &project,
        EXTEND_SPEC,
        &[("TASK-01", "done", "RF-01, RF-02, RF-03, CT-01, CT-02", "1")],
    );
    let stderr = failure_stderr(project.vostok().args(["plan", "extend", "SPEC-001"]));
    assert!(
        stderr.contains("nenhum ID da spec está sem tarefa"),
        "{stderr}"
    );
    assert!(!project
        .root()
        .join("vostok/work/plans/SPEC-001.md")
        .exists());
}

#[test]
fn extend_create_refuses_approved_spec() {
    let project = Project::new();
    spec_with_status(&project, "approved", "", EXTEND_SPEC, &[]);
    let stderr = failure_stderr(project.vostok().args(["plan", "extend", "SPEC-001"]));
    assert!(
        stderr.contains("só specs com status 'planned' ganham plano de extensão"),
        "{stderr}"
    );
    assert!(!project
        .root()
        .join("vostok/work/plans/SPEC-001.md")
        .exists());
}

#[test]
fn extend_create_refuses_open_plan() {
    let project = Project::new();
    planned_with_tasks(
        &project,
        EXTEND_SPEC,
        &[("TASK-01", "done", "RF-01, CT-01", "1")],
    );
    project
        .vostok()
        .args(["plan", "extend", "SPEC-001"])
        .assert()
        .success();
    let path = project.root().join("vostok/work/plans/SPEC-001.md");
    let before = std::fs::read_to_string(&path).unwrap();
    let stderr = failure_stderr(project.vostok().args(["plan", "extend", "SPEC-001"]));
    assert!(stderr.contains("já tem um plano atualizado"), "{stderr}");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
}

// --- Validação e ondas do plano de extensão (SPEC-058) ---

fn extension_row(id: &str, depends: &str, covers: &str) -> String {
    format!(
        "| {id} | Tarefa {id} | Faz. | tarefa | desenvolvedor | - | {depends} | - | - | {covers} |"
    )
}

fn extension_facts(ids: &[&str]) -> String {
    ids.iter()
        .map(|id| format!("| {id} | - | - | não | não | não |"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn extension_components(ids: &[&str]) -> String {
    ids.iter()
        .map(|id| format!("| {id} | - |"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Cria o plano de extensão com `vostok plan extend` e preenche as variáveis do template.
fn extension_plan(project: &Project, rows: &str, facts: &str, components: &str) {
    use vostok::support::sha256_hex;
    project
        .vostok()
        .args(["plan", "extend", "SPEC-001"])
        .assert()
        .success();
    let path = project.root().join("vostok/work/plans/SPEC-001.md");
    let content = std::fs::read_to_string(&path)
        .unwrap()
        .replace("#{PLAN_OVERVIEW}", "Tarefas novas.")
        .replace("#{PLAN_TASKS}", rows)
        .replace("#{PLAN_COMPONENTS}", components)
        .replace("#{PLAN_FACTS}", facts);
    let (front, body) = content[4..].split_once("\n---\n").unwrap();
    let hash = sha256_hex(body.as_bytes());
    let front: String = front
        .lines()
        .map(|line| {
            if line.starts_with("content_hash:") {
                format!("content_hash: {hash}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&path, format!("---\n{front}\n---\n{body}")).unwrap();
}

fn extension_tasks(project: &Project, spec_body: &str, highest_wave: &str, last: usize) {
    let owned: Vec<(String, &str, &str, String)> = (1..=last)
        .map(|n| {
            let covers = if n == 1 {
                "RF-01, RF-02, CT-01, CT-02"
            } else {
                "RF-01"
            };
            let wave = if n == last { highest_wave } else { "1" };
            (format!("TASK-{n:02}"), "done", covers, wave.to_string())
        })
        .collect();
    let tasks: Vec<(&str, &str, &str, &str)> = owned
        .iter()
        .map(|(id, status, covers, wave)| (id.as_str(), *status, *covers, wave.as_str()))
        .collect();
    planned_with_tasks(project, spec_body, &tasks);
}

fn extension_wave(plan: &str, id: &str) -> String {
    let line = tasks_section(plan)
        .lines()
        .find(|line| line.starts_with(&format!("| {id} |")))
        .unwrap()
        .to_string();
    line.split('|').nth(9).unwrap().trim().to_string()
}

#[test]
fn extend_validate_requires_only_the_ids_to_cover() {
    let project = Project::new();
    extension_tasks(&project, EXTEND_SPEC, "3", 13);
    extension_plan(
        &project,
        &extension_row("TASK-14", "-", "RF-03"),
        &extension_facts(&["TASK-14"]),
        &extension_components(&["TASK-14"]),
    );
    score(&project);
    project
        .vostok()
        .args(["plan", "validate", "SPEC-001"])
        .assert()
        .success();

    let other = Project::new();
    extension_tasks(&other, EXTEND_SPEC, "3", 13);
    extension_plan(
        &other,
        &extension_row("TASK-14", "-", "CT-01"),
        &extension_facts(&["TASK-14"]),
        &extension_components(&["TASK-14"]),
    );
    let stderr = validate_stderr(&other);
    assert!(
        stderr.contains("RF-03 não é coberto por nenhuma tarefa"),
        "{stderr}"
    );
    assert!(!stderr.contains("RF-01"), "{stderr}");
}

#[test]
fn extend_validate_refuses_an_existing_task_id() {
    let project = Project::new();
    extension_tasks(&project, EXTEND_SPEC, "3", 13);
    extension_plan(
        &project,
        &extension_row("TASK-03", "-", "RF-03"),
        &extension_facts(&["TASK-03"]),
        &extension_components(&["TASK-03"]),
    );
    let stderr = validate_stderr(&project);
    assert!(
        stderr.contains("TASK-03: a tarefa já existe na spec"),
        "{stderr}"
    );
}

#[test]
fn extend_validate_accepts_a_done_dependency() {
    let project = Project::new();
    extension_tasks(&project, EXTEND_SPEC, "3", 13);
    extension_plan(
        &project,
        &extension_row("TASK-14", "TASK-02", "RF-03"),
        &extension_facts(&["TASK-14"]),
        &extension_components(&["TASK-14"]),
    );
    score(&project);
    project
        .vostok()
        .args(["plan", "validate", "SPEC-001"])
        .assert()
        .success();
}

#[test]
fn extend_validate_ignores_components_of_existing_tasks() {
    let spec = format!(
        "{EXTEND_SPEC}\n## Componentes afetados\n\n| Caminho | Ação | Motivo |\n|---|---|---|\n| `src/a.rs` | alterar | Muda. |\n| `src/b.rs` | alterar | Muda. |\n"
    );
    let project = Project::new();
    extension_tasks(&project, &spec, "3", 13);
    extension_plan(
        &project,
        &extension_row("TASK-14", "-", "RF-03"),
        &extension_facts(&["TASK-14"]),
        "| TASK-14 | `src/a.rs` |",
    );
    score(&project);
    let output = project
        .vostok()
        .args(["plan", "validate", "SPEC-001"])
        .assert()
        .success()
        .get_output()
        .clone();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!text.contains("componente da spec sem tarefa"), "{text}");
}

#[test]
fn extend_waves_start_after_the_highest_wave() {
    let project = Project::new();
    extension_tasks(&project, EXTEND_SPEC, "3", 13);
    let rows = [
        extension_row("TASK-14", "-", "RF-03"),
        extension_row("TASK-15", "TASK-14", "RF-03"),
        extension_row("TASK-16", "TASK-02", "RF-03"),
    ]
    .join("\n");
    let ids = ["TASK-14", "TASK-15", "TASK-16"];
    extension_plan(
        &project,
        &rows,
        &extension_facts(&ids),
        &extension_components(&ids),
    );
    score(&project);
    let plan = plan_text(&project);
    assert_eq!(extension_wave(&plan, "TASK-14"), "4", "{plan}");
    assert_eq!(extension_wave(&plan, "TASK-15"), "5", "{plan}");
    assert_eq!(extension_wave(&plan, "TASK-16"), "4", "{plan}");
    project
        .vostok()
        .args(["plan", "validate", "SPEC-001"])
        .assert()
        .success();
}

#[test]
fn extend_waves_second_extension_continues() {
    let project = Project::new();
    extension_tasks(&project, EXTEND_SPEC, "5", 16);
    extension_plan(
        &project,
        &extension_row("TASK-17", "-", "RF-03"),
        &extension_facts(&["TASK-17"]),
        &extension_components(&["TASK-17"]),
    );
    let plan = plan_text(&project);
    assert!(plan.contains("\nfirst_task: TASK-17\n"), "{plan}");
    assert!(plan.contains("\nfirst_wave: 6\n"), "{plan}");
    score(&project);
    assert_eq!(extension_wave(&plan_text(&project), "TASK-17"), "6");
}

// --- Aprovação do plano de extensão (SPEC-058) ---

/// Preenche as variáveis dos arquivos da tarefa em pasta com um detalhamento válido.
fn fill_extension_task(project: &Project, id: &str, covers: &str) {
    let dir = project
        .root()
        .join(format!("vostok/work/tasks/SPEC-001/{id}"));
    let values = [
        ("TASK_READ", "- `README.md`".to_string()),
        (
            "TASK_CHANGE",
            "- `novo.txt` (criar): arquivo novo.".to_string(),
        ),
        ("TASK_STEPS", "1. Crie o arquivo.".to_string()),
        ("TASK_BASELINE", "- `true`".to_string()),
        ("TASK_VERIFY", "```bash\ntrue\n```".to_string()),
        ("TASK_EVIDENCE", "Não se aplica.".to_string()),
        (
            "TASK_DONE",
            format!("- [ ] Pronto ({covers}).\n\n  > Como comprovar: ver o arquivo `novo.txt`."),
        ),
        ("TASK_EXECUTIVE", "Resumo.".to_string()),
        ("TASK_TECHNICAL", "Detalhes.".to_string()),
    ];
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let mut text = std::fs::read_to_string(&path).unwrap();
        for (name, value) in &values {
            text = text.replace(&format!("#{{{name}}}"), value);
        }
        std::fs::write(&path, text).unwrap();
    }
}

fn file_hashes(dir: &std::path::Path) -> Vec<(String, String)> {
    use vostok::support::sha256_hex;
    let mut hashes: Vec<(String, String)> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_file())
        .map(|path| {
            (
                path.file_name().unwrap().to_string_lossy().to_string(),
                sha256_hex(&std::fs::read(&path).unwrap()),
            )
        })
        .collect();
    hashes.sort();
    hashes
}

fn run_ok(project: &Project, args: &[&str]) -> String {
    let output = project
        .vostok()
        .args(args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).unwrap()
}

/// Spec planejada com TASK-01 (`done`) e TASK-02 (`approved`) e o plano de extensão com TASK-03.
fn approvable_extension(project: &Project) {
    planned_with_tasks(
        project,
        EXTEND_SPEC,
        &[
            ("TASK-01", "done", "RF-01, RF-02, CT-01, CT-02", "1"),
            ("TASK-02", "approved", "RF-01", "1"),
        ],
    );
    extension_plan(
        project,
        &extension_row("TASK-03", "TASK-01", "RF-03"),
        &extension_facts(&["TASK-03"]),
        &extension_components(&["TASK-03"]),
    );
    score(project);
}

#[test]
fn extend_approve_creates_only_the_new_tasks() {
    let project = Project::new();
    approvable_extension(&project);
    let tasks = project.root().join("vostok/work/tasks/SPEC-001");
    let before: Vec<_> = ["TASK-01", "TASK-02"]
        .iter()
        .map(|id| file_hashes(&tasks.join(id)))
        .collect();
    run_ok(&project, &["plan", "approve", "create-tasks", "SPEC-001"]);
    fill_extension_task(&project, "TASK-03", "RF-03");
    run_ok(&project, &["plan", "approve", "validate", "SPEC-001"]);
    let stdout = run_ok(&project, &["plan", "approve", "mark-planned", "SPEC-001"]);
    assert!(
        stdout.contains("Tarefas novas de SPEC-001 aprovadas: TASK-03; a spec segue 'planned'."),
        "{stdout}"
    );
    let after: Vec<_> = ["TASK-01", "TASK-02"]
        .iter()
        .map(|id| file_hashes(&tasks.join(id)))
        .collect();
    assert_eq!(before, after);
    let mut folders: Vec<String> = std::fs::read_dir(&tasks)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
        .filter(|name| name.starts_with("TASK-"))
        .collect();
    folders.sort();
    assert_eq!(folders, ["TASK-01", "TASK-02", "TASK-03"]);
    let state = std::fs::read_to_string(tasks.join("TASK-03/estado.yml")).unwrap();
    assert!(state.contains("\nwave: 2\n"), "{state}");
    let spec = std::fs::read_to_string(
        project
            .root()
            .join("vostok/work/specs/refined/SPEC-001-exemplo.md"),
    )
    .unwrap();
    assert!(spec.contains("\nstatus: planned\n"), "{spec}");
    assert!(spec.contains("\nextend_ids: []\n"), "{spec}");
    assert!(!project
        .root()
        .join("vostok/work/plans/SPEC-001.md")
        .exists());
}

#[test]
fn extend_approve_refuses_an_existing_task_folder() {
    let project = Project::new();
    approvable_extension(&project);
    let folder = project.root().join("vostok/work/tasks/SPEC-001/TASK-03");
    std::fs::create_dir_all(&folder).unwrap();
    let stderr =
        failure_stderr(
            project
                .vostok()
                .args(["plan", "approve", "create-tasks", "SPEC-001"]),
        );
    assert!(
        stderr.contains("a pasta da tarefa TASK-03 já existe"),
        "{stderr}"
    );
    assert_eq!(std::fs::read_dir(&folder).unwrap().count(), 0);
}

#[test]
fn extend_approve_mark_planned_ignores_the_existing_tasks() {
    let project = Project::new();
    approvable_extension(&project);
    write_file(
        &project,
        "vostok/work/tasks/SPEC-001/TASK-02/execucao.md",
        "### Ler\n\n#{TASK_READ}\n",
    );
    run_ok(&project, &["plan", "approve", "create-tasks", "SPEC-001"]);
    fill_extension_task(&project, "TASK-03", "RF-03");
    run_ok(&project, &["plan", "approve", "validate", "SPEC-001"]);
    run_ok(&project, &["plan", "approve", "mark-planned", "SPEC-001"]);
    let leftover = std::fs::read_to_string(
        project
            .root()
            .join("vostok/work/tasks/SPEC-001/TASK-02/execucao.md"),
    )
    .unwrap();
    assert_eq!(leftover, "### Ler\n\n#{TASK_READ}\n");
}

#[test]
fn extend_approve_initial_plan_still_refuses_existing_folder() {
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
    std::fs::create_dir_all(project.root().join("vostok/work/tasks/SPEC-001")).unwrap();
    let stderr =
        failure_stderr(
            project
                .vostok()
                .args(["plan", "approve", "create-tasks", "SPEC-001"]),
        );
    assert!(
        stderr.contains("a pasta de tarefas de SPEC-001 já existe"),
        "{stderr}"
    );
}

// --- Trecho e Falha terminados em `=` ou `:` (SPEC-066) ---

fn plan_with_snippets(trecho: &str, falha: &str) -> (bool, String) {
    let project = Project::new();
    let evidence = format!(
        "- `saida.txt` (automática): `echo ok`\n  - Título: Saída do comando.\n  - Prova: ok.\n  - Trecho: {trecho}\n  - Falha: {falha}"
    );
    plan_task_with(&project, "true", &evidence);
    approve_validate(&project)
}

#[test]
fn trecho_termina_em_igual_e_recusado() {
    let (ok, text) = plan_with_snippets("`ITEM parada ms=`", "se aparecer `fail=1`");
    assert!(!ok, "{text}");
    assert!(text.contains("EV-01 saida.txt"), "{text}");
    assert!(text.contains("`ITEM parada ms=`"), "{text}");
    assert!(text.contains("`ITEM parada ms=...`"), "{text}");
}

#[test]
fn trecho_termina_em_dois_pontos_e_recusado() {
    let (ok, text) = plan_with_snippets("`Resumo:`", "se aparecer `fail=1`");
    assert!(!ok, "{text}");
    assert!(text.contains("`Resumo:...`"), "{text}");
}

#[test]
fn trecho_termina_falha_em_igual_e_recusada() {
    let (ok, text) = plan_with_snippets("`fail=0`", "se aparecer `fail=`");
    assert!(!ok, "{text}");
    assert!(text.contains("a Falha cita `fail=`"), "{text}");
    assert!(text.contains("valor errado literal"), "{text}");
}

#[test]
fn trecho_termina_aceita_valor_e_curinga() {
    for trecho in ["`ITEM parada ms=...`", "`fail=0`", "`ms=92 classe=a`"] {
        let (ok, text) = plan_with_snippets(trecho, "se aparecer `fail=1`");
        assert!(ok, "{trecho}: {text}");
    }
}

#[test]
fn revisor_ajuste_grava_origem_no_plano() {
    use vostok::support::sha256_hex;
    let project = Project::new();
    let spec_body = "\n# Exemplo\n\n## Requisitos funcionais\n\n| ID | Requisito |\n|---|---|\n| RF-01 | Entregar. |\n";
    let plan_body = "\n# Plano\n\n## Visão geral\n\nUma tarefa.\n\n## Tarefas\n\n| ID | Título | Descrição | Tipo | Nível | Dependências | Cobre |\n|---|---|---|---|---|---|---|\n| TASK-01 | Tarefa | Entrega. | tarefa | simples | - | RF-01 |\n\n## Fatos de esforço\n\n| ID | Arquivos a criar | Arquivos a alterar | Contrato público | Inferência | Risco alto |\n|---|---|---|---|---|---|\n| TASK-01 | 0 | 1 | não | não | não |\n\n## Histórico de ajustes\n\n| # | Data | Origem | Resumo |\n|---|---|---|---|\n";
    spec_and_plan(&project, "improvement", spec_body, "");
    let spec_hash = sha256_hex(spec_body.as_bytes());
    let plan_hash = sha256_hex(plan_body.as_bytes());
    write_file(
        &project,
        "vostok/work/plans/SPEC-001.md",
        &format!("---\nspec: SPEC-001\nspec_hash: {spec_hash}\ncontent_hash: {plan_hash}\nadjustments: 0\nadjusted_at: null\n---\n{plan_body}"),
    );
    project
        .vostok()
        .args([
            "plan",
            "adjust",
            "record",
            "SPEC-001",
            "Dependência corrigida",
            "--revisor",
        ])
        .assert()
        .success();
    let plan =
        std::fs::read_to_string(project.root().join("vostok/work/plans/SPEC-001.md")).unwrap();
    assert!(
        plan.contains("| revisor | Dependência corrigida |"),
        "{plan}"
    );
}

// --- criar só na primeira tarefa, no plano de extensão (SPEC-072) ---

fn creating_extension_spec() -> String {
    format!(
        "{EXTEND_SPEC}\n## Componentes afetados\n\n| Caminho | Ação | Motivo |\n|---|---|---|\n| `src/a.rs` | criar | Novo. |\n"
    )
}

/// Extensão com a TASK-01 original (só `estado.yml`) e a TASK-02 nova listando `src/a.rs`.
fn creating_extension(original_change: Option<&str>) -> Project {
    let project = Project::new();
    extension_tasks(&project, &creating_extension_spec(), "1", 1);
    if let Some(change) = original_change {
        let folder = "vostok/work/tasks/SPEC-001/TASK-01";
        write_file(&project, &format!("{folder}/execucao.md"), change);
        write_file(
            &project,
            &format!("{folder}/verificacao.md"),
            "### Baseline\n",
        );
        write_file(
            &project,
            &format!("{folder}/criterios.md"),
            "### Concluída quando\n",
        );
        write_file(
            &project,
            &format!("{folder}/registro.md"),
            "## Registro de execução\n",
        );
    }
    extension_plan(
        &project,
        &extension_row("TASK-02", "-", "RF-03"),
        &extension_facts(&["TASK-02"]),
        "| TASK-02 | `src/a.rs` |",
    );
    score(&project);
    project
}

const CREATES_A: &str = "### Mudar\n\n- `src/a.rs` (criar): novo.\n";

#[test]
fn creates_once_extension_counts_the_original_file_as_change() {
    let project = creating_extension(Some(CREATES_A));
    let plan = plan_text(&project);
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
fn creates_once_original_task_without_change_list_keeps_create() {
    let project = creating_extension(None);
    let plan = plan_text(&project);
    assert!(
        fact_line(&plan, "TASK-02").starts_with("| TASK-02 | 1 | 0 |"),
        "{plan}"
    );
}

#[test]
fn creates_once_extension_after_create_tasks_keeps_the_facts() {
    let project = creating_extension(None);
    assert!(fact_line(&plan_text(&project), "TASK-02").starts_with("| TASK-02 | 1 | 0 |"));
    project
        .vostok()
        .args(["plan", "approve", "create-tasks", "SPEC-001"])
        .assert()
        .success();
    write_file(
        &project,
        "vostok/work/tasks/SPEC-001/TASK-02/execucao.md",
        CREATES_A,
    );
    let output = project
        .vostok()
        .args(["plan", "validate", "SPEC-001"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("TASK-02: arquivos a criar e a alterar diferentes"),
        "{stderr}"
    );
}
