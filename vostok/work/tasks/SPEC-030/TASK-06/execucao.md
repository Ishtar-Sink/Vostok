### Ler

- `vostok/work/specs/refined/SPEC-030-trava-de-aval-e-loop-de-pedidos-do-painel.md` (RF-03, RF-08, RF-09, CT-01, CT-03, CT-04, CT-09, CT-10, CB-01, CB-07, RN-02, RN-05)
- `apps/cli/src/aval.rs` (da TASK-01: fn require, fn has_panel_approval)
- `apps/cli/src/commands/task.rs` (fn finish na linha 2135, fn run_approve, fn approve na linha 1180, fn errors_of na linha 2906, const APPROVED_BY_FIELD; arquivo com mais de 5 mil linhas: leia por busca)
- `apps/cli/src/commands/spec.rs` (fn approve na linha 1227, fn validate_refined, fn validate_adjustable, fn approval_blockers)
- `apps/cli/tests/task.rs` (fn approved_task e os testes de finish)
- `apps/cli/tests/common/mod.rs`

### Mudar

- `apps/cli/src/commands/task.rs` (alterar): aval no `finish` e no `approve` pelo chat; `approved_by: painel` sem aval na validação.
- `apps/cli/src/commands/spec.rs` (alterar): aval no `approve` pelo chat; `approved_by: painel` sem aval na validação.
- `apps/cli/tests/aval.rs` (criar): testes da trava em tarefa e spec.

### Passos

1. `apps/cli/tests/aval.rs` (criar): com `common::Project` e uma tarefa em execução pronta para concluir (siga os ajudantes de `tests/task.rs`), escreva:
   - `finish_without_lock_works_as_before` (sem `approval_lock`, conclui; CT-01);
   - `finish_with_lock_needs_the_aval` (com `approval_lock: true`, recusa citando `finalizar-tarefa` e o painel; com a linha de aval do `approved_hash`, conclui; CT-03);
   - `aval_of_another_hash_does_not_count` (aval com hash antigo é recusado citando os dois hashes; CT-04, CB-01);
   - `lock_without_avais_log_refuses` (CB-07);
   - `chat_approvals_need_the_aval_with_the_lock` (`task approve approve` e `spec approve approve` recusados com a instrução do painel; CT-09);
   - `panel_approval_without_aval_is_reported` (`approved_by: painel` sem linha no `avais.log` aparece no `task adjust validate` e no `spec validate`; CT-10).
   Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test aval` falha.
2. `apps/cli/src/commands/task.rs` (alterar): no início de `fn finish`, depois de `require_running`, leia o `approved_hash` da tarefa e chame `aval::require(&self.harness, &format!("{spec}/{task}"), "finalizar-tarefa", &hash)?` (RF-03).
3. `apps/cli/src/commands/task.rs` (alterar): em `fn run_approve`, só no caminho `Approve` vindo da CLI (`Origin::Chat`), para cada tarefa, chame `aval::require(.., "<SPEC>/<TASK>", "aprovar", <hash do conteúdo>)` antes de gravar; o `approve_from_panel` não confere (RF-08, RN-05).
4. `apps/cli/src/commands/task.rs` (alterar): em `fn errors_of`, quando `approved_by` for `painel` e `aval::has_panel_approval(work, "<SPEC>/<TASK>", approved_hash)` for falso, acrescente `approved_by: painel sem aval correspondente em avais.log` (RF-09), com a trava ligada ou não.
5. `apps/cli/src/commands/spec.rs` (alterar): em `fn approve` com `Origin::Chat`, `aval::require(.., spec_id, "aprovar", <content_hash>)`; em `fn validate_refined` e `fn validate_adjustable`, o mesmo problema do passo 4 para `approved_by: painel` com o `approved_hash` da spec.
6. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test aval --test task --test spec` passa, inclusive os golden; `cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings` sem avisos.
