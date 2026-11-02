---
spec: SPEC-027
spec_hash: 4fa8cdb48e14a0c66df9eea1c90bb95e595ed35f55582c3e208d0fd1e802e2d7
content_hash: 77d8ffa47fc383d0f343668d3d9dda33e230b458b56141b57e6519cad2ce5593
adjustments: 0
adjusted_at: null
created_at: 2026-11-02T09:46:38Z
updated_at: 2026-11-02T09:46:53Z
---

# Plano de tarefas: SPEC-027 - Estudo do painel local do Vostok sem servidor

Documento para revisão. É removido quando o plano é aprovado.

## Visão geral

O estudo foi dividido em cinco tarefas. Duas frentes independentes abrem a onda 1: o levantamento documental (inventário das ações do chat, regras de aprovação e o que muda para remover o servidor), que não toca código, e o spike de medição no branch `spike/painel-local`, que compara os servidores HTTP e o empacotamento do frontend em React e mede o binário. Com o spike pronto, o protótipo funcional monta as telas centrais com os componentes do Atria e a marca da pasta, abre dois painéis ao mesmo tempo e mostra as mudanças dos arquivos sem recarregar.

Depois do protótipo e do levantamento, uma tarefa testa no protótipo a aprovação e a edição pelo painel, a edição simultânea com o agente e um chat do Claude Code em `/loop` processando um pedido. A última tarefa consolida tudo em `docs/estudos/painel-local.md`, com a conclusão de viabilidade, a arquitetura recomendada e as specs de implementação propostas; é a única que escreve o documento, para que as anteriores rodem em paralelo sem dividir arquivos. Cada tarefa anterior registra o que achou nas próprias evidências. O código do protótipo fica só no branch e nunca entra na `main`.

## Tarefas

| ID | Título | Descrição | Tipo | Persona | Perfil | Dependências | Onda | Cobre |
|---|---|---|---|---|---|---|---|---|
| TASK-01 | Levantar ações, aprovação e remoção do servidor | Inventário das ações do chat por forma de execução, desenho da aprovação pelo painel e lista do que muda nas regras, na memória e nas pastas. | tarefa | arquiteto | julgamento | - | 1 | PQ-05, PQ-06, PQ-11 |
| TASK-02 | Medir servidor HTTP e frontend no binário | Spike no branch `spike/painel-local` que mede o binário com cada servidor HTTP e com o bundle em React e em Preact, e propõe o limite. | tarefa | desenvolvedor | mecanica | - | 1 | PQ-02, PQ-03, PQ-04 |
| TASK-03 | Montar o protótipo funcional do painel | Protótipo no spike com as telas centrais, os componentes do Atria e a marca da pasta, aberto em duas pastas ao mesmo tempo e atualizado ao vivo. | historia | ui-ux-designer | padrao | TASK-02 | 2 | PQ-01, PQ-09, PQ-10, PQ-12 |
| TASK-04 | Testar aprovação, edição simultânea e loop | Teste no protótipo da aprovação e da edição pelo painel, da edição simultânea com o agente e de um chat em `/loop` processando um pedido. | tarefa | desenvolvedor | julgamento | TASK-01, TASK-03 | 3 | PQ-06, PQ-07, PQ-08 |
| TASK-05 | Consolidar o estudo do painel local | Escreve `docs/estudos/painel-local.md` com as respostas, a viabilidade, as medições, a arquitetura recomendada, as specs propostas e as imagens do protótipo. | tarefa | arquiteto | padrao | TASK-01, TASK-02, TASK-03, TASK-04 | 4 | PQ-01, PQ-02, PQ-03, PQ-04, PQ-05, PQ-06, PQ-07, PQ-08, PQ-09, PQ-10, PQ-11, PQ-12 |

## Componentes por tarefa

Cada tarefa lista os caminhos de "Componentes afetados" da spec que ela assume.

| ID | Componentes |
|---|---|
| TASK-01 | - |
| TASK-02 | - |
| TASK-03 | - |
| TASK-04 | - |
| TASK-05 | - |

## Fatos de esforço

| ID | Arquivos a criar | Arquivos a alterar | Contrato público | Inferência | Risco alto |
|---|---|---|---|---|---|
| TASK-01 | 0 | 0 | não | sim | sim |
| TASK-02 | 0 | 0 | não | não | não |
| TASK-03 | 0 | 0 | não | sim | não |
| TASK-04 | 0 | 0 | não | sim | sim |
| TASK-05 | 0 | 0 | não | sim | não |

## Arquivos em comum por onda

Seção calculada pelo `vostok plan score` a partir dos componentes por tarefa. O Coordenador executa em sequência as tarefas de uma mesma linha.

| Onda | Arquivo | Tarefas |
|---|---|---|
| - | - | - |

## Perfis de execução

| Perfil | Descrição |
|---|---|
| mecanica | Passos diretos e repetitivos; prefere modelo rápido. |
| padrao | Implementação com decisões locais; prefere modelo intermediário. |
| julgamento | Desenho, ambiguidade e decisões amplas; prefere o modelo mais capaz. |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
