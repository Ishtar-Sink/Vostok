<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**O que foi feito.** Em `apps/cli/src/commands/task.rs`:
- o `vostok task check` grava `check_result` (`passed` quando o código de saída é 0, senão `failed`) e `checked_at` (`utc_datetime`) pelo `TaskDoc::put`, e uma linha `check` no registro pelo `record_step`. Essa linha também alimenta a atividade da TASK-05;
- uma função nova, `clear_check_result`, devolve `check_result` a `null`. Ela é chamada no `reset_results`, que cobre o ajuste em execução e a retomada sem aceite, e na retomada com `--accept-regression`, que não passa pelo `reset_results`.

**Decisões.**
- O `clear_check_result` só mexe nas tarefas que já têm o campo. Gravar `check_result: null` nas tarefas antigas mudaria o estado delas e quebraria o teste de referência `golden_task`.
- O teste antigo, que garantia que o `check` não tocava na tarefa, virou `check_records_the_result`, porque o comportamento mudou de propósito (RF-03); o baseline o exclui.

**Testes.** Em `apps/cli/tests/task.rs`: `check_records_the_result`, `adjustment_clears_the_check_result`, `resume_clears_the_check_result` e `resume_accepting_the_regression_clears_the_check_result`. Os 210 testes de `--test task` passam, sem regressão contra o baseline. Os testes foram escritos junto com o código; não foi registrado que falhavam antes da mudança.

**Pontos de atenção.** Gravar no front-matter pelo `TaskDoc` não conta como edição manual, como já acontece com o `tests_result`.

**Referências da spec.** RF-03, RN-02, CT-03, CT-04, CT-05, CB-01; contrato do `estado.yml`.
