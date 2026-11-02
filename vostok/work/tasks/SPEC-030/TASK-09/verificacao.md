### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test pedidos 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`

### Verificação

```bash
# evidência: vigiar-testes.txt
cargo test --manifest-path apps/cli/Cargo.toml --test pedidos
cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings
```

Resultado esperado: os testes de pedidos, inclusive os do `vigiar`, passam; clippy sem avisos.

### Evidências

- `vigiar-testes.txt` (automática, da verificação)
  - Prova: o `vigiar` dispara o `codex exec` uma vez por pedido, um por vez e em ordem, e para com instrução sem o `codex` (RF-15, CT-16, CB-06, RNF-04).
  - Trecho: `vigiar_runs_codex_once_per_request_in_order ... ok` e `vigiar_without_codex_stops_with_instructions ... ok`.
  - Falha: `FAILED` ou o teste preso no tempo-limite.
  - Espera: `test result: ok`
