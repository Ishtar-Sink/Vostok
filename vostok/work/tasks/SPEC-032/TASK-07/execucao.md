### Ler

- `vostok/work/specs/refined/SPEC-032-controlar-o-projeto-inteiramente-pelo-painel-v0-4-0.md`
- `apps/painel/src/shared/router.ts` e `apps/painel/src/shared/router.test.ts`
- `apps/painel/src/layout/sidebar.tsx` (items, ICONS)
- `apps/painel/src/layout/topbar.tsx` (fn crumbs)
- `apps/painel/src/main.tsx` (render das páginas)
- `apps/painel/src/features/documents/documents-page.tsx` (página de lista como exemplo)
- `apps/painel/src/shared/api.ts` (fetchActivities, da TASK-05)

### Mudar

- `apps/painel/src/features/activity/activity-page.tsx` (criar): página `/atividades` com filtros
- `apps/painel/src/features/activity/activity-page.module.css` (criar): estilos da página
- `apps/painel/src/features/activity/activity-page.test.tsx` (criar): testes da página
- `apps/painel/src/shared/router.ts` (alterar): rota `/atividades`
- `apps/painel/src/shared/router.test.ts` (alterar): teste da rota
- `apps/painel/src/layout/sidebar.tsx` (alterar): item "Atividades"
- `apps/painel/src/layout/topbar.tsx` (alterar): trilha da página
- `apps/painel/src/main.tsx` (alterar): render da página

### Passos

1. `apps/painel/src/shared/router.test.ts` (alterar): `parseRoute('/atividades')` devolve `{ page: 'activity' }`. Confira: falha antes da mudança.
2. `apps/painel/src/shared/router.ts` (alterar): acrescente `{ page: 'activity' }` ao tipo `Route` e a regra `first === 'atividades' && parts.length === 1`. Confira: o teste do passo 1 passa.
3. `apps/painel/src/features/activity/activity-page.test.tsx` (criar): `lista todas as atividades`, `filtra por tipo`, `filtra por spec` e `mostra a lista vazia com o filtro visível`, com `fetchActivities` simulado. Confira: falham antes da página existir.
4. `apps/painel/src/features/activity/activity-page.tsx` e `activity-page.module.css` (criar): título "Atividades", dois `select` com rótulo ("Tipo": todos, spec, plano, tarefa, pedido, aval; "Spec": todas e as specs que aparecem nas atividades) e a lista da mais nova para a mais antiga, no mesmo formato de item da atividade recente, buscando `fetchActivities({ tipo, spec, limite: 1000 })`. Lista vazia: "Nenhuma atividade com esses filtros.". Confira: os testes do passo 3 passam.
5. `apps/painel/src/main.tsx` (alterar): `{route.page === 'activity' && <ActivityPage version={version} />}`. `apps/painel/src/layout/topbar.tsx` (alterar): em `crumbs`, o caso `activity` com `[{ label: 'Atividades' }]`. Confira: `pnpm --dir apps/painel exec tsc --noEmit`.
6. `apps/painel/src/layout/sidebar.tsx` (alterar): em `items`, depois de "Tarefas", `{ id: 'activity', href: '/atividades', label: 'Atividades', icon: ICONS.activity }`, com um ícone novo em `ICONS` no mesmo estilo dos outros. Confira: o print `atividades.png` mostra o item na sidebar.
7. `apps/cli` (conferir): antes do print, compile o binário de release com a rota nova da TASK-05: `cargo build --release --features painel --manifest-path apps/cli/Cargo.toml`. Confira: `apps/cli/target/release/vostok --version` responde.
