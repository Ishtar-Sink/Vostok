---
id: SPEC-022
title: "Baseline sem rodar evidências que só podem falhar antes da tarefa"
status: implemented
author: "Victor Batalha"
created_at: 2026-10-31T12:41:33Z
updated_at: 2026-10-31T17:04:50Z
tags: []
type: improvement
source: "draft/SPEC-022-baseline-sem-rodar-evidencias-que-so-podem-falhar-antes-da-tarefa.md"
refined_at: 2026-10-31T13:56:17Z
adjustments: 0
adjusted_at: null
content_hash: 1866ed13ec94cbe469a813de254e43388fe8ee0e744cc8fa7e73471f99a5b487
approved_at: 2026-10-31T14:02:57Z
approved_hash: 1866ed13ec94cbe469a813de254e43388fe8ee0e744cc8fa7e73471f99a5b487
---

# Baseline sem rodar evidências que só podem falhar antes da tarefa - Especificação Refinada

O baseline deixa de rodar evidências que só podem falhar antes da tarefa: o subitem `Requer:` declara os caminhos `criar` de que a evidência depende, as evidências no baseline ganham um tempo-limite configurável com exceção por evidência aprovada pelo usuário, e o `vostok task start` mostra progresso. Melhoria da issue #8 (`[harness-improvement]`); estudo de apoio: `docs/estudos/baseline-de-evidencias.md`.

## Objetivo

Fazer o início de uma tarefa com evidências caras levar segundos, e não dezenas de minutos, sem perder o antes e depois das evidências que dizem algo antes da tarefa, e deixar visível o progresso de um baseline longo.

## Estado atual

- O `vostok task start` (`start` em `apps/cli/src/commands/task.rs`) confere a tarefa, coleta o baseline (`collect_baseline`) e só então marca `in_progress` e grava as linhas `início` e `baseline` no registro; nada é impresso durante a coleta.
- `collect_baseline` roda os comandos de "Baseline" e, com `evidence_baseline_dir`, o comando de cada evidência `(automática)` (`automatic_evidences`), sem tempo-limite, e grava tudo junto no fim.
- O `vostok task evidence` avisa quando a saída final é igual à de antes; o `evidencias.html` (0.2.5) mostra a saída de antes ao lado da de depois.
- As evidências `(automática, da verificação)`, `(diff)` e `(print)` não rodam no baseline.
- Nas tarefas arquivadas: 50 evidências rodaram no baseline (25 tarefas), metade terminou com erro antes da tarefa, e o aviso de "saída igual" nunca apareceu.
- Na issue #8, duas evidências (suíte ponta a ponta com tempo-limite de 30 s por teste e Lighthouse com login) fizeram o `vostok task start` passar de 10 minutos, sem sinal de progresso.

## Escopo

### Dentro do escopo

- Subitem `Requer:` com caminhos `criar` da tarefa.
- Tempo-limite das evidências no baseline (`evidence_baseline_timeout`, padrão 120 s).
- Exceção por evidência com `Tempo-limite:`, aprovada pelo usuário com a tarefa.
- Progresso visível no `vostok task start`.
- Orientação na `approve-plan`, na `approve-task` e na `qa-lead`.

### Fora do escopo

- Tempo-limite nos comandos de "Baseline" da regressão.
- Desligar o baseline de evidências por padrão (estudo, B5).
- Marcação `(automática, sem baseline)` (estudo, B1).
- Pular evidências automaticamente pelos caminhos citados (estudo, B3).

## Requisitos funcionais

| ID | Requisito |
|---|---|
| RF-01 | Uma evidência automática pode ter o subitem `` Requer: `caminho`[, `caminho`…] ``; no baseline, se algum caminho não existe, o comando não roda e a saída gravada é `não executada: <caminho> ainda não existe`. |
| RF-02 | A validação de tarefa e o `vostok plan approve validate` recusam um caminho de `Requer:` que não seja um caminho `criar` de "Mudar" da própria tarefa. |
| RF-03 | Para uma evidência não executada no baseline, o `vostok task evidence` não compara com o baseline nem emite o aviso de "saída igual". |
| RF-04 | `config/tasks.yml` ganha `evidence_baseline_timeout` (padrão 120 segundos); uma evidência que passa do tempo no baseline é interrompida, e a saída gravada é a produzida até ali seguida de `interrompida após <N> s`. |
| RF-05 | Uma evidência pode ter o subitem `Tempo-limite: <segundos>` (0 sem limite), que substitui o padrão só para ela no baseline. |
| RF-06 | O `vostok task approve validate` e o `vostok task approve approve` listam as evidências com `Tempo-limite:` acima do padrão, para o usuário ver a exceção que aprova. |
| RF-07 | O `vostok task start` marca `in_progress` e grava a linha `início` antes de coletar o baseline, e imprime cada comando (do Baseline e das evidências) ao começá-lo. |
| RF-08 | A `approve-plan` e a `qa-lead` orientam `Requer:` para evidências que dependem do que a tarefa cria e `Tempo-limite:` só para evidências legitimamente lentas; a `approve-task` mostra as exceções ao pedir a aprovação. |
| RF-09 | O `evidencias.html` mostra, no "antes" de uma evidência não executada ou interrompida, a linha gravada no baseline. |

## Requisitos não funcionais

| ID | Requisito | Como verificar |
|---|---|---|
| RNF-01 | Compatibilidade: tarefas sem `Requer:` e sem `Tempo-limite:` coletam o baseline como hoje, salvo o tempo-limite padrão. | Teste de tarefa no formato atual. |
| RNF-02 | Sem crates novas; o tempo-limite usa a biblioteca padrão e encerra o processo e os filhos do comando. | `Cargo.toml` sem dependência nova; teste com `sleep`. |

## Regras de negócio

| ID | Regra |
|---|---|
| RN-01 | A exceção de tempo-limite só vale com a aprovação do usuário: ela é escrita no planejamento e aprovada com a tarefa (`approve-task`). |
| RN-02 | `Requer:` só cita caminhos que a própria tarefa cria. |
| RN-03 | O tempo-limite vale só no baseline das evidências; na finalização, a evidência roda sem limite do `vostok`. |

## Fluxos

### Fluxo principal

1. O `vostok task start` confere a tarefa, marca `in_progress` e grava `início` (RF-07).
2. Roda cada comando de Baseline, imprimindo-o ao começar (RF-07).
3. Para cada evidência automática: se falta um caminho de `Requer:`, grava "não executada" (RF-01); senão, roda com o tempo-limite da evidência ou o padrão, imprimindo o comando, e grava a saída ou "interrompida" (RF-04, RF-05).
4. Grava `baseline` no registro.
5. Na finalização, o `vostok task evidence` roda as evidências normalmente e só compara as que rodaram inteiras no baseline (RF-03).

### Fluxos alternativos

- **FA-01 (passo 3), evidência lenta e legítima:** o planejamento escreve `Tempo-limite: 900`; a `approve-task` mostra a exceção, e o usuário aprova a tarefa com ela (RF-05, RF-06, RN-01).
- **FA-02 (passo 2 ou 3), falha na coleta:** a tarefa fica `in_progress` sem a linha `baseline`; `vostok task baseline <SPEC_ID> <TASK_ID>` coleta de novo, como hoje.

## Casos de borda

| ID | Situação | Comportamento esperado | Requisitos |
|---|---|---|---|
| CB-01 | `Requer:` com caminho que não é `criar` | Validação recusa | RF-02 |
| CB-02 | `Requer:` com todos os caminhos já existentes no início | O comando roda no baseline | RF-01 |
| CB-03 | `Tempo-limite: 0` | Sem limite para essa evidência | RF-05 |
| CB-04 | `Tempo-limite:` não numérico | Validação recusa | RF-05 |
| CB-05 | Comando que abre processos filhos e passa do tempo | O grupo de processos é encerrado | RF-04, RNF-02 |
| CB-06 | Evidência interrompida no baseline | Sem aviso de "saída igual"; o "antes" mostra a interrupção | RF-03, RF-09 |
| CB-07 | `Tempo-limite:` abaixo do padrão | Vale, sem aparecer como exceção | RF-05, RF-06 |

## Contratos e interfaces

Evidências:

```markdown
- `e2e.txt` (automática): `pnpm --filter web e2e --reporter=list`
  - Prova: as telas novas abrem.
  - Requer: `src/app/registro.ts`
- `lighthouse.txt` (automática): `pnpm --filter web lighthouse`
  - Prova: desempenho da tela nova.
  - Tempo-limite: 900
```

Configuração:

```yaml
evidence_baseline_timeout: 120
```

Gravação no baseline:

```text
$ pnpm --filter web e2e --reporter=list
exit: 1
não executada: src/app/registro.ts ainda não existe
```

Saída do `vostok task start`:

```text
Tarefa TASK-03 iniciada: status 'in_progress'. Coletando o baseline:
- [1] cargo test --test task -- regression_
- [evidência e2e.txt] não executada: src/app/registro.ts ainda não existe
- [evidência lighthouse.txt] pnpm --filter web lighthouse (tempo-limite 900 s)
Baseline de TASK-03 coletado em …
```

## Componentes afetados

| Caminho | Ação | Motivo |
|---|---|---|
| `apps/cli/src/commands/task.rs` | alterar | `Requer:`, tempo-limite, exceção, progresso no `start` e listagem das exceções na aprovação |
| `apps/cli/src/commands/plan.rs` | alterar | Validação de `Requer:` e `Tempo-limite:` no plano |
| `apps/cli/src/commands/finalize.rs` | alterar | "Antes" de evidência não executada ou interrompida |
| `apps/cli/tests/task.rs` | alterar | Testes de `Requer:`, tempo-limite, exceção e progresso |
| `apps/cli/tests/plan.rs` | alterar | Testes da validação no plano |
| `apps/cli/tests/finalize.rs` | alterar | Teste do "antes" |
| `packages/harness/config/tasks.yml` | alterar | Chave `evidence_baseline_timeout` |
| `packages/harness/skills/default/approve-plan/SKILL.md` | alterar | Orientação de `Requer:` e `Tempo-limite:` |
| `packages/harness/skills/default/approve-task/SKILL.md` | alterar | Mostrar as exceções ao pedir a aprovação |
| `packages/harness/personas/qa-lead.md` | alterar | Regra de dependência e tempo das evidências |

## Premissas e dependências

- O `vostok update` acrescenta `evidence_baseline_timeout`; sem a chave, vale 120 s.
- `Requer:` e `Tempo-limite:` entram como campos opcionais de evidência, aceitos mesmo sem estar em `evidence_optional_fields`, como o `Espera:`.

## Riscos

| Risco | Impacto | Mitigação |
|---|---|---|
| Padrão de 120 s corta uma evidência lenta e útil | médio | Exceção por evidência, aprovada pelo usuário |
| Encerrar o processo deixa processos filhos (servidor) vivos | médio | Encerrar o grupo de processos (CB-05) |
| Marcar `in_progress` antes do baseline deixa tarefa sem baseline numa falha | baixo | `vostok task baseline` coleta de novo; `tests` já recusa tarefa sem baseline |

## Casos de teste

| ID | Cenário | Passos | Resultado esperado | Requisitos |
|---|---|---|---|---|
| CT-01 | `Requer:` ausente no disco | Evidência com `Requer:` de caminho `criar`; `start` | Baseline grava "não executada", comando não roda | RF-01 |
| CT-02 | `Requer:` presente | Caminho criado antes do `start` | Comando roda | RF-01 |
| CT-03 | `Requer:` inválido | Caminho que não é `criar`; validar tarefa e plano | Recusa nos dois | RF-02 |
| CT-04 | Sem comparação | Evidência não executada; `evidence` | Sem aviso de "saída igual" | RF-03 |
| CT-05 | Tempo-limite | `evidence_baseline_timeout: 1`; evidência `sleep 5` | Interrompida em cerca de 1 s com a linha de interrupção | RF-04 |
| CT-06 | Exceção | `Tempo-limite: 0` numa evidência `sleep 2` com padrão 1 | Roda inteira | RF-05 |
| CT-07 | Exceção listada | `approve validate` com `Tempo-limite: 900` | A saída lista a evidência e o tempo | RF-06 |
| CT-08 | Progresso | `start` | `in_progress` e `início` antes do baseline; cada comando impresso | RF-07 |
| CT-09 | Orientação | Ler `approve-plan`, `approve-task` e `qa-lead` | Textos descrevem `Requer:`, `Tempo-limite:` e a exceção | RF-08 |
| CT-10 | Antes no documento | `finalize build` com evidência não executada | O "antes" mostra a linha gravada | RF-09 |
| CT-11 | Tarefa antiga | Tarefa sem os subitens | Baseline como hoje | RNF-01 |

## Critérios de aceite

- [ ] CA-01: Evidência com `Requer:` ausente não roda no baseline e não gera aviso de "saída igual" (RF-01, RF-03).
- [ ] CA-02: `Requer:` só aceita caminhos `criar` da tarefa, na tarefa e no plano (RF-02).
- [ ] CA-03: Evidência que passa do tempo-limite no baseline é interrompida e registrada (RF-04).
- [ ] CA-04: `Tempo-limite:` substitui o padrão e aparece para o usuário na aprovação da tarefa (RF-05, RF-06).
- [ ] CA-05: O `vostok task start` mostra a tarefa `in_progress` e cada comando enquanto coleta (RF-07).
- [ ] CA-06: As skills e a `qa-lead` orientam `Requer:` e `Tempo-limite:` (RF-08).
- [ ] CA-07: O documento de evidências mostra o motivo no "antes" (RF-09).
- [ ] CA-08: Tarefas sem os subitens coletam o baseline como antes (RNF-01).

## Referências

| Tipo | Referência | Uso |
|---|---|---|
| documentação | `docs/estudos/baseline-de-evidencias.md` | Dados do baseline das evidências e alternativas B1 a B6 |
| documentação | Issue #8 de Ishtar-Sink/Vostok | Caso real, propostas e alternativas |
| arquivo | `apps/cli/src/commands/task.rs` | `start`, `collect_baseline`, `automatic_evidences`, `evidence`, `evidence_fields` |
| arquivo | `apps/cli/src/commands/plan.rs` | `evidence_errors` e `evidence_fields` do plano |
| arquivo | `apps/cli/src/commands/finalize.rs` | `before_html` |
| arquivo | `packages/harness/config/tasks.yml` | `evidence_baseline_dir` |

## O que mudou

| # | Assunto | Estado no Original | Estado Refinado |
|---|---|---|---|
| 1 | Como pular o baseline | Marcação ou `Requer:` (issue) | `Requer:` com caminhos `criar` |
| 2 | Tempo-limite | Não citado | Padrão de 120 s, com exceção por evidência aprovada pelo usuário |
| 3 | Progresso | Linha `início` e comandos impressos (issue) | Como na issue |

## Registro de decisões

| # | Decisão | Alternativa descartada | Motivo |
|---|---|---|---|
| 1 | `Requer:` com caminhos `criar` | Marcação; baseline opcional | resposta 5A |
| 2 | Tempo-limite de 120 s | 60 s; sem tempo-limite | resposta 6A |
| 3 | Progresso como na issue | Só imprimir | resposta 7A |
| 4 | Exceção de tempo por evidência (`Tempo-limite:`), aprovada pelo usuário com a tarefa e listada na aprovação | Flag no `start`; pedir no meio da coleta | pedido do usuário ("deve ser possível abrir exceções com aval do usuário") |
| 5 | Tipo `improvement` | `feature` | inferência A |
| 6 | Tempo-limite só nas evidências do baseline | Também nos comandos de Baseline | inferência E |
| 7 | Evidência pulada sem aviso de "saída igual" e com o motivo no "antes" | Comparar mesmo assim | inferência F |
| 8 | Tarefas antigas como antes, salvo o tempo-limite padrão | Sem tempo-limite para elas | inferência G |
| 9 | `approve-plan` e `qa-lead` orientam `Requer:` e `Tempo-limite:` | Só documentação | inferência H |

## Pendências

| # | Pendência | Opções em aberto |
|---|---|---|
| - | Nenhuma | - |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
