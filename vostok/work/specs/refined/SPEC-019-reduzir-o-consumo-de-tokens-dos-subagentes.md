---
id: SPEC-019
title: "Reduzir o consumo de tokens dos subagentes"
status: implemented
author: "Victor Batalha"
created_at: 2026-10-31T10:15:05Z
updated_at: 2026-10-31T11:09:29Z
tags: []
type: improvement
source: "draft/SPEC-019-reduzir-o-consumo-de-tokens-dos-subagentes.md"
refined_at: 2026-10-31T10:23:56Z
adjustments: 0
adjusted_at: null
content_hash: cc5f5b5baf02e25fe7fa29ab199fb02ce7b22f0b3ce687f67fa47ccede07694c
approved_at: 2026-10-31T10:28:30Z
approved_hash: cc5f5b5baf02e25fe7fa29ab199fb02ce7b22f0b3ce687f67fa47ccede07694c
---

# Reduzir o consumo de tokens dos subagentes - Especificação Refinada

Reduz o consumo de tokens do Claude no uso do harness: a `execute-task` passa a executar no Coordenador as ondas de uma tarefa e a usar subagentes só para tarefas paralelas, com um limite configurável (padrão 3); o subagente recebe um briefing curto (`vostok task brief`) em vez do contexto obrigatório e da skill inteiros, lê trechos em vez de arquivos inteiros, roda a verificação com saída resumida (`vostok task check`), tem `maxTurns` e devolve um retorno curto; a `finalize-spec` roda no Coordenador. Estudo de apoio: `docs/estudos/consumo-de-tokens.md`.

## Objetivo

Diminuir o consumo de tokens por tarefa executada, com o Coordenador fazendo o máximo possível e os subagentes só o estritamente necessário, sem perder a garantia do baseline nem a rastreabilidade das tarefas.

## Estado atual

- A `execute-task` dispara um subagente por tarefa em toda onda, com no máximo 6 ao mesmo tempo (texto fixo em [execute-task-10] e no passo 5); a execução na própria sessão (FA-01) só vale sem subagentes na ferramenta.
- O pedido ao subagente manda rodar `vostok context load --persona <persona>` (cerca de 7,9 mil tokens: contexto obrigatório com 17 arquivos de regras e a persona) e `vostok context load --skill execute-task` (3,2 mil) e seguir a skill (`SKILL.md`, 3,3 mil, com os passos do Coordenador); somado ao `vostok task load` (3,2 mil), o subagente carrega cerca de 17,6 mil tokens antes de ler código.
- A lista "Ler" cita arquivos inteiros e o subagente os lê inteiros: `apps/cli/src/commands/task.rs` (cerca de 34,7 mil tokens) e `apps/cli/tests/task.rs` (17,2 mil), relidos a cada turno.
- O subagente roda a "Verificação" com os comandos crus (`cargo test` inteiro), com a saída completa no contexto.
- As definições `.claude/agents/vostok-<persona>.md` (geradas por `render_agent` em `apps/cli/src/routing.rs`) têm só `name`, `description` e `model: inherit`, sem limite de turnos.
- O Coordenador consolida relendo diffs; a finalização de spec foi feita por subagentes (cerca de 70 mil tokens cada).
- Totais observados em 2026-10-30: 55 mil a 92 mil tokens por subagente de tarefa (estudo, seção "Totais observados").

## Escopo

### Dentro do escopo

- A1: execução no Coordenador para ondas de uma tarefa e para tarefas em sequência (arquivos em comum); subagentes só para 2 ou mais tarefas paralelas da onda.
- A2: chave `max_subagents` em `config/tasks.yml`, padrão 3, teto 6, lida pela `execute-task`.
- A3: comando `vostok task brief <SPEC_ID> <TASK_ID>` e pedido ao subagente reduzido a ele.
- A4: orientação de "Ler" com trechos na `approve-plan` e leitura por busca e trecho no roteiro do subagente.
- A5: `maxTurns` nas definições de agente do Claude Code.
- A6: retorno curto do subagente e consolidação sem reler diffs.
- A7: comando `vostok task check <SPEC_ID> <TASK_ID>`, que roda a "Verificação" sem registrar status e imprime só o resumo e as falhas.
- A8: `finalize-spec` executada no Coordenador, sem subagente.
- Medição manual antes e depois com `/usage`, registrada como evidência.

### Fora do escopo

- `effort` nas definições de agente (a definição é por persona, não por perfil).
- Mudanças nas definições de agente do Codex e do Cursor.
- Medição automática de tokens pelo `vostok` (o harness não vê o consumo).
- Mudança no modelo escolhido por perfil (`profile_models`).

## Requisitos funcionais

| ID | Requisito |
|---|---|
| RF-01 | A `execute-task` executa no Coordenador, na própria sessão e na ordem dos IDs, a onda com uma tarefa e as tarefas que rodam em sequência por arquivo em comum; só dispara subagentes para 2 ou mais tarefas da onda que podem rodar em paralelo. |
| RF-02 | `config/tasks.yml` ganha a chave `max_subagents` (padrão 3); a `execute-task` usa esse valor como limite de subagentes ao mesmo tempo, e um valor fora de 1 a 6 vale 6 com aviso de `vostok task status`. |
| RF-03 | `vostok task brief <SPEC_ID> <TASK_ID>` imprime, só para uma tarefa `in_progress` com baseline coletado: as regras de limites e segurança do contexto obrigatório (`rules/agent/boundaries.md` e as regras de segurança do projeto mapeadas para `execute-task`), a persona da tarefa, o roteiro do subagente (carregar, conferir baseline, testes, código, retorno) e a seção "Execução" da tarefa. |
| RF-04 | A saída do `vostok task brief` fica abaixo de 40% da soma das saídas de `vostok context load --persona <persona>`, `vostok context load --skill execute-task` e `vostok task load` para a mesma tarefa. |
| RF-05 | O pedido ao subagente passa a ser: rodar `vostok task brief <SPEC_ID> <TASK_ID>` e seguir o que ele imprime; o subagente não roda `vostok context load` nem lê a `SKILL.md` da `execute-task`. |
| RF-06 | A `approve-plan` orienta "Ler" a citar, para arquivo grande (mais de 400 linhas), a função, o teste ou a faixa de linhas (`` - `caminho` (fn nome) ``); o roteiro do briefing manda ler esses arquivos por busca e trecho, nunca inteiros. |
| RF-07 | As definições `.claude/agents/vostok-<persona>.md` geradas pelo `vostok init`/`vostok update` têm `maxTurns`, com valor fixo por persona no código (`desenvolvedor` e `ui-ux-designer` 60, demais 30). |
| RF-08 | `vostok task check <SPEC_ID> <TASK_ID>` executa o bloco da "Verificação" como o `vostok task tests`, sem mudar status, tentativas nem registro; grava a saída em `validation/check.out` e imprime só o código de saída e, em falha, as últimas 30 linhas. O roteiro do briefing manda usá-lo em vez dos comandos crus. |
| RF-09 | O subagente devolve um bloco fixo de até 5 linhas (resultado, arquivos mudados, verificação pelo `vostok task check`, em aberto); a `execute-task` consolida com `vostok task status`, a linha `baseline` do registro e `git diff --stat`, sem reler os diffs. |
| RF-10 | A `finalize-spec` diz que é executada no Coordenador, uma spec por vez, sem disparar subagente. |

## Requisitos não funcionais

| ID | Requisito | Como verificar |
|---|---|---|
| RNF-01 | A redução é comprovada por medição: a mesma tarefa média da CLI, executada em sessão nova no fluxo atual (0.2.3) e no fluxo novo, tem consumo menor no `/usage`, com os números registrados. | Evidência manual com a saída do `/usage` dos dois modos. |
| RNF-02 | Os comandos novos são determinísticos e não usam rede nem crates novas. | Testes rodam duas vezes com o mesmo resultado; `Cargo.toml` sem dependência nova. |

## Regras de negócio

| ID | Regra |
|---|---|
| RN-01 | O Coordenador faz o máximo possível; subagente só existe para paralelismo real (2 ou mais tarefas paralelas). |
| RN-02 | O briefing nunca omite as regras de limites e de segurança: o subagente continua sem commit, sem stage e sem ler segredos. |
| RN-03 | A garantia do baseline (SPEC-016) não muda: o Coordenador inicia toda tarefa da onda antes de executar ou disparar a primeira. |

## Fluxos

### Fluxo principal

1. O usuário pede para executar uma onda; o Coordenador inicia todas as tarefas da onda (`vostok task start`, que coleta o baseline).
2. Onda de uma tarefa, ou grupo em sequência: o Coordenador executa cada tarefa na própria sessão, com `vostok task brief` como roteiro.
3. Duas ou mais tarefas paralelas: o Coordenador dispara até `max_subagents` subagentes, cada um com o pedido de rodar `vostok task brief` e seguir.
4. Cada subagente lê trechos, implementa, roda `vostok task check` e devolve o bloco curto.
5. O Coordenador consolida com `vostok task status`, a linha `baseline` e `git diff --stat` e responde ao usuário.

### Fluxos alternativos

- **FA-01, ferramenta sem subagentes:** igual ao passo 2 para todas as tarefas.
- **FA-02, `vostok task check` falha:** o subagente corrige e roda de novo; sem conseguir, devolve `blocker` com as linhas impressas.
- **FA-03, tarefa sem baseline ao pedir o briefing:** `vostok task brief` recusa com o comando de coleta.

## Casos de borda

| ID | Situação | Comportamento esperado | Requisitos |
|---|---|---|---|
| CB-01 | `max_subagents` ausente (projeto antigo) | Vale 3; o `vostok update` acrescenta a chave | RF-02 |
| CB-02 | `max_subagents` 0, negativo, não numérico ou maior que 6 | Vale 6 com aviso | RF-02 |
| CB-03 | Onda com 2 tarefas que dividem arquivos | Rodam em sequência no Coordenador, sem subagente | RF-01 |
| CB-04 | `vostok task brief` em tarefa `approved`, `done` ou sem baseline | Recusa com a mensagem do status e o comando a rodar | RF-03 |
| CB-05 | Tarefa sem persona (`-` ou `null`) | O briefing usa a persona `desenvolvedor` | RF-03 |
| CB-06 | `vostok task check` em tarefa bloqueada | Recusa como o `vostok task tests` | RF-08 |
| CB-07 | Definição de agente editada à mão sem `maxTurns` | O `vostok update` regrava como hoje regrava as definições | RF-07 |

## Contratos e interfaces

- `vostok task brief <SPEC_ID> <TASK_ID>`: imprime em Markdown as seções `# Regras`, `# Persona`, `# Roteiro` e `# Tarefa`; recusa com `a tarefa <TASK_ID> está com status '<status>'; o briefing só sai para tarefa 'in_progress' com baseline coletado`.
- `vostok task check <SPEC_ID> <TASK_ID>`: sucesso imprime `Verificação de <TASK_ID> passou (saída em <caminho>)`; falha imprime `Verificação de <TASK_ID> falhou (código <N>); últimas linhas (<caminho>):` e as 30 últimas linhas, e sai com código 1.
- `config/tasks.yml`: `max_subagents: 3`.
- `.claude/agents/vostok-<persona>.md`: front-matter com `maxTurns: <N>` depois de `model: inherit`.

## Componentes afetados

| Caminho | Ação | Motivo |
|---|---|---|
| `apps/cli/src/commands/task.rs` | alterar | Comandos `brief` e `check` (RF-03, RF-04, RF-08) e aviso de `max_subagents` no `status` (RF-02) |
| `apps/cli/src/routing.rs` | alterar | `maxTurns` no `render_agent` do Claude Code (RF-07) |
| `packages/harness/config/tasks.yml` | alterar | Chave `max_subagents` (RF-02) |
| `packages/harness/skills/default/execute-task/SKILL.md` | alterar | Coordenador por padrão, limite configurável, pedido pelo briefing, consolidação curta (RF-01, RF-02, RF-05, RF-09) |
| `packages/harness/skills/default/approve-plan/SKILL.md` | alterar | "Ler" com trechos (RF-06) |
| `packages/harness/skills/default/finalize-spec/SKILL.md` | alterar | Execução no Coordenador (RF-10) |
| `apps/cli/tests/task.rs` | alterar | Testes de `brief`, `check` e do aviso de `max_subagents` |
| `apps/cli/tests/init_update.rs` | alterar | Teste do `maxTurns` nas definições geradas |
| `apps/cli/tests/base.rs` | alterar | O `ct11_execute_task_runs_a_task_or_a_wave` confere `no máximo 6 subagentes`, que muda |

## Premissas e dependências

- O Claude Code aceita `maxTurns` no front-matter das definições de subagente (documentação `sub-agents`).
- O `vostok update` acrescenta chaves novas de topo em `config/tasks.yml` sem mudar as existentes.
- A medição do RNF-01 é feita pelo usuário ou pelo Coordenador com `/usage` em sessões novas; o `vostok` não mede tokens.

## Riscos

| Risco | Impacto | Mitigação |
|---|---|---|
| Briefing omite contexto que o subagente precisava | Médio | Regras de limites e segurança sempre presentes (RN-02); o roteiro manda ler as regras do projeto citadas na tarefa quando necessário |
| Ler por trecho deixa o subagente sem o contexto ao redor | Médio | Passos citam funções e testes; o roteiro permite ampliar o trecho por busca |
| Chat principal longo fica mais caro ao executar tarefas | Médio | Só ondas de uma tarefa ou em sequência rodam nele; a medição do RNF-01 confirma o ganho |
| `maxTurns` corta uma tarefa grande | Baixo | Valor folgado para `desenvolvedor` (60); o subagente pode ser retomado |

## Casos de teste

| ID | Cenário | Passos | Resultado esperado | Requisitos |
|---|---|---|---|---|
| CT-01 | Briefing de tarefa em execução | Tarefa `in_progress` com baseline; `vostok task brief` | Imprime as quatro seções e a "Execução" da tarefa | RF-03 |
| CT-02 | Briefing menor | Mesma tarefa; comparar o tamanho do `brief` com a soma dos três comandos atuais | Menos de 40% | RF-04 |
| CT-03 | Briefing recusado | Tarefa `approved` e tarefa `in_progress` sem baseline | Recusa com a mensagem do contrato | RF-03, CB-04 |
| CT-04 | Persona padrão | Tarefa sem persona | Briefing com a persona `desenvolvedor` | CB-05 |
| CT-05 | Check passa e falha | Verificação `true` e `false` | Mensagens do contrato; status, tentativas e registro inalterados; `check.out` gravado | RF-08 |
| CT-06 | Check bloqueada | Tarefa `blocked` | Recusa | CB-06 |
| CT-07 | `max_subagents` | Valores 3, ausente, 0 e 9 em `config/tasks.yml`; `vostok task status` | Sem aviso para 3 e ausente; aviso para 0 e 9 | RF-02, CB-01, CB-02 |
| CT-08 | `maxTurns` gerado | `vostok init` com Claude Code | `.claude/agents/vostok-desenvolvedor.md` com `maxTurns: 60` e `vostok-qa.md` com `maxTurns: 30` | RF-07 |
| CT-09 | Textos das skills | `grep` na `execute-task`, `approve-plan` e `finalize-spec` | Regra do Coordenador por padrão, `max_subagents`, pedido pelo `vostok task brief`, consolidação por `git diff --stat`, "Ler" com trechos e finalização no Coordenador | RF-01, RF-05, RF-06, RF-09, RF-10 |
| CT-10 | Medição | Mesma tarefa média da CLI em sessão nova na 0.2.3 e no fluxo novo; `/usage` | Consumo menor no fluxo novo, números registrados | RNF-01 |

## Critérios de aceite

- [ ] CA-01: Ondas de uma tarefa e tarefas em sequência rodam no Coordenador, e subagentes só para tarefas paralelas, até `max_subagents` (padrão 3). (RF-01, RF-02)
- [ ] CA-02: O subagente recebe só o `vostok task brief`, menor que 40% da carga atual, com as regras de limites e segurança. (RF-03, RF-04, RF-05)
- [ ] CA-03: O subagente lê trechos, verifica pelo `vostok task check`, tem `maxTurns` e devolve um bloco curto consolidado sem reler diffs. (RF-06, RF-07, RF-08, RF-09)
- [ ] CA-04: A finalização de spec roda no Coordenador. (RF-10)
- [ ] CA-05: A medição com `/usage` mostra consumo menor no fluxo novo para a mesma tarefa. (RNF-01)

## Referências

| Tipo | Referência | Uso |
|---|---|---|
| Estudo | docs/estudos/consumo-de-tokens.md | Medições, técnicas e alavancas A1 a A8 |
| Estudo | docs/estudos/multiagentes.md | Limite de 6 subagentes por onda |
| Skill | packages/harness/skills/default/execute-task/SKILL.md ([execute-task-10], passos 5 e 6, pedido ao subagente) | Fluxo atual |
| Código | apps/cli/src/routing.rs (render_agent) | Geração das definições de agente |
| Código | apps/cli/src/commands/task.rs (tests, load, status) | Base dos comandos novos |
| Documentação | code.claude.com/docs/en/costs e /sub-agents | maxTurns, modelo por subagente, prompts focados |

## O que mudou

| # | Assunto | Estado no Original | Estado Refinado |
|---|---|---|---|
| 1 | Alavancas | "Limitar agentes, passar só o necessário, Coordenador faz o máximo" | A1 a A8 do estudo (decisão 1B) |
| 2 | Quando usar subagente | Não definido | Só para 2 ou mais tarefas paralelas (decisão 2A) |
| 3 | Limite | Fixo em 6 | `max_subagents`, padrão 3 (decisão 3A) |
| 4 | Comprovação | Não definida | Critérios determinísticos e medição com `/usage` (decisão 4A) |

## Registro de decisões

| # | Decisão | Alternativa descartada | Motivo |
|---|---|---|---|
| 1 | Todas as alavancas A1 a A8 (usuário, 1B) | Só as de maior custo | Pedido do usuário |
| 2 | Coordenador para onda de uma tarefa e tarefas em sequência (usuário, 2A) | Também `mecanica`; sempre no Coordenador | Paralelismo real justifica o piso do subagente |
| 3 | `max_subagents` padrão 3, teto 6 (usuário, 3A; inferência F) | Padrão 2; fixo | Configurável sem passar do limite estudado |
| 4 | Critérios determinísticos e medição manual (usuário, 4A) | Meta numérica fixa | O `vostok` não mede tokens |
| 5 | Briefing como comando `vostok task brief` (inferência B) | Texto maior no pedido | Determinístico e testável |
| 6 | "Ler" com trechos só por orientação (inferência C) | Validação nova | Nem todo arquivo grande tem função citável |
| 7 | `maxTurns` só no Claude Code, por persona, sem `effort` (inferência D) | `effort` por perfil | A definição é por persona |
| 8 | Retorno em bloco fixo e consolidação por `git diff --stat` (inferência E) | Reler diffs | Menos turnos no contexto mais caro |
| 9 | Verificação resumida como comando `vostok task check` (inferência) | Filtro por hook | Funciona em qualquer ferramenta |
| 10 | Limite de 40% para o briefing e de 400 linhas para "arquivo grande" (inferência) | Sem limite | Critério verificável a partir das medições do estudo |

## Pendências

| # | Pendência | Opções em aberto |
|---|---|---|
| - | Nenhuma | - |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
