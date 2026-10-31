# Estudo de evidências e finalização de tarefas

Data do estudo: 2026-10-31. Versão: `vostok` 0.2.4. Subsídio da spec da issue #7 (https://github.com/Ishtar-Sink/Vostok/issues/7) e da percepção do usuário de que as evidências "não evidenciam nada de fato".

## Escopo e método

O estudo responde quatro perguntas:

1. Como a finalização de uma tarefa roda hoje: Verificação, regressão e evidências.
2. Quanto as tarefas já entregues repetem comandos entre essas três etapas.
3. O que as evidências gravadas realmente comprovam.
4. Quais mudanças resolvem a issue #7 e tornam as evidências relevantes, e o custo de cada uma.

A base são o código de `apps/cli/src/commands/task.rs` (`tests`, `regression`, `evidence`), a skill `approve-plan`, a persona `qa-lead` e as 95 tarefas arquivadas em `vostok/work/.archive/` (SPEC-002 a SPEC-019), com 150 itens de evidência distintos. A classificação dos itens foi feita por padrão do comando (script no fim deste estudo), conferida por amostra.

## Como funciona hoje

A skill `finish-task` roda três comandos, cada um executando os próprios comandos do zero:

| Etapa | Comando | O que executa | O que grava |
|---|---|---|---|
| Testes | `vostok task tests` | O bloco `bash` de "Verificação", inteiro, num só `bash -c` com `set -eo pipefail` | `validation/tests.out`, uma saída só para o bloco |
| Regressão | `vostok task regression` | Cada comando de "Baseline", de novo | `validation/regression/NN.out`, comparado com `baseline/NN.out` |
| Evidências | `vostok task evidence` | O comando de cada evidência automática, de novo | O arquivo da evidência, com a saída crua |

- A Verificação é um bloco único: o `vostok` não sabe onde termina um comando e começa outro, por isso nada pode apontar para a saída de um comando dela.
- A evidência automática é só a saída crua do comando. Os subitens `Prova`, `Trecho` e `Falha`, escritos no planejamento, são texto para pessoas: o `vostok` não confere se o `Trecho` aparece na saída.
- O único controle de relevância é o aviso de "igual ao baseline": o `vostok task baseline` grava a saída de cada evidência antes da mudança, e o `vostok task evidence` avisa, sem bloquear, quando a saída final é igual.
- Uma evidência que termina com erro bloqueia a etapa; uma que termina sem erro, mas sem nada que prove a tarefa, passa.

## Repetição de comandos nas tarefas entregues

Contagem nas 95 tarefas arquivadas:

| Padrão | Tarefas | Parte |
|---|---|---|
| Evidência roda o mesmo `cargo test` (mesmo alvo) que a Verificação já rodou | 36 | 38% |
| Verificação roda a suíte inteira do `cargo` | 40 | 42% |
| Comando de "Baseline" idêntico, em texto, a um comando da Verificação | 14 | 15% |

- O caso relatado na issue (16 tarefas de tela, a suíte do pacote rodando duas vezes por tarefa) é o mesmo padrão, agravado pelo filtro ignorado depois do `--`.
- Neste repositório, a suíte inteira do `cargo` leva cerca de 1 minuto. Numa tarefa que a roda na Verificação e de novo numa evidência, a finalização gasta o dobro do necessário.

## O que as evidências comprovam

Os 150 itens de evidência, por tipo de comando:

| Tipo | Itens | Parte | O que comprova |
|---|---|---|---|
| Leitura de arquivo (`grep`, `sed`, `cat`) | 83 | 55% | Que um texto está escrito num arquivo |
| Execução do produto (`vostok …`, `cargo run …`) | 18 | 12% | Um comportamento observável |
| Saída de teste (`cargo test …`) | 17 | 11% | Que os testes passaram, repetindo a Verificação |
| Manual | 15 | 10% | Depende do que foi anexado |
| Outros (scripts de teste, cópias temporárias) | 11 | 7% | Em geral, um comportamento observável |
| Nomes de teste (`cargo test … \| grep '^test'`) | 6 | 4% | Que testes com aqueles nomes passaram |

Os exemplos da SPEC-019 mostram o problema:

- `skills-coordenador.txt` é um `grep` na skill que a própria tarefa escreveu. Ele prova que a tarefa escreveu o texto, o que já se sabe pelo diff. Não prova que a skill funciona.
- `token-commands-tests.txt` é a lista `test brief_… ok`. Prova que os testes passaram, o que a Verificação já provou, e não mostra o que o comando novo imprime.
- `medicao-usage.md`, a única manual, é a que de fato evidencia algo: números lidos fora do código, com antes e depois.

O diagnóstico, em cinco causas:

1. **Evidência tautológica.** Mais da metade lê o que a tarefa escreveu. A regra atual ("busca algo que só existe depois da tarefa") é satisfeita por qualquer `grep` no texto novo, e por isso empurra para esse padrão.
2. **Evidência que repete a Verificação.** A saída de teste só diz "passou", e a Verificação já diz isso com código 0.
3. **Pouca evidência de comportamento.** Só cerca de 1 em 5 mostra o produto rodando sobre um caso, com entrada e saída.
4. **A prova não é conferida.** `Trecho` é prosa; ninguém confere que a linha esperada está no arquivo, e o critério de "Concluída quando" que cita a evidência é marcado por quem executou.
5. **Sem antes e depois.** O baseline da evidência existe, mas só gera um aviso; o documento de evidências mostra a saída final, sem o contraste.

## O que torna uma evidência relevante

Critérios tirados da prática de teste e da documentação de teste:

- **Comportamento, não implementação.** Mostra o que um usuário observaria: a saída de um comando, um arquivo gerado, uma tela, uma mensagem de erro. Não mostra o código-fonte ou o texto que a tarefa escreveu.
- **Caso concreto, com entrada e saída.** Como num log de teste (ISO/IEC/IEEE 29119-3): o que foi feito, o resultado esperado e o resultado obtido. Uma transcrição `$ comando` seguida da saída já cumpre isso.
- **Ligada a um critério de aceite.** Cada evidência responde a um critério ou a um ID da spec (CT, RF); sem isso, é só um arquivo.
- **Antes e depois, quando a tarefa muda um comportamento.** O contraste é o que convence quem não executou a tarefa.
- **Conferida por máquina.** Como nos testes de aprovação (*approval* ou *golden testing*): a saída é comparada com um padrão esperado, e a falha é visível. Um padrão esperado, escrito no planejamento, transforma a evidência de "arquivo gravado" em "prova conferida".
- **Única fonte.** A evidência é a saída que comprovou a tarefa, e não uma segunda execução que pode provar outra coisa.

Para entregas de texto (skills, regras, documentação), em que não há comportamento a executar, a evidência honesta é o trecho do diff (`git diff -- <arquivo>`), que mostra antes e depois, ou a execução de quem consome o texto (por exemplo, um teste que carrega a skill), e não um `grep` do texto novo.

## Análise das propostas da issue

| Proposta da issue | Avaliação | Custo |
|---|---|---|
| 1. Verificação focada no escopo, conferida no planejamento | Necessária. Tem o mesmo espírito da orientação que o Baseline já tem; resolve a suíte inteira em 42% das tarefas | Baixo: texto em `approve-plan` e `qa-lead` |
| 1, opcional. Aviso para argumentos depois de `--` num script do gerenciador de pacotes | Útil, mas estreito: vale para `pnpm`/`npm`/`yarn` e não para o `cargo`, em que o `--` é correto | Baixo: um aviso no `vostok plan approve validate` |
| 2. Saída de cada comando da Verificação em `validation/verify-NN.out` e evidência "da verificação" | Resolve a repetição em 38% das tarefas e garante a única fonte. Exige dividir o bloco em comandos (risco com linhas continuadas por `\` e *heredocs*) | Médio: mudança em `tests`, `evidence` e na validação de tarefa |
| 3. Regressão reaproveita a saída de um comando idêntico da Verificação | Correta e segura (só com texto idêntico, na mesma validação). Ganho menor: 15% das tarefas | Baixo, depois da proposta 2 |

A issue não trata da relevância. As propostas 1 a 3 reduzem tempo e repetição, mas uma evidência reaproveitada da Verificação continua sendo "os testes passaram". A relevância pede mudanças próprias, abaixo.

## Propostas para a spec

| ID | Proposta | Origem | Resolve |
|---|---|---|---|
| P1 | Verificação focada no escopo de "Mudar"; cada comando de teste rodado uma vez no planejamento, conferindo que seleciona só o escopo e que as opções fazem efeito | Issue, 1 | Suíte inteira, filtro ignorado |
| P2 | Aviso no `vostok plan approve validate` para argumentos depois de `--` em `pnpm`/`npm`/`yarn` | Issue, 1 opcional | Filtro ignorado |
| P3 | `vostok task tests` grava a saída de cada comando da Verificação (`validation/verify-NN.out`); evidência automática `(automática, da verificação): N` copia essa saída sem rodar de novo | Issue, 2 | Teste duas vezes; única fonte |
| P4 | `vostok task regression` reaproveita a saída de um comando de "Baseline" idêntico a um da Verificação | Issue, 3 | Typecheck duas vezes |
| P5 | Subitem `Espera:` com um texto que o `vostok task evidence` procura na saída; ausente, a etapa falha com a evidência e o texto | Estudo | A prova não é conferida |
| P6 | Orientação de relevância na `approve-plan` e no `qa-lead`: evidência de comportamento (execução do produto sobre o caso do CT, com `$ comando` e saída); entrega de texto usa o trecho do diff; proibido `grep`/`sed`/`cat` do próprio texto novo como evidência única | Estudo | Tautologia; pouca evidência de comportamento |
| P7 | A validação de tarefa recusa evidência automática que só lê um caminho de "Mudar" (`grep`, `sed`, `cat`, `head` sobre ele) | Estudo | Tautologia, barrada no planejamento |
| P8 | O documento de evidências mostra o antes (saída do baseline da evidência) ao lado do depois, quando existir | Estudo | Sem antes e depois |

P1, P3, P5 e P6 dão o maior efeito. P7 e P8 dependem de decisão: P7 pode ser rígida demais para entregas de texto, e P8 muda o `vostok finalize build`.

## Riscos

| Risco | Mitigação |
|---|---|
| Dividir a Verificação em comandos quebra blocos com `\`, *heredoc* ou funções | Dividir por comando de topo (linha sem continuação); bloco que não se divide segue como hoje, sem `verify-NN.out` |
| `Espera:` frágil (texto que muda com a ferramenta) | Texto curto e estável, como o nome do teste ou a mensagem do produto; a validação exige que o texto não esteja na saída do baseline |
| Recusar `grep` deixa entregas de texto sem evidência | Entrega de texto usa `git diff -- <arquivo>`; P7 só recusa quando o comando lê um caminho de "Mudar" e não há outra evidência |
| Tarefas já planejadas na 0.2.4 | As mudanças valem para o planejamento novo; o formato antigo de evidência continua aceito (formatos antigos importam depois da v0.1) |

## Perguntas para o refinamento

1. Escopo: uma spec com tudo (P1 a P8) ou a issue #7 (P1 a P4) separada da relevância (P5 a P8)?
2. P3: a evidência "da verificação" aponta pelo número do comando (`1`) ou por um rótulo no bloco?
3. P5: `Espera:` obrigatório em toda evidência automática ou só recomendado?
4. P7: recusar na validação ou só orientar no planejamento?
5. P8: entra agora ou fica para depois?

## Fontes

- Código: `apps/cli/src/commands/task.rs` (`run_verification`, `tests`, `regression`, `evidence`, `VERIFY_PRELUDE`).
- Skill `packages/harness/skills/default/approve-plan/SKILL.md`, variáveis `#{TASK_BASELINE}`, `#{TASK_VERIFY}`, `#{TASK_EVIDENCE}` e `#{TASK_DONE}`.
- Persona `packages/harness/personas/qa-lead.md`.
- Tarefas arquivadas em `vostok/work/.archive/SPEC-*/tasks/`.
- [Issue #7 de Ishtar-Sink/Vostok](https://github.com/Ishtar-Sink/Vostok/issues/7).
- ISO/IEC/IEEE 29119-3, documentação de teste (log de teste com resultado esperado e obtido): [ISO/IEC 29119](https://en.wikipedia.org/wiki/ISO/IEC_29119).
- Testes de aprovação: [Approval Tests](https://approvaltests.com/).
- Especificação por exemplo: [Specification by Example (Martin Fowler)](https://martinfowler.com/bliki/SpecificationByExample.html).

## Como a contagem foi feita

A classificação lê, em cada tarefa arquivada, os itens `` - `arquivo` (automática|manual) `` de "Evidências" e o comando de cada um: leitura de arquivo quando começa por `grep`, `sed`, `cat`, `head`, `awk`, `diff`, `ls`, `find` ou `wc`; nomes de teste quando há `cargo test` com `grep`; saída de teste quando há só `cargo test`; execução do produto quando chama `vostok`, `cargo run` ou um binário de `target/`. A repetição compara o alvo do `cargo test` (texto antes do `--`) das evidências com o da Verificação, e o texto exato dos comandos de "Baseline" com as linhas da Verificação.
