### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test spec 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort | grep -v 'adjust_history_'`
- `cargo test --manifest-path apps/cli/Cargo.toml --test aval 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort | grep -v 'panel_without_aval_cites_the_spec_log'`
- `cargo test --manifest-path apps/cli/Cargo.toml --test task 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`
- `cargo test --manifest-path apps/cli/Cargo.toml --lib commands::task:: 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`
- `cargo clippy --manifest-path apps/cli/Cargo.toml --features painel --all-targets -q -- -D warnings`

### Verificação

```bash
# evidência: historico-ajuste.txt
cargo test --manifest-path apps/cli/Cargo.toml --test spec -- adjust_history_ approve_records_the_status_history
# evidência: mensagem-aval.txt
cargo test --manifest-path apps/cli/Cargo.toml --test aval -- panel_without_aval_cites_the_spec_log panel_approval_without_aval_is_reported
cargo test --manifest-path apps/cli/Cargo.toml --test spec
cargo test --manifest-path apps/cli/Cargo.toml --test aval
cargo test --manifest-path apps/cli/Cargo.toml --test task
cargo clippy --manifest-path apps/cli/Cargo.toml --features painel --all-targets -q -- -D warnings
```

### Evidências

- `historico-ajuste.txt` (automática, da verificação)
  - Prova: o ajuste, do agente ou manual, que derruba a aprovação grava a volta para refined no `status_history`, depois da linha da aprovação, e o ajuste de uma spec só refinada não grava nada (RF-06, CT-08, CB-04).
  - Trecho: `test adjust_history_records_the_return_to_refined ... ok`, `test adjust_history_manual_adjust_records_the_return_to_refined ... ok`, `test adjust_history_unchanged_for_a_refined_spec ... ok`, `test approve_records_the_status_history ... ok` e `test result: ok. 4 passed`.
  - Falha: algum desses testes com `FAILED`, com a mensagem do histórico terminando em ` approved` em vez de ` refined`, ou `running 0 tests`.
- `mensagem-aval.txt` (automática, da verificação)
  - Prova: a validação da CLI de uma spec e de uma tarefa aprovadas pelo painel sem aval cita o log de avais da spec, `work/avais/SPEC-001.log`, e não cita `avais.log` (RF-07, CT-09).
  - Trecho: `test panel_without_aval_cites_the_spec_log ... ok`, `test panel_approval_without_aval_is_reported ... ok` e `test result: ok. 2 passed`.
  - Falha: algum dos dois com `FAILED`, com a recusa ainda citando `avais.log`.
