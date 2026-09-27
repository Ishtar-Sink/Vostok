---
name: completion
description: Conclusão de uma tarefa que alterou arquivos ou executou um fluxo, inclusive o fim de uma skill.
usage: |
  SUMMARY: o resultado em uma frase.
  CHANGES: lista `- caminho: o que mudou`, um item por arquivo ou grupo de arquivos.
  VERIFICATION: comandos executados e resultado. Diga também o que não foi verificado.
  MEMORY_RECORDS: opcional. Entradas criadas ou alteradas na memória, `- ID: título`. Sem entradas, omita a seção.
  PENDING: opcional. O que ficou em aberto. Sem pendências, omita a seção.
---

**#{SUMMARY}**

## O que foi feito

#{CHANGES}

## Verificação

#{VERIFICATION}

## Memória

#{MEMORY_RECORDS}

## Pendências

#{PENDING}
