### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test base 2>&1 | grep -E '^test ' | sort`

### Verificação

```bash
cargo test --manifest-path apps/cli/Cargo.toml --test base
```

Resultado esperado: os testes de `base` passam.

### Evidências

- `approve-plan.diff` (diff): `packages/harness/skills/default/approve-plan/SKILL.md`
  - Prova: o executivo e o técnico escritos no detalhamento (RF-04, CT-07).
  - Trecho: as linhas com `#{TASK_EXECUTIVE}` e `#{TASK_TECHNICAL}`.
  - Falha: diff sem esse trecho.
  - Espera: `TASK_TECHNICAL`
- `finish-task.diff` (diff): `packages/harness/skills/default/finish-task/SKILL.md`
  - Prova: o enriquecimento na conclusão e a recusa do documento igual ao do início (RF-08, CT-07).
  - Trecho: as linhas com "enriquecidos".
  - Falha: diff sem esse trecho.
  - Espera: `enriquecid`
- `approve-task.diff` (diff): `packages/harness/skills/default/approve-task/SKILL.md`
  - Prova: a recusa de documento no template na aprovação (RF-04, CT-07).
  - Trecho: a linha nova do passo 3.
  - Falha: diff sem esse trecho.
  - Espera: `template`
- `adjust-task.diff` (diff): `packages/harness/skills/default/adjust-task/SKILL.md`
  - Prova: o ajuste dos documentos como enriquecimento (RF-08, CT-07).
  - Trecho: a frase nova sobre os documentos.
  - Falha: diff sem esse trecho.
  - Espera: `enriquecimento`
- `executivo-template.diff` (diff): `packages/harness/templates/task/executivo.md`
  - Prova: o comentário do ciclo novo no template do executivo (RF-09, CT-07).
  - Trecho: o comentário novo.
  - Falha: diff sem esse trecho.
  - Espera: `planejamento`
- `tecnico-template.diff` (diff): `packages/harness/templates/task/tecnico.md`
  - Prova: o comentário do ciclo novo no template do técnico (RF-09, CT-07).
  - Trecho: o comentário novo.
  - Falha: diff sem esse trecho.
  - Espera: `planejamento`
- `arquiteto.diff` (diff): `packages/harness/personas/arquiteto.md`
  - Prova: o arquiteto escreve o `tecnico.md` no detalhamento (RF-04, CT-07).
  - Trecho: as linhas com `tecnico.md`.
  - Falha: diff sem esse trecho.
  - Espera: `tecnico.md`
