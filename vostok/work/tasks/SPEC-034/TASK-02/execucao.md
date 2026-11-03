### Ler

- `vostok/work/specs/refined/SPEC-034-corrigir-o-fluxo-do-painel-antes-da-v0-4-0.md` (RF-02, RF-03, RF-04, RF-05, RF-07, CT-02 a CT-07, CT-09, CB-01, CB-03, decisões 1, 2, 3, 8 e 10)
- `vostok/work/tasks/SPEC-034/TASK-01/execucao.md` (campo `plan_outdated` ao lado de `plan` na visão geral e em `GET /api/specs/<ID>`)
- `apps/painel/src/shared/attention.ts`
- `apps/painel/src/shared/attention.test.ts`
- `apps/painel/src/shared/action-bar.tsx`
- `apps/painel/src/shared/api.ts` (interfaces `OpenSpec` e `SpecDetail`)
- `apps/painel/src/features/spec/spec-page.tsx`
- `apps/painel/src/features/spec/spec-page.module.css` (classe `.note`)
- `apps/painel/src/features/spec/spec-page.test.tsx`
- `apps/painel/src/features/task/task-page.test.tsx` (fn `mockFetch` e `state`)
- `apps/painel/src/ui/badge/badge.tsx` (tom `warning` para pendência comum)
- `apps/cli/src/painel/actions.rs` (fn que trata `planejar-spec`, `ajustar-plano` e `finalizar-tarefa`: confirmar que os três pedidos já são aceitos)

### Mudar

- `apps/painel/src/shared/api.ts` (alterar): campo `plan_outdated: boolean` em `OpenSpec` e `SpecDetail`, entregue pela API da TASK-01
- `apps/painel/src/shared/attention.ts` (alterar): kinds `redo-plan` e `adjust-plan`, `specActions` com o plano desatualizado, Pendências sem "Aprovar plano" para plano desatualizado, "Pedir finalização" em `validating` e a ajuda das aprovações com `work/avais/<SPEC>.log`
- `apps/painel/src/shared/attention.test.ts` (alterar): testes de CT-02, CT-03, CT-05, CT-06, CT-07 e CT-09
- `apps/painel/src/shared/action-bar.tsx` (alterar): pedidos `planejar-spec` (`redo-plan`) e `ajustar-plano` (`adjust-plan`, com texto obrigatório) e a mensagem de sucesso da aprovação com o log da spec
- `apps/painel/src/features/spec/spec-page.tsx` (alterar): aviso "Plano desatualizado" no cabeçalho, no documento e na visão do plano, e o sinal passado para `specActions`
- `apps/painel/src/features/spec/spec-page.test.tsx` (alterar): testes do aviso, de "Refazer o plano", de "Ajustar plano" e da mensagem de sucesso da aprovação
- `apps/painel/src/features/task/task-page.test.tsx` (alterar): teste do pedido de finalização na tarefa em `validating`
- `apps/painel/scripts/print.sh` (alterar): pasta do projeto do print pela variável `PAINEL_PASTA`, com a raiz do repositório como padrão
- `apps/painel/scripts/fixture-fluxo.sh` (criar): monta numa pasta um projeto de demonstração com uma spec aprovada de novo depois do plano (SPEC-001) e uma tarefa em validação com print sem conferência (SPEC-002/TASK-01), para os prints

### Passos

1. `apps/painel/src/shared/api.ts` (alterar): em `interface OpenSpec` e em `interface SpecDetail`, logo depois de `plan: boolean;`, acrescente `plan_outdated: boolean;`. Confira: `pnpm --dir apps/painel exec tsc --noEmit` sem erros (os fixtures de teste usam `as unknown as Overview` ou objetos soltos e não quebram).
2. `apps/painel/src/shared/attention.test.ts` (alterar): acrescente, sem mudar os testes que existem:
   - No `describe('taskActions')`: `oferece finalizar na tarefa em validação`: `kinds('validating')` contém `finish-task` e `adjust-task` e não contém `execute-task`.
   - No `describe('specActions')`:
     - `troca aprovar por refazer com o plano desatualizado` (CT-02): `specActions(meta, true, [], true)` (meta `approved`) contém `redo-plan`, não contém `approve-plan`, `adjust-plan` nem `plan-spec`.
     - `não oferece ação de plano na spec refinada com plano desatualizado` (CT-05, CB-01): `specActions({ ...meta, status: 'refined' }, true, [], true).map((item) => item.kind)` é igual a `['approve-spec', 'adjust-spec']`.
     - `oferece ajustar e aprovar com o plano em dia` (CT-06): `specActions(meta, true, [], false)` contém `approve-plan` e `adjust-plan` e não contém `redo-plan`; o índice de `approve-plan` é menor que o de `adjust-plan`.
   - No `describe('attentionItems')`: `não lista aprovar plano com o plano desatualizado` (CT-03): uma visão geral com `specs: [{ meta, plan: true, plan_outdated: true, tasks: [] }]` não tem item `approve-plan`; com `plan_outdated: false`, tem.
   - Novo `describe('ACTION_LABELS')` (importe `ACTION_LABELS`): `cita o log de avais da spec na ajuda das aprovações` (CT-09): o `how` de `approve-spec` e o de `approve-task` contêm `work/avais/<SPEC>.log` e não contêm `avais.log`.
   Confira: `pnpm --dir apps/painel exec vitest run src/shared/attention.test.ts` com os testes novos falhando e os antigos passando.
3. `apps/painel/src/shared/attention.ts` (alterar):
   - Em `ActionKind`, acrescente `| 'redo-plan' | 'adjust-plan'`.
   - Em `attentionItems`, troque `for (const { meta, plan, tasks } of overview.specs)` por `for (const { meta, plan, plan_outdated, tasks } of overview.specs)` e a condição `if (plan)` do item `approve-plan` por `if (plan && !plan_outdated)`. Nenhum item novo em Pendências.
   - Em `ACTION_LABELS`:
     - `'approve-spec'.how`: `'O painel chama a mesma aprovação da CLI (`vostok spec approve`), grava a origem `painel` e o aval no log de avais da spec, `work/avais/<SPEC>.log`.'`
     - `'approve-task'.how`: `'O painel chama a mesma aprovação da CLI (`vostok task approve`), grava a origem `painel` e o aval no log de avais da spec, `work/avais/<SPEC>.log`.'`
     - novo `'redo-plan': { button: 'Refazer o plano', tag: 'Pedido ao agente', how: 'A spec mudou depois do plano, e ele não cobre a versão aprovada. O painel grava um pedido de planejamento; o agente em loop recria o plano pela skill plan-tasks.' }`
     - novo `'adjust-plan': { button: 'Ajustar plano', tag: 'Pedido ao agente', how: 'Você descreve o ajuste; o painel grava um pedido e o agente em loop o aplica pela skill adjust-plan.' }`
   - Em `PRIMARY`, acrescente `'redo-plan'` (`adjust-plan` fica secundário, como `adjust-spec`).
   - Em `specActions`, mude a assinatura para `specActions(meta: { id: string; title: string; status: string }, plan: boolean, tasks: TaskSummary[], planOutdated = false)` e troque a linha `if (plan) items.push({ ...base, key: 'approve-plan', kind: 'approve-plan' });` por:
     ```ts
     if (plan && !planOutdated) {
       items.push({ ...base, key: 'approve-plan', kind: 'approve-plan' });
       items.push({ ...base, key: 'adjust-plan', kind: 'adjust-plan' });
     }
     if (plan && planOutdated && meta.status === 'approved') items.push({ ...base, key: 'redo-plan', kind: 'redo-plan' });
     ```
     A linha do `plan-spec` (`meta.status === 'approved' && !plan`) não muda.
   - Em `taskActions`, troque `if (state.status === 'in_progress')` por `if (state.status === 'in_progress' || state.status === 'validating')` na linha do `finish-task`.
   Confira: os testes do passo 2 passam e os antigos continuam passando.
4. `apps/painel/src/shared/action-bar.tsx` (alterar):
   - Em `REQUEST_ACTION`, acrescente `'redo-plan': 'planejar-spec'` e `'adjust-plan': 'ajustar-plano'`.
   - Em `NEEDS_TEXT`, acrescente `'adjust-plan'` (o rótulo do campo fica o padrão, "O que ajustar").
   - Em `performAction`, troque a mensagem de sucesso da aprovação por ``return { ok: true, message: `Aprovado pelo painel: ${approved.join(', ')}. O aval ficou registrado em vostok/work/avais/${item.spec}.log.` };``.
   Confira: `pnpm --dir apps/painel exec tsc --noEmit` sem erros e `grep -rn 'avais\.log' apps/painel/src` sem resultado.
5. `apps/painel/src/features/spec/spec-page.test.tsx` (alterar): acrescente, usando `detail`, `requests` e `lastButton` do arquivo:
   - `mostra o aviso de plano desatualizado na spec` (CT-04, RF-02, RF-03): corpo `{ ...detail('vostok/work/specs/approved/SPEC-001.md', 'approved'), plan: true, plan_outdated: true, tasks: [] }`, view `documento`. Confira `await screen.findByText('Plano desatualizado')`, `screen.getByText('Desatualizado: a spec mudou depois do plano.')`, `getByRole('button', { name: 'Refazer o plano' })` presente e `queryByRole('button', { name: 'Aprovar plano' })` e `queryByRole('button', { name: 'Ajustar plano' })` nulos.
   - `mostra o aviso de plano desatualizado na visão do plano` (CT-04): o mesmo corpo, view `plano`, com `/api/planos` respondendo `json({ id: 'SPEC-001', path: 'p.md', markdown: '## Plano\nx' })`. Confira `await screen.findByText(/A spec mudou depois deste plano/)` e `screen.queryByText('Revisão')` nulo.
   - `pede refazer o plano desatualizado` (RF-03): o corpo do primeiro teste; clique em "Refazer o plano", depois em `lastButton('Refazer o plano')`, e confira `fetch` com `'/api/pedidos?acao=planejar-spec&spec=SPEC-001'` e `expect.objectContaining({ method: 'POST' })`.
   - `pede o ajuste do plano em dia com texto` (CT-06, RF-04): corpo com `status` `approved`, `plan: true`, `plan_outdated: false`, `tasks: []`. Confira que não há texto `Plano desatualizado`; clique em "Ajustar plano", confira `lastButton('Ajustar plano').disabled` verdadeiro, preencha `getByRole('textbox', { name: 'O que ajustar' })` com `Dividir a TASK-02`, clique em `lastButton('Ajustar plano')` e confira `fetch` com `'/api/pedidos?acao=ajustar-plano&spec=SPEC-001'` e `expect.objectContaining({ method: 'POST', body: 'Dividir a TASK-02' })`.
   - `cita o log de avais da spec ao aprovar` (CT-09, RF-07): corpo com `status` `refined`, `plan: false`, `tasks: []`; o simulado responde `json({ aprovadas: ['SPEC-001'] })` para `/api/aprovacoes`. Clique em "Aprovar spec" e em `lastButton('Aprovar spec')`; confira que `await screen.findByRole('status')` contém `vostok/work/avais/SPEC-001.log`.
   Confira: falham antes do passo 6, menos o de CT-09, que já passa depois do passo 4.
6. `apps/painel/src/features/spec/spec-page.tsx` (alterar):
   - Troque `const { meta, tasks, plan } = spec.data;` por `const { meta, tasks, plan, plan_outdated } = spec.data;` e crie `const outdated = plan && plan_outdated;`.
   - No `headerActions`, logo depois do `Badge` do status, acrescente `{outdated ? <Badge tone="warning">Plano desatualizado</Badge> : null}` e passe o sinal: `<ActionBar items={specActions(meta, plan, tasks, outdated)} />`.
   - No `<dl className={page.meta}>` do documento, depois de "Ajustes", acrescente `{outdated ? <div><dt>Plano</dt><dd>Desatualizado: a spec mudou depois do plano.</dd></div> : null}`.
   - Passe `outdated={outdated}` e `status={meta.status}` para `PlanView` e acrescente as props `outdated: boolean; status: string` à assinatura. No `aside` do `PlanView`, com `outdated`, troque o título "Revisão" e a nota por `<p className={page.asideTitle}>Plano desatualizado</p>` e `<p className={styles.note}>{status === 'approved' ? 'A spec mudou depois deste plano, e ele não cobre a versão aprovada. Use "Refazer o plano" para criar um plano novo.' : 'A spec mudou depois deste plano e espera uma nova aprovação. Aprove a spec antes de refazer o plano.'}</p>`; sem `outdated`, o aside continua como está.
   Confira: `pnpm --dir apps/painel exec vitest run src/features/spec/` com os testes do passo 5 e os antigos passando.
7. `apps/painel/src/features/task/task-page.test.tsx` (alterar): acrescente `pede a finalização da tarefa em validação` (CT-07, CB-03): `mockFetch(taskDetail({ status: 'validating' }))`, renderize `<TaskPage spec="SPEC-001" task="TASK-02" version={0} />`, clique em `await screen.findByRole('button', { name: 'Pedir finalização' })`, depois no último botão "Pedir finalização" (o do diálogo, por `getAllByRole(...).at(-1)`), e confira com `waitFor` que `fetch` foi chamado com `'/api/pedidos?acao=finalizar-tarefa&spec=SPEC-001&tarefa=TASK-02'` e `expect.objectContaining({ method: 'POST' })`. Confira: passa depois do passo 3, sem mudar `task-page.tsx` (a barra vem de `taskActions`).
8. `apps/painel` (conferir): `pnpm --dir apps/painel exec tsc --noEmit` e `grep -rn 'avais\.log' apps/painel/src`. Confira: sem erros e sem resultado no `grep`.
9. `apps/painel/scripts/print.sh` (alterar): troque `(cd "$ROOT" && exec "$BIN" painel --sem-navegador --porta "$API_PORT")` por `(cd "$ROOT" && exec "$BIN" painel --sem-navegador --porta "$API_PORT" --pasta "${PAINEL_PASTA:-$ROOT}")` e cite `PAINEL_PASTA` no comentário de uso. Confira: sem a variável, o print de `/specs` continua mostrando as specs do repositório.
10. `apps/painel/scripts/fixture-fluxo.sh` (criar, com `chmod +x`): script bash com `set -euo pipefail` que recebe a pasta de destino vazia, roda `git init -q` e `"${PAINEL_BIN:-vostok}" init --yes` nela e grava em `vostok/work/`: `specs/refined/SPEC-001-exportar-a-lista-de-tarefas.md` com `status: approved` e um `approved_hash`, e `plans/SPEC-001.md` com um `spec_hash` diferente (plano desatualizado); `specs/refined/SPEC-002-...md` com `status: planned` e a pasta `tasks/SPEC-002/TASK-01/` com `estado.yml` em `status: validating` e os arquivos da tarefa, com uma evidência `(print)` sem conferência no registro. Confira: com o painel do binário de release na pasta, `GET /api/specs/SPEC-001` traz `"plan_outdated":true` e `GET /api/tarefas/SPEC-002/TASK-01` traz `validating`.
11. `apps/painel` e `apps/cli` (conferir): gere o bundle e o binário com a API da TASK-01 e as telas novas: `pnpm --dir apps/painel build` e depois `cargo build --release --features painel --manifest-path apps/cli/Cargo.toml`. Confira: `apps/cli/target/release/vostok --version` responde.
