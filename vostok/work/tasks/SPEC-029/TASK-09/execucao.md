### Ler

- `vostok/work/specs/refined/SPEC-029-painel-local-do-vostok.md` (RF-03, RF-09, "Contratos e interfaces")
- `docs/estudos/painel-local/prototipo.md` (telas e retorno do usuário)
- `/home/victor/projetos/ai-hub-painel/apps/painel/src/features/task/task-page.tsx`
- `/home/victor/projetos/ai-hub-painel/apps/painel/src/features/task/task-page.module.css`
- `/home/victor/projetos/ai-hub-painel/apps/painel/src/features/documents/documents-page.tsx`
- `/home/victor/projetos/ai-hub-painel/apps/painel/src/features/documents/documents-page.module.css`
- `apps/painel/src/shared/` (api.ts, attention.ts, action-bar.tsx e edit-dialog.tsx, da TASK-03)
- `apps/painel/src/ui/` (componentes do Atria)

### Mudar

- `apps/painel/src/features/task/task-page.tsx` (alterar): tarefa com executivo, técnico, critérios e evidências primeiro, a parte do agente abaixo e "Editar" em cada parte menos o registro.
- `apps/painel/src/features/documents/documents-page.tsx` (alterar): executivo, técnico e evidências das specs arquivadas.
- `apps/painel/src/features/task/task-page.module.css` (alterar): estilos da tarefa.
- `apps/painel/src/features/documents/documents-page.module.css` (alterar): estilos dos documentos.

### Passos

1. Escreva primeiro os testes com Vitest e Testing Library (a tarefa mostra as partes na ordem do protótipo e as evidências com imagem; "Editar" não aparece no registro nem numa tarefa `done`; os documentos listam as specs arquivadas com os três documentos). Use `fetch` simulado com os JSON do contrato da spec. Confira: `pnpm --dir apps/painel exec vitest run src/features/task src/features/documents` falha.
2. Troque as páginas provisórias da TASK-03 pela implementação do spike (`features/task/task-page.tsx`, `features/task/task-page.module.css`, `features/documents/documents-page.tsx`, `features/documents/documents-page.module.css`). Importe ações, diálogos e pendências só de `shared/` e componentes só de `ui/`, nunca de outra feature. Mantenha os estados de carregamento, vazio e erro e o foco visível (RNF-05).
3. Confira: `pnpm --dir apps/painel exec tsc --noEmit`, os testes e `pnpm --dir apps/painel build` passam; tire os prints e confira com o painel do spike que mostram dados reais.
