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

// --- Nome de evidência inválido recusado na aprovação do plano (SPEC-010) ---

#[test]
fn evidence_name_with_a_path_is_refused_by_the_plan_validation() {
    let project = Project::new();
    let write = |relative: &str, content: &str| {
        let path = project.root().join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    };
    write(
        "vostok/work/plans/SPEC-001.md",
        "# Plano\n\n## Tarefas\n\n| ID | Título | Descrição | Tipo | Nível | Dependências | Cobre |\n|---|---|---|---|---|---|---|\n| TASK-01 | Tarefa | Entrega. | tarefa | simples | - | RF-01 |\n",
    );
    write(
        "vostok/work/tasks/SPEC-001/TASK-01.md",
        &format!(
            "---\nid: TASK-01\nspec: SPEC-001\ntitle: \"Tarefa\"\neffort: simples\ndepends_on: []\ncovers: [RF-01]\ntype: tarefa\nlabels: [SPEC-001, simples]\nstatus: pending\ncontent_hash: null\nadjustments: 0\nadjusted_at: null\napproved_at: null\napproved_hash: null\nattempts: 0\ntests_result: null\nregression_result: null\nevidence_result: null\n---\n\n# TASK-01: Tarefa\n\n## Execução\n\n### Ler\n\n- `README.md`\n\n### Mudar\n\n- `novo.txt` (criar): arquivo novo.\n\n### Passos\n\n1. Crie o arquivo.\n\n### Baseline\n\n- `true`\n\n### Verificação\n\n```bash\ntrue\n```\n\n### Evidências\n\n- `docs/x/saida.txt` (automática): `echo ok`\n{SUB}\n\n### Concluída quando\n\n- [ ] Pronto (RF-01).\n\n  > Como comprovar: ver o arquivo.\n\n## Descrição\n\n### Resumo executivo\n\nResumo.\n\n### Detalhes técnicos\n\nDetalhes.\n\n### Card\n\n- **Título:** Tarefa\n- **Tipo:** tarefa\n- **Descrição:** Descrição.\n- **Critérios de aceite:**\n  - [ ] Pronto.\n- **Rótulos:** SPEC-001, simples\n- **Estimativa:** simples\n- **Bloqueada por:** -\n\n## Registro de execução\n\n| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n\n## Histórico de ajustes\n\n| # | Data | Origem | Resumo |\n|---|---|---|---|\n",
            SUB = "  - Prova: ok.\n  - Trecho: ok.\n  - Falha: erro."
        ),
    );
    let refused = failure_stderr(
        project
            .vostok()
            .args(["plan", "approve", "validate", "SPEC-001"]),
    );
    assert!(refused.contains("docs/x/saida.txt"), "{refused}");
    assert!(refused.contains("sem caminho"), "{refused}");
}
