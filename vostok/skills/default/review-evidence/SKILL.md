---
name: review-evidence
description: "Trata de uma vez o lote de tarefas cujas evidências o usuário conferiu: monta o lote com vostok task reviews, pergunta uma única vez pelo que ficou sem decisão, finaliza em sequência as tarefas aceitas pelos passos da finish-task, corrige as recusadas pelos passos da execute-task e responde com a tabela do lote. Use quando o usuário avisar que conferiu evidências no painel, como \"conferi\", \"conferi a TASK-03 e a TASK-05\" ou \"conferi a SPEC-052\", para tratar o lote de tarefas de uma vez."
allowed-tools: Read Edit Write Glob Grep Bash
metadata:
  version: "1.0.0"
  author: "Victor Batalha"
  scripts: "vostok context load, vostok task reviews, vostok task evidence, vostok task status"
  templates: "templates/responses/question.md templates/responses/blocker.md templates/responses/completion.md"
---

# Conferir evidências em lote

Depois que o usuário confere as evidências de várias tarefas no painel e avisa no chat, esta skill trata o lote inteiro num pedido só: finaliza as tarefas aceitas, corrige as recusadas e pergunta uma única vez pelo que ficou sem decisão. Ela só orquestra: cada tarefa segue os passos da `finish-task` ou da `execute-task`.

## Persona

Esta skill atua como QA, carregada no passo 1 pelo `vostok context load`. Na correção de uma recusa (passo 6), a tarefa segue a persona do `vostok task brief`, como na `execute-task`. Ao terminar, inclusive numa parada, o chat volta à persona Coordenador; nenhum estado de persona ativa é gravado.

## Termos

- **Lote:** as tarefas tratadas neste pedido. Sem citação, as da spec em execução com status `in_progress`, `executed` ou `validating`. Com citação, só as tarefas ou as specs citadas.
- **Aceita:** tarefa do lote com todas as evidências que exigem aval conferidas, no painel ou no chat.
- **Recusada:** tarefa do lote com alguma evidência recusada.
- **Sem decisão:** tarefa do lote com alguma evidência ainda a conferir.
- **Fora do lote:** tarefa citada que não aguarda conferência (por exemplo, `done` ou `approved`).

## Regras

[review-evidence-01]: A conferência é sempre do usuário. Registre `--reviewed` ou `--rejected` só com a decisão dele dada na pergunta do passo 4, e só para as tarefas e evidências que a resposta citar (RN-01).
[review-evidence-02]: O que o usuário já decidiu no painel vale sem nova pergunta. Nunca registre de novo pelo chat uma evidência que o painel já conferiu ou recusou (RN-02).
[review-evidence-03]: O aviso "conferi" vale como pedido de finalizar as tarefas aceitas e de corrigir as recusadas do lote, sem nova pergunta (RN-03).
[review-evidence-04]: Uma tarefa bloqueada (falha de teste, regressão ou critério sem comprovação) para só ela: as demais do lote seguem, e o bloqueio vai para a resposta (RN-04).
[review-evidence-05]: Os passos e as regras da `finish-task` e da `execute-task` valem dentro do lote. Esta skill só os orquestra e não pula nenhum deles (RN-05).
[review-evidence-06]: Não faça commit nem stage, e não finalize a spec. Commit e finalização da spec são pedidos separados do usuário.

## Checklist

Copie e marque o progresso:

```
- [ ] 1. Carregar o contexto
- [ ] 2. Montar o lote
- [ ] 3. Separar as tarefas
- [ ] 4. Perguntar pelo que ficou sem decisão
- [ ] 5. Finalizar as aceitas
- [ ] 6. Corrigir as recusadas
- [ ] 7. Responder ao usuário
```

## Passos

### 1. Carregar o contexto

Execute:

```bash
vostok context load --skill review-evidence --persona qa
```

### 2. Montar o lote

Execute, com a spec e as tarefas citadas, se houver:

```bash
vostok task reviews [SPEC_ID] [TASK_ID...]
```

O comando imprime uma tabela por spec, com uma linha por tarefa: status, evidências a conferir, aceitas (com a origem) e recusadas (com a origem e o motivo). Ele já considera as decisões feitas no painel.

- Código de saída 2: há mais de uma spec em execução e nenhuma foi citada. **PARE** e pergunte qual, com o template `question`, uma opção por spec da lista do comando (FA-01).
- `Nenhuma tarefa aguardando conferência.`: informe isso numa resposta curta e encerre (FA-02).

### 3. Separar as tarefas

Com a tabela do passo 2, separe as tarefas do lote:

- **Aceita:** coluna "A conferir" e coluna "Recusadas" com `-`.
- **Recusada:** coluna "Recusadas" com algum item. Uma tarefa com recusa e evidência a conferir é recusada e também entra na pergunta do passo 4 (CB-02).
- **Sem decisão:** coluna "A conferir" com algum item e "Recusadas" com `-`.
- **Fora do lote:** tarefa citada com status fora de `in_progress`, `executed` e `validating`. Não a trate e cite-a na resposta com o status (CB-01).

### 4. Perguntar pelo que ficou sem decisão

Sem evidência a conferir no lote, pule para o passo 5.

Faça uma única pergunta, com o template `question`, só pelas evidências da coluna "A conferir", agrupadas por tarefa no formato da `execute-task`: um bloco por tarefa (`TASK-NN — título`) e, em cada bloco, cada evidência pelo ID e pelo título (`EV-02 Menu da conta aberto a 1280 px`), com o que olhar nela (o subitem `Trecho`). Não anexe nem cole o arquivo: o usuário confere no painel.

Com a resposta:

- Evidência aceita: `vostok task evidence <SPEC_ID> <TASK_ID> --reviewed <EV-NN> --motivo "<palavras do usuário>"`.
- Evidência recusada: `vostok task evidence <SPEC_ID> <TASK_ID> --rejected <EV-NN> --motivo "<motivo do usuário>"`. A tarefa passa a ser recusada.
- O usuário diz que conferiu no painel: rode o `vostok task reviews` do passo 2 de novo e separe as tarefas outra vez, sem registrar nada.
- Aval genérico ("pode marcar", "tudo certo"): confirme com o template `question` a quais tarefas e evidências ele vale antes de registrar, como na `execute-task`.
- Tarefa que a resposta não citou: fica como `aguardando decisão` e não é tratada neste pedido (CB-04).

### 5. Finalizar as aceitas

Finalize cada tarefa aceita, uma por vez, na ordem dos IDs, pelos passos 3 a 8 da `finish-task`: testes, regressão, evidências, persistência, critérios, documentos da tarefa e conclusão.

- Espera de fechamento (`closes_after`): se a tarefa esperada está no lote, finalize-a antes. Se ela não está no lote ou não pode ser finalizada, a tarefa que espera fica como `aguardando decisão`, com a tarefa esperada no motivo (CB-03).
- Tarefa que bloqueia (falha nos testes depois do máximo de tentativas, regressão ou critério sem comprovação): pare só ela, guarde o bloqueio para a resposta e siga para a próxima (RN-04, FA-03). Não retome nem aceite diferença sem a decisão do usuário.
- Ao concluir, cite as tarefas que o `vostok task finish` desbloqueou.

### 6. Corrigir as recusadas

Para cada tarefa recusada, na ordem dos IDs:

1. Leia o motivo de cada recusa na coluna "Recusadas" (ou no `conferencias.md` da tarefa).
2. Corrija o que o motivo aponta, pelos passos da `execute-task`: siga o `vostok task brief <SPEC_ID> <TASK_ID>` e mude só os arquivos de "Mudar".
3. Gere de novo as evidências com `vostok task evidence <SPEC_ID> <TASK_ID> --antecipada`. A tarefa fica `corrigida, a conferir de novo`.

Correção que exige mudar a tarefa (passos, verificação, evidências) ou a spec: pare só essa tarefa, sem ajuste por conta própria, e cite a recusa na resposta como `bloqueada`, sugerindo a skill `adjust-task` ou `adjust-spec` (FA-04).

### 7. Responder ao usuário

Execute `vostok commit check` e responda com o template `completion`:

- **O que foi feito:** a [tabela do lote](#tabela-do-lote), com uma linha por tarefa, inclusive as que ficaram fora do lote.
- **Verificação:** os comandos de cada tarefa finalizada (testes, regressão, evidências e conclusão) e o resultado, e as evidências geradas de novo nas corrigidas.
- **Pendências:** os bloqueios, com o comando que falhou, e as evidências a conferir de novo, pelo ID e pelo título.
- **Status das tarefas:** a saída de `vostok task status <SPEC_ID>` de cada spec envolvida, sem alteração.

Esta resposta encerra a persona da skill: o chat volta à persona Coordenador.

## Formatos de resposta

Respostas ao usuário usam os templates de `templates/responses/`. Formatos próprios desta skill:

### Tabela do lote

```markdown
| Tarefa | Resultado | Motivo |
|---|---|---|
| SPEC-052 TASK-02 | finalizada | - |
| SPEC-052 TASK-04 | corrigida, a conferir de novo | EV-02 recusada no painel: cor errada |
| SPEC-052 TASK-05 | bloqueada | regressão no comando [2] do baseline |
| SPEC-052 TASK-06 | aguardando decisão | EV-01 sem resposta |
| SPEC-052 TASK-01 | fora do lote | status done |
```
