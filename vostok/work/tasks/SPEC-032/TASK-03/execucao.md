### Ler

- `vostok/work/specs/refined/SPEC-032-controlar-o-projeto-inteiramente-pelo-painel-v0-4-0.md`
- `apps/cli/src/commands/task.rs` (fn check, fn reset_results, fn resume, fn record_step, fn tests)
- `apps/cli/tests/task.rs` (teste check_passes_and_fails_without_touching_the_task)

### Mudar

- `apps/cli/src/commands/task.rs` (alterar): `check_result` e `checked_at` no `vostok task check`, limpos no ajuste e no resume
- `apps/cli/tests/task.rs` (alterar): testes do resultado da conferência

### Passos

1. `apps/cli/tests/task.rs` (alterar): troque `check_passes_and_fails_without_touching_the_task` por `check_records_the_result`: depois de um check que passa, o front-matter tem `check_result: passed` e `checked_at`, e o registro tem uma linha `check` com resultado `passed`; depois de um que falha, `check_result: failed`. Acrescente `adjustment_clears_the_check_result` (check que passa, depois `vostok task adjust record`, e `check_result` volta a `null`) e `resume_clears_the_check_result`. Confira: os testes falham antes da mudança.
2. `apps/cli/src/commands/task.rs` (alterar): em `fn check`, depois de `run_verification`, grave `check_result` (`passed` com código 0, `failed` senão) e `checked_at` com `utc_datetime()` pelo `TaskDoc::at(&file).set`, e chame `self.record_step(&file, "check", <resultado>, "")`. Atualize o comentário da função, que hoje diz que ela não muda o front-matter. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test task -- check_`.
3. `apps/cli/src/commands/task.rs` (alterar): em `fn reset_results`, inclua `check_result` na lista zerada (`null`); no caminho de aceite do `fn resume` (`--accept-regression`), também grave `check_result: null`. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test task -- clears_the_check_result`.
4. `apps/cli` (conferir): `cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings` e `cargo test --manifest-path apps/cli/Cargo.toml --test task`. Confira: sem avisos e sem falhas.
