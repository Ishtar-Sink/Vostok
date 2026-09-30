# Cenários com agente

Roteiro dos cenários que exercitam as skills `adjust-spec`, `approve-spec`, `plan-tasks`, `adjust-plan` e `approve-plan` com um agente de verdade, e os resultados da última execução. Os scripts são cobertos por `tests/run-tests.sh`; aqui fica o que depende do agente: identificar a spec, decompor o pedido, perguntar ou aplicar.

## Como executar

1. Monte as cópias descartáveis:

   ```bash
   packages/harness/tests/fixtures/agent-scenarios.sh <pasta>
   ```

   Cada cenário recebe uma cópia do projeto com as specs de exemplo:

   | Spec | Título | Status |
   |---|---|---|
   | SPEC-001 | Exportar memória em JSON | refinada, com RF-01 a RF-03, RNF-01, RNF-02 (`jq`), RN-01 (obsoletas não saem), CB-01 e CB-02 |
   | SPEC-002 | Listar tarefas | refinada |
   | SPEC-003 | Painel de métricas | rascunho |

   Os cenários `ct04`, `ct09` e `ap02` não têm a SPEC-002; o `ct05` e o `ap04` só têm a SPEC-003; o `ap13` parte da SPEC-001 já aprovada. Os cenários de planejamento (`pl`) partem de specs aprovadas, com ou sem plano; veja a tabela deles.

2. Para cada cenário, inicie um agente numa sessão nova com a pasta do cenário como raiz do projeto. Se houver, passe o contexto da conversa e depois a mensagem do usuário. O agente não recebe resposta do usuário: quando o harness manda parar e aguardar, a mensagem dele é o resultado.

3. Confira a mensagem final e os arquivos: quais specs mudaram (compare com `cksum`), o "Histórico de ajustes" e se `adjust-spec.sh --validate` passa.

## Cenários de ajuste

| Cenário | Caso de teste | Contexto e mensagem do usuário | Resultado esperado |
|---|---|---|---|
| ct01 | CT-01 | Contexto: o agente acabou de refinar a SPEC-002 nesta conversa. Mensagem: "Ficou bom. Só acrescenta que o script também deve aceitar um filtro opcional por status." | Ajusta a SPEC-002 sem perguntar qual spec. |
| ct02 | CT-02 | "Na SPEC-001, o caminho do arquivo de saída deve vir de config/memory.yml em vez de ser fixo no código." | Ajusta a SPEC-001. |
| ct03 | CT-03 | "Acrescenta na spec um caso de borda para quando o arquivo de saída já existir." | Pergunta qual spec, com as duas candidatas, sem editar. |
| ct04 | CT-04 | "Remove o RNF-02 da spec, não quero depender de nenhuma ferramenta específica para montar o JSON." | Ajusta a única refinada sem perguntar. |
| ct05 | CT-05 | "Ajusta a spec refinada: coloca um requisito de log das operações." | Informa que não há spec refinada e para. |
| ct06 | CT-06 | "Na SPEC-003, ajusta para ter um requisito de exportar o painel em PDF." | Informa que só specs refinadas são ajustadas. |
| ct09 | CT-09 | "Na spec do export, acrescenta que o JSON precisa trazer todas as entradas da memória, inclusive as obsoletas, pro painel mostrar o histórico completo." | 1 rodada de perguntas sobre o conflito com a RN-01, sem editar. |
| ct13 | CT-13 | "Revisei a SPEC-001. Quero que o tipo all deixe de existir, porque o painel vai pedir um tipo por vez, e o arquivo de saída passa a se chamar memory-export.json, na mesma pasta. Ah, e acho que falta um critério de aceite garantindo que o JSON gerado é válido." | Três ajustes aplicados e listados; uma linha no histórico. |
| ct14 | CT-14 | "Ajustes na SPEC-001: 1. Remova o CB-02. 2. O RF-01 deve exportar também a data de criação da entrada. 3. Acrescente um risco: arquivo grande demais para o painel carregar." | Três ajustes aplicados e listados; uma linha no histórico. |
| ct15 | CT-15 | "SPEC-001, na seção Requisitos funcionais, RF-03: onde está \"`memory/export.json`\", coloque \"`work/exports/memory.json`\"." | Só o trecho citado muda. |
| ct16 | CT-16 | "Na SPEC-001: 1. O RF-02 deve aceitar só o tipo decision. 2. O RF-02 deve aceitar todos os tipos menos issue. 3. Remova o RF-07." | 1 rodada de perguntas com a contradição e o RF-07 inexistente, sem editar. |

## Resultados dos cenários de ajuste

Execução de 2026-09-27, com Claude Code (Opus 5.5) como agente.

| Cenário | Resultado | Arquivos alterados | Observação |
|---|---|---|---|
| ct01 | passou | refinada da SPEC-002 | Novo RF-03 com `--status`, CB-02, CT-02, CT-03 e CA-02. Duas inferências registradas. |
| ct02 | passou | refinada da SPEC-001 | RF-03, fluxo, FA-02, CB-03, contrato, componentes, CT-03, CT-04 e CA-03 atualizados. |
| ct03 | passou | nenhum | Perguntou entre SPEC-001 e SPEC-002 com o template `question`. |
| ct04 | passou | refinada da SPEC-001 | Removeu RNF-02 e, por inferência, a premissa e o risco ligados ao `jq`. |
| ct05 | passou | nenhum | Informou que não há spec refinada. |
| ct06 | passou | nenhum | Informou que a SPEC-003 está em `draft`. |
| ct09 | passou | nenhum | Perguntou sobre o conflito com a RN-01 e notou que o CA-01 também seria afetado. |
| ct13 | passou | refinada da SPEC-001 | Três ajustes listados; RF-02, CT-02 e CA-02 removidos; novo CA-03. Uma linha no histórico. |
| ct14 | passou | refinada da SPEC-001 | Três ajustes listados; CT-01 atualizado junto com o RF-01. Uma linha no histórico. |
| ct15 | passou, com ressalva | refinada da SPEC-001 | Trocou o RF-03 e também o passo 3 do fluxo, que citava o mesmo caminho, registrando isso como inferência ([adjust-spec-06]). |
| ct16 | passou | nenhum | Uma rodada com as duas perguntas: RF-02 contraditório e RF-07 inexistente. |

Em todos os cenários que ajustaram a spec, `--record-manual` rodou antes da edição, `--validate` passou, o histórico ganhou uma única linha `agente` e nenhum rascunho foi alterado.

## Cenários de aprovação

Os casos de teste citados são os da spec de aprovação (skill `approve-spec`).

| Cenário | Caso de teste | Mensagem do usuário | Resultado esperado |
|---|---|---|---|
| ap01 | CT-01 | "Aprovo a SPEC-001." | Aprova a SPEC-001: status `approved`, `approved_hash` igual ao `content_hash` e rascunho removido. |
| ap02 | CT-02 | "Pode aprovar a spec." | Aprova a única refinada sem perguntar. |
| ap03 | CT-03 | "Pode aprovar a spec." | Pergunta qual spec, com as duas candidatas, sem aprovar. |
| ap04 | CT-04 | "Aprova a spec refinada." | Informa que não há spec refinada e para. |
| ap13 | CT-13 | "Na SPEC-001, o RF-03 deve gravar a saída em `work/exports/memory.json`." | Ajusta a spec aprovada, informa que a aprovação caiu e acrescenta uma linha a "O que mudou". |

## Resultados dos cenários de aprovação

Execução de 2026-09-27, com Claude Code (Opus 5.5) como agente.

| Cenário | Resultado | Arquivos alterados | Observação |
|---|---|---|---|
| ap01 | passou | refinada e rascunho da SPEC-001 | `--validate` e `--approve`; `approved_hash` igual ao `content_hash`; rascunho removido. |
| ap02 | passou | refinada e rascunho da SPEC-001 | Identificou a única refinada sem perguntar. |
| ap03 | passou | nenhum | Perguntou entre SPEC-001 e SPEC-002 com o template `question`. |
| ap04 | passou | nenhum | Informou que só há a SPEC-003, em `draft`. |
| ap13 | passou | refinada da SPEC-001 | Status voltou a `refined`, `approved_at` e `approved_hash` `null`; a resposta avisou que a aprovação caiu; "O que mudou" ganhou a linha 2. |

## Cenários de planejamento

Os casos de teste citados são os da spec de granularização (skills `plan-tasks`, `adjust-plan` e `approve-plan`). O plano da fixture (`plan` em `fixtures/agent-scenarios.sh`) tem três tarefas para a SPEC-001: TASK-01 exporta um tipo, TASK-02 exporta todos e TASK-03 reúne os testes.

| Cenário | Caso de teste | Specs e plano | Mensagem do usuário | Resultado esperado |
|---|---|---|---|---|
| pl01 | CT-01 | SPEC-001 aprovada, sem plano | "Planeja as tarefas da SPEC-001." | Plano criado, preenchido, pontuado e válido; resposta com a visão geral, as tarefas e os níveis. |
| pl03 | CT-03 | SPEC-001 e SPEC-002 aprovadas, sem plano | "Quebra em tarefas a spec aprovada." | Pergunta qual spec, com as duas candidatas, sem criar nada. |
| pl06 | CT-06 | SPEC-001 aprovada, com plano | "No plano da SPEC-001, divida a TASK-01 em duas: uma para a exportação de um tipo e outra para a gravação em memory/export.json. E remova a TASK-03, passando cada caso de teste para a tarefa que ele testa." | Ajusta o plano ou pergunta pelo que ficou ambíguo, sem editar antes da resposta. |
| pl06b | CT-06 | SPEC-001 aprovada, com plano | "No plano da SPEC-001, divida a TASK-01 em duas: uma para a exportação de um tipo (RF-01) e outra para a gravação em memory/export.json (RF-03). E remova a TASK-03: o CT-01 passa para a tarefa de gravação e o CT-02 para a tarefa do tipo all." | Dois ajustes aplicados, níveis recalculados e uma linha `agente` no histórico do plano. |
| pl07 | CT-07, CT-18 | SPEC-001 aprovada, com plano | "Aprovo o plano de tarefas da SPEC-001." | Tarefas criadas e detalhadas, plano removido e spec `planned`; tarefas prontas para o board. |
| pl11 | CT-11 | SPEC-001 planejada | "Na SPEC-001, acrescenta um RF para exportar também em CSV." | Recusa, informando que mudar a spec exige um novo ciclo. |
| pl13a | CT-13 | SPEC-001 com plano e SPEC-002 sem plano, ambas aprovadas | "Planeja a SPEC-002." | Planeja a SPEC-002 sem tocar no plano da SPEC-001. |
| pl13b | CT-13 | Como o pl13a | "Planeja a SPEC-001." | Informa que o plano existe e não o altera. |
| pl13c | CB-14 | Como o pl13a | "Planeja a spec aprovada." | Planeja a SPEC-002, a única sem plano, sem perguntar. |

Confira também se `plan-tasks.sh --validate` (ou `approve-plan.sh --validate`) passa e se `adjust-plan.sh --record-manual` responde "Nenhuma edição manual.".

## Resultados dos cenários de planejamento

Execução de 2026-09-28, com Claude Code (Opus 5.5) como agente.

| Cenário | Resultado | Arquivos alterados | Observação |
|---|---|---|---|
| pl01 | passou | plano da SPEC-001 | Duas tarefas (moderada e simples), cobrindo RF-01 a RF-03, CT-01 e CT-02; `--validate` passou. |
| pl03 | passou | nenhum | Perguntou entre SPEC-001 e SPEC-002 com o template `question`. |
| pl06 | passou, com pergunta | nenhum | Perguntou para qual tarefa vai o CT-01, que testa as duas metades da TASK-01 dividida; não editou antes da resposta ([adjust-plan-05]). A mensagem era ambígua; o pl06b repete o pedido sem a ambiguidade. |
| pl06b | passou | plano da SPEC-001 | Dois ajustes listados; níveis recalculados; uma linha `agente` no histórico; dependências renumeradas registradas como inferência. |
| pl07 | falhou (bug no script) | tarefas da SPEC-001 | O `--validate` recusava `alterar` num arquivo criado pela TASK-01, de que a TASK-02 depende. O agente parou com o template `blocker` sem trocar a ação. Corrigido: caminhos criados por tarefas de que a tarefa depende contam como existentes. |
| pl07 (repetição) | passou | spec, plano e tarefas da SPEC-001 | Três tarefas detalhadas, plano removido e spec `planned`. Na TASK-02, resumo executivo em linguagem de negócio, detalhes técnicos com RF-02 e RN-01, card completo, `type` e `labels` no front-matter (CT-18). `load-task.sh` não imprime a "Descrição". |
| pl11 | passou | nenhum | Recusou e informou que mudar a spec `planned` exige um novo ciclo. |
| pl13a | passou | plano da SPEC-002 | Plano da SPEC-001 intacto. |
| pl13b | passou | nenhum | Informou que o plano da SPEC-001 existe e está atualizado, sem alterá-lo. |
| pl13c | passou | plano da SPEC-002 | Escolheu a única spec aprovada sem plano, sem perguntar. |

Em todos os cenários, os planos e as tarefas criados passaram na validação, `adjust-plan.sh --record-manual` respondeu "Nenhuma edição manual." e nenhum arquivo fora do esperado mudou (conferido com `cksum`).
