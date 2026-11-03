### Ler

- `vostok/work/specs/refined/SPEC-032-controlar-o-projeto-inteiramente-pelo-painel-v0-4-0.md`
- `docs/estudos/painel-ponta-a-ponta.md` (seção "Fontes de atividade", da TASK-01)
- `apps/cli/src/painel/read.rs` (fn route, fn activity, fn activity_rows, struct ActivityRow, testes)
- `apps/cli/src/painel/server.rs` (fn dispatch e testes)
- `apps/cli/src/aval.rs` (formato do avais.log)
- `apps/cli/src/commands/pedidos.rs` (campos dos pedidos)
- `apps/painel/src/shared/api.ts` (Activity, fetch)

### Mudar

- `apps/cli/src/painel/read.rs` (alterar): fontes de atividade e rota `GET /api/atividades`
- `apps/cli/src/painel/server.rs` (alterar): teste de roteamento de `/api/atividades`
- `apps/painel/src/shared/api.ts` (alterar): tipo `Activity` com `type` e `target`, e a função `fetchActivities`

### Passos

1. `apps/cli/src/painel/read.rs` (alterar, testes): acrescente `atividades_reunem_todas_as_fontes` (fixture com registro de tarefa aberta e arquivada, histórico de ajustes de uma spec e de um plano, um pedido e uma linha no `avais.log`; `GET /api/atividades` traz os tipos tarefa, spec, plano, pedido e aval, do mais novo para o mais antigo), `atividades_filtram_por_tipo_spec_e_limite` e `avais_malformados_sao_ignorados`. Confira: falham antes da mudança.
2. `apps/cli/src/painel/read.rs` (alterar): troque `ActivityRow` por um registro com `date`, `type` (`tarefa`, `spec`, `plano`, `pedido`, `aval`), `spec`, `target` (ID da tarefa, do pedido ou da spec), `step`, `result` e `detail`; mantenha `activity_rows` para o registro das tarefas e acrescente leitores para: tarefas arquivadas em `<archive>/<SPEC>/tasks/`, a seção "Histórico de ajustes" das specs e dos planos, os pedidos de `work/pedidos/` (criação por `criado_em` e estado atual por `atualizado_em`) e o `avais.log` (ignorando linha com menos de 4 colunas). Confira: `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::read`.
3. `apps/cli/src/painel/read.rs` (alterar): em `fn route`, acrescente `["api", "atividades"]`, lendo `limite` (padrão 15), `tipo` e `spec` de `query_value`. O `overview` continua com o campo `activity` (as 20 mais novas), para não quebrar o painel antes das TASK-06 e TASK-07. Confira: os três testes do passo 1 passam.
4. `apps/cli/src/painel/server.rs` (alterar): acrescente o teste `atividades_sao_servidas_sem_token`, que faz `dispatch` de `GET /api/atividades?limite=5` e espera 200. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::server`.
5. `apps/cli/src/painel/read.rs` (alterar, teste): acrescente `atividades_respondem_em_ate_500_ms` com uma fixture de 1.000 linhas de registro, medindo `GET /api/atividades?limite=15` com `Instant` (RNF-01). Confira: passa.
6. `apps/painel/src/shared/api.ts` (alterar): `Activity` ganha `type` e `target`; acrescente `fetchActivities({ limite, tipo, spec })`, que chama `/api/atividades` com a query. Confira: `pnpm --dir apps/painel exec tsc --noEmit`.
7. Se o estudo da TASK-01 apontou alguma fonte como cara demais, **PARE** e pergunte ao usuário antes do passo 2; tirar uma fonte é um ajuste desta tarefa.
