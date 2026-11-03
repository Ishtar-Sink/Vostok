<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**O que foi feito.**
- `apps/painel/src/features/dashboard/dashboard-page.tsx`:
  - `RecentActivity` busca a lista com `fetchActivities({ limite })` (TASK-05), começando em 5.
  - O seletor ao lado do `h2` é um grupo de botões (`role="group"`, `aria-label="Quantidade de atividades"`, `aria-pressed` no escolhido) com 5, 10 e 15.
  - O link "Ver todas" no fim da lista leva a `/atividades`.
  - Cada item leva à página do alvo pelo `type`, mostra a data com `formatDateTime` e passa as datas ISO do detalhe por `formatDatesIn` (os dois em `api.ts`, TASK-07).
- `dashboard-page.module.css`:
  - `.activity` perdeu `max-height` e `overflow-y`.
  - `.limits` reproduz o `.filters` da página de specs (`specs-page.module.css`) em versão menor: padding de 3px e 8px, `--font-size-xs` e números tabulares.

**Decisões.**
- O usuário pediu um seletor no estilo do filtro de specs, ou nenhum se não ficasse bom; ele aprovou o resultado. O `select` nativo foi trocado no ajuste 1.
- O `overview` ainda envia `activity`, mas o painel não usa mais esse campo. Removê-lo fica para uma limpeza futura.

**Verificação.**
- Testes "mostra cinco atividades por padrão", "troca a quantidade pelo seletor" (clica em 10 e 15 e confere o `limite` pedido e o `aria-pressed`) e "leva a todas as atividades". Os testes falharam antes da mudança e agora passam 8 de 8, com o `tsc` limpo.
- O print `atividade-recente.png` foi conferido pelo usuário.

**Referências da spec.** RF-05, CT-07, CB-02, RNF-02, RNF-03.
