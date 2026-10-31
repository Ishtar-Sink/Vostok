---
id: SPEC-021
title: "Regressão que não é regressão"
status: implemented
author: "Victor Batalha"
created_at: 2026-10-31T12:08:25Z
updated_at: 2026-10-31T15:24:41Z
tags: []
type: improvement
source: "draft/SPEC-021-regressao-que-nao-e-regressao.md"
refined_at: 2026-10-31T13:56:17Z
adjustments: 0
adjusted_at: null
content_hash: 660f2e18870e4c412b0ff60ade1b1a3a8c3882f09ea32480e7989bf56fb76374
approved_at: 2026-10-31T14:02:57Z
approved_hash: 660f2e18870e4c412b0ff60ade1b1a3a8c3882f09ea32480e7989bf56fb76374
---

# Regressão que não é regressão - Especificação Refinada

A regressão da finalização deixa de bloquear quando a tarefa só acrescenta linhas a uma saída (testes novos numa lista) e quando o planejamento declarou que o comando muda de propósito; uma linha nova de alerta continua bloqueando, e aceitar uma diferença deixa de obrigar a rodar tudo de novo. Estudo de apoio: `docs/estudos/regressao-falsa.md`.

## Objetivo

Acabar com os bloqueios de regressão que não são regressão (8 de 8 nas tarefas arquivadas), mantendo o bloqueio quando algo que existia some, muda ou piora, e reduzir o custo do aceite quando o bloqueio acontecer.

## Estado atual

- O `vostok task start` grava, para cada comando de "Baseline", `$ comando`, `exit: N` e a saída (`baseline/NN.out`, `recorded_run` em `apps/cli/src/commands/task.rs`).
- O `vostok task regression` compara a gravação inteira, byte a byte, depois de tirar dos dois lados as linhas com textos de `regression_ignore` (`without_ignored`); qualquer diferença bloqueia a tarefa.
- Comandos que citam um caminho `criar` ou pasta nova são ignorados por inteiro (SPEC-017, SPEC-018); comando idêntico a um rotulado da Verificação reaproveita a saída gravada (SPEC-020).
- O aceite (`vostok task resume … --accept-regression N`) zera tentativas e resultados, e a tarefa precisa rodar testes, regressão e evidências de novo.
- A `approve-plan` orienta medir o que a tarefa não pode quebrar, filtrar os nomes de testes novos e tirar a contagem; mesmo assim, o filtro por substring do `cargo test` pegou testes novos na SPEC-020.
- Nas 95 tarefas arquivadas, 8 bloqueios por regressão, todos aceitos: 3 por saída que só cresceu (testes novos) e 5 por Baseline que media a própria entrega (contagens e validação).

## Escopo

### Dentro do escopo

- Acréscimo que não bloqueia, com linhas de alerta (`regression_alert`).
- Item de Baseline `(muda): motivo`, mostrado sem bloquear e registrado como esperado.
- Aceite que mantém os testes aprovados e roda só a regressão de novo.
- Orientação na `approve-plan` e na `qa-lead` sobre o `muda` e sobre o critério novo.

### Fora do escopo

- Conferir os comandos de Baseline no planejamento (estudo, A5): volta se os bloqueios por medir a entrega continuarem.
- Comparação por teste conforme o formato de cada executor (estudo, A3).
- Mudar o tratamento dos comandos que medem a entrega (SPEC-017 e SPEC-018).

## Requisitos funcionais

| ID | Requisito |
|---|---|
| RF-01 | O `vostok task regression` aceita como acréscimo uma diferença em que o código de saída é o mesmo do baseline e todas as linhas do baseline aparecem na saída atual, na mesma ordem, depois de aplicar o `regression_ignore`. |
| RF-02 | Um acréscimo com alguma linha nova que contém um texto de `regression_alert` (sem diferenciar maiúsculas) bloqueia como hoje, citando a linha. |
| RF-03 | `config/tasks.yml` ganha `regression_alert`, com o padrão `FAILED`, `error`, `warning` e `panicked`; sem a chave, valem esses textos. |
| RF-04 | O registro e a saída do `vostok task regression` informam a quantidade de comandos com acréscimo e, na saída, cada comando com o número de linhas novas. |
| RF-05 | Um item de Baseline pode ser `` - `comando` (muda): motivo ``; a regressão roda o comando, grava a diferença em `NN.diff`, não bloqueia e registra o comando como mudança esperada, com o motivo. |
| RF-06 | `vostok task resume <SPEC_ID> <TASK_ID> "decisão" --accept-regression <N>` mantém `tests_result: passed` e o `evidence_result`, zera só o `regression_result`, e a tarefa segue com `vostok task regression`. |
| RF-07 | A `approve-plan`, a `finish-task` e a persona `qa-lead` descrevem o critério novo (acréscimo e alerta), o `(muda)` para comandos que a tarefa muda de propósito e o aceite que roda só a regressão de novo. |

## Requisitos não funcionais

| ID | Requisito | Como verificar |
|---|---|---|
| RNF-01 | Compatibilidade: baselines coletados antes desta versão e tarefas sem `(muda)` são comparados com o critério novo, sem migração. | Teste com um baseline gravado no formato atual. |
| RNF-02 | Sem crates novas; a comparação por subsequência é linear no tamanho das saídas. | `Cargo.toml` sem dependência nova. |

## Regras de negócio

| ID | Regra |
|---|---|
| RN-01 | Algo que existia e some, muda ou piora (linha removida ou alterada, código de saída diferente) continua sendo regressão. |
| RN-02 | Uma linha nova de alerta é regressão, mesmo num acréscimo. |
| RN-03 | O `(muda)` é decidido no planejamento, aprovado com a tarefa, e nunca escrito durante a execução. |
| RN-04 | Os comandos que medem a entrega (SPEC-017 e SPEC-018) continuam ignorados antes de qualquer regra desta spec. |

## Fluxos

### Fluxo principal

1. O `vostok task regression` roda cada comando de Baseline (ou reaproveita a saída da Verificação).
2. Comando que mede a entrega: ignorado (RN-04).
3. Comando `(muda)`: diferença gravada e registrada como esperada, sem bloquear (RF-05).
4. Saída igual: conta como igual.
5. Saída diferente com o mesmo código de saída e o baseline contido em ordem: acréscimo; com linha de alerta nova, bloqueia (RF-01, RF-02).
6. Os demais casos bloqueiam como hoje.
7. A saída e o registro informam iguais, acréscimos, mudanças esperadas, aceitos e ignorados (RF-04).

### Fluxos alternativos

- **FA-01 (passo 6), bloqueio aceito pelo usuário:** `vostok task resume … --accept-regression N` mantém os testes e as evidências e o agente roda só `vostok task regression` antes do `finish` (RF-06).
- **FA-02 (passo 5), alerta num acréscimo:** a saída cita o comando e a linha de alerta; o usuário decide como hoje.

## Casos de borda

| ID | Situação | Comportamento esperado | Requisitos |
|---|---|---|---|
| CB-01 | Lista de testes com testes novos e a linha de contagem | Acréscimo: a contagem sai pelo `regression_ignore` e os nomes novos são linhas novas | RF-01 |
| CB-02 | Teste que passava aparece como `FAILED` | Bloqueia: a linha `ok` sumiu e há linha de alerta | RF-01, RF-02 |
| CB-03 | Código de saída 0 vira 1 sem mudança no texto | Bloqueia | RF-01 |
| CB-04 | Saída reordenada com as mesmas linhas | Bloqueia (as linhas do baseline não estão na mesma ordem) | RF-01 |
| CB-05 | Item `(muda)` sem motivo | A validação de tarefa recusa | RF-05 |
| CB-06 | `(muda)` num comando que mede a entrega | Ignorado como entrega (RN-04) | RF-05 |
| CB-07 | Aceite numa tarefa com testes reprovados | Mantém o resultado dos testes como está; a finalização exige testes aprovados | RF-06 |

## Contratos e interfaces

Baseline com mudança esperada:

```markdown
- `cargo test --manifest-path apps/cli/Cargo.toml --test task 2>&1 | grep -E '^test ' | sort`
- `ls rules/agent | wc -l` (muda): a tarefa acrescenta a regra rule-suggestions.md
```

Configuração (`config/tasks.yml`):

```yaml
regression_alert:
  - FAILED
  - error
  - warning
  - panicked
```

Saída do `vostok task regression` sem bloqueio:

```text
Sem regressão em TASK-01: 1 igual(is) ao baseline; 1 com acréscimo; 1 mudança esperada.
- [2] cargo test …: acréscimo de 3 linha(s)
- [3] ls rules/agent | wc -l: muda (a tarefa acrescenta a regra rule-suggestions.md)
```

## Componentes afetados

| Caminho | Ação | Motivo |
|---|---|---|
| `apps/cli/src/commands/task.rs` | alterar | Critério de acréscimo e alerta, `(muda)`, aceite que mantém testes e evidências |
| `apps/cli/tests/task.rs` | alterar | Testes de acréscimo, alerta, `(muda)` e aceite |
| `packages/harness/config/tasks.yml` | alterar | Chave `regression_alert` |
| `packages/harness/skills/default/approve-plan/SKILL.md` | alterar | Orientação do `(muda)` e do critério novo no Baseline |
| `packages/harness/skills/default/finish-task/SKILL.md` | alterar | Retomada que roda só a regressão de novo |
| `packages/harness/personas/qa-lead.md` | alterar | Regra sobre comando que a tarefa muda de propósito |

## Premissas e dependências

- Os baselines gravam `$ comando` e `exit: N` nas duas primeiras linhas, como hoje.
- O `vostok update` acrescenta `regression_alert` em projetos antigos; sem a chave, valem os padrões.

## Riscos

| Risco | Impacto | Mitigação |
|---|---|---|
| Linha nova de problema sem texto de alerta passa como acréscimo | médio | Lista configurável; o registro lista os acréscimos para quem confere |
| `(muda)` usado para esconder uma regressão | médio | Vem do planejamento e é aprovado pelo usuário com a tarefa (RN-03) |
| Saída com ordem instável vira bloqueio | baixo | A `approve-plan` já orienta ordenar saídas |

## Casos de teste

| ID | Cenário | Passos | Resultado esperado | Requisitos |
|---|---|---|---|---|
| CT-01 | Testes novos numa lista | Baseline lista testes; a tarefa acrescenta dois testes | Regressão passa com 1 acréscimo | RF-01, RF-04 |
| CT-02 | Linha removida | A saída perde uma linha do baseline | Bloqueia | RF-01 |
| CT-03 | Alerta num acréscimo | Linha nova `test x ... FAILED` | Bloqueia citando a linha | RF-02 |
| CT-04 | Código de saída diferente | Mesma saída, código 1 | Bloqueia | RF-01 |
| CT-05 | Alerta configurável | `regression_alert` com `ERRO`; linha nova com `erro` | Bloqueia | RF-03 |
| CT-06 | Mudança esperada | Item `(muda)` com contagem 5 para 6 | Passa, `NN.diff` gravado e registro com o motivo | RF-05 |
| CT-07 | `(muda)` sem motivo | Validar a tarefa | Recusa | RF-05 |
| CT-08 | Aceite barato | Bloqueio, `resume --accept-regression 1` | `tests_result` e `evidence_result` mantidos; `regression` passa; `finish` conclui sem rodar testes de novo | RF-06 |
| CT-09 | Orientação | Ler `approve-plan`, `finish-task` e `qa-lead` | Textos descrevem acréscimo, alerta, `(muda)` e o aceite | RF-07 |
| CT-10 | Baseline antigo | Baseline gravado no formato atual com saída que cresce | Passa como acréscimo | RNF-01 |

## Critérios de aceite

- [ ] CA-01: Testes novos numa lista do Baseline não bloqueiam a regressão (RF-01, RF-04).
- [ ] CA-02: Linha removida, alterada, código de saída diferente ou linha nova de alerta continuam bloqueando (RF-01, RF-02).
- [ ] CA-03: `regression_alert` é configurável, com os padrões da spec (RF-03).
- [ ] CA-04: Um item `(muda)` mostra a diferença sem bloquear e registra o motivo (RF-05).
- [ ] CA-05: O aceite mantém testes e evidências e exige só a regressão de novo (RF-06).
- [ ] CA-06: As skills e a `qa-lead` descrevem o critério novo, o `(muda)` e o aceite (RF-07).
- [ ] CA-07: Baselines já coletados funcionam com o critério novo (RNF-01).

## Referências

| Tipo | Referência | Uso |
|---|---|---|
| documentação | `docs/estudos/regressao-falsa.md` | Os 8 bloqueios, as causas e as alternativas A1 a A6 |
| arquivo | `apps/cli/src/commands/task.rs` | `regression`, `recorded_run`, `without_ignored`, `resume`, `regression_acceptances` |
| arquivo | `packages/harness/config/tasks.yml` | `regression_ignore` |
| arquivo | `packages/harness/skills/default/approve-plan/SKILL.md` | Orientação atual do Baseline |
| memória | ISS-001 | O aceite por número continua valendo |

## O que mudou

| # | Assunto | Estado no Original | Estado Refinado |
|---|---|---|---|
| 1 | Critério da regressão | Não definido | Acréscimo não bloqueia; alerta e perda continuam bloqueando |
| 2 | Comando que muda de propósito | Não definido | Item `(muda): motivo` |
| 3 | Aceite | Não definido | Mantém testes e evidências |

## Registro de decisões

| # | Decisão | Alternativa descartada | Motivo |
|---|---|---|---|
| 1 | Acréscimo com linhas de alerta | Qualquer acréscimo passa; comparação exata | resposta 1A |
| 2 | Mudança declarada `(muda)` | Evitar o comando no planejamento | resposta 2A |
| 3 | Aceite que roda só a regressão | Zerar tudo, como hoje | resposta 3A |
| 4 | Conferir o Baseline no planejamento fica fora | Entrar agora | resposta 4A |
| 5 | Tipo `improvement` | `fix` | inferência A |
| 6 | Comandos da entrega ignorados antes das regras novas | Aplicar o acréscimo também a eles | inferência B |
| 7 | Alerta sem diferenciar maiúsculas, só nas linhas novas | Em toda a saída | inferência C |
| 8 | Linha que some ou muda bloqueia mesmo com acréscimos | Tolerar | inferência D |
| 9 | Tarefas antigas usam o critério novo sem migração | Manter o antigo para elas | inferência G |
| 10 | `approve-plan` e `qa-lead` orientam o `(muda)` | Só documentação | inferência H |

## Pendências

| # | Pendência | Opções em aberto |
|---|---|---|
| - | Nenhuma | - |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
