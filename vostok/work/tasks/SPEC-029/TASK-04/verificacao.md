### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`

### Verificação

```bash
# evidência: leitura-testes.txt
cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::read
cargo clippy --manifest-path apps/cli/Cargo.toml --features painel --all-targets -- -D warnings
```

Resultado esperado: os testes de leitura passam; clippy sem avisos.

### Evidências

- `leitura-testes.txt` (automática, da verificação)
  - Prova: a API lê specs, planos, tarefas, evidências, documentos, configurações e marca dos arquivos, recusa caminhos fora de `vostok/` e usa os tokens padrão sem marca (RF-02, RF-04, CT-03, CB-05).
  - Trecho: os testes `painel::read::` com `ok` e `test result: ok`.
  - Falha: `FAILED`.
  - Espera: `test result: ok`
