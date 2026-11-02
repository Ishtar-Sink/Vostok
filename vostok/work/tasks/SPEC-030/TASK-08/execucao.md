### Ler

- `vostok/work/specs/refined/SPEC-030-trava-de-aval-e-loop-de-pedidos-do-painel.md` (RF-14, CT-15, RN-03, RN-04, "Fluxos")
- `docs/estudos/painel-local/testes.md` (seção "Pedido ao agente e loop (PQ-08)": a instrução usada no teste e os tempos)
- `packages/harness/templates/skill/skill.md` (template de uma skill)
- `packages/harness/skills/default/finish-task/SKILL.md` (exemplo de skill completa)
- `packages/harness/context/skill-catalog.md`
- `packages/harness/config/context.yml` (mapa de contexto por skill)
- `apps/cli/src/painel/actions.rs` (const REQUEST_ACTIONS: as 11 ações)

### Mudar

- `packages/harness/skills/default/process-requests/SKILL.md` (criar): a skill com a instrução do loop.
- `packages/harness/context/skill-catalog.md` (alterar): linha da skill `process-requests`.
- `packages/harness/config/context.yml` (alterar): entrada da skill no mapa de contexto.

### Passos

1. `packages/harness/skills/default/process-requests/SKILL.md` (criar): a partir de `packages/harness/templates/skill/skill.md`, com `name: process-requests` e uma `description` que diga quando usar ("processar os pedidos do painel", "/loop", "atender pedidos"). O corpo segue o padrão das outras skills (Termos, Regras, Checklist, Passos):
   - Passo 1: `vostok pedidos listar --estado pendente`; sem pedido, encerre sem responder.
   - Passo 2: para o primeiro pedido, em ordem de ID: `vostok pedidos iniciar PED-NNN`.
   - Passo 3: mapa da ação para a skill: `aprovar-plano` → approve-plan, `planejar-spec` → plan-tasks, `ajustar-spec` → adjust-spec, `ajustar-plano` → adjust-plan, `ajustar-tarefa` → adjust-task, `executar-tarefa` → execute-task, `finalizar-tarefa` → finish-task, `desbloquear-tarefa` → finish-task (retomada), `finalizar-spec` → finalize-spec, `responder-pergunta` → a pergunta citada, `fazer-commit` → approve-commit com `--pedido PED-NNN`. O texto de `## Pedido` é a mensagem do usuário.
   - Passo 4: `vostok pedidos concluir PED-NNN` com o resultado, ou `vostok pedidos recusar PED-NNN` com o motivo (RN-04: pedido ambíguo, pedido de print a conferir ou recusa da CLI viram `recusado`).
   - Regras: um pedido por vez; nunca editar o arquivo do pedido; nunca gravar aval; a conferência de print continua do usuário.
   - Passo 0, "Armar o loop": com um intervalo no argumento (`/process-requests 1m`, `30s`, `5m`), no Claude Code, a skill arma o loop chamando a skill `loop` com `<intervalo> /process-requests` (sem intervalo, para o loop não se armar de novo) e segue para uma rodada; no Codex, ela orienta `vostok pedidos vigiar --agente codex --intervalo <segundos>`. Sem intervalo, faz uma rodada só.
   - Seção "Como armar o loop": `/process-requests 1m` no Claude Code e `vostok pedidos vigiar --agente codex` no Codex; a `description` cita o intervalo.
2. `packages/harness/context/skill-catalog.md` (alterar): uma linha na tabela "Padrão" com o nome, a descrição igual à do front-matter e o caminho `../skills/default/process-requests`.
3. `packages/harness/config/context.yml` (alterar): entrada `process-requests:` com `config/tasks.yml`, na mesma forma das vizinhas.
4. Confira: `cargo build --manifest-path apps/cli/Cargo.toml` passa (o pacote é embutido) e `cargo test --manifest-path apps/cli/Cargo.toml --test init_update --test context_memory_rules` passa.
