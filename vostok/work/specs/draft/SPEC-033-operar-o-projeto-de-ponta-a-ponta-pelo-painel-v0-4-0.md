---
id: SPEC-033
title: "Operar o projeto de ponta a ponta pelo painel (v0.4.0)"
status: draft
author: "Victor Batalha"
created_at: 2026-11-03T15:35:49Z
updated_at: 2026-11-03T15:35:49Z
tags: []
---

# Operar o projeto de ponta a ponta pelo painel (v0.4.0)

Segunda parte da v0.4.0: operar o projeto inteiramente pelo painel, sem precisar interagir diretamente com o chat do agente. Parte do estudo preliminar da SPEC-032.

## Contexto

A v0.4.0 entrega o controle do projeto usando o vostok inteiramente pelo painel. No refinamento da SPEC-032, ela foi dividida em duas specs: a SPEC-032 entrega o estudo preliminar e os itens já claros (status, quadro, pendências e atividades), e esta spec entrega a operação de ponta a ponta, a partir do que o estudo decidir.

O painel deve servir para operar o projeto de ponta a ponta: criar specs, refinar specs e fazer todos os processos que hoje se fazem pelo agente no Claude Code. A preferência atual é o caminho do pedido: o usuário escreve no painel, o agente em loop faz e as perguntas dele voltam como pendências que o usuário responde no painel. A viabilidade de um chat dentro do painel é avaliada no estudo.

Os arquivos de comunicação entre agente e painel não podem se acumular a ponto de confundir o usuário, e o agente precisa carregá-los de forma eficiente, sem ler todos a cada rodada. Hoje o `vostok pedidos` lê todos os arquivos da pasta para achar os pendentes, e o `avais.log` só cresce.

> Quanto mais detalhes, mais assertivo será o refinamento._

## Recursos

- Criar e refinar specs pelo painel.
- Fazer pelo painel todos os processos que hoje se fazem pelo agente, exceto as skills do próprio harness (report-harness-bug, suggest-harness-*, validate-harness-bug, create-skill) e as de configuração inicial (init-harness, adjust-rules, customize-documents).
- Perguntas do agente gravadas para o painel e respondidas pelo usuário como pendências.
- Arquivos de comunicação (pedidos, perguntas e avais) organizados para não se acumularem e para o agente ler só o necessário; separar por pastas de status é uma ideia a avaliar.
- Mudanças de status que o estudo apontar como faltantes, posicionadas em fluxos já existentes.
- A versão sobe para 0.4.0 no fim desta spec.

## Detalhes Técnicos

- Depende do estudo da SPEC-032, em `docs/estudos/painel-ponta-a-ponta.md`.
- Hoje o painel lê `work/perguntas/` e tem a ação `responder-pergunta`, mas nenhuma skill nem comando grava perguntas lá (`apps/cli/src/painel/read.rs`).
- Hoje `vostok pedidos listar` e `vigiar` leem todos os arquivos de `work/pedidos/` a cada rodada (`apps/cli/src/commands/pedidos.rs`).
- As pastas `pedidos/` e `perguntas/` estão vazias neste repositório e no app da RD.

## Roteiro de Testes

_Se aplicável e/ou conhecido, insira o roteiro de testes. Não preencher fará o agente inferir com base no restante das informações. Pode ser ajustado posteriormente._

## Critérios de Aceite
- [ ]
