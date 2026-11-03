### Ler

- `vostok/work/specs/refined/SPEC-034-corrigir-o-fluxo-do-painel-antes-da-v0-4-0.md` (RF-01, CT-01, CB-01, CB-02, riscos e decisões 7 e 8)
- `apps/cli/src/commands/plan.rs` (fn record_manual_at, struct Workspace com fn new, fn plan_path e fn find_spec, fn plan_is_outdated perto da linha 506)
- `apps/cli/src/painel/read.rs` (fn harness de `Layout`, fn overview perto da linha 164, fn spec perto da linha 518, fn commits como exemplo de chamada à CLI, teste overview_lists_open_and_archived_specs_activity_and_questions)
- `apps/cli/tests/painel.rs`
- `apps/cli/tests/common/mod.rs`

### Mudar

- `apps/cli/src/commands/plan.rs` (alterar): `pub fn plan_outdated(harness: &Harness, spec_id: &str) -> bool`, que reaproveita `plan_is_outdated` para o painel
- `apps/cli/src/painel/read.rs` (alterar): campo `"plan_outdated"` ao lado de `"plan"` no JSON de cada spec do `overview` e no da `spec`
- `apps/cli/tests/painel.rs` (alterar): testes do campo `plan_outdated` na visão geral e na spec (CT-01, CB-02)

### Passos

1. `apps/cli/tests/painel.rs` (alterar): depois de `pending_task`, acrescente o helper `fn spec_with_plan(approved_hash: &str, status: &str, plan_hash: Option<&str>) -> Project`, que cria um `Project::new()` e grava, com o `write` do arquivo, `vostok/work/specs/refined/SPEC-001-plano.md` com o conteúdo `---\nid: SPEC-001\ntitle: \"Plano\"\nstatus: {status}\ntype: fix\napproved_hash: {approved_hash}\n---\n\n# Plano\n` e, quando `plan_hash` é `Some`, `vostok/work/plans/SPEC-001.md` com `---\nspec: SPEC-001\nspec_hash: {plan_hash}\n---\n\n# Plano de tarefas\n`. Acrescente também `fn assert_plan_fields(project: &Project, expected: &str)`, que sobe o painel com `start(project.root())`, faz `get` de `/api/visao` e de `/api/specs/SPEC-001`, confere `assert_status(..., "200")` nas duas respostas e `assert!(resposta.contains(expected), "{resposta}")` em cada uma. Depois, os quatro testes, no fim do arquivo:
   - `plan_outdated_true_when_the_spec_was_approved_again` (CT-01): `spec_with_plan("novo", "approved", Some("antigo"))` e `assert_plan_fields(&project, "\"plan\":true,\"plan_outdated\":true")`.
   - `plan_outdated_false_when_the_hashes_match` (CT-01): `spec_with_plan("abc", "approved", Some("abc"))` e `"\"plan\":true,\"plan_outdated\":false"`.
   - `plan_outdated_true_when_the_spec_lost_the_approval` (RF-01, CB-01): `spec_with_plan("null", "refined", Some("abc"))` e `"\"plan\":true,\"plan_outdated\":true"`.
   - `plan_outdated_false_without_a_plan` (CB-02): `spec_with_plan("abc", "approved", None)` e `"\"plan\":false,\"plan_outdated\":false"`.
   Confira: `cargo test --manifest-path apps/cli/Cargo.toml --features painel --test painel -- plan_outdated_` compila e os quatro testes falham (o campo ainda não existe).
2. `apps/cli/src/commands/plan.rs` (alterar): logo depois de `fn record_manual_at`, acrescente a função abaixo, com o comentário de documentação; não mude `plan_is_outdated`. Confira: `cargo build --manifest-path apps/cli/Cargo.toml` sem erro.

   ```rust
   /// Se o plano da spec está desatualizado, pela regra do `vostok plan validate`; falso sem plano
   /// ou quando a spec ou a configuração não podem ser lidas, para o painel nunca recusar a leitura.
   pub fn plan_outdated(harness: &Harness, spec_id: &str) -> bool {
       let workspace = Workspace::new(harness.clone());
       let (Ok(plan), Ok(spec)) = (workspace.plan_path(spec_id), workspace.find_spec(spec_id)) else {
           return false;
       };
       plan.is_file() && workspace.plan_is_outdated(&plan, &spec).unwrap_or(false)
   }
   ```
3. `apps/cli/src/painel/read.rs` (alterar): importe `use crate::commands::plan::plan_outdated;` junto dos outros `use crate::commands::...`. Em `fn overview`, antes do `let open`, crie `let harness = self.harness();`; no `map` de cada spec, depois de `let plan = ...`, crie `let plan_outdated = plan_outdated(&harness, &spec.id);` e troque o formato por `"{{\"meta\":{},\"plan\":{plan},\"plan_outdated\":{plan_outdated},\"tasks\":[{}]}}"`. Em `fn spec`, depois de `let plan = ...`, crie `let plan_outdated = plan_outdated(&self.harness(), safe_id);` e troque o formato por `"{{\"meta\":{},\"path\":{},\"plan\":{plan},\"plan_outdated\":{plan_outdated},\"archived\":{archived},\"markdown\":{},\"tasks\":[{}]}}"`. Não mude o campo `"plan"` nem a ordem dos demais. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --features painel --test painel -- plan_outdated_` com os quatro testes `ok`.
4. `apps/cli` (conferir): `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::read`, `cargo test --manifest-path apps/cli/Cargo.toml --features painel --test painel` e `cargo test --manifest-path apps/cli/Cargo.toml --test plan` sem falhas, e `cargo fmt --manifest-path apps/cli/Cargo.toml` seguido de `cargo clippy --manifest-path apps/cli/Cargo.toml --features painel --all-targets -- -D warnings` sem avisos.
