### Ler

- `vostok/work/specs/refined/SPEC-030-trava-de-aval-e-loop-de-pedidos-do-painel.md` (RF-04, RF-05, RF-06, RF-07, CT-05, CT-06, CT-07, CT-08, CB-04, "Contratos e interfaces")
- `apps/cli/src/aval.rs` (da TASK-01: fn require, fn request_state, fn request_text_hash)
- `apps/cli/tests/task.rs` (ajudantes para montar um projeto com spec, plano e tarefas)
- `apps/cli/src/commands/plan.rs` (fn mark_planned na linha 1122, fn run_approve)
- `apps/cli/src/commands/finalize.rs` (fn archive na linha 3062, fn archive_problems)
- `apps/cli/src/commands/commit.rs` (enum CommitCommand, fn run, fn approve, fn commit)
- `apps/cli/tests/plan.rs`, `apps/cli/tests/finalize.rs` e `apps/cli/tests/commit_skill.rs` (ajudantes de cada comando)

### Mudar

- `apps/cli/src/commands/plan.rs` (alterar): aval `aprovar-plano` no `mark-planned`.
- `apps/cli/src/commands/finalize.rs` (alterar): aval `finalizar-spec` no `archive`.
- `apps/cli/src/commands/commit.rs` (alterar): `--pedido` e aval `fazer-commit` no `commit` e no `approve`.
- `apps/cli/tests/trava_plano_commit.rs` (criar): testes da trava em plano, arquivamento e commit.

### Passos

1. `apps/cli/tests/trava_plano_commit.rs` (criar): com `common::Project` e os ajudantes dos testes de cada comando, escreva:
   - `mark_planned_needs_the_plan_aval` (CT-05): recusa sem aval; com a linha `SPEC-001`, `content_hash` do plano, `aprovar-plano`, marca;
   - `archive_needs_the_spec_aval` (CT-06): o mesmo com `finalizar-spec` e o `content_hash` da spec;
   - `commit_needs_a_request_with_aval` (CT-07): sem `--pedido`, recusa; com `--pedido PED-001` sem aval, recusa; com o aval e o pedido `em_andamento`, faz o commit;
   - `commit_refuses_a_finished_request` (CB-04): pedido `feito` é recusado;
   - `commit_approve_needs_the_request` (CT-08).
   Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test trava_plano_commit` falha nesses testes.
2. `apps/cli/src/commands/plan.rs` (alterar): em `fn mark_planned`, antes de gravar, `aval::require(&harness, spec_id, "aprovar-plano", <content_hash do plano>)?` (RF-04).
3. `apps/cli/src/commands/finalize.rs` (alterar): em `fn archive`, antes de `archive_problems`, `aval::require(.., &self.id, "finalizar-spec", <content_hash da spec>)?` (RF-05).
4. `apps/cli/src/commands/commit.rs` (alterar): em `CommitCommand`, `Commit { #[arg(long)] pedido: Option<String>, paths: Vec<String> }` e `Approve { #[arg(long)] pedido: Option<String>, code: Option<String> }`. Crie `fn require_request(harness, pedido: Option<&str>) -> Result<()>`: com a trava desligada, `Ok`; ligada, exige o `--pedido`, `aval::request_state` com `acao: fazer-commit` e `estado: em_andamento`, e `aval::require(.., pedido, "fazer-commit", aval::request_text_hash(..))`. Chame-a no início de `fn commit` e de `fn approve` (RF-06, RF-07).
5. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test trava_plano_commit --test plan --test finalize --test commit_skill` passa; `cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings` sem avisos.
