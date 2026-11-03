<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**O que foi feito.**
- `apps/cli/src/painel/read.rs`: `check_result` (gravado pelo `vostok task check`, TASK-03) entra no resumo de cada tarefa do `GET /api/visao`.
- `apps/painel/src/shared/api.ts`: `TaskSummary` ganha `check_result?: string`.
- `apps/painel/src/shared/attention.ts`: em `attentionItems`, a pendência `finish-task` exige `status === 'in_progress'`, `check_result === 'passed'` e as dependências concluídas. `taskActions` não mudou, então a tela da tarefa segue com "Pedir finalização".

**Decisões.** Sem conferência (tarefas antigas ou recém-iniciadas) e com `failed`, a tarefa não é pendência: uma pendência é só o que trava o agente e precisa da decisão do usuário (RN-02).

**Verificação.** Teste novo em `attention.test.ts`, "só pede finalização com a conferência passada", com os três casos (sem conferência, `passed`, `failed`). Ele falhou antes da mudança. Os testes existentes de `taskActions` continuam passando, e a regressão só acrescenta esse teste.

**Pontos de atenção.** O campo novo em `read.rs` não tem um teste Rust próprio; quem o cobre é o teste do painel.

**Referências da spec.** RF-04, RN-02, CT-06, CB-01.
