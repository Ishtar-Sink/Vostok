### Ler

- `vostok/work/specs/refined/SPEC-029-painel-local-do-vostok.md` (RF-03, RF-09, "Contratos e interfaces")
- `docs/estudos/painel-local/prototipo.md` (telas e retorno do usuário)
- `/home/victor/projetos/ai-hub-painel/apps/painel/src/features/specs/specs-page.tsx`
- `/home/victor/projetos/ai-hub-painel/apps/painel/src/features/specs/specs-page.module.css`
- `/home/victor/projetos/ai-hub-painel/apps/painel/src/features/spec/spec-page.tsx`
- `/home/victor/projetos/ai-hub-painel/apps/painel/src/features/spec/spec-page.module.css`
- `/home/victor/projetos/ai-hub-painel/apps/painel/src/features/board/board.tsx`
- `/home/victor/projetos/ai-hub-painel/apps/painel/src/features/board/board.module.css`
- `/home/victor/projetos/ai-hub-painel/apps/painel/src/features/board/tasks-page.tsx`
- `apps/painel/src/shared/` (api.ts, attention.ts, action-bar.tsx e edit-dialog.tsx, da TASK-03)
- `apps/painel/src/ui/` (componentes do Atria)

### Mudar

- `apps/painel/src/features/specs/specs-page.tsx` (alterar): tabela das specs abertas, arquivadas ou todas.
- `apps/painel/src/features/spec/spec-page.tsx` (alterar): spec com índice, metadados, tarefas, ações, abas de documento, quadro e plano, e o botão "Editar o arquivo".
- `apps/painel/src/features/board/tasks-page.tsx` (alterar): quadro de todas as tarefas abertas, com filtro por spec.
- `apps/painel/src/features/board/board.tsx` (alterar): quadro kanban reaproveitado pela spec e pela lista de tarefas.
- `apps/painel/src/features/specs/specs-page.module.css` (alterar): estilos da lista.
- `apps/painel/src/features/spec/spec-page.module.css` (alterar): estilos da spec.
- `apps/painel/src/features/board/board.module.css` (alterar): estilos do quadro.

### Passos

1. Escreva primeiro os testes com Vitest e Testing Library (a lista filtra abertas e arquivadas; a spec mostra o índice e as tarefas; o quadro agrupa por status; uma spec arquivada não tem "Editar" (CB-07); o "Editar o arquivo" abre o `shared/edit-dialog` com o conteúdo de `/api/fonte`). Use `fetch` simulado com os JSON do contrato da spec. Confira: `pnpm --dir apps/painel exec vitest run src/features/specs src/features/spec src/features/board` falha.
2. Troque as páginas provisórias da TASK-03 pela implementação do spike (`features/specs/specs-page.tsx`, `features/specs/specs-page.module.css`, `features/spec/spec-page.tsx`, `features/spec/spec-page.module.css`, `features/board/board.tsx`, `features/board/board.module.css`, `features/board/tasks-page.tsx`). Importe ações, diálogos e pendências só de `shared/` e componentes só de `ui/`, nunca de outra feature. Mantenha os estados de carregamento, vazio e erro e o foco visível (RNF-05).
3. Confira: `pnpm --dir apps/painel exec tsc --noEmit`, os testes e `pnpm --dir apps/painel build` passam; tire os prints e confira com o painel do spike que mostram dados reais.
