### Ler

- `vostok/work/specs/refined/SPEC-030-trava-de-aval-e-loop-de-pedidos-do-painel.md` (RF-11, CT-12)
- `apps/painel/src/shared/attention.ts` (type ActionKind, ACTION_LABELS)
- `apps/painel/src/shared/action-bar.tsx` (fn request, o diálogo do texto do pedido)
- `apps/painel/src/features/inbox/inbox-page.tsx`
- `apps/painel/src/features/inbox/inbox-page.test.tsx`
- `apps/painel/src/shared/api.ts` (fn act)

### Mudar

- `apps/painel/src/shared/attention.ts` (alterar): ação `fazer-commit` com rótulo e explicação.
- `apps/painel/src/features/inbox/inbox-page.tsx` (alterar): botão "Pedir commit" no cabeçalho da tela, com o diálogo do texto do pedido.
- `apps/painel/src/features/inbox/inbox-page.test.tsx` (alterar): testes do botão.

### Passos

1. `apps/painel/src/features/inbox/inbox-page.test.tsx` (alterar): testes `mostra o botão Pedir commit` e `grava o pedido de commit com o texto` (o clique abre o diálogo; confirmar chama `POST /api/pedidos?acao=fazer-commit` com o texto). Confira: `pnpm --dir apps/painel exec vitest run src/features/inbox` falha.
2. `apps/painel/src/shared/attention.ts` (alterar): inclua `'fazer-commit'` em `ActionKind` e em `ACTION_LABELS`, com `button: 'Pedir commit'`, `tag: 'Commit'` e `how` dizendo que o agente faz o commit pela skill de commit com o aval do painel.
3. `apps/painel/src/features/inbox/inbox-page.tsx` (alterar): no cabeçalho da página (`PageHeader`), acrescente o botão secundário "Pedir commit", que abre o mesmo diálogo de texto usado pelas outras ações (`ActionBar`/`request` de `shared/action-bar.tsx`) com a ação `fazer-commit` e sem spec. Depois de gravar, mostre a mensagem do pedido gravado.
4. Confira: `pnpm --dir apps/painel exec tsc --noEmit`, `pnpm --dir apps/painel exec vitest run src/features/inbox src/shared` e `pnpm --dir apps/painel build` passam.
