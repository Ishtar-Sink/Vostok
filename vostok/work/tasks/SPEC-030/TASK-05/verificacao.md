### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test context_memory_rules --test commit_skill 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`

### Verificação

```bash
# evidência: regras-testes.txt
cargo test --manifest-path apps/cli/Cargo.toml --test context_memory_rules --test commit_skill
```

Resultado esperado: os testes das regras e da skill de commit passam.

### Evidências

- `regras-testes.txt` (automática, da verificação)
  - Prova: as regras continuam válidas e carregáveis depois da mudança (RF-16).
  - Trecho: `test result: ok` nas duas suítes.
  - Falha: `FAILED`.
  - Espera: `test result: ok`
- `boundaries.md.diff` (diff): `packages/harness/rules/agent/boundaries.md`
  - Prova: a regra do pedido do painel com aval como pedido do usuário e a proibição de gravar aval (RF-16, CT-17, RN-01).
  - Trecho: a regra `[agent/boundaries-05]` nova e a mudança em `[agent/boundaries-01]`.
  - Falha: diff vazio ou sem a proibição de gravar o `avais.log`.
- `finish-task.diff` (diff): `packages/harness/skills/default/finish-task/SKILL.md`
  - Prova: a finalização aceita o pedido do painel (RF-16, CT-17).
  - Trecho: a regra `[finish-task-08]` citando o pedido `finalizar-tarefa`.
  - Falha: diff vazio.
- `approve-commit.diff` (diff): `packages/harness/skills/default/approve-commit/SKILL.md`
  - Prova: o commit aceita o pedido `fazer-commit` e leva `--pedido` (RF-16, CT-17).
  - Trecho: a regra `[approve-commit-01]` e o `--pedido PED-NNN`.
  - Falha: diff vazio.
