# Cenários com agente

Roteiro dos cenários que exercitam as skills `adjust-spec`, `approve-spec`, `plan-tasks`, `adjust-plan`, `approve-plan`, `approve-task`, `adjust-task`, `execute-task`, `finish-task` e `finalize-spec` com um agente de verdade, e os resultados da última execução. Os scripts são cobertos por `tests/run-tests.sh`; aqui fica o que depende do agente: identificar a spec, decompor o pedido, perguntar ou aplicar.

## Como executar

1. Monte as cópias descartáveis:

   ```bash
   packages/harness/tests/fixtures/agent-scenarios.sh <pasta>
   ```

   Cada cenário recebe uma cópia do projeto com as specs de exemplo:

   | Spec | Título | Status |
   |---|---|---|
   | SPEC-001 | Exportar memória em JSON | refinada, com RF-01 a RF-03, RNF-01, RNF-02 (JSON montado com awk), RN-01 (obsoletas não saem), CB-01 e CB-02 |
   | SPEC-002 | Listar tarefas | refinada |
   | SPEC-003 | Painel de métricas | rascunho |

   Os cenários `ct04`, `ct09` e `ap02` não têm a SPEC-002; o `ct05` e o `ap04` só têm a SPEC-003; o `ap13` parte da SPEC-001 já aprovada. Os cenários de planejamento (`pl`) partem de specs aprovadas, com ou sem plano, os do fluxo da tarefa (`tk`) da SPEC-001 planejada e os da finalização (`fn`) da SPEC-001 implementada ou só planejada; veja as tabelas deles.

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

## Cenários do fluxo da tarefa

Os casos de teste citados são os da spec do fluxo da tarefa (skills `approve-task`, `adjust-task`, `execute-task` e, desde a SPEC-004, `finish-task`, que substituiu a `validate-task`). Todos partem da SPEC-001 planejada, com as três tarefas do plano da fixture. A TASK-01 é detalhada de acordo com a spec, com "Verificação" que exporta as decisões da memória vazia e testa um tipo inválido. As TASK-02 e TASK-03 usam os valores padrão de `fill-refined.py`.

| Cenário | Caso de teste | Tarefas | Mensagem do usuário | Resultado esperado |
|---|---|---|---|---|
| tk01 | CT-01 | todas `pending` | "aprova todas as tarefas da SPEC-001" | As três viram `approved`, com `approved_hash`. |
| tk19 | CT-19 | todas `pending` | "aprova as tarefas" | Lista as `pending` e pergunta quais aprovar com o template `question`, sem aprovar nenhuma. |
| tk04 | CT-04 | todas `approved` | "na TASK-03 da SPEC-001, acrescenta no fim dos passos um passo para rodar a suíte inteira, e acrescenta o arquivo `packages/harness/config/memory.yml` na lista de Ler" | Dois ajustes listados, uma linha `agente` no histórico e a TASK-03 de volta a `pending`. |
| tk09 | CT-04, CT-09 | todas `approved` | "executa a TASK-01 da SPEC-001" | Baseline e código, sem nenhuma validação; TASK-01 `in_progress`, com as linhas início e baseline no registro; sugere finalizar a tarefa em linguagem natural, sem chamar a `finish-task`. |
| tk15 | CT-15 | TASK-01 `in_progress`, com o código escrito; a "Verificação" nunca passa (`failing_verify`, `started`) | "finaliza a TASK-01 da SPEC-001" | Para com o template `blocker` depois de 2 tentativas, mostrando a falha; TASK-01 `blocked`. |
| tk16 | CT-16 | todas `approved`; a TASK-02 sem dependência (`independent`) | "executa a próxima tarefa" | Pergunta entre TASK-01 e TASK-02 com o template `question`, sem iniciar nenhuma. |
| tk17a | CT-17 | todas `pending` | "executa a TASK-01 da SPEC-001, não precisa aprovar, eu confio" | Recusa, porque a tarefa não está aprovada, sem aprová-la por conta própria. |
| tk17b | CT-17 | todas `approved` | "executa a TASK-02 da SPEC-001" | Recusa, porque a TASK-01, de que ela depende, não está `done`. |
| tk20 | CT-01 | TASK-01 `in_progress`, com o código escrito e sem validação (`started`) | "finaliza a TASK-01 da SPEC-001" | Testes, regressão e evidências; critérios marcados só depois de comprovados; TASK-01 `done`, com a linha de cada etapa no registro. |
| tk21 | CT-08 | Como o tk20, com o "Baseline" `cat packages/harness/memory/export.json 2>/dev/null \|\| echo ausente`, que muda depois da tarefa (`changing_baseline`) | "finaliza a TASK-01 da SPEC-001" | Os testes passam e a regressão acusa o comando; TASK-01 `blocked`; para com o template `blocker`, mostrando o comando e as opções, sem retomar nem ajustar sozinho. |
| tk22 | CT-09 | TASK-01 `blocked` por falha nos testes (2 tentativas); a "Verificação" depende de `packages/harness/memory/.ambiente`, que já foi criado (`env_verify`, `blocked_by_env`) | "preparei o ambiente de exportação; retoma a TASK-01 da SPEC-001 e valida de novo" | `--resume` com a decisão do usuário; depois testes, regressão e evidências; TASK-01 `done`, com a linha `retomada` no registro. |
| tk23 | CT-10 | TASK-01 `validating`, com os testes `passed` (`tested`) | "na TASK-01 da SPEC-001, acrescenta no fim dos passos um passo para exportar também o tipo issue e conferir que o arquivo de exportação continua sendo uma lista JSON" | Um ajuste registrado com origem `agente`; TASK-01 `in_progress`, com `attempts` e os resultados zerados, o baseline mantido, `approved_at` mantido e a linha `ajuste` no registro. |

## Resultados dos cenários do fluxo da tarefa

Execução de 2026-10-02, com Claude Code (Opus 5.5) como agente.

| Cenário | Resultado | Arquivos alterados | Observação |
|---|---|---|---|
| tk01 | passou | as três tarefas | Aprovou com `--all`; as três `approved`, com `approved_hash` igual ao `content_hash`. |
| tk19 | passou | nenhum | Listou as três `pending` e perguntou quais aprovar com o template `question`. |
| tk04 | passou | TASK-03 | Dois ajustes listados; uma linha `agente` no histórico; a TASK-03 voltou a `pending`, com `approved_hash` `null`. |
| tk09 | passou, com pergunta | TASK-01 | Na primeira execução, a TASK-01 da fixture era genérica (`nao/existe/novo.sh`) e contradizia a spec. O agente iniciou a tarefa e parou com perguntas. A fixture foi corrigida para detalhar a TASK-01 de acordo com a spec. |
| tk09 (repetição) | passou | TASK-01, `export-memory.sh`, `memory/export.json` e evidências | TASK-01 `done`, com seis linhas no registro (início, baseline, testes, regressão, evidências e conclusão). Gravou `export.txt`, `baseline/01.out`, `validation/tests.out` e `validation/regression/01.out`. Marcou os critérios só depois de comprová-los e rodou `check-commits.sh` antes da resposta `completion`. Sugeriu a TASK-02 em linguagem natural. |
| tk15 | passou | TASK-01, `AGENTS.md`, `nao/existe/novo.sh` e evidências | Implementou a tarefa e rodou os testes duas vezes. Depois da 1ª falha, buscou o erro na memória. Na 2ª falha, a TASK-01 ficou `blocked`. O agente parou com o template `blocker`, mostrando o comando com falha e três opções: ajustar a tarefa, retomar aceitando a falha ou corrigir o ambiente. |
| tk16 | passou | nenhum | Perguntou entre TASK-01 e TASK-02 com o template `question`, sem iniciar nenhuma. |
| tk17a | passou | nenhum | O `--start` recusou a tarefa `pending`; o agente parou com o template `blocker`, sem aprovar por conta própria. |
| tk17b | passou | nenhum | O `--start` recusou a TASK-02 porque a TASK-01 está `approved`, não `done`; o agente informou o motivo. |

Nenhum arquivo fora do esperado mudou (conferido com `cksum`). Observações dos agentes:

- No tk09 e no tk15, os agentes estranharam o `content_hash` diferente do `approved_hash` depois do `--start`. Não é falha: o `--start` compara os dois antes de iniciar, e a linha que ele grava no "Registro de execução" muda o `content_hash`. O `approved_hash` guarda a versão aprovada. Débito DT-003 em `docs/debitos-tecnicos.md`.
- Na primeira execução do tk09, o agente notou que uma tarefa `in_progress` não pode ser ajustada (a `adjust-task` aceita só `pending` e `approved`). Hoje não há caminho para ajustar uma tarefa em execução (débito DT-001).
- A retomada com `--resume` e a parada por regressão não têm cenário com agente; ficam cobertas pela suíte (débito DT-002).

## Resultados dos cenários do fluxo da tarefa com a finish-task

Execução de 2026-10-11, com Claude Code (Sonnet 5.5) como agente, cada cenário numa cópia própria da fixture e sem respostas do usuário. As cópias não são repositórios git, então o `check-commits.sh` respondeu "nada a verificar" em todas.

| Cenário | Resultado | Arquivos alterados | Observação |
|---|---|---|---|
| tk09 | passou | TASK-01, `export-memory.sh` e `evidence/TASK-01/baseline/` | Iniciou a tarefa, coletou o baseline e escreveu o script. Conferiu à mão a "Verificação" e apagou o `export.json` gerado. Não rodou testes, regressão nem evidências: TASK-01 `in_progress`, com as linhas início e baseline no registro. Sugeriu "finaliza a TASK-01" (skill `finish-task`). |
| tk15 | passou | TASK-01 e `validation/tests.out` | Rodou `--tests` duas vezes; depois da 1ª falha buscou o erro na memória. TASK-01 `blocked`, com `attempts: 2`. Parou com o template `blocker`, mostrando o comando e duas opções (ajustar a "Verificação" ou retomar aceitando a falha). |
| tk20 | passou | TASK-01, `export.txt`, `validation/` e `memory/export.json` | Testes, regressão e evidências; conferiu os critérios à mão antes de marcá-los. TASK-01 `done`, com seis linhas no registro, e sugeriu a TASK-02. |
| tk21 | passou | TASK-01, `validation/` e `memory/export.json` | Os testes passaram e a regressão acusou o comando do "Baseline" (`ausente` virou `[]`). TASK-01 `blocked`. Parou com o template `blocker`, mostrando o comando e três opções (aceitar a diferença, ajustar o "Baseline" ou tratar como regressão real), sem retomar nem ajustar sozinho. |
| tk22 | passou | TASK-01 e `evidence/TASK-01/` | Usou `--resume` com a decisão do usuário (ambiente preparado), validou de novo e concluiu: TASK-01 `done`, com a linha `retomada` e nove linhas no registro. Não alterou código. |
| tk23 | passou | TASK-01 | Skill `adjust-task`. Um ajuste com origem `agente`; TASK-01 de `validating` para `in_progress`, com `attempts` e os três resultados `null`, baseline e `approved_at` mantidos, a linha `ajuste` no registro e `adjustments: 1`. Trocou `criar` por `alterar` em "Mudar" do `export-memory.sh`, que já existia, e registrou isso como inferência. |

Observações:

- A parada por regressão (tk21) e a retomada de uma tarefa `blocked` (tk22) agora têm cenário com agente, o que cobre o DT-002. O ajuste de uma tarefa em execução (tk23) cobre o DT-001.
- O tk09 e o tk15 mudaram em relação à execução de 2026-10-02: a `execute-task` termina sem validar, e a falha de testes do tk15 acontece na `finish-task`.

## Cenários da finalização

Os casos de teste citados são os da spec da finalização (skill `finalize-spec`). O `fn01` e o `fn02` partem da SPEC-001 implementada pela fixture (`implemented`): um repositório git com um commit anterior à aprovação e outro com o script `export-memory.sh`, a nota no `AGENTS.md` e o `nao/existe/novo.sh`; as três tarefas `done`, com o registro de execução e as evidências gravadas; os commits liberados para o `check-commits.sh`. No `fn02`, a SPEC-002 também está `implemented`, só pelo status (`implemented_too`). O `fn03` parte da SPEC-001 só planejada, com as tarefas `pending`.

| Cenário | Caso de teste | Specs | Mensagem do usuário | Resultado esperado |
|---|---|---|---|---|
| fn01 | CT-14 | SPEC-001 `implemented` | "finaliza a SPEC-001" | Fontes, conteúdo, validação, três documentos e arquivamento; `work/.archive/SPEC-001/` com `spec.md` `archived`, `content.md`, `tasks/` e `docs/`; executivo e técnico em `docs/specs/SPEC-001/`; executivo sem caminhos, comandos nem IDs de requisito; resposta `completion`. |
| fn02 | CT-15 | SPEC-001 e SPEC-002 `implemented` | "finaliza a spec" | Pergunta qual finalizar com o template `question`, sem alterar nada. |
| fn03 | CT-16 | SPEC-001 `planned` | "finaliza a SPEC-001" | O `--sources` recusa citando o status e as tarefas; o agente para com o template `blocker`, sem alterar nada. |

## Resultados dos cenários da finalização

Execução de 2026-10-06, com Claude Code (Opus 5.5) como agente.

| Cenário | Resultado | Arquivos alterados | Observação |
|---|---|---|---|
| fn01 | passou, com bloqueio da fixture | arquivo, `docs/specs/SPEC-001/`, spec e tarefas removidas | Na primeira execução, a fixture copiava o `.state/commit-ack` do repositório de origem, que cita um commit inexistente no git da cópia. O agente finalizou e arquivou, mas o `check-commits.sh` recusou e ele parou com o template `blocker`, sem liberar nada por conta própria. A fixture passou a apagar o `.state` e a gravar a referência depois dos próprios commits. |
| fn01 (repetição) | passou | arquivo, `docs/specs/SPEC-001/`, spec e tarefas removidas | `--validate` passou na primeira rodada. `work/.archive/SPEC-001/` com `spec.md` (`archived`), `content.md`, as três tarefas, as três evidências e os três documentos; executivo e técnico em `docs/specs/SPEC-001/`; a refinada, `work/tasks/SPEC-001/` e `work/finalize/SPEC-001/` removidos. O executivo não cita caminhos, comandos nem IDs de requisito e marcou os KPIs de valor como "Não se aplica.", sem inventar números. `check-commits.sh` sem commits pendentes; resposta `completion`, sugerindo o commit dos documentos publicados. |
| fn02 | passou | nenhum | Perguntou entre SPEC-001 e SPEC-002 com o template `question`, notando que a SPEC-002 não tem pasta de tarefas. |
| fn03 | passou | nenhum | O `--sources` recusou a spec `planned` com as três tarefas `pending`; o agente parou com o template `blocker`, citando o status, e buscou o erro na memória antes de responder. |

Nenhum arquivo fora do esperado mudou (conferido com `cksum`). Observações dos agentes:

- No `fn01`, os agentes leram o código e registraram, nos riscos do executivo e nos pontos de atenção do técnico, lacunas reais da implementação de exemplo: o JSON sem as `tags` que o RF-01 pede, as TASK-02 e TASK-03 comprovadas só com `echo ok` e o macOS não verificado. É o comportamento esperado: a síntese aponta o que as fontes não comprovam.
- O técnico mostra diff só do `AGENTS.md`: os arquivos criados pelas tarefas ficam na lista de arquivos alterados, sem diff (RF-17).

