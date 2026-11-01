### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test finalize 2>&1 | grep -E '^test ' | grep -v golden_finalize | sort`

### Verificação

```bash
# evidência: content-tests.txt
cargo test --manifest-path apps/cli/Cargo.toml --test finalize -- content_
cargo test --manifest-path apps/cli/Cargo.toml
cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings
```

Resultado esperado: os testes `content_` e toda a suíte passam, e o clippy termina sem avisos.

### Evidências

- `content-tests.txt` (automática, da verificação)
  - Prova: a validação e a geração das seções novas do executivo, do técnico e das evidências (RF-06, RF-07, RF-08, RF-11, CT-05, CT-06, CT-08).
  - Trecho: as linhas `test content_... ok`.
  - Falha: `FAILED` ou ausência dos testes `content_`.
  - Espera: `content_validate_refuses_an_empty_new_section ... ok`
- `finalize-spec.diff` (diff): `packages/harness/skills/default/finalize-spec/SKILL.md`
  - Prova: as regras de escrita do estudo na skill (RF-11, CT-05).
  - Trecho: o passo 4 novo.
  - Falha: diff sem as regras de formato.
  - Espera: `prosa`
- `tecnico-novo.png` (print): `cargo run --quiet --manifest-path apps/cli/Cargo.toml -- finalize build SPEC-025 >/dev/null 2>&1; CHROME="$(ls -d ~/.cache/ms-playwright/chromium-*/chrome-linux64/chrome | sort | tail -1)"; "$CHROME" --headless=new --no-sandbox --disable-gpu --hide-scrollbars --window-size=1280,3600 --screenshot="$VOSTOK_EVIDENCE/tecnico-novo.png" "file://$PWD/vostok/work/finalize/SPEC-025/tecnico.html"`
  - Prova: o técnico gerado com as seções novas de sustentação (RF-07, RF-08, CT-06).
  - Trecho: as seções de operação, mapa do código e limites.
  - Falha: as seções antigas do técnico.
