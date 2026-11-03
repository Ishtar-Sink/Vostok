### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test base 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`
- `cargo build --manifest-path apps/cli/Cargo.toml -q && apps/cli/target/debug/vostok --version` (muda): a versão passa de `vostok 0.3.0` para `vostok 0.4.0` (RF-15).

### Verificação

```bash
cargo build --manifest-path apps/cli/Cargo.toml
# evidência: versao.txt
apps/cli/target/debug/vostok --version
# evidência: versao-testes.txt
cargo test --manifest-path apps/cli/Cargo.toml --test base
```

Resultado esperado: o binário compilado responde `vostok 0.4.0` e os testes de `--test base` passam.

### Evidências

- `versao.txt` (automática, da verificação)
  - Prova: o binário compilado da CLI responde a versão nova (RF-15, CT-16).
  - Trecho: a linha `vostok 0.4.0`.
  - Falha: `vostok 0.3.0` ou erro de compilação.
  - Espera: `vostok 0.4.0`
- `versao-testes.txt` (automática, da verificação)
  - Prova: o teste da versão passa com 0.4.0 e nada mais de `--test base` quebrou (RF-15, CT-16).
  - Trecho: `test ct01_version_is_0_1_0 ... ok` e `test result: ok`.
  - Falha: `FAILED` ou `test result: FAILED`.
