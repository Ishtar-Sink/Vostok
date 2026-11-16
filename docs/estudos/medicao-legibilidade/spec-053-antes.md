---
id: SPEC-053
title: "Conferência de evidências em lote pelo chat"
status: refined
author: "Victor Batalha"
created_at: 2026-11-16T10:39:51Z
updated_at: 2026-11-16T10:44:50Z
tags: []
type: feature
source: "draft/SPEC-053-conferencia-de-evidencias-em-lote-pelo-chat.md"
refined_at: 2026-11-16T10:44:50Z
adjustments: 0
adjusted_at: null
content_hash: 09a240451c63884892f83ecc88aa2fae3287383d8235b01cd09ee7900811da85
---

# Conferência de evidências em lote pelo chat - Especificação Refinada

Skill nova `review-evidence`: o usuário confere as evidências de várias tarefas no painel, avisa no chat só que conferiu, e o agente percorre o lote, finaliza as tarefas aceitas, corrige as recusadas e pergunta só pelo que ficou sem decisão.

## Objetivo

Hoje, depois de conferir no painel as evidências de várias tarefas (por exemplo, de 9), o usuário volta ao chat e trata cada tarefa num pedido, porque a `finish-task` finaliza uma tarefa por pedido. O resultado esperado é uma skill em que basta dizer "conferi" (ou citar as tarefas) para o agente tratar o lote inteiro: finalizar as aceitas, corrigir as recusadas e perguntar uma única vez pelo que falta.

## Escopo

### Dentro do escopo

- Skill `review-evidence` em `packages/harness/skills/default/review-evidence/`, com entrada no catálogo e no mapa de contexto.
- Lote padrão: as tarefas da spec em execução que aguardam conferência; ou as tarefas e specs citadas pelo usuário.
- Leitura das decisões já tomadas no painel (`conferencias.md`), sem registrá-las de novo.
- Uma única pergunta pelas evidências ainda sem decisão, agrupadas por tarefa.
- Finalização, em sequência, de cada tarefa com todas as evidências aceitas, pelos passos da `finish-task`.
- Correção, em seguida, de cada tarefa com evidência recusada, pelos passos da `execute-task`, com nova geração das evidências.
- Subcomando `vostok task reviews` que lista as tarefas pendentes de conferência e o estado de cada evidência.

### Fora do escopo

- Mudar o painel ou as rotas de conferência (DEC-007).
- Conferir evidências pelo agente: a conferência continua sendo do usuário.
- Executar tarefas aprovadas que ainda não começaram.
- Finalizar a spec (`finalize-spec`) ao fim do lote.
- Fazer commit das tarefas finalizadas.

## Requisitos funcionais

| ID | Requisito |
|---|---|
| RF-01 | O sistema deve ter a skill `review-evidence`, acionada quando o usuário avisa que conferiu evidências, com ou sem citar tarefas. |
| RF-02 | Sem tarefas citadas, a skill deve montar o lote com as tarefas da spec em execução que estão `executed` ou `in_progress` com evidência a conferir ou recusada. |
| RF-03 | Com tarefas ou specs citadas, a skill deve limitar o lote a elas. |
| RF-04 | O comando `vostok task reviews` deve listar, para uma spec ou para as tarefas citadas, cada tarefa com o status e as evidências a conferir, aceitas e recusadas, com a origem (chat ou painel). |
| RF-05 | Sem spec informada, o `vostok task reviews` deve usar a spec em execução e, havendo mais de uma, listar as candidatas e recusar. |
| RF-06 | A skill deve perguntar, numa única mensagem, só pelas evidências do lote ainda sem decisão, agrupadas por tarefa. |
| RF-07 | A skill deve finalizar, em sequência, cada tarefa do lote com todas as evidências aceitas, seguindo os passos da `finish-task`. |
| RF-08 | A skill deve corrigir, em seguida, cada tarefa com evidência recusada, seguindo o motivo da recusa pelos passos da `execute-task`, e gerar de novo as evidências dela. |
| RF-09 | A skill deve responder com uma tabela do lote: tarefa, resultado (finalizada, corrigida e a conferir de novo, bloqueada ou aguardando decisão) e o motivo. |

## Requisitos não funcionais

| ID | Requisito | Como verificar |
|---|---|---|
| RNF-01 | Economia: o lote é montado por um único comando, sem o agente abrir tarefa por tarefa. | Teste do `vostok task reviews` com várias tarefas numa saída só. |
| RNF-02 | Compatibilidade: as tarefas no arquivo único e as em pasta são listadas. | Testes com os dois formatos de tarefa. |
| RNF-03 | Portabilidade: a skill segue o formato padrão e ganha os redirecionamentos de Claude Code, Codex e Cursor pelo `vostok init` e `vostok update`. | Teste do `init` com a skill nova nos três destinos. |

## Regras de negócio

| ID | Regra |
|---|---|
| RN-01 | A conferência é sempre do usuário: a skill nunca registra `--reviewed` ou `--rejected` sem a decisão dele no painel ou no chat (finish-task-09). |
| RN-02 | Evidência já decidida no painel não é registrada de novo pelo chat (RF-23 da SPEC-051). |
| RN-03 | O aviso "conferi" vale como pedido de finalizar as tarefas aceitas e de corrigir as recusadas do lote, sem nova pergunta. |
| RN-04 | Uma tarefa bloqueada (falha de teste ou regressão) para só ela: as demais do lote seguem, e o bloqueio vai para a resposta. |
| RN-05 | Os passos da `finish-task` e da `execute-task` valem dentro do lote; a skill só os orquestra. |

## Fluxos

### Fluxo principal

1. O usuário confere as evidências no painel e avisa no chat ("conferi", "conferi a TASK-03 e a TASK-05" ou "conferi a SPEC-052").
2. A skill carrega o contexto e roda `vostok task reviews`, com as tarefas ou a spec citadas, se houver.
3. A skill separa o lote em aceitas, recusadas e com evidência sem decisão.
4. Havendo evidência sem decisão, a skill faz uma única pergunta por elas e registra as respostas com `--reviewed` ou `--rejected`.
5. A skill finaliza cada tarefa aceita, em sequência, pelos passos da `finish-task`.
6. A skill corrige cada tarefa recusada pelos passos da `execute-task` e gera de novo as evidências com `--antecipada`.
7. A skill responde com a tabela do lote, as evidências a conferir de novo e o `vostok task status` das specs envolvidas.

### Fluxos alternativos

- FA-01 (passo 2): mais de uma spec em execução sem citação: a skill pergunta qual, com uma opção por spec.
- FA-02 (passo 2): nenhuma tarefa aguardando conferência: a skill informa e encerra.
- FA-03 (passo 5): uma tarefa bloqueia na finalização: a skill registra o bloqueio e segue com as demais (RN-04).
- FA-04 (passo 6): a correção de uma recusa exige mudar a tarefa ou a spec: a skill para só essa tarefa e cita a recusa na resposta, sem ajuste por conta própria.

## Casos de borda

| ID | Situação | Comportamento esperado | Requisitos |
|---|---|---|---|
| CB-01 | Tarefa citada que não aguarda conferência (done ou approved) | Fica fora do lote e aparece na resposta com o status | RF-03, RF-09 |
| CB-02 | Evidência recusada e outra sem decisão na mesma tarefa | Pergunta pela sem decisão; a tarefa vai para a correção | RF-06, RF-08 |
| CB-03 | Tarefa com espera de fechamento (`closes_after`) ainda não concluída | Finaliza antes a tarefa esperada se ela estiver no lote; senão fica aguardando | RF-07 |
| CB-04 | Usuário responde a pergunta só para parte das tarefas | As citadas seguem; as outras ficam como aguardando decisão | RF-06, RF-09 |
| CB-05 | Tarefa no formato de arquivo único | Listada e tratada como as em pasta | RF-04, RNF-02 |

## Contratos e interfaces

Novo subcomando:

```
vostok task reviews [SPEC_ID] [TASK_ID...]
```

Saída, uma tabela por spec:

```
### Conferências da SPEC-052

| Tarefa | Status | A conferir | Aceitas | Recusadas |
|---|---|---|---|---|
| TASK-02 | executed | - | EV-01 (painel), EV-02 (painel) | - |
| TASK-04 | in_progress | EV-03 | EV-01 (chat) | EV-02 (painel): motivo |
```

Sem `SPEC_ID` e com mais de uma spec em execução, sai com código 2 e lista as specs candidatas. Spec em execução: status `planned`, com ao menos uma tarefa `in_progress`, `executed` ou `validating`.

Skill nova, no formato das demais:

```
packages/harness/skills/default/review-evidence/SKILL.md
  name: review-evidence
  scripts: vostok context load, vostok task reviews, vostok task evidence, vostok task status
```

## Componentes afetados

| Caminho | Ação | Motivo |
|---|---|---|
| `packages/harness/skills/default/review-evidence/SKILL.md` | criar | Skill do lote (RF-01 a RF-03, RF-06 a RF-09). |
| `packages/harness/context/skill-catalog.md` | alterar | Entrada da skill no catálogo. |
| `packages/harness/config/context.yml` | alterar | Regras e personas carregadas pela skill. |
| `packages/harness/skills/default/finish-task/SKILL.md` | alterar | A regra de uma tarefa por pedido passa a aceitar o lote da `review-evidence`. |
| `packages/harness/skills/default/execute-task/SKILL.md` | alterar | Indica a `review-evidence` quando o usuário volta ao chat depois de conferir no painel. |
| `apps/cli/src/commands/task.rs` | alterar | Subcomando `reviews` (RF-04, RF-05). |
| `apps/cli/tests/task.rs` | alterar | Testes do `vostok task reviews`. |
| `apps/cli/tests/init_update.rs` | alterar | A skill nova chega aos redirecionamentos (RNF-03). |

## Premissas e dependências

- As decisões do painel ficam no `conferencias.md` de cada tarefa, como na SPEC-051, e são lidas por `review_states` em `apps/cli/src/commands/task.rs`.
- O `vostok init` e o `vostok update` geram os redirecionamentos das skills de `packages/harness/skills/default/`.
- A SPEC-052 está concluída; o lote não depende dela.

## Riscos

| Risco | Impacto | Mitigação |
|---|---|---|
| Um lote grande consome muito contexto numa só resposta | médio | Finalização em sequência, com a saída de cada comando resumida na tabela do lote. |
| A correção automática de uma recusa faz algo diferente do que o usuário quis | médio | A correção segue só o motivo gravado; se ele exigir ajuste da tarefa, para só essa tarefa (FA-04). |
| A skill registra uma conferência que o usuário não deu | alto | Só registra respostas dadas na pergunta do passo 4 (RN-01, RN-02). |

## Casos de teste

| ID | Cenário | Passos | Resultado esperado | Requisitos |
|---|---|---|---|---|
| CT-01 | Lote sem citação | Três tarefas `executed` da spec em execução, todas aceitas no painel; avisar "conferi" | As três finalizadas em sequência e a tabela com as três | RF-01, RF-02, RF-07, RF-09 |
| CT-02 | Lote citado | Avisar "conferi a TASK-02" com duas tarefas pendentes | Só a TASK-02 é tratada | RF-03 |
| CT-03 | Listagem do comando | `vostok task reviews SPEC-NNN` com tarefas aceitas, recusadas e a conferir | Tabela com as três colunas e a origem de cada decisão | RF-04, RNF-01 |
| CT-04 | Spec ambígua | `vostok task reviews` com duas specs em execução | Código 2 e as duas specs listadas | RF-05 |
| CT-05 | Evidência sem decisão | Uma tarefa com EV-02 sem decisão | Uma única pergunta, só pela EV-02 | RF-06 |
| CT-06 | Recusa no painel | Uma tarefa com EV-01 recusada | A tarefa é corrigida pelo motivo e as evidências geradas de novo | RF-08 |
| CT-07 | Bloqueio no lote | Uma tarefa falha nos testes | Ela fica bloqueada e as demais são finalizadas | RF-07, RF-09 |
| CT-08 | Formato antigo | Tarefa em arquivo único com evidência a conferir | Aparece na listagem | RF-04, RNF-02 |
| CT-09 | Redirecionamentos | `vostok init` num projeto novo | `review-evidence` nos três destinos | RF-01, RNF-03 |

## Critérios de aceite

- [ ] CA-01: Depois de conferir no painel, dizer só "conferi" faz o agente tratar todas as tarefas da spec em execução que aguardavam conferência (RF-01, RF-02, RF-07).
- [ ] CA-02: Citar tarefas ou uma spec limita o lote a elas (RF-03).
- [ ] CA-03: O `vostok task reviews` lista numa saída o estado das conferências de cada tarefa (RF-04, RF-05).
- [ ] CA-04: O agente pergunta uma única vez, só pelas evidências sem decisão (RF-06).
- [ ] CA-05: As recusadas são corrigidas e as aceitas finalizadas, e a resposta traz a tabela do lote (RF-08, RF-09).

## Referências

| Tipo | Referência | Uso |
|---|---|---|
| arquivo | `vostok/skills/default/finish-task/SKILL.md` | Passo 5 (conferência e painel) e regra de uma tarefa por pedido. |
| arquivo | `vostok/skills/default/execute-task/SKILL.md` | Passo 7, volta ao chat depois de conferir no painel. |
| arquivo | `apps/cli/src/commands/task.rs` | `TaskCommand`, `REVIEWS_FILE` e `review_states`. |
| arquivo | `packages/harness/context/skill-catalog.md` | Formato da entrada de skill. |
| arquivo | `packages/harness/config/context.yml` | Mapa de contexto por skill. |
| memória | DEC-007 | Escrita do painel restrita à conferência e às configurações. |
| memória | DEC-006 | Origem das conferências feitas no painel. |

## O que mudou

| # | Assunto | Estado no Original | Estado Refinado |
|---|---|---|---|
| 1 | Ações sobre o lote | Só "iterar" sobre as tarefas | Finaliza as aceitas e corrige as recusadas |
| 2 | Lote padrão | Tarefas aguardando conferência | Só as da spec em execução |
| 3 | Evidência sem decisão | Não tratado | Uma única pergunta por elas |
| 4 | Comando da CLI | Não citado | `vostok task reviews` |

## Registro de decisões

| # | Decisão | Alternativa descartada | Motivo |
|---|---|---|---|
| 1 | Finalizar cada tarefa aceita | Só mostrar o resultado | resposta 1 |
| 2 | Perguntar só pelas evidências sem decisão | Tratar "conferi" como aceite | resposta 2 |
| 3 | Corrigir as recusadas em seguida | Listar e perguntar | resposta 3 |
| 4 | Lote padrão é a spec em execução | Todas as specs | resposta 4 |
| 5 | Nome `review-evidence` | Outro nome | inferência A |
| 6 | Subcomando `vostok task reviews` | Agente abrir tarefa por tarefa | inferência B |
| 7 | Não registrar de novo o que o painel decidiu | Registrar no chat também | inferência C |
| 8 | Lote por tarefa, spec ou "todas" | Só por tarefa | inferência D |

## Pendências

| # | Pendência | Opções em aberto |
|---|---|---|
| - | Nenhuma | - |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
