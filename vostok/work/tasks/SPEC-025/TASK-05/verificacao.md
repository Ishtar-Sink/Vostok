### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test finalize 2>&1 | grep -E '^test ' | grep -v golden_finalize | sort`

### Verificação

```bash
# evidência: visual-tests.txt
cargo test --manifest-path apps/cli/Cargo.toml --test finalize -- visual_
cargo test --manifest-path apps/cli/Cargo.toml --test finalize
cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings
```

Resultado esperado: os testes `visual_` e os goldens da finalização passam, e o clippy termina sem avisos.

### Evidências

- `visual-tests.txt` (automática, da verificação)
  - Prova: tema claro padrão, nenhuma requisição externa e títulos do técnico das tarefas renderizados (RF-10, RF-12, CT-09, CT-10).
  - Trecho: as linhas `test visual_... ok`.
  - Falha: `FAILED` ou ausência dos testes `visual_`.
  - Espera: `visual_task_technical_headings_render_as_headings ... ok`
- `executivo-novo.png` (print): `T="$(mktemp -d)"; A=vostok/work/.archive/SPEC-023; mkdir -p "$T/vostok/work/specs/refined" "$T/vostok/work/tasks" "$T/vostok/work/finalize/SPEC-023"; cp -r packages/harness/. "$T/vostok/"; sed 's/^status: .*/status: implemented/' "$A/spec.md" > "$T/vostok/work/specs/refined/SPEC-023-tarefa-como-pasta.md"; cp -r "$A/tasks" "$T/vostok/work/tasks/SPEC-023"; cp "$A/content.md" "$T/vostok/work/finalize/SPEC-023/"; git -C "$T" init -q; git -C "$T" add -A; git -C "$T" -c user.name=v -c user.email=v@v commit -qm base; M="$PWD/apps/cli/Cargo.toml"; (cd "$T" && cargo run --quiet --manifest-path "$M" -- finalize build SPEC-023 >/dev/null 2>&1); CHROME="$(ls -d ~/.cache/ms-playwright/chromium-*/chrome-linux64/chrome | sort | tail -1)"; "$CHROME" --headless=new --no-sandbox --disable-gpu --hide-scrollbars --window-size=1280,3600 --screenshot="$VOSTOK_EVIDENCE/executivo-novo.png" "file://$T/vostok/work/finalize/SPEC-023/executivo.html"`
  - Prova: o executivo gerado no modelo escolhido (RF-10, CT-10).
  - Trecho: o executivo da SPEC-023 (arquivada), gerado numa cópia temporária com o harness desta versão, inteiro no tema claro.
  - Falha: o visual antigo, escuro com faixa.
- `evidencias-novo.png` (print): `T="$(mktemp -d)"; A=vostok/work/.archive/SPEC-023; mkdir -p "$T/vostok/work/specs/refined" "$T/vostok/work/tasks" "$T/vostok/work/finalize/SPEC-023"; cp -r packages/harness/. "$T/vostok/"; sed 's/^status: .*/status: implemented/' "$A/spec.md" > "$T/vostok/work/specs/refined/SPEC-023-tarefa-como-pasta.md"; cp -r "$A/tasks" "$T/vostok/work/tasks/SPEC-023"; cp "$A/content.md" "$T/vostok/work/finalize/SPEC-023/"; git -C "$T" init -q; git -C "$T" add -A; git -C "$T" -c user.name=v -c user.email=v@v commit -qm base; M="$PWD/apps/cli/Cargo.toml"; (cd "$T" && cargo run --quiet --manifest-path "$M" -- finalize build SPEC-023 >/dev/null 2>&1); CHROME="$(ls -d ~/.cache/ms-playwright/chromium-*/chrome-linux64/chrome | sort | tail -1)"; "$CHROME" --headless=new --no-sandbox --disable-gpu --hide-scrollbars --window-size=1280,3600 --screenshot="$VOSTOK_EVIDENCE/evidencias-novo.png" "file://$T/vostok/work/finalize/SPEC-023/evidencias.html"`
  - Prova: o documento de evidências no modelo escolhido (RF-09, CT-07).
  - Trecho: as peças de evidência da SPEC-023 (arquivada), geradas numa cópia temporária, no visual novo.
  - Falha: o visual antigo.
