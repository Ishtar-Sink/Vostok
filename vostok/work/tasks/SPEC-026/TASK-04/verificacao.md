### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test init_update 2>&1 | grep -E '^test ' | sort`
- `cargo test --manifest-path apps/cli/Cargo.toml --test context_memory_rules 2>&1 | grep -E '^test ' | sort`

### Verificação

```bash
# evidência: skill-tests.txt
cargo test --manifest-path apps/cli/Cargo.toml --test init_update
cargo test --manifest-path apps/cli/Cargo.toml --test context_memory_rules
```

Resultado esperado: as duas suítes passam com a skill nova no catálogo e no mapa de contexto.

### Evidências

- `customize-documents.diff` (diff): `packages/harness/skills/default/customize-documents/SKILL.md`
  - Prova: a skill com detecção, proposta, pergunta sem candidatas e gravação só com confirmação (RF-01, RF-03, RN-06).
  - Trecho: os passos 3 a 6 e as regras `[customize-documents-01]` a `[customize-documents-03]`.
  - Falha: passo que grava sem pergunta ou ausência do caminho sem candidatas.
- `skill-tests.txt` (automática, da verificação)
  - Prova: a skill nova entra no catálogo e no contexto sem quebrar a instalação nem o carregamento (RF-01).
  - Trecho: `test result: ok` nas duas suítes.
  - Falha: `FAILED`.
- `skill-comandos.txt` (automática): `T="$(mktemp -d)"; cp -r packages/harness/. "$T/vostok/"; mkdir -p "$T/src" "$T/public"; printf ':root {\n  --color-gray: #6b7280;\n  --color-primary: #0f766e;\n  --color-accent: #f59e0b;\n}\n' > "$T/src/theme.css"; printf '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><circle cx="8" cy="8" r="7"/></svg>\n' > "$T/public/favicon.svg"; M="$PWD/apps/cli/Cargo.toml"; cd "$T" && cargo run --quiet --manifest-path "$M" -- context load --skill customize-documents | grep -c 'vostok documents brand' && cargo run --quiet --manifest-path "$M" -- documents detect && cargo run --quiet --manifest-path "$M" -- documents brand --primary "#0f766e" --secondary "#f59e0b" --name "Teste" --icon public/favicon.svg | tail -3 && grep -A5 '^brand:' vostok/config/documents.yml`
  - Prova: a sequência que a skill executa, num repositório de exemplo: o contexto da skill carrega, o `detect` propõe as cores de marca e o `brand` grava a proposta confirmada (RF-01, CT-12).
  - Trecho: o `detect` com `--color-primary  #0f766e` e `--color-accent  #f59e0b` antes do cinza, e o grupo `brand` com `name: "Teste"`, `icon: "public/favicon.svg"`, `primary: "#0f766e"` e `secondary: "#f59e0b"`.
  - Falha: contexto sem a skill, cinza antes das cores de marca ou grupo `brand` vazio.
- `skill-sem-cores.txt` (automática): `T="$(mktemp -d)"; cp -r packages/harness/. "$T/vostok/"; M="$PWD/apps/cli/Cargo.toml"; cd "$T" && cargo run --quiet --manifest-path "$M" -- documents detect`
  - Prova: num repositório sem cores, o `detect` diz que não achou nada, o caso em que a skill pergunta as cores (RF-03, CT-13).
  - Trecho: `Nenhuma cor encontrada no repositório.`
  - Falha: alguma cor listada ou erro.
