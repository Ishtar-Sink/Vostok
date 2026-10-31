# Estudo do baseline das evidências

Data do estudo: 2026-10-31. Versão: `vostok` 0.2.5. Subsídio da SPEC-022 e da issue #8 (https://github.com/Ishtar-Sink/Vostok/issues/8).

## Escopo e método

O estudo responde três perguntas:

1. Para que serve rodar as evidências no baseline, e quanto isso ajudou até agora.
2. O que a SPEC-020 (0.2.5) já mudou nesse ponto.
3. Quais alternativas resolvem o caso da issue (evidências caras que só podem falhar antes da tarefa) e o custo de cada uma.

A base é o código de `collect_baseline` e `evidence` em `apps/cli/src/commands/task.rs`, o `vostok finalize build` (antes e depois) e as 25 tarefas arquivadas que têm baseline de evidências.

## Como funciona hoje

- O `vostok task start` coleta o baseline no mesmo comando: os comandos de "Baseline" e, com `evidence_baseline_dir` configurado, o comando de cada evidência `(automática)`. Tudo é gravado junto, no fim: ou o baseline existe inteiro, ou não existe.
- Durante a coleta, a tarefa continua `approved`, sem linha no registro e sem nada impresso.
- O `vostok task evidence` compara a saída final de cada evidência com a de antes e avisa, sem bloquear, quando são iguais.
- Desde a 0.2.5, o `evidencias.html` mostra a saída de antes ao lado da de depois.
- As evidências `(automática, da verificação)`, `(diff)` e `(print)`, da 0.2.5, não rodam no baseline.

## O que o baseline das evidências rendeu

| Medida | Valor |
|---|---|
| Tarefas arquivadas com baseline de evidências | 25 |
| Evidências executadas no baseline | 50 |
| Saídas com código 0 antes da tarefa | 25 |
| Saídas com código 1 ou 2 antes da tarefa | 25 |
| Vezes em que o aviso de "saída igual ao baseline" apareceu | 0 |

- O aviso nunca disparou. As 50 execuções só serviram, até a 0.2.5, para gravar uma saída que ninguém leu.
- Metade terminava com erro antes da tarefa, como a regra pede ("algo que só existe depois da tarefa"). Para essas, a saída de antes é uma falha conhecida.
- A 0.2.5 deu um uso novo à saída de antes: o contraste no `evidencias.html`. Ele só tem valor quando a saída de antes diz algo útil, por exemplo um comando do produto que mudou de resposta. Para uma suíte ponta a ponta que falha por tempo-limite, o "antes" é uma lista de tempos-limite.

## O caso da issue

- Duas evidências caras numa tarefa de integração: a suíte ponta a ponta inteira (192 execuções, cada uma esperando 30 s antes de falhar) e o Lighthouse com login.
- O `vostok task start` passou de 10 minutos, com estimativa de mais de 30, sem nenhum sinal de progresso.
- O contorno (um `if test -f …` em cada comando) derruba a aprovação e esconde a dependência dentro do comando.

## Alternativas

| ID | Alternativa | Resolve | Risco | Custo |
|---|---|---|---|---|
| B1 | **Marcação `(automática, sem baseline)`** (issue, 1) | O caso da issue, por escolha explícita | O planejamento esquecer a marcação | Baixo |
| B2 | **Subitem `Requer:`** com caminhos `criar` da tarefa (issue, 2): sem os caminhos, o comando não roda no baseline | O caso da issue, com a dependência explícita e validada | Dependência que não é um arquivo (servidor, variável) não se expressa | Médio |
| B3 | **Automático pelos caminhos `criar`:** uma evidência cujo comando cita um caminho `criar` ou pasta nova não roda no baseline, como já acontece na regressão | Parte do caso: o comando da issue não cita a rota criada | Cobre pouco | Baixo |
| B4 | **Tempo-limite no baseline das evidências** (`evidence_baseline_timeout`, por exemplo 60 s): passou do tempo, grava "interrompida após 60 s" e segue | Qualquer evidência cara, sem marcação | O valor padrão pode cortar uma evidência legítima e lenta | Baixo |
| B5 | **Baseline de evidências opcional:** desligado por padrão, ligado por evidência (`(automática, com baseline)`) quando o antes e depois importa | O caso da issue e o custo de todas as 50 execuções | Perde o antes e depois da 0.2.5 onde ninguém marcar | Baixo |
| B6 | **Progresso visível** (issue, 4): a linha `início` no registro antes do baseline e cada comando impresso ao começar | A sensação de travado, em qualquer baseline | Nenhum | Baixo |

## Recomendação

- **B6** entra de qualquer jeito: é barato e resolve a falta de rastro.
- Para o custo, **B2 com B4**: o `Requer:` deixa a dependência explícita e validada no planejamento, e o tempo-limite protege contra o caso que ninguém previu. B1 é mais simples, mas não explica por que a evidência pula o baseline.
- B5 é a opção mais econômica, porque os dados mostram que o baseline das evidências quase não rendeu. Mas ela troca o padrão do harness e tira o antes e depois que a 0.2.5 acabou de ganhar; vale como decisão do usuário, e não como recomendação.

## Perguntas para o refinamento

1. Como a evidência pula o baseline: `Requer:` (B2), marcação (B1) ou baseline opcional (B5)?
2. Entra o tempo-limite (B4)? Com qual valor padrão?
3. O progresso visível (B6) entra como está na issue?

## Fontes

- `apps/cli/src/commands/task.rs` (`start`, `collect_baseline`, `automatic_evidences`, `evidence`).
- `apps/cli/src/commands/finalize.rs` (`before_html`).
- `packages/harness/config/tasks.yml` (`evidence_baseline_dir`).
- Baselines das tarefas arquivadas em `vostok/work/.archive/SPEC-*/tasks/evidence/*/baseline/evidence/`.
- [Issue #8 de Ishtar-Sink/Vostok](https://github.com/Ishtar-Sink/Vostok/issues/8).
