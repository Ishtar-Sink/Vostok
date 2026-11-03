---
id: SPEC-034
title: "Corrigir o fluxo do painel antes da v0.4.0"
status: implemented
author: "Victor Batalha"
created_at: 2026-11-03T21:57:10Z
updated_at: 2026-11-03T22:39:37Z
tags: []
type: fix
source: "draft/SPEC-034-corrigir-o-fluxo-do-painel-antes-da-v0-4-0.md"
refined_at: 2026-11-03T22:02:55Z
adjustments: 0
adjusted_at: null
content_hash: ca7896fbcfd11db6a7167dff3f7bfde92d8aa176471d3f69eee90ebd7563ce52
approved_at: 2026-11-03T22:05:23Z
approved_hash: ca7896fbcfd11db6a7167dff3f7bfde92d8aa176471d3f69eee90ebd7563ce52
approved_by: chat
---

# Corrigir o fluxo do painel antes da v0.4.0 - Especificação Refinada

Corrige quatro defeitos do fluxo do painel entregue pela SPEC-033, achados ao operar um projeto de demonstração de ponta a ponta: o plano desatualizado oferecido para aprovação, a falta de "Ajustar plano", a tarefa em validação sem botão de finalizar e o binário que embute telas antigas. Também corrige o histórico de status do ajuste e os textos que ainda citam `avais.log`.

## Objetivo

Deixar o fluxo de spec, plano e tarefa operável só pelo painel, sem precisar gravar pedidos pela API com `curl`, antes de publicar a v0.4.0 (TASK-13 da SPEC-033).

## Defeito

- **Comportamento atual:** (1) Com a spec ajustada e aprovada de novo depois do plano, o painel oferece "Aprovar plano" sobre um plano que não cobre os IDs novos e não oferece "Planejar"; (2) não há botão para ajustar um plano em dia; (3) depois da conferência dos prints, a tarefa fica em `validating` só com "Pedir ajuste"; (4) o ajuste que derruba a aprovação não grava a volta para refined no `status_history`; (5) a ajuda dos diálogos de aprovação, a mensagem de sucesso e a validação da CLI citam `work/avais.log`; (6) o `cargo build --features painel` embute o `apps/painel/dist` que existir, mesmo mais antigo que o código.
- **Comportamento esperado:** (1) O plano desatualizado aparece como tal, sem "Aprovar plano", com "Refazer o plano" na spec aprovada; (2) o plano em dia tem "Ajustar plano"; (3) a tarefa em `validating` tem "Finalizar"; (4) o `status_history` registra a volta para refined; (5) os textos citam o log de avais da spec, `work/avais/<SPEC>.log`; (6) o build não embute um `dist` mais antigo que `apps/painel/src`.
- **Como reproduzir:** No projeto de demonstração: refinar, aprovar e planejar uma spec; ajustar a spec pelo painel e aprová-la de novo; abrir a spec (só "Aprovar plano") e rodar `vostok plan validate` (IDs novos sem tarefa). Executar e finalizar pelo painel uma tarefa com print, aceitar o print e abrir a tarefa (só "Pedir ajuste"). Ler o `status_history` da spec ("approved; approved"). Alterar um arquivo de `apps/painel/src`, rodar `cargo build --release --features painel` e abrir o painel (tela antiga).
- **Causa raiz:** `specActions` e o `overview` do painel usam só a existência do arquivo do plano (`apps/cli/src/painel/read.rs`, `plan` booleano); a regra de plano desatualizado existe só em `plan_is_outdated` (`apps/cli/src/commands/plan.rs`), privada. `taskActions` oferece "Finalizar" só para `in_progress`. `drop_approval` (`apps/cli/src/commands/spec.rs`) muda o status sem chamar `record_status`. Os textos não foram atualizados quando a TASK-03 da SPEC-033 passou o aval para logs por spec. O `build.rs` só reconstrói o painel quando falta `dist/index.html`.

## Escopo

### Dentro do escopo

- Plano desatualizado no painel: sinal na API, aviso na spec e no plano, "Refazer o plano" no lugar de "Aprovar plano".
- "Ajustar plano" para o plano em dia.
- "Finalizar" para a tarefa em `validating`.
- Volta para refined no `status_history` quando o ajuste derruba a aprovação.
- Textos do aval com o log por spec.
- Recusa do build com `dist` mais antigo que `apps/painel/src`.

### Fora do escopo

- Os pontos de experiência da SPEC-035 (estado visível, uma ação principal, fila guiada, perguntas com radio e checkbox, lote, ondas no quadro, Atividades acionável, tabelas).
- Painéis que não encerram: os processos órfãos vieram do binário do spike e de um teste interrompido, não do código atual (resposta 4A).
- A atualização da DEC-003 (inferência G).
- Mudanças nas skills `plan-tasks`, `adjust-plan`, `approve-plan` e `finish-task`: elas já tratam os casos.

## Requisitos funcionais

| ID | Requisito |
|---|---|
| RF-01 | O sistema deve expor, na visão geral e na spec do painel, se o plano da spec está desatualizado, pela mesma regra da CLI: `spec_hash` do plano diferente do `approved_hash` da spec, ou spec sem aprovação. |
| RF-02 | O painel deve mostrar o plano desatualizado com o aviso "Plano desatualizado" na spec e na visão do plano, sem "Aprovar plano" na spec nem em Pendências. |
| RF-03 | Com o plano desatualizado e a spec aprovada, o painel deve oferecer "Refazer o plano", que grava o pedido `planejar-spec`. |
| RF-04 | Com o plano em dia, o painel deve oferecer "Ajustar plano", com texto obrigatório, que grava o pedido `ajustar-plano`. |
| RF-05 | O painel deve oferecer "Finalizar" na tarefa em `validating`, gravando o pedido `finalizar-tarefa`. |
| RF-06 | O ajuste que derruba a aprovação de uma spec deve gravar a volta para refined no `status_history`. |
| RF-07 | A ajuda dos diálogos de aprovar spec e tarefa, a mensagem de sucesso da aprovação e a mensagem de validação da CLI devem citar o log de avais da spec (`work/avais/<SPEC>.log`), e nenhum texto deve citar `avais.log` como destino. |
| RF-08 | Com a feature `painel` e `apps/painel/dist/index.html` mais antigo que o arquivo mais novo de `apps/painel/src`, o `build.rs` deve reconstruir o `dist` com `pnpm` e, se a reconstrução falhar, recusar o build com mensagem que cita `pnpm --dir apps/painel build`. |

## Casos de borda

| ID | Situação | Comportamento esperado | Requisitos |
|---|---|---|---|
| CB-01 | Spec de volta a refined com um plano antigo | O plano aparece como desatualizado, sem botão; a spec mostra só as ações dela ("Aprovar spec", "Pedir ajuste") | RF-01, RF-02 |
| CB-02 | Spec aprovada sem plano | Continua oferecendo "Planejar tarefas", sem aviso | RF-01, RF-03 |
| CB-03 | Tarefa em `validating` com print ainda não conferido | "Finalizar" aparece; a finish-task diz o que falta e a tarefa continua em `validating` | RF-05 |
| CB-04 | Ajuste de uma spec só refinada | O `status_history` não ganha linha, porque o status não muda | RF-06 |
| CB-05 | Build sem a feature `painel` ou sem `apps/painel/dist` | Comportamento atual: sem a feature nada muda; sem o `dist`, o `build.rs` tenta o `pnpm build` como hoje | RF-08 |

## Componentes afetados

| Caminho | Ação | Motivo |
|---|---|---|
| `apps/cli/src/commands/plan.rs` | alterar | Expor a regra de plano desatualizado para o painel. |
| `apps/cli/src/painel/read.rs` | alterar | Campo de plano desatualizado na visão geral e na spec. |
| `apps/cli/src/commands/spec.rs` | alterar | `drop_approval` grava a volta para refined no `status_history`. |
| `apps/cli/src/commands/task.rs` | alterar | Mensagem `PANEL_WITHOUT_AVAL` com o log por spec. |
| `apps/cli/build.rs` | alterar | Recusar `dist` mais antigo que `apps/painel/src`. |
| `apps/painel/src/shared/attention.ts` | alterar | Ações de plano desatualizado, "Ajustar plano", "Finalizar" em `validating` e textos do aval. |
| `apps/painel/src/shared/action-bar.tsx` | alterar | Pedidos `ajustar-plano` (com texto) e a mensagem de sucesso da aprovação. |
| `apps/painel/src/features/spec/spec-page.tsx` | alterar | Aviso de plano desatualizado na spec e na visão do plano. |
| `apps/cli/tests/spec.rs` | alterar | Teste do `status_history` no ajuste. |
| `apps/cli/tests/painel.rs` | alterar | Teste do campo de plano desatualizado. |
| `apps/painel/src/shared/attention.test.ts` | alterar | Testes das ações novas. |
| `apps/painel/src/features/spec/spec-page.test.tsx` | alterar | Teste do aviso de plano desatualizado. |

## Premissas e dependências

- As skills `plan-tasks` (recria o plano desatualizado), `adjust-plan` (recusa o desatualizado) e `finish-task` (diz o que falta na tarefa em validação) já tratam os casos; o painel só oferece os pedidos certos.
- A CLI do painel já aceita os pedidos `planejar-spec`, `ajustar-plano` e `finalizar-tarefa` (`apps/cli/src/painel/actions.rs`).
- A TASK-13 da SPEC-033 (versão 0.4.0) é executada depois desta spec (inferência F).

## Riscos

| Risco | Impacto | Mitigação |
|---|---|---|
| A recusa do `build.rs` atrapalhar o ciclo de desenvolvimento, que edita `apps/painel/src` e roda `cargo test --features painel` | Testes param até rodar `pnpm --dir apps/painel build` | A mensagem diz o comando exato; inferência H: o `build.rs` tenta reconstruir antes de recusar |
| A regra de plano desatualizado divergir entre o painel e a CLI | O painel oferece uma ação que a CLI recusa | O painel usa a função da CLI, sem repetir a regra (inferência B) |
| Comparar datas de arquivo depois de um `git checkout`, que muda as datas | Recusa ou aceite errado do `dist` | A comparação usa o arquivo mais novo de `apps/painel/src` contra `dist/index.html`, e a reconstrução resolve os dois casos |

## Casos de teste

| ID | Cenário | Passos | Resultado esperado | Requisitos |
|---|---|---|---|---|
| CT-01 | Plano desatualizado na API | Criar plano, mudar o `approved_hash` da spec e ler a visão geral e a spec | O campo de plano desatualizado vem verdadeiro; com hashes iguais, falso | RF-01 |
| CT-02 | Ações da spec com plano desatualizado | `specActions` com spec aprovada e plano desatualizado | Sem "Aprovar plano", com "Refazer o plano" (`planejar-spec`) | RF-02, RF-03 |
| CT-03 | Pendências com plano desatualizado | Visão geral com plano desatualizado | Nenhum item "Aprovar plano" para a spec | RF-02 |
| CT-04 | Aviso na página da spec | Renderizar a spec com plano desatualizado | Aviso "Plano desatualizado" na spec e na visão do plano | RF-02 |
| CT-05 | Spec refined com plano antigo | `specActions` com spec refined e plano | Só "Aprovar spec" e "Pedir ajuste"; nenhum botão de plano | RF-02 |
| CT-06 | Ajustar plano em dia | `specActions` com plano em dia; enviar o diálogo | "Ajustar plano" com texto obrigatório grava `ajustar-plano` | RF-04 |
| CT-07 | Finalizar em validação | `taskActions` com `validating`; enviar o diálogo | "Finalizar" grava `finalizar-tarefa` | RF-05 |
| CT-08 | Histórico no ajuste | Aprovar uma spec, ajustá-la com `vostok spec adjust record` e ler o front-matter | O `status_history` termina com a volta para refined | RF-06 |
| CT-09 | Textos do aval | Buscar `avais.log` em `apps/painel/src` e na mensagem de `apps/cli/src/commands/task.rs`; aprovar pelo painel | Nenhum texto cita `avais.log`; a mensagem de sucesso cita o log da spec | RF-07 |
| CT-10 | Build com `dist` velho | Tocar um arquivo de `apps/painel/src` depois do `dist` e rodar `cargo build --features painel` sem `pnpm` no `PATH` | O build termina com erro e a mensagem cita `pnpm --dir apps/painel build` | RF-08 |
| CT-11 | Build com `dist` em dia | Rodar `pnpm --dir apps/painel build` e depois `cargo build --features painel` | O build termina sem erro | RF-08 |

## Critérios de aceite

- [ ] CA-01: com o plano desatualizado, o painel mostra o aviso e "Refazer o plano" e não oferece "Aprovar plano" (RF-01, RF-02, RF-03).
- [ ] CA-02: com o plano em dia, o painel oferece "Ajustar plano" com texto obrigatório (RF-04).
- [ ] CA-03: a tarefa em `validating` tem "Finalizar" (RF-05).
- [ ] CA-04: o `status_history` registra a volta para refined no ajuste que derruba a aprovação (RF-06).
- [ ] CA-05: nenhum texto do painel ou da CLI cita `avais.log` como destino do aval (RF-07).
- [ ] CA-06: o build com a feature `painel` não embute um `dist` mais antigo que `apps/painel/src` (RF-08).

## Referências

| Tipo | Referência | Uso |
|---|---|---|
| Arquivo | `apps/cli/src/commands/plan.rs` | `plan_is_outdated` e `outdated_message`: a regra do plano desatualizado |
| Arquivo | `apps/cli/src/painel/read.rs` | `overview` e a spec: o `plan` booleano pela existência do arquivo |
| Arquivo | `apps/painel/src/shared/attention.ts` | `specActions`, `taskActions` e os textos com `avais.log` |
| Arquivo | `apps/painel/src/shared/action-bar.tsx` | Mensagem de sucesso da aprovação e o mapa de pedidos |
| Arquivo | `apps/cli/src/commands/spec.rs` | `drop_approval` sem `record_status` |
| Arquivo | `apps/cli/src/commands/task.rs` | `PANEL_WITHOUT_AVAL` |
| Arquivo | `apps/cli/build.rs` | `prepare_painel_bundle`: só reconstrói sem `dist/index.html` |
| Arquivo | `vostok/skills/default/adjust-plan/SKILL.md` | Plano desatualizado não é ajustado, é recriado pela plan-tasks |
| Arquivo | `vostok/skills/default/approve-plan/SKILL.md` | Impedimento de plano desatualizado |
| Memória | DEC-003 | Painel local; ainda cita `work/avais.log` |
| Spec | SPEC-033 | Fluxo do painel entregue; TASK-03 (avais por spec), TASK-10 (conferência de prints), TASK-13 (versão 0.4.0) |
| Demonstração | Projeto de demonstração operado pelo usuário em 2026-11-03 | Reprodução dos defeitos |

## O que mudou

| # | Assunto | Estado no Original | Estado Refinado |
|---|---|---|---|
| 1 | Saída do plano desatualizado | "Ajustar plano" | "Refazer o plano" pelo `planejar-spec`; "Ajustar plano" só para o plano em dia |
| 2 | Painéis que não encerram | Defeito da spec | Fora do escopo: órfãos do spike e de um teste interrompido |
| 3 | Telas antigas no binário | Nota para a TASK-13 | Requisito: o `build.rs` recusa `dist` velho |
| 4 | Tarefa em validação | Botão para concluir depois da conferência | "Finalizar" em `validating`, e a finish-task diz o que falta |

## Registro de decisões

| # | Decisão | Alternativa descartada | Motivo |
|---|---|---|---|
| 1 | Plano desatualizado: aviso e "Refazer o plano" (`planejar-spec`) | Manter "Ajustar plano" mudando a adjust-plan | resposta 1A |
| 2 | "Ajustar plano" com texto obrigatório no plano em dia | Deixar para a SPEC-035 | resposta 2A |
| 3 | "Finalizar" também em `validating` | Só com todos os prints conferidos; pedido automático no último aceite | resposta 3A |
| 4 | Painéis órfãos fora do escopo, processos encerrados à parte | Encerramento automático do painel | resposta 4A |
| 5 | O `build.rs` recusa `dist` mais antigo que o código | Só um passo na TASK-13 | resposta 5A |
| 6 | Tipo `fix` | `improvement` | inferência A |
| 7 | O painel usa a regra da CLI para o plano desatualizado | Repetir a regra no frontend | inferência B |
| 8 | Spec refined com plano antigo mostra só as ações da spec | Oferecer "Refazer o plano" sem aprovação | inferência C |
| 9 | `drop_approval` grava o `status_history` | Gravar só na aprovação | inferência D |
| 10 | Textos citam `work/avais/<SPEC>.log` | Não citar arquivo nenhum | inferência E |
| 11 | TASK-13 depois desta spec, por sequência | Mudar as dependências da TASK-13 | inferência F |
| 12 | DEC-003 atualizada na finalização da SPEC-033 | Nesta spec | inferência G |
| 13 | O `build.rs` tenta reconstruir o `dist` com `pnpm` antes de recusar | Recusar sempre | inferência H |

## Pendências

| # | Pendência | Opções em aberto |
|---|---|---|
| - | Nenhuma | - |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
