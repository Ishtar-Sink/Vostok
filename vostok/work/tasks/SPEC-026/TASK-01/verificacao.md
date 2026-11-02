### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test init_update 2>&1 | grep -E '^test ' | sort`
- `cargo test --manifest-path apps/cli/Cargo.toml --test finalize 2>&1 | grep -E '^test ' | sort`
- `cargo run --quiet --manifest-path apps/cli/Cargo.toml -- --help` (muda): a lista de comandos ganha `documents`

### Verificação

```bash
# evidência: brand-tests.txt
cargo test --manifest-path apps/cli/Cargo.toml --test documents -- brand_
cargo test --manifest-path apps/cli/Cargo.toml --test init_update
cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings
```

Resultado esperado: os testes `brand_` e os de `init_update` passam, e o clippy termina sem avisos.

### Evidências

- `brand-tests.txt` (automática, da verificação)
  - Prova: o comando grava a identidade, deriva os dois temas, passa no contraste, volta ao padrão e recusa entradas inválidas (RF-04, RF-05, RF-09, RF-10, RF-11, CT-03, CT-04, CT-08, CT-09, CT-10, CT-14).
  - Trecho: as linhas `test brand_... ok`.
  - Falha: `FAILED` ou ausência dos testes `brand_`.
  - Espera: `brand_text_pairs_pass_aa_in_both_themes ... ok`
- `brand-saida.txt` (automática): `T="$(mktemp -d)"; cp -r packages/harness/. "$T/vostok/"; M="$PWD/apps/cli/Cargo.toml"; cd "$T" && cargo run --quiet --manifest-path "$M" -- documents brand --primary "#facc15" --name "Atria"`
  - Requer: `apps/cli/src/commands/documents.rs`
  - Prova: a saída real do comando com uma cor clara, com o antes e depois de cada token e o aviso de ajuste (RF-09, RF-11, CB-04).
  - Trecho: linhas `light.primary: #6d28d9 -> ...` e `aviso: light.primary ajustado de #facc15 para ...`.
  - Falha: erro de comando desconhecido ou nenhuma linha de aviso.
