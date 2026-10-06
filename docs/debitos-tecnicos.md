# Débitos técnicos

Problemas conhecidos que ficaram para depois: lacunas de comportamento, de cobertura de testes ou de clareza. Cada débito tem um ID fixo (`DT-NNN`), é registrado quando é encontrado e só sai da lista de abertos quando for resolvido ou descartado, com o motivo.

## Resumo

| ID | Título | Área | Prioridade | Status |
|---|---|---|---|---|
| DT-001 | Tarefa em execução não pode ser ajustada | harness: fluxo da tarefa | alta | aberto |
| DT-002 | Retomada e regressão sem cenário com agente | harness: testes com agente | média | aberto |
| DT-003 | Hash da tarefa diverge do hash aprovado após o início | harness: fluxo da tarefa | baixa | aberto |

Prioridade: **alta** bloqueia um fluxo real; **média** deixa um comportamento sem prova; **baixa** confunde, mas não quebra nada.

## DT-001: Tarefa em execução não pode ser ajustada

- **Área:** harness, skill `adjust-task`
- **Encontrado em:** 2026-10-02, SPEC-002 TASK-08, primeira execução do cenário `tk09`
- **Prioridade:** alta
- **Status:** aberto

**Descrição.** A `adjust-task` aceita só tarefas `pending` ou `approved`. Quando a execução revela que a tarefa está errada (passos, verificação ou arquivos que não batem com a spec), não há caminho para corrigi-la: ela está `in_progress` e só sai desse estado para `validating`, `blocked` ou `done`.

**Impacto.** O agente fica sem saída válida. Ele precisa parar e perguntar, e o usuário também não tem um comando para devolver a tarefa a um estado ajustável. A alternativa seria editar o arquivo à mão, fora do fluxo.

**Proposta.** Definir como uma tarefa em execução volta a ser ajustável. Uma opção é a `adjust-task` aceitar `in_progress` e `blocked`, derrubar a aprovação, descartar o baseline e as evidências e registrar o ajuste. Outra é uma etapa explícita de "devolver para ajuste". Exige decisão do usuário antes de implementar.

**Ocorrências.** 2026-10-06, SPEC-003 TASK-05: o usuário mudou o foco e o visual do documento executivo com a tarefa já em validação. O ajuste foi registrado à mão no histórico de ajustes da tarefa, com os arquivos novos em "Mudar" e o critério novo em "Concluída quando".

**Referências.** `packages/harness/skills/default/adjust-task/`, `packages/harness/tests/agent-scenarios.md` (resultado do `tk09`).

## DT-002: Retomada e regressão sem cenário com agente

- **Área:** harness, `tests/agent-scenarios.md`
- **Encontrado em:** 2026-10-02, SPEC-002 TASK-08
- **Prioridade:** média
- **Status:** aberto

**Descrição.** O CA-05 da SPEC-002 pede que os cenários tristes com agente parem na falha de testes e na regressão e retomem com `--resume`. Os cenários com agente cobrem só a parada por falha de testes (`tk15`). A parada por regressão e a retomada de uma tarefa `blocked` com a decisão do usuário têm cobertura apenas na suíte (`tests/run-tests.sh`).

**Impacto.** Não há prova de que o agente reage bem à regressão, nem de que conduz a retomada como o fluxo manda: registrar a decisão e voltar a validar.

**Proposta.** Acrescentar à fixture um cenário com regressão (baseline que muda depois da tarefa) e um cenário de retomada, que parte de uma tarefa `blocked` e tem como mensagem do usuário uma das opções do `blocker`.

**Referências.** `packages/harness/tests/fixtures/agent-scenarios.sh`, `packages/harness/skills/default/validate-task/`.

## DT-003: Hash da tarefa diverge do hash aprovado após o início

- **Área:** harness, skills `execute-task` e `validate-task`
- **Encontrado em:** 2026-10-02, SPEC-002 TASK-08, cenários `tk09` e `tk15`
- **Prioridade:** baixa
- **Status:** aberto

**Descrição.** O `--start` compara o `approved_hash` com o conteúdo antes de iniciar. Depois, cada etapa grava uma linha no "Registro de execução" e atualiza o `content_hash`, que deixa de bater com o `approved_hash`. O comportamento está correto, mas dois agentes estranharam a diferença e a citaram como possível falha.

**Impacto.** Gera dúvida no agente e no usuário, e pode levar alguém a "consertar" algo que não está quebrado.

**Proposta.** Explicar a diferença na `SKILL.md` da `execute-task` (e no front-matter, se fizer sentido), ou excluir o "Registro de execução" do cálculo do hash.

**Referências.** `packages/harness/skills/default/execute-task/scripts/execute-task.sh`, `packages/harness/scripts/lib/task.sh` (`task_record_step`).

## Como registrar um débito

1. Use o próximo ID livre (`DT-NNN`) e acrescente uma linha ao resumo.
2. Crie a seção com área, origem (data e onde foi encontrado), prioridade, status, descrição, impacto, proposta e referências.
3. Ao resolver, mude o status para `resolvido` e cite o commit ou a tarefa; ao descartar, mude para `descartado` e diga o motivo. Não apague a seção.
