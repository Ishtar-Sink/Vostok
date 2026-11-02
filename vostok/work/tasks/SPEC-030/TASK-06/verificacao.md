### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test task --test spec 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`

### Verificação

```bash
# evidência: trava-tarefa-spec.txt
cargo test --manifest-path apps/cli/Cargo.toml --test aval
cargo test --manifest-path apps/cli/Cargo.toml --lib aval
cargo test --manifest-path apps/cli/Cargo.toml --test task --test spec
cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings
```

Resultado esperado: os testes da trava, os de tarefa e de spec (com os golden) passam; clippy sem avisos.

### Evidências

- `trava-tarefa-spec.txt` (automática, da verificação)
  - Prova: sem a trava nada muda; com ela, concluir e aprovar pelo chat exigem o aval do mesmo hash; aprovações do painel sem aval são acusadas (RF-03, RF-08, RF-09, CT-01, CT-03, CT-04, CT-09, CT-10).
  - Trecho: `finish_without_lock_works_as_before ... ok`, `finish_with_lock_needs_the_aval ... ok`, `aval_of_another_hash_does_not_count ... ok`, `panel_approval_without_aval_is_reported ... ok`.
  - Falha: `FAILED`, inclusive nos golden.
  - Espera: `test result: ok`
