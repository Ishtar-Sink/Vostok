<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**O que foi feito.**
- **Página.**
  - `apps/painel/src/features/activity/` (novos): `activity-page.tsx`, `activity-page.module.css` e `activity-page.test.tsx`. A página tem o título "Atividades" e dois `SelectField`: "Tipo" e "Spec", este com as specs que já apareceram. A lista vem de `fetchActivities({ tipo, spec, limite: 1000 })` e mostra `EmptyState` quando está vazia.
  - `router.ts` ganhou a rota `/atividades` (`{ page: 'activity' }`), `sidebar.tsx` o item "Atividades" com um ícone novo, `topbar.tsx` a trilha e `main.tsx` o render da página.
- **Hora no registro (ajuste 1).**
  - `apps/cli/src/commands/task.rs`: `record_step` grava `utc_now()` inteiro na coluna Data (`2026-11-03T16:27:56Z`), em vez de só os 10 primeiros caracteres.
  - `apps/cli/src/commands/finalize.rs`: `task_done_date` lê só os 10 primeiros caracteres, então os documentos finais continuam com a data.
  - `apps/cli/tests/common/golden.rs`: `normalize` trata a hora do registro como volátil (`| <DATETIME> |` vira `| <DATE> |`), porque as fixtures dos scripts bash só têm a data.
  - A ordenação de `GET /api/atividades` (TASK-05) não mudou: ela compara as datas como texto, e uma data ISO com hora fica depois da mesma data sem hora.
- **Datas formatadas.** `apps/painel/src/shared/api.ts` ganhou:
  - `formatDateTime`: no fuso local, mostra "hoje, HH:MM", "ontem, HH:MM" ou "DD/MM/AAAA, HH:MM", e só o dia quando o valor não tem hora;
  - `formatDatesIn`: formata as datas ISO dentro de um texto, como o detalhe "aprovada em ...".

**Decisões.**
- A hora vai em UTC, no formato ISO, como os outros campos de data do harness. O painel converte para o fuso local.
- O histórico de ajustes continua só com a data. Na mesma data, as linhas dele aparecem abaixo das linhas do registro que têm hora.

**Verificação.**
- Testes:
  - Rust: `record_has_the_time`, que falhou antes da mudança;
  - painel: `formatDateTime` e `formatDatesIn`, os quatro da página e o da rota.
- Também passaram a suíte completa da CLI, inclusive `golden_task` e `finalize`, os testes do `painel::` e o `clippy -D warnings`.
- A saída de `record_has_the_time` fica em `validation/tests.out`, não em `atividades-pagina-testes.txt`, que só guarda a saída do vitest.
- O print `atividades.png` foi tirado com a CLI nova instalada e conferido pelo usuário.

**Pontos de atenção.**
- A hora só aparece em linhas gravadas pela CLI nova; quem usa a 0.3.0 continua gravando só a data.
- O formato da coluna Data do `registro.md` mudou: leitores que esperam exatamente `AAAA-MM-DD` precisam cortar os 10 primeiros caracteres.

**Referências da spec.** RF-06, CT-08, CB-05, RNF-03.
