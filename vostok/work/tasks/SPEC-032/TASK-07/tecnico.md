<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Página nova em `features/activity/`, no padrão de `documents-page` (feature com componente, CSS module e teste juntos, [project/architecture-04]), consumindo `fetchActivities` da TASK-05. Rota, trilha, render e item da sidebar seguem os das outras páginas.

**Decisões.**
- Filtros como `select` com rótulo, acessíveis por teclado (RNF-03), em vez de chips.
- A página pede até 1.000 atividades de uma vez, sem paginação; paginação fica para quando o volume pedir, e o estudo da TASK-01 mede.

**Riscos.** Mudanças em `main.tsx`, `topbar.tsx` e `sidebar.tsx`, que outras telas usam; o `tsc` e os testes de `src/layout` conferem.

**Referências da spec.** RF-06, RNF-03, CT-08, CB-05.

**Arquivos em comum com a onda 3.** Nenhum.
