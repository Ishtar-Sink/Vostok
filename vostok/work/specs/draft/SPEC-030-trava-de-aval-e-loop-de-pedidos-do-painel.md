---
id: SPEC-030
title: "Trava de aval e loop de pedidos do painel"
status: draft
author: "Victor Batalha"
created_at: 2026-11-02T12:57:17Z
updated_at: 2026-11-02T12:57:17Z
tags: []
---

# Trava de aval e loop de pedidos do painel

Fecha o ciclo do painel local do lado do agente: a CLI passa a conferir o aval do usuário antes de concluir, commitar e aprovar, e um chat em loop processa os pedidos gravados pelo painel, no Claude Code e no Codex.

## Contexto

Recurso novo, ligado à memória `spec-trava-aval` e às specs propostas pelo estudo da SPEC-027 (`docs/estudos/painel-local.md`), condensadas pelo usuário para executar com paralelismo. Hoje as regras de aprovação são só texto: a CLI não distingue quem a chama, e um agente pode concluir, commitar ou aprovar sem o usuário.

O estudo testou um chat do Claude Code em `/loop 1m` processando pedidos do painel sem mudança nas skills (`docs/estudos/painel-local/testes.md`). Esta spec usa os formatos do estudo (`avais.log` e `PED-NNN.md`), então pode começar junto com a SPEC-029 ou logo depois de ela fixar esses formatos.

## Recursos

- Conferência do aval do ID e do hash antes de `task finish`, `commit commit`, `commit approve`, `plan approve mark-planned` e `finalize archive`, quando a trava estiver ligada.
- `validate` de spec, plano e tarefa acusam `approved_by: painel` sem aval correspondente.
- Regras: aprovação, liberação ou retomada feita no painel vale como pedido do usuário; o agente nunca chama o endpoint do painel nem grava campos de aval ou de origem.
- Pedidos de ações travadas (finalizar tarefa, aprovar plano, finalizar spec, commit) exigem o aval do painel.
- Instrução padrão do loop no harness, para o chat processar `vostok/work/pedidos/`: estados `pendente`, `em_andamento`, `feito`, `recusado`, com `## Resultado`.
- Processo externo para o Codex, que dispara `codex exec` quando aparece um pedido pendente (por exemplo `vostok painel --agente codex`).

## Detalhes Técnicos

- Aval: linha em `vostok/work/avais.log` (data, ID, hash aprovado, origem), só de acréscimo, gravada pelo painel (SPEC-029).
- `rules/agent/boundaries.md` e as regras `[approve-spec-01]`, `[approve-plan-01]`, `[approve-task-01]`, `[finish-task-08]`, `[finalize-spec-01]`, `[approve-commit-01]` passam a aceitar o aval do painel.
- A memória `spec-trava-aval` também cita regras `ask` no `.claude/settings.json` como camada extra por ferramenta.
- Achados do teste do loop: a espera é dominada pelo intervalo (cerca de 1 min); o loop precisa de permissão para os comandos `vostok`; um loop mais curto quando há pedido pendente aproxima do tempo real.
- Risco conhecido: um agente com shell no mesmo usuário do sistema ainda pode escrever na pasta; o aval torna a forja visível e presa ao hash, sem impedi-la.

## Roteiro de Testes

1. Com a trava ligada, `vostok task finish` sem aval correspondente é recusado; com o aval, conclui.
2. `commit commit` e `commit approve` sem aval são recusados.
3. Aval de outro hash não vale para a versão atual.
4. Tarefa com `approved_by: painel` sem linha no `avais.log` é acusada pelo `validate`.
5. Chat do Claude Code com a instrução padrão processa um pedido do painel até `feito` ou `recusado`.
6. Processo do Codex processa um pedido pendente com `codex exec`.

## Critérios de Aceite
- [ ] A CLI recusa concluir, commitar e aprovar sem o aval do usuário quando a trava está ligada.
- [ ] Aprovações do painel sem aval correspondente são acusadas na validação.
- [ ] As regras do harness aceitam o aval do painel como pedido do usuário.
- [ ] Um chat em loop processa os pedidos do painel no Claude Code e no Codex.
