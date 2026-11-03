<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**O que foi feito.**
- **`apps/cli/src/painel/read.rs`:** nova rota `GET /api/atividades` (`fn activities`), lida dos arquivos a cada requisição, sem índice (RN-03). Ela junta cinco fontes:
  - `task_rows`: registro das tarefas abertas;
  - `archived_task_rows`: registro das tarefas arquivadas;
  - `adjustment_rows`: histórico de ajustes das specs e dos planos;
  - `request_rows`: pedidos, com a criação e o estado atual;
  - `approval_rows`: `avais.log`, sem as linhas malformadas.

  As linhas são ordenadas por `sort_newest_first`. Os filtros são `limite` (padrão `DEFAULT_ACTIVITIES = 15`), `tipo` e `spec`.
- **`apps/cli/src/painel/server.rs`:** só o teste de roteamento, porque todo `GET /api/*` já ia para `read::route`.
- **`apps/painel/src/shared/api.ts`:** o tipo `Activity` ganhou `type` (`ActivityType`: tarefa, spec, plano, pedido, aval) e `target`, e o arquivo ganhou `ActivityFilter` e `fetchActivities`.

**Decisões.**
- O campo `activity` do `overview` continua, para o painel não quebrar até a TASK-06 passar a usar a rota nova.
- Como os pedidos guardam só o último estado, cada pedido gera no máximo duas atividades. As transições ficam para o histórico de status proposto para a SPEC-033.

**Verificação.**
- Testes novos: `atividades_reunem_todas_as_fontes`, `atividades_filtram_por_tipo_spec_e_limite`, `avais_malformados_sao_ignorados`, `atividades_respondem_em_ate_500_ms` e `atividades_sao_servidas_sem_token`. Os 81 testes de `painel::` passam, e o `clippy` e o `tsc` ficam limpos.
- A verificação foi ajustada uma vez (ajuste 1): o `cargo test` aceita um filtro só, então `painel::read painel::server` virou `painel::`.

**Pontos de atenção.** O `api.ts` e o `read.rs` mudam de novo na TASK-04, da onda 3.

**Referências da spec.** RF-07, RN-03, RN-04, RNF-01, CT-09, CT-10, CB-03, CB-04; decisão 10; contrato de `GET /api/atividades`.
