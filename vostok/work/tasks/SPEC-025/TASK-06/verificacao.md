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
- `tecnico-novo.png` (print): `T="$(mktemp -d)"; A=vostok/work/.archive/SPEC-023; mkdir -p "$T/vostok/work/specs/refined" "$T/vostok/work/tasks"; cp -r packages/harness/. "$T/vostok/"; sed 's/^status: .*/status: implemented/' "$A/spec.md" > "$T/vostok/work/specs/refined/SPEC-023-tarefa-como-pasta.md"; cp -r "$A/tasks" "$T/vostok/work/tasks/SPEC-023"; git -C "$T" init -q; git -C "$T" add -A; git -C "$T" -c user.name=v -c user.email=v@v commit -qm base; M="$PWD/apps/cli/Cargo.toml"; (cd "$T" && cargo run --quiet --manifest-path "$M" -- finalize create SPEC-023 >/dev/null 2>&1); python3 -c 'import re,sys; p=sys.argv[1]; s=open(p).read(); ids=re.findall(r"^\| (RN?F-\d+) \|", open(sys.argv[2]).read(), re.M); v={"TECH_CODE_MAP": "| Caminho | Papel | Testes |\n|---|---|---|\n| apps/cli/src/commands/task.rs | camada da tarefa | task_layer_ |", "TECH_OPERATION": "1. Converter a spec com vostok task migrate.", "EXEC_NUMBERS": "Não se aplica.", "TECH_NUMBERS": "Não se aplica.", "EVIDENCE_COVERAGE": "| ID | O que foi verificado | Evidência |\n|---|---|---|\n" + "\n".join("| %s | comportamento | TASK-10 / equivalence-tests.txt |" % i for i in ids)}; open(p, "w").write(re.sub(r"#\{([A-Z0-9_]+)\}", lambda m: v.get(m.group(1), "Texto de exemplo do conteúdo."), s))' "$T/vostok/work/finalize/SPEC-023/content.md" "$T/vostok/work/specs/refined/SPEC-023-tarefa-como-pasta.md"; (cd "$T" && cargo run --quiet --manifest-path "$M" -- finalize build SPEC-023 >/dev/null 2>&1); CHROME="$(ls -d ~/.cache/ms-playwright/chromium-*/chrome-linux64/chrome | sort | tail -1)"; "$CHROME" --headless=new --no-sandbox --disable-gpu --hide-scrollbars --window-size=1280,3600 --screenshot="$VOSTOK_EVIDENCE/tecnico-novo.png" "file://$T/vostok/work/finalize/SPEC-023/tecnico.html"`
  - Prova: o técnico gerado com as seções novas de sustentação (RF-07, RF-08, CT-06).
  - Trecho: o técnico da SPEC-023 (arquivada), com conteúdo de exemplo no modelo novo, gerado numa cópia temporária: as seções de operação, mapa do código e limites.
  - Falha: as seções antigas do técnico.
