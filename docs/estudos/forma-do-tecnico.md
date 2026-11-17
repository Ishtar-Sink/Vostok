# Forma do técnico

## Problema

O usuário disse que o técnico atual "é muito ruim de ler, principalmente a abordagem que separa os passos por ponto e virgula misturado a diversos blocos de codigo". Três pontos tornam o texto difícil:

- Os passos da abordagem vêm encadeados numa frase só, separados por ponto e vírgula.
- Trechos de código entram no meio da frase, e a leitura para a cada bloco.
- As decisões e os riscos ficam misturados com a descrição da abordagem, sem separação visual.

Os três exemplos a seguir reescrevem o mesmo técnico, `vostok/work/.archive/SPEC-057/tasks/TASK-03/tecnico.md`, sem mudar o conteúdo, só a forma.

## Exemplo A: passos numerados

**Abordagem**

1. O `vostok task tests` grava `tests_code_hash` e `tests_checks_hash` quando os testes passam, e grava os dois como `null` na falha (RF-01, RF-02).
2. O ajuste compara os dois hashes com o estado atual e mantém `tests_result` e `regression_result` só quando os dois são iguais (RN-01).
3. Em qualquer caso, o ajuste zera `attempts` e `evidence_result`.
4. Se só o código de "Mudar" mudou, a mensagem diz isso e os testes são refeitos.
5. O `finish` recusa a tarefa quando o código de "Mudar" mudou depois dos testes e indica `vostok task tests` (RF-03, RN-01).

**Decisões**

- Impressão só dos arquivos de "Mudar". Descartado o `git diff` do repositório, porque zeraria os testes quando outra tarefa da onda mexe no repositório.
- Hash próprio de Baseline e Verificação. Descartado reaproveitar o `verification_hash`, porque ele cobre as Evidências e zeraria tudo num ajuste só de `Trecho:`.
- Tarefa sem os campos zera tudo, como hoje. Descartado calcular a impressão no ajuste, porque tarefas antigas não têm o que comparar (RNF-03, CB-01).
- `evidence_result` é zerado mesmo com os testes mantidos. Descartado mantê-lo, porque as evidências são geradas de novo e a conferência por hash (SPEC-046) decide o que fica.

**Riscos**

- Código mudado fora de "Mudar" não altera a impressão. A regressão cobre o caso em que isso quebra algo.
- Um `finish` sem a checagem aceitaria testes antigos. O teste `ajuste_mantem_finish_pede_testes_quando_o_codigo_muda_depois` cobre esse caminho (RNF-02).

**Bom para:** quem quer seguir a ordem da abordagem passo a passo.

**Custa:** ocupa mais linhas, e as decisões ficam longas quando o motivo é extenso.

## Exemplo B: tabelas

**O que muda**

| Onde | O que muda | Por quê |
|---|---|---|
| `apps/cli/src/commands/task.rs`, `code_fingerprint` | Impressão só dos arquivos de "Mudar", gravada como `tests_code_hash` quando os testes passam | Mantém os testes quando o código não muda (RF-01, RF-02) |
| `apps/cli/src/commands/task.rs`, `checks_hash` | Impressão de Baseline e Verificação, gravada como `tests_checks_hash` | Ajuste só no `Trecho:` não zera os testes (RN-01) |
| `apps/cli/src/commands/task.rs`, ajuste | Compara os dois hashes e zera `attempts` e `evidence_result` em qualquer caso | Só mantém testes com o código e os checks iguais (RF-03) |
| `apps/cli/src/commands/task.rs`, `finish` | Recusa quando o código de "Mudar" mudou depois dos testes | Impede aceitar testes antigos (RF-03) |
| `apps/cli/tests/task.rs` | Seis testes `ajuste_mantem_` | Cobre verificação igual e alterada, código alterado e tarefa antiga (RNF-02, RNF-03) |

**Decisões**

| Decisão | Descartado | Motivo |
|---|---|---|
| Impressão só dos arquivos de "Mudar" | `git diff` do repositório | Outra tarefa da onda mexendo no repositório zeraria os testes |
| Hash próprio de Baseline e Verificação | Reaproveitar `verification_hash` | Ele cobre as Evidências e zeraria tudo num ajuste de `Trecho:` |
| Tarefa sem os campos zera tudo | Calcular a impressão no ajuste | Tarefas antigas não têm o que comparar (RNF-03, CB-01) |
| `evidence_result` sempre zerado | Mantê-lo junto com os testes | As evidências são geradas de novo, e a conferência por hash (SPEC-046) decide |

**Riscos**

- Código mudado fora de "Mudar" não altera a impressão. A regressão cobre o caso em que isso quebra algo.
- Um `finish` sem a checagem aceitaria testes antigos. O teste `ajuste_mantem_finish_pede_testes_quando_o_codigo_muda_depois` cobre esse caminho (RNF-02).

**Bom para:** quem compara arquivo a arquivo e decisão a decisão.

**Custa:** as tabelas ficam largas no painel estreito, e o motivo de cada linha precisa caber numa célula.

## Exemplo C: em camadas

**Em uma frase**

O `vostok task tests` grava uma impressão do código e dos checks quando os testes passam, e o ajuste e o `finish` só mantêm ou exigem os testes conforme essa impressão muda.

**Como funciona**

Quando os testes passam, o `vostok task tests` grava duas impressões: uma dos arquivos de "Mudar" e outra do texto de Baseline e Verificação. Quando os testes falham, grava as duas como vazias.

O ajuste compara as duas com o estado atual. Se os testes passaram e nada mudou em nenhuma, mantém o resultado dos testes e da regressão. Em qualquer caso, zera as tentativas e as evidências, porque elas são geradas de novo.

O `finish` recusa a tarefa quando o código de "Mudar" mudou depois dos testes e manda rodar `vostok task tests`. Tarefa sem os campos novos segue o caminho antigo.

**Onde no código**

- `apps/cli/src/commands/task.rs`, `code_fingerprint`: impressão dos arquivos de "Mudar".
- `apps/cli/src/commands/task.rs`, `checks_hash`: impressão de Baseline e Verificação.
- `apps/cli/src/commands/task.rs`, `tested_code` e `code_changed_after_tests`: dizem se o código mudou depois dos testes.
- `apps/cli/tests/task.rs`: seis testes `ajuste_mantem_`.

**Decisões e riscos**

- Impressão só dos arquivos de "Mudar". Descartado o `git diff` do repositório, porque outra tarefa da onda o faria zerar os testes.
- Hash próprio de Baseline e Verificação. Descartado o `verification_hash`, que cobre as Evidências.
- Código mudado fora de "Mudar" não altera a impressão. A regressão cobre o caso em que isso quebra algo.

**Bom para:** quem precisa entender a ideia antes dos detalhes.

**Custa:** esconde a ordem exata dos passos e os nomes das decisões, que ficam só no nível de camada.

## Forma escolhida

O usuário escolheu a mistura do Exemplo C com o Exemplo B: "a parte em camadas dá o panorama geral em forma de prosa, e depois exibe uma tabela detalhando o que for pertinente como no exemplo B".

O técnico abre com a frase e a prosa do Exemplo C, sem código no meio, e detalha o resto em tabelas como no Exemplo B. Não há passos encadeados por ponto e vírgula nem blocos de código na explicação.

```markdown
**Em uma frase**

Uma frase com o que a tarefa faz.

**Como funciona**

Dois ou três parágrafos curtos, sem código, com o panorama: o que acontece, em que ordem e por quê.

**O que muda**

| Onde | O que muda | Por quê |
|---|---|---|
| `caminho`, função | O que muda nesse ponto | Motivo, com os IDs da spec |

**Decisões**

| Decisão | Descartado | Motivo |
|---|---|---|
| O que foi escolhido | A alternativa descartada | Por que ela foi descartada |

**Riscos**

- Um risco por item, com o que o cobre.

**Referências**

Os IDs da spec cobertos e os arquivos em comum com tarefas da mesma onda.
```
