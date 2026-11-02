### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`
- `cargo test --manifest-path apps/cli/Cargo.toml --test task --test spec -- approve 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`

### Verificação

```bash
# evidência: acoes-testes.txt
cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::actions
cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::server
cargo test --manifest-path apps/cli/Cargo.toml --test context_memory_rules
cargo clippy --manifest-path apps/cli/Cargo.toml --features painel --all-targets -- -D warnings
```

Resultado esperado: os testes de edição, configuração (com a validação das regras), aprovação e pedidos passam; os testes do servidor passam com o encaminhamento de `/arquivos`; os testes de regras continuam passando; clippy sem avisos.

### Evidências

- `acoes-testes.txt` (automática, da verificação)
  - Prova: aprovação com `approved_by: painel` e aval; edição com hash, 409 e registro manual; recusas 400, 403 e de tamanho; configuração gravada; pedido no formato (RF-06, RF-09, RF-10, RF-11, CT-04, CT-08 a CT-12, CB-02 a CB-04).
  - Trecho: os testes `painel::actions::` com `ok` e `test result: ok`.
  - Falha: `FAILED`.
  - Espera: `test result: ok`
