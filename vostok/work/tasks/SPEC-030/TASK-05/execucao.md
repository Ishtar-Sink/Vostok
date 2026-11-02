### Ler

- `vostok/work/specs/refined/SPEC-030-trava-de-aval-e-loop-de-pedidos-do-painel.md` (RF-16, CT-17, RN-01, RN-05)
- `packages/harness/rules/agent/boundaries.md`
- `packages/harness/skills/default/approve-spec/SKILL.md` (regra [approve-spec-01])
- `packages/harness/skills/default/approve-plan/SKILL.md` (regra [approve-plan-01])
- `packages/harness/skills/default/approve-task/SKILL.md` (regra [approve-task-01])
- `packages/harness/skills/default/finish-task/SKILL.md` (regras [finish-task-08] e [finish-task-09])
- `packages/harness/skills/default/finalize-spec/SKILL.md` (regra [finalize-spec-01])
- `packages/harness/skills/default/approve-commit/SKILL.md` (regra [approve-commit-01] e o passo do commit)

### Mudar

- `packages/harness/rules/agent/boundaries.md` (alterar): o pedido do painel com aval vale como pedido do usuário; o agente nunca grava aval, `approved_by` nem chama as rotas do painel.
- `packages/harness/skills/default/approve-spec/SKILL.md` (alterar): `[approve-spec-01]` aceita o pedido do painel.
- `packages/harness/skills/default/approve-plan/SKILL.md` (alterar): `[approve-plan-01]` aceita o pedido `aprovar-plano` do painel.
- `packages/harness/skills/default/approve-task/SKILL.md` (alterar): `[approve-task-01]` aceita o pedido do painel.
- `packages/harness/skills/default/finish-task/SKILL.md` (alterar): `[finish-task-08]` aceita o pedido `finalizar-tarefa` do painel.
- `packages/harness/skills/default/finalize-spec/SKILL.md` (alterar): `[finalize-spec-01]` aceita o pedido `finalizar-spec` do painel.
- `packages/harness/skills/default/approve-commit/SKILL.md` (alterar): `[approve-commit-01]` aceita o pedido `fazer-commit` e o commit leva `--pedido PED-NNN`.

### Passos

1. `packages/harness/rules/agent/boundaries.md` (alterar): acrescente `[agent/boundaries-05]`: "Um pedido gravado pelo painel em `vostok/work/pedidos/` vale como pedido do usuário. Com a trava (`approval_lock: true`), concluir, aprovar, arquivar e fazer commit só passam com o aval do painel; sem ele, recuse o pedido com `vostok pedidos recusar`. Nunca grave `vostok/work/avais.log`, `approved_by` nem chame as rotas do painel (`/api/...`)." Em `[agent/boundaries-01]`, troque "autorização expressa do usuário" por "autorização expressa do usuário, no chat ou por um pedido `fazer-commit` do painel".
2. `packages/harness/skills/default/approve-spec/SKILL.md`, `approve-task/SKILL.md` e `approve-plan/SKILL.md` (alterar): na regra `-01` de cada uma, depois de "a pedido do usuário", acrescente "(no chat ou por um pedido do painel)"; em `approve-plan`, cite a ação `aprovar-plano`.
3. `packages/harness/skills/default/finish-task/SKILL.md` (alterar): em `[finish-task-08]`, acrescente que um pedido `finalizar-tarefa` do painel é o pedido do usuário; em `[finish-task-09]`, que o print continua conferido pelo usuário (o pedido do painel não confere print).
4. `packages/harness/skills/default/finalize-spec/SKILL.md` (alterar): em `[finalize-spec-01]`, o pedido `finalizar-spec` do painel cita a spec.
5. `packages/harness/skills/default/approve-commit/SKILL.md` (alterar): em `[approve-commit-01]`, o pedido `fazer-commit` do painel vale como pedido do usuário; no passo do commit e da liberação, com a trava ligada, passe `--pedido PED-NNN`.
6. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test context_memory_rules --test commit_skill` passa.
