### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test base 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`

### Verificação

```bash
# evidência: pedidos-testes.txt
cargo test --manifest-path apps/cli/Cargo.toml --test pedidos
cargo test --manifest-path apps/cli/Cargo.toml --test base
cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings
```

Resultado esperado: os testes de pedidos e os de base passam; clippy sem avisos.

### Evidências

- `pedidos-testes.txt` (automática, da verificação)
  - Prova: os pedidos mudam de estado só pelo comando, com o resultado gravado, e as transições inválidas são recusadas (RF-13, CT-14, CB-05, RN-03).
  - Trecho: `iniciar_moves_pending_to_em_andamento ... ok`, `iniciar_refuses_a_request_not_pending ... ok`, `concluir_writes_the_result ... ok`.
  - Falha: `FAILED`.
  - Espera: `test result: ok`
