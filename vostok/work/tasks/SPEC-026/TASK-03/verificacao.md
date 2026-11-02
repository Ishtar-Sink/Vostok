### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test finalize 2>&1 | grep -E '^test ' | sort`

### Verificação

```bash
# evidência: brand-build-tests.txt
cargo test --manifest-path apps/cli/Cargo.toml --test finalize -- brand_ coverage_
cargo test --manifest-path apps/cli/Cargo.toml --test finalize
cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings
```

Resultado esperado: os testes `brand_` e `coverage_`, os `visual_*` e os goldens da finalização passam, e o clippy termina sem avisos.

### Evidências

- `brand-build-tests.txt` (automática, da verificação)
  - Prova: nome e ícone no topo, identidade atual sem configuração, recusa de ícone inválido e IDs sem quebra (RF-06, RF-07, RF-08, RF-12, CT-05, CT-06, CT-07, CT-11, CB-02, CB-03, CB-08).
  - Trecho: as linhas `test brand_... ok` e `test coverage_ids_do_not_wrap ... ok`.
  - Falha: `FAILED` ou ausência desses testes.
  - Espera: `brand_icon_is_embedded_in_the_header ... ok`
- `executivo-marca.png` (print): `T="$(mktemp -d)"; A=vostok/work/.archive/SPEC-025; mkdir -p "$T/vostok/work/specs/refined" "$T/vostok/work/tasks" "$T/vostok/work/finalize/SPEC-025"; cp -r packages/harness/. "$T/vostok/"; sed 's/^status: .*/status: implemented/' "$A/spec.md" > "$T/vostok/work/specs/refined/SPEC-025-documentos-finais-com-visual-e-conteudo-profissionais.md"; cp -r "$A/tasks" "$T/vostok/work/tasks/SPEC-025"; cp "$A/content.md" "$T/vostok/work/finalize/SPEC-025/"; printf '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><circle cx="8" cy="8" r="7" fill="#0f766e"/></svg>' > "$T/logo.svg"; sed -i 's/^  name: ""/  name: "Atria"/; s/^  icon: ""/  icon: "logo.svg"/' "$T/vostok/config/documents.yml"; git -C "$T" init -q; git -C "$T" add -A; git -C "$T" -c user.name=v -c user.email=v@v commit -qm base; M="$PWD/apps/cli/Cargo.toml"; (cd "$T" && cargo run --quiet --manifest-path "$M" -- finalize build SPEC-025 >/dev/null 2>&1); CHROME="$(ls -d ~/.cache/ms-playwright/chromium-*/chrome-linux64/chrome | sort | tail -1)"; "$CHROME" --headless=new --no-sandbox --disable-gpu --hide-scrollbars --window-size=1280,900 --screenshot="$VOSTOK_EVIDENCE/executivo-marca.png" "file://$T/vostok/work/finalize/SPEC-025/executivo.html"`
  - Prova: o executivo da SPEC-025 (arquivada) com o nome "Atria" e o ícone no topo (RF-06, RF-07, CT-05, CT-06).
  - Trecho: o topo com o círculo verde e "Atria" no lugar do quadrado e de "Vostok".
  - Falha: "Vostok" com o quadrado de gradiente.
- `cobertura-ids.png` (print): `T="$(mktemp -d)"; A=vostok/work/.archive/SPEC-025; mkdir -p "$T/vostok/work/specs/refined" "$T/vostok/work/tasks" "$T/vostok/work/finalize/SPEC-025"; cp -r packages/harness/. "$T/vostok/"; sed 's/^status: .*/status: implemented/' "$A/spec.md" > "$T/vostok/work/specs/refined/SPEC-025-documentos-finais-com-visual-e-conteudo-profissionais.md"; cp -r "$A/tasks" "$T/vostok/work/tasks/SPEC-025"; cp "$A/content.md" "$T/vostok/work/finalize/SPEC-025/"; git -C "$T" init -q; git -C "$T" add -A; git -C "$T" -c user.name=v -c user.email=v@v commit -qm base; M="$PWD/apps/cli/Cargo.toml"; (cd "$T" && cargo run --quiet --manifest-path "$M" -- finalize build SPEC-025 >/dev/null 2>&1); CHROME="$(ls -d ~/.cache/ms-playwright/chromium-*/chrome-linux64/chrome | sort | tail -1)"; "$CHROME" --headless=new --no-sandbox --disable-gpu --hide-scrollbars --window-size=1280,1500 --screenshot="$VOSTOK_EVIDENCE/cobertura-ids.png" "file://$T/vostok/work/finalize/SPEC-025/evidencias.html"`
  - Prova: o mapa de cobertura com os IDs numa linha só (RF-12, CT-11).
  - Trecho: as linhas `RNF-01` a `RNF-05` da tabela, cada ID numa linha.
  - Falha: `RNF-` e o número em linhas separadas.
