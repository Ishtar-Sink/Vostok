### Ler

- `vostok/work/specs/refined/SPEC-033-operar-o-projeto-de-ponta-a-ponta-pelo-painel-v0-4-0.md` (RF-05, CT-05, CT-06, CT-18, RNF-04, "Contratos e interfaces")
- `vostok/work/tasks/SPEC-033/TASK-08/execucao.md` (pedidos `criar-spec`, `refinar-spec` e `executar-onda` e o parâmetro da onda)
- `apps/painel/src/shared/attention.ts`
- `apps/painel/src/shared/attention.test.ts`
- `apps/painel/src/shared/action-bar.tsx`
- `apps/painel/src/features/specs/specs-page.tsx`
- `apps/painel/src/features/specs/specs-page.test.tsx`
- `apps/painel/src/features/spec/spec-page.tsx`
- `apps/painel/src/features/spec/spec-page.test.tsx`
- `apps/painel/src/features/inbox/inbox-page.tsx` (fn RequestCommit: botão no cabeçalho que abre o `ActionDialog`, exemplo do "Nova spec")

### Mudar

- `apps/painel/src/shared/attention.ts` (alterar): ações `create-spec`, `refine-spec` e `execute-wave`, com rótulos, e a próxima onda executável em `specActions`
- `apps/painel/src/shared/attention.test.ts` (alterar): testes de "Refinar" e da próxima onda
- `apps/painel/src/shared/action-bar.tsx` (alterar): pedidos `criar-spec`, `refinar-spec` e `executar-onda`, o texto do assunto e o parâmetro `onda`
- `apps/painel/src/features/specs/specs-page.tsx` (alterar): botão "Nova spec" no cabeçalho, com o diálogo do assunto
- `apps/painel/src/features/specs/specs-page.test.tsx` (alterar): testes do "Nova spec"
- `apps/painel/src/features/spec/spec-page.test.tsx` (alterar): testes de "Refinar" e "Executar onda" na página da spec

### Passos

1. `apps/painel/src/shared/attention.test.ts` (alterar): no `describe('specActions')`, acrescente:
   - `oferece refinar para a spec em rascunho`: `specActions({ ...meta, status: 'draft' }, false, [])` tem `refine-spec` e não tem `adjust-spec`.
   - `oferece executar a próxima onda executável`: com `meta` em `planned` e as tarefas `TASK-01` `done` (onda 1), `TASK-02` `approved` (onda 2, `depends_on` `[TASK-01]`), `TASK-03` `approved` (onda 2, sem dependência) e `TASK-04` `approved` (onda 3, `depends_on` `[TASK-02]`), o item `execute-wave` tem `wave` igual a `'2'`, `tasks` igual a `['TASK-02', 'TASK-03']` e `title` igual a `'Onda 2: TASK-02, TASK-03'`, e não há item `execute-task`.
   - `não oferece executar onda sem tarefa pronta`: `meta` em `planned` com `TASK-01` `approved` e `depends_on` `[TASK-09]` não tem `execute-wave`.
   Use o helper `task` do arquivo, acrescentando o parâmetro `wave` (padrão `'1'`). Confira: os três falham antes do passo 2.
2. `apps/painel/src/shared/attention.ts` (alterar):
   - Em `ActionKind`, acrescente `'create-spec' | 'refine-spec' | 'execute-wave'`; em `AttentionItem`, o campo opcional `wave?: string`.
   - Em `ACTION_LABELS`: `'create-spec': { button: 'Nova spec', tag: 'Pedido ao agente', how: 'Você descreve o assunto; o painel grava um pedido e o agente em loop cria o rascunho pela skill create-spec.' }`, `'refine-spec': { button: 'Refinar', tag: 'Pedido ao agente', how: 'O painel grava um pedido de refinamento; o agente em loop refina pela skill refine-spec e, se tiver dúvidas, as perguntas aparecem em Pendências.' }` e `'execute-wave': { button: 'Executar onda', tag: 'Pedido ao agente', how: 'O painel grava um pedido com a spec e a onda; o agente em loop executa as tarefas prontas dessa onda pela skill execute-task.' }`.
   - Em `PRIMARY`, acrescente `'create-spec'`, `'refine-spec'` e `'execute-wave'`.
   - Em `specActions`: com `meta.status === 'draft'`, acrescente `{ ...base, key: 'refine-spec', kind: 'refine-spec' }` antes dos demais. Troque o bloco do `execute-ready` por: com `meta.status === 'planned'` e `ready.length > 0`, calcule `const wave = String(Math.min(...ready.map((task) => Number(task.wave))))`, `const ids = ready.filter((task) => task.wave === wave).map((task) => task.id)` e acrescente `{ ...base, key: 'execute-wave', kind: 'execute-wave', wave, tasks: ids, title: 'Onda ' + wave + ': ' + ids.join(', ') }`. O `execute-task` por tarefa continua só em `taskActions`.
   Confira: os testes do passo 1 passam e os que já existiam continuam passando.
3. `apps/painel/src/shared/action-bar.tsx` (alterar): em `REQUEST_ACTION`, acrescente `'create-spec': 'criar-spec'`, `'refine-spec': 'refinar-spec'` e `'execute-wave': 'executar-onda'`; em `NEEDS_TEXT`, `'create-spec'`; em `TEXT_LABEL`, `'create-spec': 'Assunto da spec'`. Em `perform`, depois de `task`, monte `const wave = item.wave ? '&onda=' + encodeURIComponent(item.wave) : '';` e acrescente `wave` ao fim da URL do pedido, depois de `task` (`/api/pedidos?acao=<ação><spec><tarefa><onda>`). Confira: `pnpm --dir apps/painel exec tsc --noEmit` sem erros.
4. `apps/painel/src/features/specs/specs-page.test.tsx` (alterar): acrescente `grava o pedido de nova spec com o assunto`: clique em `getByRole('button', { name: 'Nova spec' })`, confira que o último botão "Nova spec" (o do diálogo) está `disabled` sem texto, preencha `getByRole('textbox', { name: 'Assunto da spec' })` com `Tela de login`, clique no último "Nova spec" e confira `fetch` chamado com `'/api/pedidos?acao=criar-spec'` e `expect.objectContaining({ method: 'POST', body: 'Tela de login' })`, e o `getByRole('status')` com `PED-7` (o simulado responde `json({ id: 'PED-7' })` para `/api/pedidos`). Acrescente também `mostra a recusa do pedido de nova spec`: o simulado responde `new Response('ação desconhecida', { status: 400 })` e o `getByRole('alert')` contém `ação desconhecida`. Confira: falham antes do passo 5.
5. `apps/painel/src/features/specs/specs-page.tsx` (alterar): crie no arquivo `function NewSpec()`, no molde do `RequestCommit` de `inbox-page.tsx`: um `Button variant="primary"` com `ACTION_LABELS['create-spec'].button` e o `ActionDialog` com o item `const NEW_SPEC: AttentionItem = { key: 'new-spec', kind: 'create-spec', spec: '', title: '', reason: '', href: '/specs' }`. Passe `actions={<NewSpec />}` ao `PageHeader`. No `EmptyState`, troque a descrição por `Troque o filtro ou crie uma spec com "Nova spec".`. Confira: os testes do passo 4 passam.
6. `apps/painel/src/features/spec/spec-page.test.tsx` (alterar): acrescente:
   - `pede o refinamento da spec em rascunho`: com `detail(...)` em `draft`, `plan: false` e sem tarefas, clique em "Refinar", depois no último "Refinar" (o do diálogo), e confira `fetch` com `'/api/pedidos?acao=refinar-spec&spec=SPEC-001'` e `method: 'POST'`.
   - `pede a execução da próxima onda`: com a spec em `planned`, `plan: false` e as tarefas `TASK-01` `done` (onda 1) e `TASK-02` `approved` (onda 2, `depends_on: '[TASK-01]'`), confira o texto `Onda 2: TASK-02` no diálogo depois de clicar em "Executar onda", clique no último "Executar onda" e confira `fetch` com `'/api/pedidos?acao=executar-onda&spec=SPEC-001&onda=2'`.
   - `não oferece refinar fora do rascunho`: com a spec em `planned`, `queryByRole('button', { name: 'Refinar' })` é `null`.
   O simulado responde `json({ id: 'PED-8' })` para `/api/pedidos` e o `detail` para `/api/specs/`. Confira: passam depois dos passos 2 e 3, sem mudança em `spec-page.tsx` (a barra já vem de `specActions`).
7. `apps/painel` (conferir): `pnpm --dir apps/painel exec tsc --noEmit`. Confira: sem erros.
8. `apps/cli` (conferir): antes dos prints, compile o binário de release com as ações novas da TASK-08: `cargo build --release --features painel --manifest-path apps/cli/Cargo.toml`. Confira: `apps/cli/target/release/vostok --version` responde.
