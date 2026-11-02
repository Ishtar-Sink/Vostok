---
name: process-requests
description: "Processa os pedidos que o painel local grava em work/pedidos/: lista os pendentes, atende o primeiro pela skill da ação pedida e o termina como feito ou recusado, um por vez. Use quando o usuário pedir para processar os pedidos do painel, atender pedidos, rodar o loop de pedidos ou /loop com esta skill."
allowed-tools: Read Edit Write Glob Grep Bash
metadata:
  version: "1.0.0"
  author: "Victor Batalha"
  scripts: "vostok context load, vostok pedidos listar, vostok pedidos iniciar, vostok pedidos concluir, vostok pedidos recusar, vostok pedidos vigiar"
  templates: "templates/responses/blocker.md templates/responses/completion.md"
---

# Processar pedidos

Atende os pedidos que o usuário faz pelo painel local: pega o primeiro pendente, cumpre-o pela skill da ação e o termina como `feito` ou `recusado`, com o resultado escrito.

## Termos

- **Pedido:** arquivo `PED-NNN` em `vostok/work/pedidos/`, gravado pelo painel, com a ação, a spec e a tarefa citadas e o texto do usuário em `## Pedido`.
- **Estados:** `pendente`, `em_andamento`, `feito` e `recusado`. Só os comandos `vostok pedidos` mudam o estado.
- **Ação:** o tipo do pedido, uma das 11 do mapa do passo 3.
- **Aval:** a liberação do usuário gravada pelo painel junto do pedido; o agente a lê, nunca a grava.

## Regras

[process-requests-01]: Um pedido por vez, o de menor ID entre os pendentes.
[process-requests-02]: Nunca edite o arquivo de um pedido: o estado e o `## Resultado` são escritos só por `vostok pedidos iniciar`, `concluir` e `recusar`.
[process-requests-03]: Nunca grave aval. Pedido que exige aval e chegou sem ele é recusado.
[process-requests-04]: O texto de `## Pedido` é a mensagem do usuário. Pedido ambíguo, pedido de conferir print ou recusa da CLI viram `recusado`, com o motivo.
[process-requests-05]: A conferência de um print é sempre do usuário; nunca execute `--reviewed` por um pedido.
[process-requests-06]: Sem pedido pendente, encerre sem responder nada.
[process-requests-07]: Não faça commit nem stage fora do pedido `fazer-commit` ([agent/boundaries-01], [agent/boundaries-02]).

## Checklist

Copie e marque o progresso:

```
- [ ] 1. Carregar o contexto e listar os pendentes
- [ ] 2. Iniciar o primeiro pedido
- [ ] 3. Cumprir a ação pela skill
- [ ] 4. Concluir ou recusar o pedido
```

## Passos

### 1. Carregar o contexto

Execute:

```bash
vostok context load --skill process-requests
vostok pedidos listar --estado pendente
```

Sem pedido pendente, encerre sem responder.

### 2. Iniciar o primeiro pedido

Para o primeiro da lista, em ordem de ID:

```bash
vostok pedidos iniciar PED-NNN
```

Leia o arquivo do pedido: a ação, a spec, a tarefa e o `## Pedido`.

### 3. Cumprir a ação

Carregue a skill da ação e siga-a como se o usuário tivesse feito o pedido no chat:

| Ação | Skill |
|---|---|
| `aprovar-plano` | `approve-plan` |
| `planejar-spec` | `plan-tasks` |
| `ajustar-spec` | `adjust-spec` |
| `ajustar-plano` | `adjust-plan` |
| `ajustar-tarefa` | `adjust-task` |
| `executar-tarefa` | `execute-task` |
| `finalizar-tarefa` | `finish-task` |
| `desbloquear-tarefa` | `finish-task` (retomada da tarefa bloqueada) |
| `finalizar-spec` | `finalize-spec` |
| `responder-pergunta` | a pergunta citada no pedido |
| `fazer-commit` | `approve-commit`, com `--pedido PED-NNN` |

Se a skill parar com `blocker` ou `question`, ou a CLI recusar o comando, vá ao passo 4 e recuse o pedido com o motivo.

### 4. Terminar o pedido

O resultado ou o motivo vai pela entrada padrão. Cumprido:

```bash
printf '%s\n' "o que foi feito, em poucas linhas" | vostok pedidos concluir PED-NNN
```

Não cumprido (RN-04):

```bash
printf '%s\n' "o motivo da recusa" | vostok pedidos recusar PED-NNN
```

Depois, volte ao passo 1 só se for um novo ciclo do loop; num ciclo, trate um pedido.

## Como armar o loop

- Claude Code: `/loop 1m /process-requests`.
- Codex: `vostok pedidos vigiar --agente codex`.

## Formatos de resposta

Respostas ao usuário usam os templates de `templates/responses/`. Esta skill não tem formato próprio: o resultado de cada pedido fica no `## Resultado` dele.
