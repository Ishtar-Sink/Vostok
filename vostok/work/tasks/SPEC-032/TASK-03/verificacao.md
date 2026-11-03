### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test task 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort | grep -v 'check_passes_and_fails_without_touching_the_task\|check_records_the_result\|clears_the_check_result'`

### Verificação

```bash
# evidência: check-testes.txt
cargo test --manifest-path apps/cli/Cargo.toml --test task -- check_ clears_the_check_result
cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings
```

### Evidências

- `check-testes.txt` (automática, da verificação)
  - Prova: o `vostok task check` grava o resultado e a data, e o ajuste e o resume os limpam (RF-03, CT-03, CT-04, CT-05).
  - Trecho: `test check_records_the_result ... ok`, `test adjustment_clears_the_check_result ... ok` e `test resume_clears_the_check_result ... ok`.
  - Falha: algum desses testes com `FAILED`.
