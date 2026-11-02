### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort | grep -v 'request_actions_are'`
- `cargo test --manifest-path apps/cli/Cargo.toml --features painel --test painel_server 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`

### Verificação

```bash
# evidência: painel-aval.txt
cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::actions
cargo test --manifest-path apps/cli/Cargo.toml --features painel --test painel_server
cargo clippy --manifest-path apps/cli/Cargo.toml --features painel --all-targets -- -D warnings
```

Resultado esperado: os testes de ações e do servidor passam; clippy sem avisos.

### Evidências

- `painel-aval.txt` (automática, da verificação)
  - Prova: a aprovação grava o aval com a ação, e os pedidos travados gravam o aval com o hash do alvo; os demais não gravam (RF-10, RF-12, CT-11, CT-13).
  - Trecho: `approval_aval_has_the_action_column ... ok`, `finish_request_records_the_aval ... ok`, `commit_request_records_the_aval ... ok`.
  - Falha: `FAILED` em qualquer teste de `painel::actions`.
  - Espera: `test result: ok`
