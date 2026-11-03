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
- `apps/cli/src/commands/task.rs` (alterar): data e hora (UTC, ISO) na coluna Data do registro de execução
- `apps/cli/src/commands/finalize.rs` (alterar): a data da conclusão lida do registro ignora a hora
- `apps/cli/tests/common/golden.rs` (alterar): a hora do registro conta como volátil na comparação
- `apps/cli/tests/task.rs` (alterar): teste da hora no registro
- `apps/painel/src/shared/api.ts` (alterar): `formatDateTime`, com data e hora locais
- `apps/painel/src/shared/api.test.ts` (criar): testes do `formatDateTime`

### Passos

1. `apps/painel/src/shared/router.test.ts` (alterar): `parseRoute('/atividades')` devolve `{ page: 'activity' }`. Confira: falha antes da mudança.
2. `apps/painel/src/shared/router.ts` (alterar): acrescente `{ page: 'activity' }` ao tipo `Route` e a regra `first === 'atividades' && parts.length === 1`. Confira: o teste do passo 1 passa.
3. `apps/painel/src/features/activity/activity-page.test.tsx` (criar): `lista todas as atividades`, `filtra por tipo`, `filtra por spec` e `mostra a lista vazia com o filtro visível`, com `fetchActivities` simulado. Confira: falham antes da página existir.
4. `apps/painel/src/features/activity/activity-page.tsx` e `activity-page.module.css` (criar): título "Atividades", dois `select` com rótulo ("Tipo": todos, spec, plano, tarefa, pedido, aval; "Spec": todas e as specs que aparecem nas atividades) e a lista da mais nova para a mais antiga, no mesmo formato de item da atividade recente, buscando `fetchActivities({ tipo, spec, limite: 1000 })`. Lista vazia: "Nenhuma atividade com esses filtros.". Confira: os testes do passo 3 passam.
5. `apps/painel/src/main.tsx` (alterar): `{route.page === 'activity' && <ActivityPage version={version} />}`. `apps/painel/src/layout/topbar.tsx` (alterar): em `crumbs`, o caso `activity` com `[{ label: 'Atividades' }]`. Confira: `pnpm --dir apps/painel exec tsc --noEmit`.
6. `apps/painel/src/layout/sidebar.tsx` (alterar): em `items`, depois de "Tarefas", `{ id: 'activity', href: '/atividades', label: 'Atividades', icon: ICONS.activity }`, com um ícone novo em `ICONS` no mesmo estilo dos outros. Confira: o print `atividades.png` mostra o item na sidebar.
7. `apps/cli` (conferir): antes do print, compile o binário de release com a rota nova da TASK-05: `cargo build --release --features painel --manifest-path apps/cli/Cargo.toml`. Confira: `apps/cli/target/release/vostok --version` responde.
8. `apps/cli/tests/task.rs` (alterar): acrescente `record_has_the_time`: depois de `vostok task start`, a última linha do `registro.md` tem na coluna Data um valor `AAAA-MM-DDTHH:MM:SSZ`. Confira: falha antes da mudança.
9. `apps/cli/src/commands/task.rs` (alterar): em `record_step`, grave `utc_now()` inteiro na coluna Data, em vez de `utc_now()[..10]`. Linhas antigas, só com a data, continuam válidas. Confira: o teste do passo 8 passa.
10. `apps/cli/src/commands/finalize.rs` (alterar): onde a data é lida do registro (`task_done_date` e a linha do tempo), use só os 10 primeiros caracteres, para os documentos finais continuarem só com a data. `apps/cli/tests/common/golden.rs` (alterar): em `normalize`, numa linha de tabela do registro (`| N | ... |`), o `<DATETIME>` da coluna Data vira `<DATE>`, porque a hora é volátil como a data. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test task --test finalize` passa.
11. `apps/painel/src/shared/api.test.ts` (criar) e `apps/painel/src/shared/api.ts` (alterar): `formatDateTime(value)` mostra `hoje, 14:32`, `ontem, 14:32` ou `03/11/2026, 14:32`, no fuso local, para um valor ISO com hora; um valor só com a data mostra `hoje`, `ontem` ou `03/11/2026`; vazio mostra `—`. Datas ISO dentro do detalhe da atividade (como `aprovada em 2026-11-03T15:46:07Z`) também saem formatadas, pela função `formatDatesIn(text)`. Confira: os testes passam.
12. `apps/painel/src/features/activity/activity-page.tsx` (alterar): use `formatDateTime` na data e `formatDatesIn` no detalhe de cada item. Confira: no print `atividades.png`, datas com hora, em ordem da mais nova para a mais antiga.
