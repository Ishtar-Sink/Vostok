---
id: SPEC-030
title: "Trava de aval e loop de pedidos do painel"
status: planned
author: "Victor Batalha"
created_at: 2026-11-02T12:57:17Z
updated_at: 2026-11-02T15:35:56Z
tags: []
type: feature
source: "draft/SPEC-030-trava-de-aval-e-loop-de-pedidos-do-painel.md"
refined_at: 2026-11-02T15:19:31Z
adjustments: 0
adjusted_at: null
content_hash: 11cc5bc975fefba915cc21dfa4ace8aa20c882c6451ece2a1d1a379e27e5b0d5
approved_at: 2026-11-02T15:27:49Z
approved_hash: 11cc5bc975fefba915cc21dfa4ace8aa20c882c6451ece2a1d1a379e27e5b0d5
approved_by: chat
---

# Trava de aval e loop de pedidos do painel - Especificação Refinada

Fecha o ciclo do painel local do lado do agente. Com a trava ligada, a CLI só conclui, aprova e faz commit com o aval do usuário dado pelo painel. Um chat em loop, no Claude Code, ou um processo próprio, no Codex, processa os pedidos gravados pelo painel.

## Objetivo

Hoje as regras de aprovação são só texto das skills: a CLI não distingue quem a chama, e um agente pode concluir uma tarefa, fazer commit ou aprovar sem o usuário (memória `spec-trava-aval`). A SPEC-029 entregou o painel, com o aval em `vostok/work/avais.log` e os pedidos em `vostok/work/pedidos/`, mas nada do lado do agente confere o aval nem processa os pedidos. Esta spec faz a CLI exigir o aval, opcionalmente por projeto, e dá ao harness um jeito padrão de processar os pedidos no Claude Code e no Codex.

## Escopo

### Dentro do escopo

- Chave `approval_lock` em `vostok/config/main.yml`, desligada por padrão.
- Leitura do `avais.log` pela CLI e conferência do aval antes de:
  - `task finish`, `plan approve mark-planned` e `finalize archive`;
  - `commit commit` e `commit approve`;
  - `spec approve approve` e `task approve approve`.
- `approved_by: painel` sem aval correspondente acusado pela validação de spec e de tarefa.
- No painel: aval gravado junto com os pedidos de ações travadas e a ação nova `fazer-commit`.
- Comando `vostok pedidos` para listar e mudar o estado dos pedidos.
- Skill `process-requests`, com a instrução padrão do loop.
- `vostok pedidos vigiar --agente codex`, que dispara `codex exec` para cada pedido pendente.
- Regras do harness: o pedido do painel com aval vale como pedido do usuário.

### Fora do escopo

- Regras `ask` no `.claude/settings.json` ou no Codex (resposta 4).
- Impedir que um processo do mesmo usuário escreva na pasta: o aval torna a forja visível e presa ao hash, sem impedi-la.
- Trava ligada por padrão ou sem chave (resposta 1).
- Loop acordado pelo próprio arquivo (sem intervalo); o loop do Claude Code segue com `/loop`.
- Processar pedidos em paralelo: um pedido por vez.

## Requisitos funcionais

| ID | Requisito |
|---|---|
| RF-01 | A CLI deve ler a chave `approval_lock` de `vostok/config/main.yml`; sem a chave, a trava fica desligada. |
| RF-02 | A CLI deve ler `vostok/work/avais.log` aceitando linhas de 4 colunas (formato da SPEC-029, lidas como ação `aprovar`) e de 5 colunas (data, ID, hash, origem e ação). |
| RF-03 | Com a trava ligada, `vostok task finish` deve exigir um aval `finalizar-tarefa` do `SPEC/TASK` com o `approved_hash` atual da tarefa. |
| RF-04 | Com a trava ligada, `vostok plan approve mark-planned` deve exigir um aval `aprovar-plano` da spec com o hash atual do plano. |
| RF-05 | Com a trava ligada, `vostok finalize archive` deve exigir um aval `finalizar-spec` da spec com o hash atual da spec refinada. |
| RF-06 | Com a trava ligada, `vostok commit commit` deve exigir `--pedido PED-NNN`, com um aval `fazer-commit` desse pedido e o pedido em `em_andamento`. |
| RF-07 | Com a trava ligada, `vostok commit approve` deve exigir `--pedido PED-NNN` nas mesmas condições do RF-06. |
| RF-08 | Com a trava ligada, `vostok spec approve approve` e `vostok task approve approve` devem exigir um aval `aprovar` de cada item com o hash atual. |
| RF-09 | `vostok spec validate`, `vostok task adjust validate` e as validações de aprovação devem acusar `approved_by: painel` sem linha de aval do mesmo ID e hash, com a trava ligada ou não. |
| RF-10 | O painel deve gravar o aval (com a ação e o hash atual do alvo) ao gravar um pedido `finalizar-tarefa`, `aprovar-plano`, `finalizar-spec` ou `fazer-commit`. |
| RF-11 | O painel deve aceitar a ação `fazer-commit`, com um botão "Pedir commit" na tela de pendências e o texto do pedido. |
| RF-12 | A aprovação pelo painel deve passar a gravar o aval com 5 colunas, com a ação `aprovar`. |
| RF-13 | O comando `vostok pedidos` deve listar os pedidos e mudar o estado de um pedido para `em_andamento`, `feito` ou `recusado`, gravando `atualizado_em` e o texto de `## Resultado`. |
| RF-14 | A skill `process-requests` deve trazer a instrução do loop: para cada pedido pendente, em ordem, iniciar, executar pela skill da ação e concluir ou recusar com o motivo. |
| RF-15 | O comando `vostok pedidos vigiar --agente codex` deve conferir a pasta no intervalo de `--intervalo` (padrão 15 s) e disparar `codex exec` com a instrução da skill para cada pedido pendente, um por vez. |
| RF-16 | As regras `[agent/boundaries-01]` e as das skills `approve-spec`, `approve-plan`, `approve-task`, `finish-task`, `finalize-spec` e `approve-commit` devem aceitar o pedido do painel com aval como pedido do usuário. |

## Requisitos não funcionais

| ID | Requisito | Como verificar |
|---|---|---|
| RNF-01 | Com a trava desligada, o comportamento é o da 0.3.0. | A suíte atual passa sem mudança de golden. |
| RNF-02 | Compatibilidade: o `avais.log` e os `PED-NNN.md` gravados pela 0.3.0 continuam válidos. | Teste com as linhas e os pedidos no formato da 0.3.0. |
| RNF-03 | A recusa por falta de aval diz o que falta e como obtê-lo pelo painel. | Teste confere a mensagem. |
| RNF-04 | O `vostok pedidos vigiar` não precisa do painel aberto e não processa dois pedidos ao mesmo tempo. | Teste com dois pedidos e um `codex` simulado. |

## Regras de negócio

| ID | Regra |
|---|---|
| RN-01 | Só o painel grava o `avais.log`, e só por acréscimo; o agente nunca grava aval, `approved_by` nem chama as rotas do painel. |
| RN-02 | Um aval vale só para o ID, a ação e o hash que ele cita; um ajuste depois do aval o invalida. |
| RN-03 | O estado de um pedido muda só pelo `vostok pedidos`; o agente não edita o front-matter do pedido à mão. |
| RN-04 | O loop nunca decide pelo usuário: pedido ambíguo ou recusado pela CLI vai para `recusado`, com o motivo em `## Resultado`. |
| RN-05 | Com a trava ligada, uma ação travada pedida pelo chat também precisa do aval do painel. |

## Fluxos

### Fluxo principal

1. O usuário liga `approval_lock: true` em `vostok/config/main.yml` e abre `vostok painel`.
2. Na tarefa concluída pelo agente, o usuário clica em "Pedir finalização".
3. O painel grava `PED-NNN.md` com `acao: finalizar-tarefa` e acrescenta ao `avais.log` a linha com o `SPEC/TASK`, o `approved_hash` e a ação.
4. O chat em `/loop` com a skill `process-requests` acha o pedido e roda `vostok pedidos iniciar PED-NNN`.
5. O chat executa a skill `finish-task`; o `vostok task finish` acha o aval e conclui.
6. O chat roda `vostok pedidos concluir PED-NNN` com o resultado; o painel mostra o pedido como `feito`.

### Fluxos alternativos

- **FA-01 (passo 5):** sem aval correspondente, o `task finish` recusa citando a ação e o hash; o chat roda `vostok pedidos recusar PED-NNN` com a mensagem.
- **FA-02 (passo 4):** no Codex, o `vostok pedidos vigiar --agente codex` acha o pedido e dispara `codex exec` com a instrução da skill.
- **FA-03 (passo 2):** para commit, o usuário clica em "Pedir commit"; o chat roda `vostok commit commit --pedido PED-NNN`.
- **FA-04 (passo 1):** com a trava desligada, os comandos não conferem aval, e os pedidos seguem o mesmo fluxo.

## Casos de borda

| ID | Situação | Comportamento esperado | Requisitos |
|---|---|---|---|
| CB-01 | Aval com hash de uma versão anterior da tarefa | Recusado, citando o hash do aval e o atual | RF-03, RN-02 |
| CB-02 | Linha do `avais.log` com 4 colunas | Lida como ação `aprovar` | RF-02, RNF-02 |
| CB-03 | Linha malformada no `avais.log` | Ignorada na conferência e acusada pela validação | RF-02 |
| CB-04 | `commit commit --pedido` de um pedido `feito` | Recusado: o pedido precisa estar `em_andamento` | RF-06 |
| CB-05 | `vostok pedidos iniciar` de um pedido que não está `pendente` | Recusado, sem mudar o arquivo | RF-13 |
| CB-06 | `codex` ausente no PATH | `vostok pedidos vigiar` para com a instrução de instalar | RF-15 |
| CB-07 | Trava ligada e `avais.log` inexistente | Toda ação travada é recusada com a instrução do painel | RF-03, RNF-03 |

## Contratos e interfaces

```yaml
# vostok/config/main.yml
approval_lock: false   # true: concluir, aprovar e fazer commit exigem o aval do painel
```

```text
# vostok/work/avais.log (tabulações; só acréscimo)
<data UTC>	<ID>	<hash>	painel	<ação>
2026-11-03T10:00:00Z	SPEC-030/TASK-02	9f1c…	painel	finalizar-tarefa
2026-11-03T10:05:00Z	PED-004	3ab2…	painel	fazer-commit
```

```bash
vostok pedidos listar [--estado pendente|em_andamento|feito|recusado]
vostok pedidos iniciar PED-NNN
vostok pedidos concluir PED-NNN    # resultado pela entrada padrão
vostok pedidos recusar PED-NNN     # motivo pela entrada padrão
vostok pedidos vigiar --agente codex [--intervalo 15]
vostok commit commit --pedido PED-NNN <caminhos>
vostok commit approve --pedido PED-NNN [CODE]
```

O hash do aval é o `approved_hash` da tarefa (`finalizar-tarefa`), o SHA-256 do plano (`aprovar-plano`), o da spec refinada (`finalizar-spec`), o do item aprovado (`aprovar`) e o do arquivo do pedido no momento da criação (`fazer-commit`).

## Componentes afetados

| Caminho | Ação | Motivo |
|---|---|---|
| `apps/cli/src/aval.rs` | criar | Leitura do `avais.log`, da chave `approval_lock` e conferência do aval |
| `apps/cli/src/lib.rs` | alterar | Registrar o módulo `aval` |
| `apps/cli/src/commands/task.rs` | alterar | Aval no `finish` e no `approve`; `approved_by: painel` sem aval na validação |
| `apps/cli/src/commands/spec.rs` | alterar | Aval no `approve`; `approved_by: painel` sem aval na validação |
| `apps/cli/src/commands/plan.rs` | alterar | Aval no `mark-planned` |
| `apps/cli/src/commands/finalize.rs` | alterar | Aval no `archive` |
| `apps/cli/src/commands/commit.rs` | alterar | `--pedido` e aval no `commit` e no `approve` |
| `apps/cli/src/commands/pedidos.rs` | criar | Subcomandos `listar`, `iniciar`, `concluir`, `recusar` e `vigiar` |
| `apps/cli/src/commands/mod.rs` | alterar | Registrar o comando `pedidos` |
| `apps/cli/src/cli.rs` | alterar | Argumentos de `pedidos` e o `--pedido` do commit |
| `apps/cli/src/painel/actions.rs` | alterar | Aval nos pedidos travados, ação `fazer-commit` e aval de 5 colunas |
| `apps/painel/src/features/inbox/inbox-page.tsx` | alterar | Botão "Pedir commit" |
| `apps/painel/src/shared/attention.ts` | alterar | Ação `fazer-commit` na lista de ações |
| `apps/cli/tests/aval.rs` | criar | Testes da trava em cada comando |
| `apps/cli/tests/pedidos.rs` | criar | Testes do `vostok pedidos` e do `vigiar` com um `codex` simulado |
| `packages/harness/config/main.yml` | alterar | Chave `approval_lock: false` comentada |
| `packages/harness/rules/agent/boundaries.md` | alterar | Pedido do painel com aval como pedido do usuário |
| `packages/harness/skills/default/approve-spec/SKILL.md` | alterar | Regra `[approve-spec-01]` aceita o aval |
| `packages/harness/skills/default/approve-plan/SKILL.md` | alterar | Regra `[approve-plan-01]` aceita o aval |
| `packages/harness/skills/default/approve-task/SKILL.md` | alterar | Regra `[approve-task-01]` aceita o aval |
| `packages/harness/skills/default/finish-task/SKILL.md` | alterar | Regra `[finish-task-08]` aceita o aval |
| `packages/harness/skills/default/finalize-spec/SKILL.md` | alterar | Regra `[finalize-spec-01]` aceita o aval |
| `packages/harness/skills/default/approve-commit/SKILL.md` | alterar | Regra `[approve-commit-01]` e o `--pedido` |
| `packages/harness/skills/default/process-requests` | criar | Skill com a instrução do loop |
| `packages/harness/context/skill-catalog.md` | alterar | Entrada da skill `process-requests` |
| `packages/harness/config/context.yml` | alterar | Mapa de contexto da skill nova |

## Premissas e dependências

- A SPEC-029 está na `main` (commits `21e4438` a `9c1ff2e`), com o `avais.log` de 4 colunas e os `PED-NNN.md` com `estado: pendente`.
- O `codex exec` aceita a instrução como argumento e roda sem interação quando os comandos do `vostok` estão liberados no sandbox (estudo da SPEC-027).
- O `/loop` do Claude Code roda a instrução da skill a cada intervalo, como no teste do estudo.
- Os projetos recebem as skills e as regras novas pelo `vostok update`.

## Riscos

| Risco | Impacto | Mitigação |
|---|---|---|
| Um agente com shell grava um aval forjado | alto | O aval fica visível no `avais.log` e preso ao hash; impedir a escrita fica fora do escopo |
| A trava ligada trava o fluxo de quem não abre o painel | médio | Desligada por padrão; a recusa explica como dar o aval |
| Mudança no `actions.rs` quebra as rotas do painel da 0.3.0 | médio | Os testes da SPEC-029 continuam na suíte |
| O `codex exec` para pedindo permissão | médio | O `vigiar` registra a saída e recusa o pedido depois do tempo-limite |
| Mudar seis skills ao mesmo tempo gera conflito nas tarefas | baixo | Cada skill num caminho próprio em Componentes afetados |

## Casos de teste

| ID | Cenário | Passos | Resultado esperado | Requisitos |
|---|---|---|---|---|
| CT-01 | Trava desligada | Sem `approval_lock`, rodar `task finish` sem aval | Conclui como hoje | RF-01, RNF-01 |
| CT-02 | Aval nos dois formatos | `avais.log` com uma linha de 4 e uma de 5 colunas | As duas são lidas; a de 4 vira `aprovar` | RF-02, RNF-02 |
| CT-03 | Finish com e sem aval | Trava ligada; `task finish` sem e com o aval `finalizar-tarefa` | Recusa e depois conclui | RF-03, RNF-03 |
| CT-04 | Aval de outro hash | Aval com hash antigo da tarefa | Recusado, citando os dois hashes | RF-03, RN-02 |
| CT-05 | Aprovar plano | Trava ligada; `mark-planned` sem e com aval | Recusa e depois marca | RF-04 |
| CT-06 | Arquivar spec | Trava ligada; `finalize archive` sem e com aval | Recusa e depois arquiva | RF-05 |
| CT-07 | Commit | Trava ligada; `commit commit` sem `--pedido`, com pedido sem aval e com aval | Recusa duas vezes e depois faz o commit | RF-06 |
| CT-08 | Liberar commits | Trava ligada; `commit approve` sem e com `--pedido` válido | Recusa e depois libera | RF-07 |
| CT-09 | Aprovar spec e tarefa | Trava ligada; `spec approve approve` e `task approve approve` sem aval | Recusados com a instrução do painel | RF-08 |
| CT-10 | Validação | Tarefa com `approved_by: painel` sem linha no `avais.log` | `task adjust validate` acusa | RF-09 |
| CT-11 | Pedido travado grava aval | `POST /api/pedidos?acao=finalizar-tarefa` | `PED-NNN.md` e a linha de aval com o hash | RF-10 |
| CT-12 | Pedir commit | `POST /api/pedidos?acao=fazer-commit` e o botão na tela | Pedido e aval gravados; botão visível | RF-11 |
| CT-13 | Aprovação com 5 colunas | Aprovar tarefa pelo painel | Linha termina em `painel	aprovar` | RF-12 |
| CT-14 | Estados do pedido | `iniciar`, `concluir` e `recusar` | Estado, `atualizado_em` e `## Resultado` gravados; iniciar fora de `pendente` recusa | RF-13 |
| CT-15 | Loop do Claude Code | Chat com `/loop 1m` e a skill; um pedido `ajustar-tarefa` | Pedido termina `feito` ou `recusado` com resultado | RF-14, RN-04 |
| CT-16 | Vigiar com Codex | Dois pedidos pendentes e um `codex` simulado no PATH | Duas execuções, uma por vez, na ordem | RF-15, RNF-04 |
| CT-17 | Regras | Ler as regras das seis skills e de `boundaries.md` | Todas citam o aval do painel como pedido do usuário | RF-16 |

## Critérios de aceite

- [ ] CA-01: com a trava ligada, a CLI recusa concluir, aprovar, arquivar e fazer commit sem o aval do painel, e aceita com ele (RF-01, RF-03, RF-04, RF-05, RF-06, RF-07, RF-08).
- [ ] CA-02: o `avais.log` da 0.3.0 continua válido, e aprovações do painel sem aval são acusadas na validação (RF-02, RF-09, RF-12).
- [ ] CA-03: o painel grava o aval com os pedidos travados e oferece o pedido de commit (RF-10, RF-11).
- [ ] CA-04: um chat em loop no Claude Code e o `vostok pedidos vigiar` no Codex processam os pedidos até `feito` ou `recusado` (RF-13, RF-14, RF-15).
- [ ] CA-05: as regras do harness aceitam o pedido do painel com aval como pedido do usuário (RF-16).

## Referências

| Tipo | Referência | Uso |
|---|---|---|
| memória | `spec-trava-aval` | Origem da trava e das regras `ask` |
| memória | DEC-003 | Painel local, aval em `avais.log` e pedidos processados por um chat em loop |
| documentação | `docs/estudos/painel-local/testes.md` | Teste do `/loop 1m`, tempos e desenho do Codex |
| documentação | `docs/specs/SPEC-029/tecnico.html` | Como o painel aprova e grava os pedidos |
| arquivo | `apps/cli/src/painel/actions.rs` | Formato do `avais.log`, `PED-NNN.md` e as 10 ações |
| arquivo | `apps/cli/src/commands/task.rs` | `Origin`, `approve_from_panel`, `finish` e a validação |
| arquivo | `apps/cli/src/commands/commit.rs` | `commit approve` com código de liberação |
| arquivo | `packages/harness/rules/agent/boundaries.md` | Regra de commit a mudar |
| arquivo | `packages/harness/config/main.yml` | Onde entra a chave da trava |

## O que mudou

| # | Assunto | Estado no Original | Estado Refinado |
|---|---|---|---|
| 1 | Trava | "quando a trava estiver ligada", sem dizer onde | Chave `approval_lock` em `main.yml`, desligada por padrão |
| 2 | Commit | Conferência no `commit commit` e no `commit approve` | Pedido `fazer-commit` no painel e `--pedido` nos dois comandos |
| 3 | Codex | `vostok painel --agente codex`, como exemplo | Comando próprio `vostok pedidos vigiar --agente codex` |
| 4 | Regras `ask` | Citadas como camada extra | Fora do escopo |
| 5 | Formato do aval | Data, ID, hash e origem | Acrescenta a ação, mantendo as linhas de 4 colunas |
| 6 | Estados do pedido | Mudados pelo chat | Mudados pelo `vostok pedidos` |

## Registro de decisões

| # | Decisão | Alternativa descartada | Motivo |
|---|---|---|---|
| 1 | Trava opcional, desligada por padrão | Ligada por padrão ou sempre ligada | resposta 1 |
| 2 | Commit recebe o aval de um pedido `fazer-commit` | Commit fora da trava; botão de liberar commits | resposta 2 |
| 3 | Codex com comando próprio, `vostok pedidos vigiar` | Laço dentro do `vostok painel`; Codex fora da spec | resposta 3 |
| 4 | Sem regras `ask` | Regras `ask` como camada extra | resposta 4 |
| 5 | Chave `approval_lock` em `main.yml` | Arquivo de configuração próprio | inferência A |
| 6 | Quinta coluna com a ação no `avais.log`; 4 colunas lidas como `aprovar` | Novo arquivo de avais | inferência B |
| 7 | Hash do aval por ação (approved_hash, plano, spec, pedido) | Aval sem hash | inferência C |
| 8 | Com a trava, aprovações pelo chat exigem o aval `aprovar` | Deixar as aprovações pelo chat livres | inferência D |
| 9 | Estado do pedido mudado pelo `vostok pedidos` | Edição do front-matter pelo chat | inferência E |
| 10 | Instrução do loop como skill `process-requests` | Texto num documento | inferência F |
| 11 | Botão "Pedir commit" na tela de pendências | Botão no painel inicial | inferência G |
| 12 | `vigiar` com intervalo padrão de 15 s e um pedido por vez | Disparos em paralelo | inferência H |

## Pendências

| # | Pendência | Opções em aberto |
|---|---|---|
| - | Nenhuma | - |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
