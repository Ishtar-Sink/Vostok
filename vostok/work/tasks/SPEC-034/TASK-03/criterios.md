### Concluída quando

- [x] O ajuste que derruba a aprovação de uma spec, pelo agente ou manual, grava a volta para refined no histórico de status, depois da linha da aprovação (RF-06, CT-08).

  > Como comprovar: em `historico-ajuste.txt`, os testes `adjust_history_records_the_return_to_refined` e `adjust_history_manual_adjust_records_the_return_to_refined` aparecem com `ok`: o histórico termina com a entrada `refined`.

- [x] O ajuste de uma spec só refinada não acrescenta linha ao histórico de status (CB-04).

  > Como comprovar: em `historico-ajuste.txt`, o teste `adjust_history_unchanged_for_a_refined_spec` aparece com `ok`.

- [x] A mensagem de validação da CLI para uma aprovação do painel sem aval cita o log de avais da spec, `work/avais/<SPEC>.log`, e nenhuma mensagem da CLI cita `avais.log` como destino (RF-07, CT-09).

  > Como comprovar: em `mensagem-aval.txt`, os testes `panel_without_aval_cites_the_spec_log` e `panel_approval_without_aval_is_reported` aparecem com `ok`; a busca por `PANEL_WITHOUT_AVAL` em `apps/cli/src` não encontra nada.

- [x] A aprovação, a validação das specs e das tarefas e a trava de aval continuam funcionando como antes.

  > Como comprovar: as suítes `spec`, `aval` e `task` passam sem regressão contra o baseline, e o clippy não mostra aviso.
