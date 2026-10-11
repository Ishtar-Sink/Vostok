# Débitos técnicos

Problemas conhecidos que ficaram para depois: lacunas de comportamento, de cobertura de testes ou de clareza. Cada débito tem um ID fixo (`DT-NNN`), é registrado quando é encontrado e só sai da lista de abertos quando for resolvido ou descartado, com o motivo.

## Resumo

| ID | Título | Área | Prioridade | Status |
|---|---|---|---|---|
| DT-001 | Tarefa em execução não pode ser ajustada | harness: fluxo da tarefa | alta | resolvido |
| DT-002 | Retomada e regressão sem cenário com agente | harness: testes com agente | média | resolvido |
| DT-003 | Hash da tarefa diverge do hash aprovado após o início | harness: fluxo da tarefa | baixa | resolvido |
| DT-004 | Commits da finalização sem data final | harness: finalização da spec | baixa | resolvido |
| DT-005 | `--build` informa sucesso quando a geração falha | harness: finalização da spec | média | resolvido |
| DT-006 | Desfazer do `--archive` sem teste | harness: finalização da spec | média | resolvido |

Prioridade: **alta** bloqueia um fluxo real; **média** deixa um comportamento sem prova; **baixa** confunde, mas não quebra nada.

## DT-001: Tarefa em execução não pode ser ajustada

- **Área:** harness, skill `adjust-task`
- **Encontrado em:** 2026-10-02, SPEC-002 TASK-08, primeira execução do cenário `tk09`
- **Prioridade:** alta
- **Status:** resolvido

**Descrição.** A `adjust-task` aceita só tarefas `pending` ou `approved`. Quando a execução revela que a tarefa está errada (passos, verificação ou arquivos que não batem com a spec), não há caminho para corrigi-la: ela está `in_progress` e só sai desse estado para `validating`, `blocked` ou `done`.

**Impacto.** O agente fica sem saída válida. Ele precisa parar e perguntar, e o usuário também não tem um comando para devolver a tarefa a um estado ajustável. A alternativa seria editar o arquivo à mão, fora do fluxo.

**Proposta.** Definir como uma tarefa em execução volta a ser ajustável. Uma opção é a `adjust-task` aceitar `in_progress` e `blocked`, derrubar a aprovação, descartar o baseline e as evidências e registrar o ajuste. Outra é uma etapa explícita de "devolver para ajuste". Exige decisão do usuário antes de implementar.

**Ocorrências.** 2026-10-06, SPEC-003 TASK-05: o usuário mudou o foco e o visual do documento executivo com a tarefa já em validação. O ajuste foi registrado à mão no histórico de ajustes da tarefa, com os arquivos novos em "Mudar" e o critério novo em "Concluída quando".

**Referências.** `packages/harness/skills/default/adjust-task/`, `packages/harness/tests/agent-scenarios.md` (resultado do `tk09`).

**Resolução.** Resolvido na SPEC-004 pelas TASK-02 a TASK-05 (commits `123a608` e `5721c1b`): a `adjust-task` aceita tarefas `in_progress`, `validating` e `blocked`, mantém o baseline, zera tentativas e resultados, grava a linha `ajuste` no registro e não pede nova aprovação; tarefas `done` são recusadas. A conclusão ficou só na skill `finish-task`.

## DT-002: Retomada e regressão sem cenário com agente

- **Área:** harness, `tests/agent-scenarios.md`
- **Encontrado em:** 2026-10-02, SPEC-002 TASK-08
- **Prioridade:** média
- **Status:** resolvido

**Descrição.** O CA-05 da SPEC-002 pede que os cenários tristes com agente parem na falha de testes e na regressão e retomem com `--resume`. Os cenários com agente cobrem só a parada por falha de testes (`tk15`). A parada por regressão e a retomada de uma tarefa `blocked` com a decisão do usuário têm cobertura apenas na suíte (`tests/run-tests.sh`).

**Impacto.** Não há prova de que o agente reage bem à regressão, nem de que conduz a retomada como o fluxo manda: registrar a decisão e voltar a validar.

**Proposta.** Acrescentar à fixture um cenário com regressão (baseline que muda depois da tarefa) e um cenário de retomada, que parte de uma tarefa `blocked` e tem como mensagem do usuário uma das opções do `blocker`.

**Referências.** `packages/harness/tests/fixtures/agent-scenarios.sh`, `packages/harness/skills/default/finish-task/`.

**Resolução.** Resolvido na SPEC-004 pela TASK-06 (commit `dddd36d`): os cenários `tk21` (parada por regressão), `tk22` (retomada de uma tarefa `blocked` com a decisão do usuário) e `tk23` (ajuste em execução) foram executados com agente e passaram.

## DT-003: Hash da tarefa diverge do hash aprovado após o início

- **Área:** harness, skills `execute-task` e `finish-task`
- **Encontrado em:** 2026-10-02, SPEC-002 TASK-08, cenários `tk09` e `tk15`
- **Prioridade:** baixa
- **Status:** resolvido

**Descrição.** O `--start` compara o `approved_hash` com o conteúdo antes de iniciar. Depois, cada etapa grava uma linha no "Registro de execução" e atualiza o `content_hash`, que deixa de bater com o `approved_hash`. O comportamento está correto, mas dois agentes estranharam a diferença e a citaram como possível falha.

**Impacto.** Gera dúvida no agente e no usuário, e pode levar alguém a "consertar" algo que não está quebrado.

**Proposta.** Explicar a diferença na `SKILL.md` da `execute-task` (e no front-matter, se fizer sentido), ou excluir o "Registro de execução" do cálculo do hash.

**Referências.** `packages/harness/skills/default/execute-task/scripts/execute-task.sh`, `packages/harness/scripts/lib/task.sh` (`task_record_step`).

**Resolução.** Resolvido na SPEC-004 pela TASK-01 (commit `123a608`): o hash da tarefa ignora a seção "Registro de execução", e o `content_hash` continua igual ao `approved_hash` durante a execução. Uma edição à mão nessa seção deixou de ser detectada, risco aceito porque o registro é escrito só pelos scripts.

## DT-004: Commits da finalização sem data final

- **Área:** harness, `finalize-spec`
- **Encontrado em:** 2026-10-06, SPEC-003 TASK-12, finalização da SPEC-002
- **Prioridade:** baixa
- **Status:** resolvido

**Descrição.** As fontes da finalização listavam os commits desde o `approved_at` da spec, sem data final. O técnico da SPEC-002 trouxe commits da SPEC-003 que tocaram os mesmos caminhos.

**Impacto.** O documento técnico mostrava commits que não pertencem à spec, o que confunde quem o lê e exige revisão do agente na síntese.

**Proposta.** Limitar a lista ao `approved_at` mais antigo das specs aprovadas depois da finalizada, lidas de `refined/` e do arquivo. Sem outra spec aprovada depois, sem limite final.

**Resolução.** Resolvido na SPEC-004 pela TASK-07 (commit `123a608`): `finalize_next_approved_at` passa o limite ao `git log --until`.

**Referências.** `packages/harness/scripts/lib/finalize.sh` (`finalize_sources_commits`), `docs/specs/SPEC-002/tecnico.html`.

## DT-005: `--build` informa sucesso quando a geração falha

- **Área:** harness, `finalize-spec`
- **Encontrado em:** 2026-10-06, SPEC-003 TASK-12, finalização da SPEC-002
- **Prioridade:** média
- **Status:** resolvido

**Descrição.** O `finalize-spec.sh --build` imprimia a mensagem de sucesso mesmo quando um comando da geração falhava no meio. Na SPEC-002, o limite de tamanho de uma variável de ambiente fez o documento técnico sair quebrado, com mensagens de erro no terminal e a lista dos três documentos como se tudo estivesse certo.

**Impacto.** Um documento parcial ou quebrado podia ser conferido e publicado sem aviso.

**Proposta.** Gerar cada documento num arquivo temporário, terminar com erro se algum comando falhar ou escrever no stderr, e só então substituir o documento anterior.

**Resolução.** Resolvido na SPEC-004 pela TASK-08 (commit `123a608`): `finalize_generate` faz a geração num temporário e o `--build` para no primeiro documento com falha, mantendo os anteriores.

**Referências.** `packages/harness/scripts/lib/finalize.sh` (`finalize_generate`), `packages/harness/skills/default/finalize-spec/scripts/finalize-spec.sh`, commit `29f3fbd`.

## DT-006: Desfazer do `--archive` sem teste

- **Área:** harness, `finalize-spec`
- **Encontrado em:** 2026-10-06, SPEC-003 TASK-09
- **Prioridade:** média
- **Status:** resolvido

**Descrição.** Quando uma cópia do `--archive` falha, o `finalize_archive` remove o arquivo e os documentos publicados parciais e mantém a spec, as tarefas e o trabalho no lugar. Nenhum teste forçava essa falha, e o mutante que tirava o desfazer sobrevivia.

**Impacto.** Uma regressão no desfazer deixaria uma spec meio arquivada, sem que a suíte acusasse.

**Proposta.** Um teste que force a falha de uma cópia e confira que nada ficou pela metade, validado por mutação.

**Resolução.** Resolvido na SPEC-004 pela TASK-09 (commit `123a608`): o teste torna o destino `docs/specs` impossível de criar; sem a linha do desfazer, o caso novo falha.

**Referências.** `packages/harness/scripts/lib/finalize.sh` (`finalize_archive`), `packages/harness/tests/run-tests.sh`.

## Como registrar um débito

1. Use o próximo ID livre (`DT-NNN`) e acrescente uma linha ao resumo.
2. Crie a seção com área, origem (data e onde foi encontrado), prioridade, status, descrição, impacto, proposta e referências.
3. Ao resolver, mude o status para `resolvido` e cite o commit ou a tarefa; ao descartar, mude para `descartado` e diga o motivo. Não apague a seção.
