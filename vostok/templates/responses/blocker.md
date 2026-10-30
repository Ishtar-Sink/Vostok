---
name: blocker
description: Bloqueio que impede continuar, como um erro que persiste após as tentativas permitidas ou um script que falhou. Use ao parar por falha.
usage: |
  BLOCKER_SUMMARY: o que está bloqueado, em uma frase.
  ERROR_OUTPUT: a saída relevante do erro, sem cortar a mensagem principal.
  ATTEMPTS: lista numerada das tentativas e do resultado de cada uma.
  MEMORY_SEARCH: termos usados em `vostok memory search issue` e os IDs encontrados, ou "Nenhuma entrada encontrada".
  Opções: de 2 a 4, a primeira sendo a recomendada, com a descrição terminando em "(recomendada)".
  TASK_STATUS: obrigatória nas respostas sobre tarefas (aprovar o plano, aprovar, executar, ajustar e finalizar tarefa): a saída de `vostok task status <SPEC_ID>`, colada sem alteração ([agent/responses-07]). Nas demais respostas, omita a seção.
---

**Bloqueado: #{BLOCKER_SUMMARY}**

## Erro

```
#{ERROR_OUTPUT}
```

## O que foi tentado

#{ATTEMPTS}

## Memória consultada

#{MEMORY_SEARCH}

## Status das tarefas

#{TASK_STATUS}

## Como seguir

| Opção | Descrição |
|---|---|
| A | #{OPTION_DESCRIPTION} (recomendada) |
| B | #{OPTION_DESCRIPTION} |

Responda com a letra ou descreva como prosseguir.
