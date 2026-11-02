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
- `skill-conversa.png` (manual): print da conversa seguindo a skill num repositório de teste com uma cor `--color-primary`, mostrando a proposta e o pedido de confirmação antes do `vostok documents brand` (CT-12), e num sem cores, mostrando a pergunta (CT-13).
  - Prova: o comportamento da skill nos dois caminhos (RF-01, RF-03, CT-12, CT-13).
  - Trecho: a pergunta com as cores propostas e, no outro, a pergunta da cor principal.
  - Falha: gravação sem pergunta.
