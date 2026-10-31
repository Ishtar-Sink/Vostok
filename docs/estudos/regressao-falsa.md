# Estudo da regressão que não é regressão

Data do estudo: 2026-10-31. Versão: `vostok` 0.2.5. Subsídio da SPEC-021.

## Escopo e método

O estudo responde três perguntas:

1. Quantas vezes a regressão bloqueou uma tarefa sem regressão de verdade, e por quê.
2. Por que as correções anteriores não resolveram.
3. Quais mudanças acabam com o bloqueio falso sem perder a detecção de regressão real.

A base é o registro de execução das 95 tarefas arquivadas em `vostok/work/.archive/` (SPEC-002 a SPEC-020), os arquivos `validation/regression/accepted-NN.diff` e o código de `regression` em `apps/cli/src/commands/task.rs`.

## Como a regressão compara hoje

- O `vostok task start` grava, para cada comando de "Baseline", a linha `$ comando`, a linha `exit: N` e a saída inteira (`baseline/NN.out`).
- O `vostok task regression` roda o comando de novo e compara a gravação inteira, byte a byte, depois de tirar dos dois lados as linhas com algum texto de `regression_ignore` (`Start at`, `Duration`, `finished in`, `Time:`).
- Qualquer diferença bloqueia a tarefa. O usuário decide e o agente roda `vostok task resume … --accept-regression N`, que zera os resultados e obriga a rodar os testes, a regressão e as evidências de novo.
- Exceções automáticas já existentes: comandos que citam um caminho `criar` ou uma pasta nova da tarefa são ignorados por inteiro (SPEC-017 e SPEC-018).

## O que aconteceu nas tarefas entregues

Oito bloqueios por regressão em seis tarefas. Nenhum era regressão de verdade: todos foram aceitos pelo usuário.

| Tarefa | Comando do Baseline | O que mudou | Causa |
|---|---|---|---|
| SPEC-006 TASK-03 (2 bloqueios) | contagem de arquivos em `rules/agent/` | 5 para 6 | O Baseline media a própria entrega: a tarefa criava a sexta regra |
| SPEC-006 TASK-04 (2 bloqueios) | contagem de `SKILL.md` com `requires:` | 12 para 0 | O Baseline media o objetivo da tarefa |
| SPEC-009 TASK-02 | `vostok spec validate SPEC-009` | `exit: 0` para `exit: 1` | A validação da spec passou a recusar o `criar` de um arquivo que a tarefa criou |
| SPEC-016 TASK-01 | lista dos testes de `tests/task.rs` | 5 testes novos | Testes de outra tarefa no mesmo arquivo de teste |
| SPEC-020 TASK-01 | lista dos testes com `evidence_` e `check_` | 6 testes novos | O filtro do `cargo test` é por substring e pegou os testes novos `verify_label_evidence_*` |
| SPEC-020 TASK-02 | lista dos testes com `regression_` e `delivery_` | 3 testes novos | O mesmo: `regression_reuses_*` |

Fora do arquivo, a issue #6 (SPEC-017 e SPEC-018) foi o caso dos comandos que mediam os arquivos criados pela tarefa.

Em todas as tarefas, a linha de contagem dos testes (`test result: … N passed`) também mudou, mas ela já sai da comparação pelo `regression_ignore`, porque contém `finished in`.

## Diagnóstico

Os bloqueios têm duas causas:

1. **Saída que só cresce (3 dos 8 bloqueios):** listas de testes, de arquivos ou de itens. A tarefa acrescenta itens, e nada que existia some ou piora. A comparação exata trata o acréscimo como diferença.
2. **Baseline que mede a entrega (5 dos 8 bloqueios):** contagens e validações que a própria tarefa muda de propósito. O planejamento devia evitá-las, e não evitou.

Por que as correções anteriores não bastaram:

- Cada correção tratou uma forma do problema: caminhos `criar` (SPEC-017), pastas novas (SPEC-018) e a regra de tirar a contagem e os nomes novos (SPEC-016). Nenhuma mudou o critério da comparação.
- A orientação da `approve-plan` já manda filtrar os nomes novos de testes. Na SPEC-020, o próprio Coordenador escreveu o baseline com o filtro por substring e não percebeu. Orientação sozinha não segura um erro que só aparece na finalização.
- O aceite custa caro: cada bloqueio para a tarefa, pede uma decisão ao usuário e roda tudo de novo.

## O que é uma regressão

Para um comando de Baseline, uma regressão é algo que existia e funcionava antes e deixou de funcionar:

- o código de saída piorou (0 virou outro valor);
- uma linha que existia sumiu ou mudou;
- um teste que passava falha ou desapareceu.

Uma linha nova, num comando que continua terminando com 0, não é regressão, a menos que ela indique um problema (erro, aviso, falha).

## Alternativas

| ID | Alternativa | Resolve | Risco | Custo |
|---|---|---|---|---|
| A1 | **Acréscimo não bloqueia:** com o código de saída igual, uma diferença formada só por linhas novas passa, listada como "acréscimos" no registro | Causa 1 inteira (3 dos 8 bloqueios) | Uma linha nova de erro ou aviso passaria | Baixo: muda a decisão na `regression` |
| A2 | A1 com **linhas de alerta:** uma linha nova que contém um texto de `regression_alert` (`FAILED`, `error`, `warning`, `panicked`) continua bloqueando | Causa 1, sem o risco de A1 | Lista de alertas incompleta para alguma ferramenta | Baixo, mais uma chave em `config/tasks.yml` |
| A3 | **Comparação por teste:** reconhecer linhas de resultado de teste (`test x ... ok`, `✓`, `PASS`) e bloquear só quando um teste que passava falha ou some | Causa 1 para testes | Depende do formato de cada executor de testes | Médio |
| A4 | **Mudança esperada declarada:** um item de Baseline pode trazer `muda:` com o motivo (`` - `comando` (muda): o número de regras passa de 5 para 6 ``); a regressão mostra a diferença sem bloquear e a registra como esperada | Causa 2 quando o planejamento sabe que o número muda | O planejamento pode marcar demais e esconder uma regressão | Baixo |
| A5 | **Conferir o Baseline no planejamento:** a validação roda cada comando de Baseline uma vez e recusa o que termina com erro, e a `approve-plan` manda rodar o filtro de testes contra os nomes novos | Parte da causa 2 (SPEC-009) e os filtros errados | Planejamento mais lento | Médio |
| A6 | **Aceite mais barato:** aceitar a diferença sem zerar os testes e as evidências, que não dependem da regressão | Custo de cada bloqueio | Nenhum relevante | Baixo |

## Recomendação

- **A2** resolve a causa que voltou nas duas últimas specs (SPEC-016 e SPEC-020), sem abrir mão de pegar linhas de erro novas.
- **A6** reduz o custo dos bloqueios que continuarem.
- **A4** dá ao planejamento um jeito explícito de dizer "este número muda de propósito", em vez de evitar o comando.
- A3 fica de fora por depender do formato de cada executor; A5 pode vir depois, se os casos da causa 2 continuarem.

Com A2, A4 e A6, os oito bloqueios do arquivo teriam sido: três sem bloqueio (acréscimos), quatro sem bloqueio se o planejamento tivesse marcado as contagens da SPEC-006 como `muda`, e um (SPEC-009) bloqueado, com aceite mais barato.

## Perguntas para o refinamento

1. Acréscimo não bloqueia: com linhas de alerta (A2) ou sem (A1)?
2. Entra a mudança esperada declarada (A4)?
3. O aceite deixa de zerar testes e evidências (A6)?
4. Vale conferir os comandos de Baseline no planejamento (A5) agora ou depois?

## Fontes

- `apps/cli/src/commands/task.rs` (`regression`, `recorded_run`, `without_ignored`, `regression_acceptances`).
- `packages/harness/config/tasks.yml` (`regression_ignore`).
- `packages/harness/skills/default/approve-plan/SKILL.md` (orientação do Baseline).
- Registros e `accepted-NN.diff` das tarefas arquivadas em `vostok/work/.archive/`.
- Issue #6 de Ishtar-Sink/Vostok e as SPEC-016, SPEC-017 e SPEC-018.
