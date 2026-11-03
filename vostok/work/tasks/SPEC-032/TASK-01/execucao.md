### Ler

- `vostok/work/specs/refined/SPEC-032-controlar-o-projeto-inteiramente-pelo-painel-v0-4-0.md`
- `vostok/work/specs/draft/SPEC-033-operar-o-projeto-de-ponta-a-ponta-pelo-painel-v0-4-0.md`
- `docs/estudos/painel-local.md` (estudo da SPEC-027, formato e método)
- `apps/cli/src/painel/read.rs` (fn activity, fn activity_rows, fn questions, fn overview)
- `apps/cli/src/painel/actions.rs` (REQUEST_ACTIONS, LOCKED_ACTIONS, fn request)
- `apps/cli/src/commands/pedidos.rs`
- `apps/cli/src/aval.rs`
- `packages/harness/skills/default/process-requests/SKILL.md`
- `packages/harness/context/skill-catalog.md`
- `apps/painel/src/shared/attention.ts`

### Mudar

- `docs/estudos/painel-ponta-a-ponta.md` (criar): o estudo preliminar da v0.4.0

### Passos

1. `docs/estudos/painel-ponta-a-ponta.md` (criar): abra com "Pergunta", "Resposta curta" e "Como foi feito", no formato de `docs/estudos/painel-local.md`. Confira: o arquivo existe com os três títulos.
2. `docs/estudos/painel-ponta-a-ponta.md` (alterar): seção "Técnicas para operar sem o chat": compare o pedido em loop (caminho preferido; `process-requests` e `vostok pedidos vigiar`) com um chat dentro do painel e com qualquer outra técnica que achar, com custo de implementação, dependência de o agente estar rodando, latência de resposta e o que cada uma exige de mudança no `vostok` e nas skills. Confira: a tabela de comparação tem as três colunas de critério e a recomendação.
3. `docs/estudos/painel-ponta-a-ponta.md` (alterar): seção "Processos a operar pelo painel": uma linha por skill de `packages/harness/context/skill-catalog.md`, sem as do harness (report-harness-bug, suggest-harness-*, validate-harness-bug, create-skill) e as de configuração inicial (init-harness, adjust-rules, customize-documents), com o que o painel já faz hoje (ação em `REQUEST_ACTIONS` ou aprovação direta) e o que falta. Confira: cada skill restante tem uma linha.
4. `docs/estudos/painel-ponta-a-ponta.md` (alterar): seção "Status das entidades": para spec, plano, tarefa, pedido e pergunta, onde o status é gravado hoje, quais mudanças não são gravadas e em qual fluxo existente posicioná-las (sem mecanismo novo). Confira: as cinco entidades estão na tabela.
5. `docs/estudos/painel-ponta-a-ponta.md` (alterar): seção "Perguntas do agente": hoje o painel lê `work/perguntas/` e tem `responder-pergunta`, mas nada grava perguntas; proponha o comando da CLI e como as skills de conversa (create-spec, refine-spec, adjust-*) gravam a pergunta e esperam a resposta quando atendem um pedido. Confira: a seção cita o comando proposto e as skills.
6. `docs/estudos/painel-ponta-a-ponta.md` (alterar): seção "Arquivos de comunicação": meça quantos arquivos o `vostok pedidos listar` e o painel leem hoje com 10, 100 e 1.000 pedidos (gere uma pasta descartável no scratchpad, nunca em `vostok/work/`), compare com pastas por status e com um índice, e trate o crescimento do `avais.log`. Confira: a tabela tem as medições dos três tamanhos.
7. `docs/estudos/painel-ponta-a-ponta.md` (alterar): seção "Fontes de atividade": para registro das tarefas (abertas e arquivadas), histórico de ajustes das specs e dos planos, pedidos e `avais.log`, meça o tempo de leitura com 1.000 entradas e diga se cabe na rota `GET /api/atividades` em até 500 ms (RNF-01). Confira: cada fonte tem tempo medido e uma recomendação.
8. `docs/estudos/painel-ponta-a-ponta.md` (alterar): seção "Proposta da SPEC-033": escopo recomendado, ordem e o que muda no rascunho da SPEC-033, e se alguma tarefa desta spec (TASK-05 em especial) precisa de ajuste antes de começar. Confira: a seção existe e cita a TASK-05.
