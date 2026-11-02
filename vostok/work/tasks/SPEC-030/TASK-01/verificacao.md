### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test init_update 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`
- `cargo test --manifest-path apps/cli/Cargo.toml --lib 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort | grep -v ' aval::'`

### Verificação

```bash
# evidência: aval-leitura.txt
cargo test --manifest-path apps/cli/Cargo.toml --lib aval
cargo test --manifest-path apps/cli/Cargo.toml --test init_update
cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings
```

Resultado esperado: os testes do módulo `aval` e os de `init_update` passam; clippy sem avisos.

### Evidências

- `aval-leitura.txt` (automática, da verificação)
  - Prova: as linhas de 4 e 5 colunas são lidas, a linha malformada é apontada e a chave da trava é lida com o padrão desligado (RF-01, RF-02, CT-02, CB-02, CB-03).
  - Trecho: `four_columns_read_as_aprovar ... ok`, `five_columns_keep_the_action ... ok`, `lock_is_off_without_the_key ... ok`.
  - Falha: `FAILED` ou o módulo sem testes.
  - Espera: `test result: ok`
