### Ler

- `vostok/work/specs/refined/SPEC-029-painel-local-do-vostok.md` (RF-03, RF-06, RF-11, "Contratos e interfaces")
- `docs/estudos/painel-local/prototipo.md` (telas e retorno do usuário)
- `/home/victor/projetos/ai-hub-painel/apps/painel/src/features/dashboard/dashboard-page.tsx`
- `/home/victor/projetos/ai-hub-painel/apps/painel/src/features/dashboard/dashboard-page.module.css`
- `/home/victor/projetos/ai-hub-painel/apps/painel/src/features/inbox/inbox-page.tsx`
- `/home/victor/projetos/ai-hub-painel/apps/painel/src/features/inbox/inbox.tsx`
- `/home/victor/projetos/ai-hub-painel/apps/painel/src/features/inbox/inbox.module.css`
- `apps/painel/src/shared/` (api.ts, attention.ts, action-bar.tsx e edit-dialog.tsx, da TASK-03)
- `apps/painel/src/ui/` (componentes do Atria)

### Mudar

- `apps/painel/src/features/dashboard/dashboard-page.tsx` (alterar): indicadores, "Aguardando você", specs em andamento com barra por status e ondas, atividade recente.
- `apps/painel/src/features/inbox/inbox-page.tsx` (alterar): pendências: aprovações, finalizações, bloqueios e perguntas, com as ações.
- `apps/painel/src/features/dashboard/dashboard-page.module.css` (alterar): estilos do painel.
- `apps/painel/src/features/inbox/inbox-page.module.css` (alterar): estilos das pendências.

### Passos

1. Escreva primeiro os testes com Vitest e Testing Library (o painel mostra os indicadores e as specs em andamento; as pendências listam uma tarefa `pending` com "Aprovar tarefa" e uma pergunta aberta com "Responder"; o clique em "Aprovar tarefa" chama `POST /api/aprovacoes` pelo `shared/action-bar`). Use `fetch` simulado com os JSON do contrato da spec. Confira: `pnpm --dir apps/painel exec vitest run src/features/dashboard src/features/inbox` falha.
2. Troque as páginas provisórias da TASK-03 pela implementação do spike (`features/dashboard/dashboard-page.tsx`, `features/dashboard/dashboard-page.module.css`, `features/inbox/inbox-page.tsx`, `features/inbox/inbox.tsx`, `features/inbox/inbox.module.css`). Importe ações, diálogos e pendências só de `shared/` e componentes só de `ui/`, nunca de outra feature. Mantenha os estados de carregamento, vazio e erro e o foco visível (RNF-05).
3. Confira: `pnpm --dir apps/painel exec tsc --noEmit`, os testes e `pnpm --dir apps/painel build` passam; tire os prints e confira com o painel do spike que mostram dados reais.
