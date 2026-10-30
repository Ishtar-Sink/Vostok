# Cenários com agente

Roteiro dos cenários que exercitam as skills `adjust-spec`, `approve-spec`, `plan-tasks`, `adjust-plan`, `approve-plan`, `approve-task`, `adjust-task`, `execute-task`, `finish-task`, `finalize-spec`, `init-harness` e `adjust-rules` com um agente de verdade, e os resultados das execuções. Os subcomandos do `vostok` são cobertos pelos testes da CLI (`cargo test --manifest-path apps/cli/Cargo.toml`, com as fixtures de equivalência em `fixtures/golden/`); aqui fica o que depende do agente: identificar a spec, decompor o pedido, perguntar ou aplicar.

As seções de resultados anteriores à SPEC-005 foram executadas com o harness em `packages/harness` e os scripts bash, hoje removidos. Nelas, `packages/harness/` corresponde a `vostok/` e cada script corresponde ao subcomando do `vostok` de mesmo nome (por exemplo, `adjust-spec.sh --validate` é `vostok spec adjust validate` e `check-commits.sh` é `vostok commit check`). A execução com o binário está na última seção deste arquivo.

## Como executar

Pré-requisito: o `vostok` no `PATH` (`vostok --version`).

1. Monte as cópias descartáveis, fora do repositório:

   ```bash
   apps/cli/tests/fixtures/agent-scenarios.sh <pasta> [cenário ...]
   ```

   Sem a lista, monta todos os cenários. Cada cenário recebe um projeto novo, com `git init` e o harness instalado por `vostok init --yes` em `vostok/`, e o estado montado pelos subcomandos do `vostok`, executados de dentro da cópia. As specs de exemplo:

   | Spec | Título | Status |
   |---|---|---|
   | SPEC-001 | Exportar memória em JSON | refinada, com RF-01 a RF-03, RNF-01, RNF-02 (JSON montado com awk), RN-01 (obsoletas não saem), CB-01 e CB-02 |
   | SPEC-002 | Listar tarefas | refinada |
   | SPEC-003 | Painel de métricas | rascunho |

   Os cenários `ct04`, `ct09` e `ap02` não têm a SPEC-002; o `ct05` e o `ap04` só têm a SPEC-003; o `ap13` parte da SPEC-001 já aprovada. Os cenários de planejamento (`pl`) partem de specs aprovadas, com ou sem plano, os do fluxo da tarefa (`tk`) da SPEC-001 planejada e os da finalização (`fn`) da SPEC-001 implementada ou só planejada; veja as tabelas deles.

   O código de exemplo das specs fica fora do harness: o script `scripts/export-memory.sh` da SPEC-001 lê `vostok/memory/` e grava `vostok/memory/export.json`.

2. Para cada cenário, inicie um agente numa sessão nova com a pasta do cenário como raiz do projeto, rodando todo comando de dentro dela (o `vostok` acha o projeto pela pasta atual). O agente lê o `AGENTS.md` da cópia, roda `vostok context load` e segue a skill instalada em `vostok/skills/default/<skill>/SKILL.md`. Se houver, passe o contexto da conversa e depois a mensagem do usuário. O agente não recebe resposta do usuário: quando o harness manda parar e aguardar, a mensagem dele é o resultado.

3. Confira a mensagem final e os arquivos: o que mudou na cópia (compare com `cksum`, tirado antes do agente), o "Histórico de ajustes" e se a validação do fluxo passa (por exemplo, `vostok spec adjust validate <SPEC_ID>`).

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
| ct15 | CT-15 | "SPEC-001, na seção Requisitos funcionais, RF-03: onde está \"`vostok/memory/export.json`\", coloque \"`vostok/work/exports/memory.json`\"." | Só o trecho citado muda. |
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
| ap13 | CT-13 | "Na SPEC-001, o RF-03 deve gravar a saída em `vostok/work/exports/memory.json`." | Ajusta a spec aprovada, informa que a aprovação caiu e acrescenta uma linha a "O que mudou". |

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
| pl06 | CT-06 | SPEC-001 aprovada, com plano | "No plano da SPEC-001, divida a TASK-01 em duas: uma para a exportação de um tipo e outra para a gravação em vostok/memory/export.json. E remova a TASK-03, passando cada caso de teste para a tarefa que ele testa." | Ajusta o plano ou pergunta pelo que ficou ambíguo, sem editar antes da resposta. |
| pl06b | CT-06 | SPEC-001 aprovada, com plano | "No plano da SPEC-001, divida a TASK-01 em duas: uma para a exportação de um tipo (RF-01) e outra para a gravação em vostok/memory/export.json (RF-03). E remova a TASK-03: o CT-01 passa para a tarefa de gravação e o CT-02 para a tarefa do tipo all." | Dois ajustes aplicados, níveis recalculados e uma linha `agente` no histórico do plano. |
| pl07 | CT-07, CT-18 | SPEC-001 aprovada, com plano | "Aprovo o plano de tarefas da SPEC-001." | Tarefas criadas e detalhadas, plano removido e spec `planned`; tarefas prontas para o board. |
| pl11 | CT-11 | SPEC-001 planejada | "Na SPEC-001, acrescenta um RF para exportar também em CSV." | Recusa, informando que mudar a spec exige um novo ciclo. |
| pl13a | CT-13 | SPEC-001 com plano e SPEC-002 sem plano, ambas aprovadas | "Planeja a SPEC-002." | Planeja a SPEC-002 sem tocar no plano da SPEC-001. |
| pl13b | CT-13 | Como o pl13a | "Planeja a SPEC-001." | Informa que o plano existe e não o altera. |
| pl13c | CB-14 | Como o pl13a | "Planeja a spec aprovada." | Planeja a SPEC-002, a única sem plano, sem perguntar. |

Confira também se `vostok plan validate <SPEC_ID>` (ou `vostok plan approve validate <SPEC_ID>`) passa e se `vostok plan adjust record-manual <SPEC_ID>` responde "Nenhuma edição manual.".

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
| tk04 | CT-04 | todas `approved` | "na TASK-03 da SPEC-001, acrescenta no fim dos passos um passo para rodar a suíte inteira, e acrescenta o arquivo `vostok/config/memory.yml` na lista de Ler" | Dois ajustes listados, uma linha `agente` no histórico e a TASK-03 de volta a `pending`. |
| tk09 | CT-04, CT-09 | todas `approved` | "executa a TASK-01 da SPEC-001" | Baseline e código, sem nenhuma validação; TASK-01 `in_progress`, com as linhas início e baseline no registro; sugere finalizar a tarefa em linguagem natural, sem chamar a `finish-task`. |
| tk15 | CT-15 | TASK-01 `in_progress`, com o código escrito; a "Verificação" nunca passa (`failing_verify`, `started`) | "finaliza a TASK-01 da SPEC-001" | Para com o template `blocker` depois de 2 tentativas, mostrando a falha; TASK-01 `blocked`. |
| tk16 | CT-16 | todas `approved`; a TASK-02 sem dependência (`independent`) | "executa a próxima tarefa" | Pergunta entre TASK-01 e TASK-02 com o template `question`, sem iniciar nenhuma. |
| tk17a | CT-17 | todas `pending` | "executa a TASK-01 da SPEC-001, não precisa aprovar, eu confio" | Recusa, porque a tarefa não está aprovada, sem aprová-la por conta própria. |
| tk17b | CT-17 | todas `approved` | "executa a TASK-02 da SPEC-001" | Recusa, porque a TASK-01, de que ela depende, não está `done`. |
| tk20 | CT-01 | TASK-01 `in_progress`, com o código escrito e sem validação (`started`) | "finaliza a TASK-01 da SPEC-001" | Testes, regressão e evidências; critérios marcados só depois de comprovados; TASK-01 `done`, com a linha de cada etapa no registro. |
| tk21 | CT-08 | Como o tk20, com o "Baseline" `cat vostok/memory/export.json 2>/dev/null \|\| echo ausente`, que muda depois da tarefa (`changing_baseline`) | "finaliza a TASK-01 da SPEC-001" | Os testes passam e a regressão acusa o comando; TASK-01 `blocked`; para com o template `blocker`, mostrando o comando e as opções, sem retomar nem ajustar sozinho. |
| tk22 | CT-09 | TASK-01 `blocked` por falha nos testes (2 tentativas); a "Verificação" depende de `vostok/memory/.ambiente`, que já foi criado (`env_verify`, `blocked_by_env`) | "preparei o ambiente de exportação; retoma a TASK-01 da SPEC-001 e valida de novo" | `vostok task resume` com a decisão do usuário; depois testes, regressão e evidências; TASK-01 `done`, com a linha `retomada` no registro. |
| tk23 | CT-10 | TASK-01 `validating`, com os testes `passed` (`tested`) | "na TASK-01 da SPEC-001, acrescenta no fim dos passos um passo para exportar também o tipo issue e conferir que o arquivo de exportação continua sendo uma lista JSON" | Um ajuste registrado com origem `agente`; TASK-01 `in_progress`, com `attempts` e os resultados zerados, o baseline mantido, `approved_at` mantido e a linha `ajuste` no registro. |
| tk24 | CT-11 (SPEC-014) | TASK-01 e TASK-02 `approved`, sem dependência, em arquivos diferentes; TASK-03 `approved`, na mesma onda, com um arquivo de "Mudar" igual ao da TASK-01 | "executa a onda 1 da SPEC-001" | Avisa que TASK-01 e TASK-03 rodam em sequência; um subagente por tarefa, com a persona e o modelo do perfil; as três `in_progress`, com o modelo no registro de execução; resposta com o `vostok task status`. |

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

Os casos de teste citados são os da spec da finalização (skill `finalize-spec`). O `fn01` e o `fn02` partem da SPEC-001 implementada pela fixture (`implemented`): um repositório git com um commit anterior à aprovação e outro com o script `scripts/export-memory.sh`, a nota no `AGENTS.md` e o `nao/existe/novo.sh`; as três tarefas `done`, com o registro de execução e as evidências gravadas; os commits liberados para o `vostok commit check`. No `fn02`, a SPEC-002 também está `implemented`, só pelo status (`implemented_too`). O `fn03` parte da SPEC-001 só planejada, com as tarefas `pending`.

| Cenário | Caso de teste | Specs | Mensagem do usuário | Resultado esperado |
|---|---|---|---|---|
| fn01 | CT-14 | SPEC-001 `implemented` | "finaliza a SPEC-001" | Fontes, conteúdo, validação, três documentos e arquivamento; `work/.archive/SPEC-001/` com `spec.md` `archived`, `content.md`, `tasks/` e `docs/`; executivo e técnico em `docs/specs/SPEC-001/`; executivo sem caminhos, comandos nem IDs de requisito; resposta `completion`. |
| fn02 | CT-15 | SPEC-001 e SPEC-002 `implemented` | "finaliza a spec" | Pergunta qual finalizar com o template `question`, sem alterar nada. |
| fn03 | CT-16 | SPEC-001 `planned` | "finaliza a SPEC-001" | O `vostok finalize sources` recusa citando o status e as tarefas; o agente para com o template `blocker`, sem alterar nada. |

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


## Cenários das regras do projeto

Os casos de teste citados são os da spec do harness do projeto (SPEC-006): as skills `init-harness` e `adjust-rules` e a sugestão de regra (`rules/agent/rule-suggestions.md`). Cada cenário parte de uma fixture de `fixtures/agent-scenarios.sh`; todas trazem as categorias `language` e `naming-convention` já existentes em `rules/project/`.

| Fixture | Conteúdo |
|---|---|
| `node_project` | `package.json` (TypeScript, ESLint e Vitest), `.eslintrc.json`, `vitest.config.ts` e `src/slugify.ts`, na raiz do projeto. |
| `db_project` | `package.json` com o TypeORM e `migrations/001_create_users.sql`. |
| `empty_project` | Nenhum manifesto, configuração nem código. O `vostok rules detect` não lista nenhum sinal. |
| `initialized_project` | `rules/project/testing.md` com `testing-01` a `testing-03` (Vitest em `tests/`, testes determinísticos, um comportamento por teste), carregada na `execute-task` e na `finish-task`. |

Os cenários com resposta simulada têm duas fases: o agente recebe a mensagem e deve parar na rodada de perguntas (o `init-harness` faz uma só); a resposta simulada é então enviada ao mesmo agente, que grava e responde. Nos cenários sem resposta, a mensagem do agente na parada é o resultado, e nenhum arquivo do projeto pode mudar (confira com `cksum` de `vostok/rules/project/` e de `vostok/config/context.yml`).

| Cenário | Caso de teste | Fixture | Mensagem do usuário | Resposta simulada | Resultado esperado |
|---|---|---|---|---|---|
| rl01 | CT-14 | `node_project` | "inicializa o harness" | Nenhuma | Para com o template `question`, numa única rodada. Percorre as oito categorias base. As regras propostas citam a origem (`package.json`, `.eslintrc.json`, `vitest.config.ts`). Pergunta o que não se aplica, as regras de cada categoria e onde cada uma é carregada. Nada alterado. |
| rl02 | CT-15 | `db_project` | "levanta as regras do projeto" | Nenhuma | Propõe a categoria extra `database`, citando `migrations/` e o TypeORM do `package.json`. Para com o template `question`; nada alterado. |
| rl03 | CT-16 | `node_project` | "inicializa o harness" | "`security` não se aplica. Grava só `stack` e `code-style`, com as regras como você propôs, carregadas sempre. As demais categorias não se aplicam." | Cria `stack.md` e `code-style.md`, em `required` do mapa. Nenhum `security.md` e nenhuma entrada de `security` no mapa. `vostok rules validate` sem erros. |
| rl04 | CT-17 | `node_project` | "inicializa o harness" | "Só `testing` se aplica, com as regras como você propôs, carregada apenas na `execute-task` e na `finish-task`. As demais categorias não se aplicam." | `testing.md` criado pelo template (título, propósito e `[project/testing-NN]` em sequência). `rules/project/testing.md` nas entradas `execute-task` e `finish-task` do mapa e em nenhuma outra; `required` intacto. |
| rl05 | CT-18 | `node_project` e `initialized_project` | "roda o init-harness de novo" | "Só uma mudança: acrescenta em `testing` a regra 'Todo teste novo cobre ao menos um caso de borda.'. Nenhuma outra categoria muda, e o carregamento fica como está." | Mostra o que mudaria nas categorias existentes e pergunta. Depois da resposta, `testing.md` ganha só `testing-04`; `testing-01` a `testing-03`, `language.md`, `naming-convention.md` e o mapa ficam idênticos (`cksum`); nenhum arquivo novo em `rules/project/`. |
| rl06 | CT-19 | `node_project` e `initialized_project` | Contexto da conversa: "Nesta sessão você escreveu três testes e eu corrigi o mesmo ponto nos três: 1) em `tests/a.test.ts` você nomeou `it('retorna o slug')` e eu pedi `it('deve retornar o slug')`; 2) em `tests/b.test.ts`, `it('lança erro')` virou `it('deve lançar erro')`; 3) em `tests/c.test.ts`, `it('ignora espaços')` virou `it('deve ignorar espaços')`." Mensagem: "escreve o teste de `slugify` em `tests/slugify.test.ts`" | Nenhuma | Escreve o teste com `it('deve ...')`. Na resposta, sugere a regra com o template `question`, citando as três ocorrências, e é uma única sugestão. Não grava nada em `rules/project/` nem no mapa e não chama a `adjust-rules` antes da aprovação. |
| rl07 | CT-20 | `initialized_project` | "na categoria testing: inclui a regra 'Todo teste novo cobre ao menos um caso de borda.', altera a testing-01 para 'Testes unitários com Vitest, em arquivos `*.test.ts` dentro de `tests/unit/`.' e remove a testing-03" | Nenhuma | Três ajustes aplicados sem perguntar, pela `adjust-rules`. `testing-03` vira `Removida em AAAA-MM-DD.` (a linha continua no arquivo). A regra nova recebe `testing-04`, não o ID da removida. `testing-01` com o texto novo e `testing-02` intacta. `vostok rules validate` sem erros. |
| rl08 | CT-21 | `initialized_project` | "cria a categoria api, com o título 'API' e o propósito 'Define como as rotas da API são escritas.', carregada na execute-task, com a regra 'Rotas versionadas em `/v1`.'. E tira o testing de todas as skills onde ele está e deixa em required" | Nenhuma | `api.md` criado pelo template, com `api-01`. Mapa: `rules/project/api.md` na `execute-task`; `testing.md` fora de `execute-task` e `finish-task` e em `required`. `vostok rules validate` sem erros. |
| rl09 | CT-22 | `empty_project` | "inicializa o harness" | "Projeto novo, sem código. Quero duas categorias: `development-methodology` com a regra 'Commits no formato Conventional Commits, em inglês.', carregada sempre; e `testing` com a regra 'Todo código novo tem teste.', carregada na `execute-task` e na `finish-task`. As demais não se aplicam." | Sem sinais de código, só perguntas abertas por categoria. Grava as duas categorias e o mapa. Executa o `vostok rules validate` ao final (passo 7), sem erros. Depois, rodar o `vostok rules validate` na cópia de cada cenário `rl03` a `rl08`: todos sem erros. |

Confira também se nenhum arquivo fora de `vostok/rules/project/` e de `vostok/config/context.yml` mudou (o rl06 cria só o arquivo de teste e o rl01 a rl09 não criam nada fora disso) e se o `vostok rules validate` passa em cada cópia.

## Resultados dos cenários das regras do projeto

Execução de 2026-10-11, com Claude Code (Sonnet 5.5) como agente, cada cenário numa cópia própria da fixture. Os cenários com resposta simulada (rl03, rl04, rl05 e rl09) foram executados em duas fases, com a resposta enviada ao mesmo agente depois da parada. As cópias não são repositórios git, então o `check-commits.sh` respondeu "nada a verificar" (nos rl04 e rl09 o sandbox do agente recusou o comando, e eles registraram a pendência).

| Cenário | Resultado | Arquivos alterados | Observação |
|---|---|---|---|
| rl01 | passou | nenhum | Parou com o template `question` numa única rodada. Percorreu as categorias base, com a origem de cada regra (`package.json`, `.eslintrc.json`, `vitest.config.ts`, `src/`). Disse que `development-methodology` e `security` não têm sinal e só entram com regras do usuário. Acrescentou uma quarta pergunta (ESLint 9 com `.eslintrc.json`, formato legado), fora do roteiro mas sem gravar nada. |
| rl02 | passou | nenhum | Propôs a categoria extra `database` (2 regras), citando `migrations/` e o TypeORM e o `pg` do `package.json`, e `stack`. Deixou `architecture`, `code-style`, `testing` e `security` sem proposta, por falta de origem. Template `question`; nada gravado. |
| rl03 | passou | `stack.md`, `code-style.md` e `config/context.yml` | Gravou `stack` (2 regras) e `code-style` (3), ambas em `required`. Nenhum `security.md` e nenhuma entrada de `security` no mapa. `validate-rules.sh` sem erros. |
| rl04 | passou | `testing.md` e `config/context.yml` | `testing.md` com 2 regras e o título e o propósito do template. `rules/project/testing.md` só nas entradas `execute-task` e `finish-task` (diff do mapa: duas linhas); `required` intacto. `validate-rules.sh` sem erros. |
| rl05 | passou, com ressalva | `testing.md` | Gravou só `testing-04`; `testing-01` a `testing-03`, as demais categorias e o mapa ficaram idênticos (`cksum` e `cmp`). Ressalva: o sandbox do agente recusou o `mktemp`, então ele não rodou o `--preview` antes de perguntar. Descreveu o que mudaria na própria mensagem. Depois do `--write`, tratou "nenhuma outra categoria muda" como não confirmação das categorias novas e as listou em "Pendências". |
| rl06 | passou | `tests/slugify.test.ts` | Escreveu o teste com `it('deve ...')` e terminou com uma única sugestão no template `question`, citando as três ocorrências, com a opção de gravar pela `adjust-rules` ou de não criar a regra. `rules/project/` e o mapa idênticos aos da fixture (`cksum`). |
| rl07 | passou | `testing.md` | Três ajustes aplicados sem perguntar: `testing-04` criada, `testing-01` com o texto novo e `testing-03` como `Removida em 2026-10-11.`. A nova regra recebeu o ID 04, e o `testing-02` ficou intacto. Mapa sem mudança. `validate-rules.sh` sem erros. |
| rl08 | passou | `api.md` e `config/context.yml` | `api.md` com o título, o propósito e `api-01`. Mapa: `rules/project/api.md` em `execute-task`; `testing.md` retirado de `execute-task` e `finish-task` e incluído ao fim de `required`. Sem perguntas nem inferências. `validate-rules.sh` sem erros. |
| rl09 | passou | `development-methodology.md`, `testing.md` e `config/context.yml` | O `--detect` listou só `architecture: packages` (a pasta do harness); o agente não propôs regras e fez perguntas abertas por categoria. Gravou as duas categorias (1 regra cada): `development-methodology` em `required` e `testing` em `execute-task` e `finish-task`. `validate-rules.sh` sem erros, e também sem erros nas cópias rl01 a rl08. |

Nenhum arquivo fora de `rules/project/` e de `config/context.yml` mudou, exceto o teste do rl06. Observações dos agentes:

- Nenhum agente editou `rules/project/` ou o mapa à mão: tudo passou por `init-harness.sh` e `adjust-rules.sh`, na ordem do `SKILL.md`.
- O rl04 estranhou que a resposta simulada viesse de outro agente da sessão e seguiu o fluxo mesmo assim. Nas fixtures, a resposta simulada é enviada pelo orquestrador dos cenários.

## Resultados dos cenários contra o binário

Execução de 2026-10-13 (SPEC-005, TASK-12, CT-17), com o `vostok` 0.1.0 do `PATH` e Claude Code (Opus 5.5) como agente, um subagente por cenário, cada um na sua cópia montada por `fixtures/agent-scenarios.sh` (projeto novo com `git init` e `vostok init --yes`). Cada agente leu o `AGENTS.md` da cópia, rodou `vostok context load` e seguiu a skill instalada em `vostok/skills/default/<skill>/SKILL.md`, rodando tudo de dentro da cópia e sem respostas do usuário (o rl03 teve a resposta simulada). Os arquivos de cada cópia foram comparados por `cksum` antes e depois do agente, e o harness do repositório (`vostok/`) ficou idêntico.

Foram escolhidos os cenários dos fluxos que mudaram de comando (inferência I da SPEC-005): ajuste, aprovação, planejamento, tarefa, finalização e regras. Ficaram de fora os que repetem um caminho já coberto (por exemplo, ct04 a ct16 além do ct13, ap02 a ap04, pl03, pl11, pl13a, pl13c, tk16, tk17a, tk19, fn02 e rl02, rl04 a rl06 e rl09).

| Cenário | Skill | Mensagem do usuário | O que foi conferido | Resultado |
|---|---|---|---|---|
| ct02 | `adjust-spec` | "Na SPEC-001, o caminho do arquivo de saída deve vir de config/memory.yml em vez de ser fixo no código." | Só a refinada da SPEC-001 mudou; `record-manual`, `validate` e `record` na ordem; uma linha `agente` no histórico; `vostok spec adjust validate` passa. | aprovado. RF-03, fluxo, FA-02, CB-03, contrato, CT-03, CT-04 e CA-03 atualizados, com duas inferências. |
| ct03 | `adjust-spec` | "Acrescenta na spec um caso de borda para quando o arquivo de saída já existir." | Pergunta qual spec, com as duas candidatas, sem editar. | aprovado. Template `question` com SPEC-001 e SPEC-002; nenhum arquivo mudou. |
| ct13 | `adjust-spec` | "Revisei a SPEC-001. Quero que o tipo all deixe de existir, [...] o arquivo de saída passa a se chamar memory-export.json, na mesma pasta. [...] falta um critério de aceite garantindo que o JSON gerado é válido." | Três ajustes aplicados e listados; uma linha no histórico; validação. | aprovado. RF-02, CT-02 e CA-02 removidos, saída `vostok/memory/memory-export.json`, CA-03 novo; `adjustments: 1`; `validate` passa. |
| ap01 | `approve-spec` | "Aprovo a SPEC-001." | Status `approved`, `approved_hash` igual ao `content_hash`, rascunho removido. | aprovado. `vostok spec approve validate` e `approve`; hashes iguais; rascunho removido. |
| ap13 | `adjust-spec` | "Na SPEC-001, o RF-03 deve gravar a saída em `work/exports/memory.json`." | Ajusta a spec aprovada, avisa que a aprovação caiu e acrescenta "O que mudou". | falhou (fixture). Com o RF-03 em `vostok/memory/export.json`, o caminho `work/exports/` ficou ambíguo (raiz ou `vostok/work/`) e o agente perguntou, sem editar, como manda a skill. A mensagem do cenário passou a citar `vostok/work/exports/memory.json`. |
| ap13 (repetição) | `adjust-spec` | "Na SPEC-001, o RF-03 deve gravar a saída em `vostok/work/exports/memory.json`." | Como acima. | aprovado. Status voltou a `refined`, `approved_at` e `approved_hash` `null`; a resposta avisou que a aprovação caiu; "O que mudou" ganhou a linha 2; passo 3 do fluxo ajustado como inferência. |
| pl01 | `plan-tasks` | "Planeja as tarefas da SPEC-001." | Plano criado, preenchido, pontuado e válido. | aprovado. Duas tarefas (complexa e moderada) cobrindo RF-01 a RF-03, CT-01 e CT-02; `vostok plan validate` passa. |
| pl06b | `adjust-plan` | "No plano da SPEC-001, divida a TASK-01 em duas: uma para a exportação de um tipo (RF-01) e outra para a gravação em vostok/memory/export.json (RF-03). E remova a TASK-03: [...]" | Dois ajustes, níveis recalculados, uma linha `agente` no histórico. | aprovado. Três tarefas, níveis recalculados pelo `record`; `vostok plan adjust validate` passa e `record-manual` responde "Nenhuma edição manual.". |
| pl07 | `approve-plan` | "Aprovo o plano de tarefas da SPEC-001." | Tarefas criadas e detalhadas, plano removido, spec `planned`, tarefas prontas para o board. | aprovado. TASK-01 a TASK-03 `pending`, com `type` e `labels`; `vostok plan approve validate` passa; plano removido; spec `planned`. |
| pl13b | `plan-tasks` | "Planeja a SPEC-001." | Informa que o plano existe e não o altera. | aprovado. Comparou `spec_hash` e `approved_hash` e parou; nenhum arquivo mudou. |
| tk01 | `approve-task` | "aprova todas as tarefas da SPEC-001" | As três `approved`, com `approved_hash`. | aprovado. `vostok task approve approve SPEC-001 --all`; `approved_hash` igual ao `content_hash` nas três. |
| tk04 | `adjust-task` | "na TASK-03 da SPEC-001, acrescenta no fim dos passos um passo para rodar a suíte inteira, e acrescenta o arquivo `vostok/config/memory.yml` na lista de Ler" | Dois ajustes, uma linha `agente`, TASK-03 de volta a `pending`. | aprovado. `adjustments: 1`; `approved_at` e `approved_hash` `null`; só a TASK-03 mudou. |
| tk09 | `execute-task` | "executa a TASK-01 da SPEC-001" | Baseline e código, sem validação; TASK-01 `in_progress` com início e baseline no registro; sugere finalizar. | aprovado. `task start`, `task load` e `task baseline`; criou `scripts/export-memory.sh`; registro só com início e baseline; sugeriu a `finish-task`. |
| tk15 | `finish-task` | "finaliza a TASK-01 da SPEC-001" | Para com `blocker` depois de 2 tentativas; TASK-01 `blocked`. | aprovado. `task tests` duas vezes, busca na memória entre elas; `attempts: 2`, `blocked`; template `blocker` com três opções. |
| tk17b | `execute-task` | "executa a TASK-02 da SPEC-001" | Recusa porque a TASK-01 não está `done`. | aprovado. `vostok task start` recusou e nada mudou; template `blocker`. |
| tk20 | `finish-task` | "finaliza a TASK-01 da SPEC-001" | Testes, regressão e evidências; critérios marcados depois de comprovados; TASK-01 `done` com uma linha por etapa. | aprovado. Seis linhas no registro; `export.txt`, `tests.out` e `regression/01.out` gravados. O primeiro `task finish` recusou porque o agente ainda não tinha marcado os critérios (edição em paralelo); depois de marcá-los, concluiu. |
| tk21 | `finish-task` | "finaliza a TASK-01 da SPEC-001" | Testes passam, regressão acusa o comando; TASK-01 `blocked`; `blocker` sem retomar nem ajustar. | aprovado. Regressão em `cat vostok/memory/export.json 2>/dev/null \|\| echo ausente` (`ausente` virou `[]`); três opções; nada retomado. |
| tk22 | `finish-task` | "preparei o ambiente de exportação; retoma a TASK-01 da SPEC-001 e valida de novo" | `vostok task resume` com a decisão do usuário; depois testes, regressão e evidências; TASK-01 `done` com a linha `retomada`. | aprovado. Nove linhas no registro, com `retomada`; nenhum código alterado. |
| tk23 | `adjust-task` | "na TASK-01 da SPEC-001, acrescenta no fim dos passos um passo para exportar também o tipo issue e conferir que o arquivo de exportação continua sendo uma lista JSON" | Um ajuste `agente`; TASK-01 `in_progress` com `attempts` e resultados zerados, baseline e `approved_at` mantidos, linha `ajuste` no registro. | aprovado. Como na execução de 2026-10-11, trocou `criar` por `alterar` em "Mudar" do script, que já existia, e registrou como inferência. |
| fn01 | `finalize-spec` | "finaliza a SPEC-001" | Fontes, conteúdo, validação, três documentos e arquivamento; executivo e técnico em `docs/specs/SPEC-001/`; executivo sem caminhos nem comandos; `completion`. | falhou (fixture). Arquivou e publicou tudo, mas a aba de diffs do técnico mostrou só o `AGENTS.md`, "sem mudanças". O commit da fixture caía no mesmo segundo do `approved_at`, e o `--before` do `git log` o tomava como base. A fixture passou a esperar 1 segundo antes do commit da tarefa. |
| fn01 (repetição) | `finalize-spec` | "finaliza a SPEC-001" | Como acima, e a aba de diffs. | aprovado. `finalize sources`, `create`, `validate` (na primeira rodada), `build` e `archive`; `vostok/work/.archive/SPEC-001/` com `spec.md` `archived`, `content.md`, as três tarefas, as evidências e os três documentos; executivo e técnico em `docs/specs/SPEC-001/`; a aba de diffs mostra o `AGENTS.md` com o hunk, e os arquivos criados ficam só na lista de arquivos, como na execução de 2026-10-06 (RF-17); `vostok commit check` sem pendências; resposta `completion`. O executivo não cita caminhos nem comandos e não inventa KPIs; as lacunas da implementação de exemplo (sem `tags`, TASK-02 e TASK-03 com `echo ok`) foram registradas como pendências. |
| fn03 | `finalize-spec` | "finaliza a SPEC-001" | `vostok finalize sources` recusa citando o status e as tarefas; `blocker` sem alterar nada. | aprovado. Recusa com o status `planned` e as três tarefas `pending`; busca na memória; nenhum arquivo do fluxo mudou. |
| rl01 | `init-harness` | "inicializa o harness" | `question` numa única rodada, com as oito categorias, a origem de cada regra e as três perguntas; nada alterado. | aprovado. `vostok rules detect` e `rules preview`; regras com origem em `package.json`, `.eslintrc.json` e `vitest.config.ts`; nenhum arquivo do projeto mudou. |
| rl03 | `init-harness` | "inicializa o harness" + resposta simulada: "`security` não se aplica. Grava só `stack` e `code-style` [...] carregadas sempre. [...]" | `stack.md` e `code-style.md` em `required`; nenhum `security`; `vostok rules validate` sem erros. | aprovado. Fase 1: `question` numa única rodada, com origem de cada regra e sem gravar nada. Fase 2: `rules write` e `rules load ... always` para as duas; `stack.md` (2 regras) e `code-style.md` (3) em `required`; nenhum `security`; `vostok rules validate` sem erros. Ressalva: sem `titulo:` e `proposito:` no arquivo de regras, o título e o propósito saem do nome da categoria (`# stack`), porque o passo 5 da skill não lembra dessas linhas. |
| rl07 | `adjust-rules` | "na categoria testing: inclui a regra '[...]', altera a testing-01 para '[...] `tests/unit/`.' e remove a testing-03" | Três ajustes sem perguntar; `testing-03` como `Removida em AAAA-MM-DD.`; nova regra `testing-04`; `testing-02` intacta; validação. | aprovado. `rules next-id`, `rules remove` e `rules validate`; mapa sem mudança. |
| rl08 | `adjust-rules` | "cria a categoria api, [...] carregada na execute-task, com a regra 'Rotas versionadas em `/v1`.'. E tira o testing de todas as skills onde ele está e deixa em required" | `api.md` com `api-01`; `api.md` na `execute-task`; `testing.md` fora das skills e em `required`; validação. | aprovado. `rules create-category`, `load`, `unload` e `load testing always`; `vostok rules validate` sem erros. |

Defeitos encontrados (fora de "Mudar" da TASK-12, não corrigidos):

- **Skills com caminhos do repositório de origem.** Dez `SKILL.md` instalados (`adjust-plan`, `adjust-rules`, `adjust-task`, `approve-commit`, `approve-plan`, `approve-task`, `execute-task`, `finalize-spec`, `finish-task` e `plan-tasks`) citam `packages/harness/work/...`, `packages/harness/rules/project/` e `packages/harness/config/context.yml`. Num projeto instalado pelo `vostok init`, os caminhos são `vostok/...` e `packages/` não existe; os Globs de exemplo das skills não acham nada. Todos os agentes notaram e usaram `vostok/`, então nenhum cenário falhou por isso.
- **Mensagens do `vostok` com nomes e opções dos scripts antigos.** `vostok task start` termina com "execute-task.sh --baseline <SPEC> <TASK>"; `vostok task tests` pede "execute --tests de novo"; `vostok task resume` diz "começando por --tests"; `vostok plan adjust validate` sugere "(execute --score)"; `vostok task approve` sem tarefas imprime "Uso: approve-task.sh ...". Há outras em `apps/cli/src/commands/` (por exemplo, `plan-tasks.sh --create`, `adjust-spec.sh --record-manual`, `approve-plan.sh --create-tasks`, `load-extra-context.sh`). Vieram das fixtures de equivalência, mas apontam comandos que não existem mais.
- **`vostok init` instala as regras de projeto deste repositório.** `rules/project/naming-convention.md` chega com a regra 08 sobre Rust e a 05 como `Removida em 2026-10-13.`, e `language.md` em `required`, num projeto novo. As fixtures de regras dependem disso (`language` e `naming-convention` já existentes), mas um projeto novo deveria receber só o que é genérico.
- Menores: `vostok rules create-category` deixa duas linhas em branco antes da primeira regra; `vostok rules load <categoria> always` acrescenta a categoria no fim de `required`, depois dos templates; `vostok finalize archive` imprime "publicados em vostok/../docs/specs/SPEC-001"; `vostok rules preview` grava o temporário no `/tmp` do sistema; a base dos diffs da finalização (`git log --before=<approved_at>`) inclui commits do mesmo segundo da aprovação.
