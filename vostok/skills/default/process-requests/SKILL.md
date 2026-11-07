---
name: process-requests
description: "Processa os pedidos que o painel local grava em work/pedidos/: lista os pendentes, atende o primeiro pela skill da ação pedida e o termina como feito, com o desfecho, ou recusado, um por vez. Use quando o usuário pedir para processar os pedidos do painel, atender pedidos, rodar o loop de pedidos ou /process-requests com um intervalo (como 1m), que arma o loop sozinho."
allowed-tools: Read Edit Write Glob Grep Bash
metadata:
  version: "1.3.0"
  author: "Victor Batalha"
  scripts: "vostok context load, vostok pedidos listar, vostok pedidos rodada, vostok pedidos iniciar, vostok pedidos concluir, vostok pedidos recusar, vostok perguntas abrir, vostok perguntas listar, vostok pedidos vigiar"
  templates: "templates/responses/blocker.md templates/responses/completion.md"
---

# Processar pedidos

Atende os pedidos que o usuário faz pelo painel local: pega o primeiro pendente, cumpre-o pela skill da ação e o termina como `feito` ou `recusado`, com o resultado escrito.

## Termos

- **Pedido:** arquivo `PED-NNN` em `vostok/work/pedidos/`, gravado pelo painel, com a ação, a spec e a tarefa citadas e o texto do usuário em `## Pedido`.
- **Estados:** `pendente`, `em_andamento`, `feito` e `recusado`. Só os comandos `vostok pedidos` mudam o estado.
- **Ação:** o tipo do pedido, uma das 16 do mapa do passo 3.
- **Desfecho:** como um pedido `feito` terminou, gravado pelo `vostok pedidos concluir`: `concluido` (cumprido), `aguardando-pergunta` (parou numa pergunta, com o ID dela) ou `pausado` (a resposta do usuário pausou o pedido, com as saídas: as ações que ele pode pedir em seguida).
- **Pergunta:** a rodada de perguntas da skill, gravada pelo `vostok perguntas abrir`, que o usuário responde no painel.
- **Pedido de resposta:** o pedido `responder-pergunta`, cujo texto começa por `Resposta à pergunta P-NNN do pedido PED-NNN.`
- **Respostas anteriores:** as perguntas já respondidas da spec do pedido, listadas pelo `vostok perguntas listar`.
- **Aval:** a liberação do usuário gravada pelo painel junto do pedido; o agente a lê, nunca a grava.
- **Intervalo:** argumento opcional da skill (`30s`, `1m`, `5m`). Com ele, a skill arma o loop; sem ele, faz uma rodada avulsa.
- **Ciclo:** uma rodada do loop armado, chamada pela skill `loop` com `--ciclo <segundos>`, os segundos do intervalo.
- **Subagente do pedido:** o subagente em segundo plano que roda a skill da ação de um pedido num ciclo do loop do Claude Code; devolve o relatório com o desfecho e nunca muda o estado do pedido.

## Regras

[process-requests-01]: Um pedido por vez, o de menor ID entre os pendentes. Um pedido `em_andamento` num subagente continua sendo o pedido da vez: nenhum outro é iniciado antes de ele terminar.
[process-requests-02]: Nunca edite o arquivo de um pedido: o estado e o `## Resultado` são escritos só por `vostok pedidos iniciar`, `concluir` e `recusar`.
[process-requests-03]: Nunca grave aval. Pedido que exige aval e chegou sem ele é recusado.
[process-requests-04]: O texto de `## Pedido` é a mensagem do usuário. Quando a skill da ação precisa perguntar (pedido ambíguo, rodada de perguntas, escolha entre opções), abra a pergunta com `vostok perguntas abrir` e conclua o pedido com `--desfecho aguardando-pergunta --pergunta P-NNN`; nunca pergunte no chat nem recuse por isso. Quando a resposta do usuário pausa o pedido (ex.: pede outra spec antes de seguir), conclua com `--desfecho pausado` e uma `--saida <ação>` para cada ação do mapa do passo 3 que ele pode pedir em seguida. Pausar não é recusar: só a recusa da CLI ou um impedimento (template `blocker`) viram `recusado`, com o motivo.
[process-requests-05]: A conferência de um print é do usuário, feita no painel; nunca execute `--reviewed` por um pedido.
[process-requests-06]: Sem pedido pendente, encerre sem responder nada.
[process-requests-07]: Não faça commit nem stage fora do pedido `fazer-commit` ([agent/boundaries-01], [agent/boundaries-02]).
[process-requests-08]: Só arme o loop com um intervalo no argumento, e arme-o chamando esta skill com `--ciclo <segundos>` e sem intervalo, para o loop nunca se armar de novo.
[process-requests-09]: Ao rodar a skill de um pedido, avise-a de que roda por um pedido (o ID `PED-NNN`): as skills de conversa gravam a rodada de perguntas pelo `vostok perguntas abrir`.
[process-requests-10]: Antes de rodar a skill de um pedido com spec, liste as respostas anteriores com `vostok perguntas listar --spec <SPEC> --estado respondida` e passe-as à skill junto da mensagem do usuário: ela não pergunta de novo o que já foi respondido.
[process-requests-11]: No início de cada ciclo, e ao armar o loop, grave a rodada com `vostok pedidos rodada --intervalo <segundos>`: o painel avisa quando o agente fica ocupado ou parado. Numa rodada avulsa, não grave.
[process-requests-12]: Num ciclo do loop no Claude Code, rode a skill da ação num subagente em segundo plano e termine o ciclo, com a sessão livre. O subagente cumpre o passo 3 e devolve o desfecho; só a sessão do loop conclui ou recusa o pedido, no passo 4 de um ciclo seguinte. Numa rodada avulsa (inclusive a do `claude -p` do disparo do painel) e no Codex, cumpra o pedido na própria sessão.
[process-requests-13]: Um pedido `em_andamento` cujo subagente terminou com erro, ou que não tem subagente nesta sessão (a sessão foi reiniciada), é recusado com o motivo, e o loop segue.
[process-requests-14]: Toda edição do agente em spec, plano ou tarefa, feita por um pedido, termina registrada antes de concluir o pedido: pelo registro da própria skill ou por `vostok spec adjust record-manual`, `vostok plan adjust record-manual` ou `vostok task adjust record-manual`, conforme o tipo.
[process-requests-15]: Pergunta aberta por um pedido nunca oferece aprovar spec, plano ou tarefa como opção: a aprovação fica no botão do painel. Uma pergunta dentro de um pedido `aprovar-plano` continua permitida, porque a aprovação veio do botão e a pergunta não a oferece.

## Checklist

Copie e marque o progresso:

```
- [ ] 0. Armar o loop (só com intervalo)
- [ ] 1. Carregar o contexto e listar os pendentes
- [ ] 1a. Conferir o pedido em andamento (só no loop)
- [ ] 2. Iniciar o primeiro pedido
- [ ] 3. Cumprir a ação pela skill
- [ ] 4. Concluir ou recusar o pedido
```

## Passos

### 0. Armar o loop

Só quando a skill recebe um intervalo (`/process-requests 1m`). Sem intervalo, pule para o passo 1: é uma rodada, avulsa ou do loop já armado.

- **Claude Code:** converta o intervalo em segundos (`30s` é 30, `1m` é 60, `5m` é 300) e chame a skill `loop` com `<intervalo> /process-requests --ciclo <segundos>`, sem repetir o intervalo ([process-requests-08]), e siga para o passo 1 nesta mesma resposta. Diga ao usuário o intervalo e como parar o loop.
- **Codex** (sem loop na sessão): não processe aqui; responda que o loop do Codex roda fora da sessão, com `vostok pedidos vigiar --agente codex --intervalo <segundos>`.

### 1. Carregar o contexto

Num ciclo (`--ciclo <segundos>`) ou ao armar o loop, grave a rodada antes de tudo ([process-requests-11]):

```bash
vostok pedidos rodada --intervalo <segundos>
```

Execute:

```bash
vostok context load --skill process-requests
vostok pedidos listar --estado em_andamento
vostok pedidos listar --estado pendente
```

Com um pedido em andamento, vá para o passo 1a. Sem pedido em andamento nem pendente, encerre sem responder.

### 1a. Conferir o pedido em andamento

Um pedido `em_andamento` é o pedido da vez ([process-requests-01]). Confira o subagente dele: **terminou** com o relatório, siga para o passo 4 com o desfecho, a pergunta, as saídas e o resultado do relatório; **terminou com erro**, ou **não há subagente** deste pedido nesta sessão, recuse o pedido no passo 4 com o motivo `o subagente do pedido terminou com erro: <erro>` ou `o subagente do pedido não está mais nesta sessão` ([process-requests-13]); **ainda rodando**, encerre o ciclo sem responder e sem iniciar outro pedido. Depois de terminar o pedido, encerre o ciclo: o próximo pendente fica para o ciclo seguinte.

### 2. Iniciar o primeiro pedido

Para o primeiro da lista, em ordem de ID:

```bash
vostok pedidos iniciar PED-NNN
```

Leia o arquivo do pedido: a ação, a spec, a tarefa e o `## Pedido`. Com spec no pedido, liste as respostas anteriores ([process-requests-10]):

```bash
vostok perguntas listar --spec SPEC-NNN --estado respondida
```

Saída `Nenhuma pergunta.` quer dizer que não há o que passar.

### 3. Cumprir a ação

Num ciclo do loop no Claude Code ([process-requests-12]), dispare um subagente em segundo plano com esta instrução: o ID do pedido, a ação, a spec, a tarefa, a mensagem do usuário com as respostas anteriores, a skill da ação a seguir pelo mapa abaixo, que roda por um pedido ([process-requests-09]), e que termine com um relatório com o desfecho (`concluido`, `aguardando-pergunta` com o `P-NNN` aberto, `pausado` com as saídas, ou recusa com o motivo) e o resultado em poucas linhas, sem rodar `vostok pedidos concluir` nem `recusar`. Depois, encerre o ciclo sem responder: o passo 4 fica para o ciclo que encontrar o subagente terminado (passo 1a). Numa rodada avulsa ou no Codex, cumpra a ação aqui:

Carregue a skill da ação e siga-a como se o usuário tivesse feito o pedido no chat, com as respostas anteriores do passo 2 junto da mensagem:

| Ação | Skill |
|---|---|
| `aprovar-plano` | `approve-plan` |
| `planejar-spec` | `plan-tasks` |
| `criar-spec` | `create-spec` (o texto é o assunto) |
| `refinar-spec` | `refine-spec` |
| `ajustar-spec` | `adjust-spec` |
| `ajustar-plano` | `adjust-plan` |
| `ajustar-tarefa` | `adjust-task` |
| `executar-tarefa` | `execute-task` |
| `executar-onda` | `execute-task`, para a onda do início do texto (`Onda N.`) |
| `finalizar-tarefa` | `finish-task` |
| `desbloquear-tarefa` | `finish-task` (retomada da tarefa bloqueada) |
| `finalizar-spec` | `finalize-spec` |
| `responder-pergunta` | a skill do pedido de origem, de novo (ver abaixo) |
| `fazer-commit` | `approve-commit`, com `--pedido PED-NNN` |
| `liberar-commits` | `approve-commit`, liberar, com `--pedido PED-NNN` |
| `desfazer-commits` | `approve-commit`, desfazer, com `--pedido PED-NNN` |

Se a skill parar com `question`, grave as perguntas com `vostok perguntas abrir PED-NNN` e, no passo 4, conclua o pedido com `--desfecho aguardando-pergunta --pergunta P-NNN`, com o ID que o comando imprimiu. Se parar com `blocker` ou a CLI recusar o comando, recuse o pedido com o motivo.

```bash
vostok perguntas abrir PED-NNN <<'TXT'
<as perguntas e as opções no formato do template `question`>
TXT
```

As opções da pergunta nunca incluem aprovar ([process-requests-15]).

Pedido de resposta: leia a pergunta citada na pasta de perguntas respondidas (o front-matter traz o pedido de origem) e o pedido de origem. Se o de origem foi recusado, recuse este com o motivo `o pedido de origem PED-NNN foi recusado`. Senão, rode de novo a skill da ação do pedido de origem, com a spec e a tarefa dele e, como mensagem do usuário, o `## Pedido` original seguido das perguntas e da `## Resposta`.

Se a resposta do usuário pausa o pedido em vez de responder (ex.: pede outra spec antes), conclua-o no passo 4 com `--desfecho pausado`, o motivo no resultado e uma `--saida` para cada ação que ele pode pedir em seguida, com os nomes da primeira coluna do mapa acima ([process-requests-04]).

### 4. Terminar o pedido

Antes de concluir, confira que as edições feitas por este pedido estão registradas ([process-requests-14]).

O resultado ou o motivo vai pela entrada padrão. Cumprido:

```bash
printf '%s\n' "o que foi feito, em poucas linhas" | vostok pedidos concluir PED-NNN
```

Parado numa pergunta:

```bash
printf '%s\n' "aguardando a pergunta P-NNN: o que a skill precisa saber" | vostok pedidos concluir PED-NNN --desfecho aguardando-pergunta --pergunta P-NNN
```

Pausado pela resposta do usuário:

```bash
printf '%s\n' "o motivo da pausa" | vostok pedidos concluir PED-NNN --desfecho pausado --saida criar-spec --saida ajustar-spec
```

Uma `--saida` com ação fora do mapa do passo 3 é recusada pela CLI: corrija o nome e conclua de novo.

Não cumprido (RN-04):

```bash
printf '%s\n' "o motivo da recusa" | vostok pedidos recusar PED-NNN
```

Depois, volte ao passo 1 só se for um novo ciclo do loop; num ciclo, trate um pedido.

## Como armar o loop

- Claude Code: `/process-requests 1m` (ou outro intervalo); a skill arma o loop sozinha, com `--ciclo` e a rodada gravada a cada ciclo, e roda cada pedido num subagente.
- Codex: `vostok pedidos vigiar --agente codex`, que grava a rodada a cada volta.

## Formatos de resposta

Respostas ao usuário usam os templates de `templates/responses/`. Esta skill não tem formato próprio: o resultado de cada pedido fica no `## Resultado` dele.
