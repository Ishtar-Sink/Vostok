---
id: TASK-01
spec: SPEC-019
title: "Briefing, check e limite de subagentes na CLI"
profile: padrao
persona: desenvolvedor
depends_on: []
wave: 1
covers: [RF-02, RF-03, RF-04, RF-08, CT-01, CT-02, CT-03, CT-04, CT-05, CT-06, CT-07, CB-01, CB-02, CB-04, CB-05, CB-06]
type: tarefa
labels: [SPEC-019, padrao]
status: done
content_hash: d5a19878d13802167c34d166535c21ecb572eef2408ebca4823e7b74bf22629d
adjustments: 1
adjusted_at: 2026-10-31T10:39:30Z
approved_at: 2026-10-31T10:32:31Z
approved_hash: d8378afafea1793f6963bdc70dac59381fe604c6ef5d45ec353d527dd64b0582
attempts: 0
tests_result: passed
regression_result: passed
evidence_result: passed
model: sonnet
updated_at: 2026-10-31T10:39:30Z
---

# TASK-01: Briefing, check e limite de subagentes na CLI

## Execução

### Ler

- `vostok/work/specs/refined/SPEC-019-reduzir-o-consumo-de-tokens-dos-subagentes.md` (RF-02, RF-03, RF-04, RF-08, contratos)
- `docs/estudos/consumo-de-tokens.md` (seção "Onde o harness gasta hoje")
- `apps/cli/src/commands/task.rs` (só os trechos: `enum TaskCommand`, `fn run`, `fn load`, `fn tests`, `fn require_running`, `fn status_report`, `fn existing_task`, `VERIFY_PRELUDE`; use busca e `sed -n`, não o arquivo inteiro)
- `apps/cli/src/commands/context.rs` (`load_skill`, `skill_entries`)
- `apps/cli/tests/task.rs` (helpers `complete_task`, `started_task`, `running_task`, `status_task`, `nested`, `success_stdout`, `failure_stderr`; por busca)
- `packages/harness/config/tasks.yml`

### Mudar

- `apps/cli/src/commands/task.rs` (alterar): subcomandos `brief` e `check` e o limite `max_subagents` no `status`.
- `apps/cli/tests/task.rs` (alterar): testes `brief_*`, `check_*` e `max_subagents_*`.
- `packages/harness/config/tasks.yml` (alterar): chave `max_subagents: 3`.
- `apps/cli/src/commands/context.rs` (alterar): `skill_entries` visível no crate, para o briefing reaproveitar o mapa da `execute-task`.

### Passos

1. Escreva os testes primeiro, numa seção `// --- Consumo de tokens (SPEC-019) ---` de `apps/cli/tests/task.rs`, e confirme que falham:
   - `brief_prints_rules_persona_route_and_task` (CT-01, RF-03): tarefa iniciada (`started_task(&[])`, baseline coletado pelo `start`); `vostok task brief SPEC-001 TASK-01` passa e o stdout contém, nessa ordem, `# Regras`, `# Persona`, `# Roteiro` e `# Tarefa`, o texto de `rules/agent/boundaries.md`, o de `rules/project/security.md` (se existir no projeto de teste) e a seção `## Execução` da tarefa.
   - `brief_is_smaller_than_the_current_load` (CT-02, RF-04): no mesmo projeto, `len(brief) * 100 < 40 * (len(context load --persona desenvolvedor) + len(context load --skill execute-task) + len(task load))`.
   - `brief_refuses_a_task_not_started_or_without_baseline` (CT-03, CB-04): tarefa `approved` (sem `start`) e tarefa `in_progress` sem pasta `baseline/` falham com stderr contendo `o briefing só sai para tarefa 'in_progress' com baseline coletado`.
   - `brief_without_persona_uses_desenvolvedor` (CT-04, CB-05): tarefa sem campo `persona` → `# Persona` traz o texto de `personas/desenvolvedor.md`.
   - `check_passes_and_fails_without_touching_the_task` (CT-05, RF-08): com Verificação `true`, `vostok task check` imprime `Verificação de TASK-01 passou (saída em ` e grava `evidence/TASK-01/validation/check.out`; com `false`, sai com erro, stdout contém `Verificação de TASK-01 falhou (código 1); últimas linhas`; nos dois casos o arquivo da tarefa fica igual (status, `attempts`, registro).
   - `check_refuses_a_blocked_task` (CT-06, CB-06): tarefa `blocked` → falha como o `vostok task tests` (`está bloqueada`).
   - `max_subagents_is_shown_and_invalid_values_warn` (CT-07, RF-02, CB-01, CB-02): `vostok task status SPEC-001` imprime no fim `Subagentes ao mesmo tempo: 3` com a chave em 3 e sem a chave; com `0` e com `9`, imprime `Subagentes ao mesmo tempo: 6` e o stderr tem `Aviso: max_subagents`.
2. `packages/harness/config/tasks.yml`: acrescente, com um comentário de uma linha no estilo do arquivo, `max_subagents: 3` (subagentes ao mesmo tempo numa onda da `execute-task`; de 1 a 6).
3. `apps/cli/src/commands/task.rs`:
   - `TaskCommand::Brief { spec_id, task_id }` ("Imprime o briefing do subagente.") e `TaskCommand::Check { spec_id, task_id }` ("Executa a verificação sem registrar."), com o uso `Uso: vostok task brief|check <SPEC_ID> <TASK_ID>` e as entradas em `run`.
   - `brief`: tarefa com status `in_progress` e pasta `baseline/`; senão `bail!` com `a tarefa {task} está com status '{status}'; o briefing só sai para tarefa 'in_progress' com baseline coletado` (sem baseline, acrescente `; execute: vostok task baseline {spec} {task}`). Imprime:
     - `# Regras`: o conteúdo de `rules/agent/boundaries.md`, `rules/project/security.md`, `rules/project/language.md` e dos itens `rules/...` da entrada `execute-task` do mapa usado por `vostok context load --skill` (reaproveite `skill_entries`, tornando-a `pub(crate)`); arquivo ausente é pulado com aviso no stderr.
     - `# Persona`: `personas/<persona>.md` do front-matter da tarefa; `-`, `null` ou ausente vale `desenvolvedor`.
     - `# Roteiro`: um texto fixo (constante `BRIEF_ROUTE`) com, em lista numerada: (1) conferir no registro abaixo a linha `baseline` e não mudar nada sem ela; (2) ler os arquivos de "Ler" por busca e trecho (`grep -n`, `sed -n`), nunca inteiro um arquivo com mais de 400 linhas; (3) implementar primeiro os testes pedidos e confirmar que falham; (4) seguir os "Passos" mudando só os arquivos de "Mudar"; (5) verificar com `vostok task check <SPEC_ID> <TASK_ID>` até passar; (6) sem commit, sem stage, sem `vostok task tests|regression|evidence|finish`, sem ler segredos; (7) se um passo exigir algo fora da tarefa, parar e devolver a pergunta; (8) devolver um bloco de até 5 linhas: `Resultado:`, `Arquivos:`, `Verificação:` (a última linha do `vostok task check`), `Em aberto:`. Use `<SPEC_ID>`/`<TASK_ID>` já trocados pelos IDs.
     - `# Tarefa`: a mesma saída do `vostok task load` (reaproveite o corpo de `load` numa função que devolve a `String`), mais as linhas do "Registro de execução".
   - `check`: `require_running`; executa o bloco da "Verificação" como `tests` (extraia a execução para uma função comum que recebe o nome do arquivo de saída), grava `validation/check.out` e não muda front-matter nem registro. Sucesso: `Verificação de {task} passou (saída em {caminho})`. Falha: imprime `Verificação de {task} falhou (código {N}); últimas linhas ({caminho}):` e `indented_tail(.., 30)`, e `bail!("a verificação de {task} falhou; corrija e rode de novo")`.
   - `status_report`: depois das tabelas, a linha `Subagentes ao mesmo tempo: {n}`, com `n` lido de `max_subagents` (ausente: 3; fora de 1 a 6 ou não numérico: 6 e `eprintln!("Aviso: max_subagents '{valor}' inválido em config/tasks.yml; use de 1 a 6. Vale 6.")`).
4. Rode `cargo test --manifest-path apps/cli/Cargo.toml --test task` até passar, depois `cargo fmt --manifest-path apps/cli/Cargo.toml`, `cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings` e `cargo test --manifest-path apps/cli/Cargo.toml` inteiro. Se um golden ou um teste de `status` antigo mudar só pela linha nova, ajuste a expectativa e cite isso no retorno.

### Baseline

- `cargo run --quiet --manifest-path apps/cli/Cargo.toml -- rules validate`
- `cargo test --manifest-path apps/cli/Cargo.toml --test task 2>&1 | grep -E '^test ' | grep -v '^test result' | grep -vE 'brief_|check_|max_subagents_' | sort`

### Verificação

```bash
cargo fmt --manifest-path apps/cli/Cargo.toml --check
cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path apps/cli/Cargo.toml --test task brief_
cargo test --manifest-path apps/cli/Cargo.toml --test task check_
cargo test --manifest-path apps/cli/Cargo.toml --test task max_subagents_
cargo test --manifest-path apps/cli/Cargo.toml
cargo run --quiet --manifest-path apps/cli/Cargo.toml -- rules validate
```

Resultado esperado: formatação e clippy sem avisos; os testes `brief_*` (4), `check_*` (2) e `max_subagents_*` (1) passam, e toda a suíte passa; `rules validate` imprime `Regras e mapa válidos.`

### Evidências

- `token-commands-tests.txt` (automática): `cargo test --manifest-path apps/cli/Cargo.toml --test task -- brief_ check_ max_subagents_ 2>&1 | grep -E '^test (brief|check|max_subagents)_' | sort`
  - Prova: o briefing (RF-03, RF-04), a verificação resumida (RF-08) e o limite configurável (RF-02); CT-01 a CT-07.
  - Trecho: sete linhas `test ... ok`, entre elas `brief_is_smaller_than_the_current_load ... ok`.
  - Falha: `FAILED` ou menos de sete linhas.

### Concluída quando

- [x] `vostok task brief` imprime regras, persona, roteiro e tarefa e recusa tarefa não iniciada ou sem baseline (RF-03, CT-01, CT-03, CT-04).

  > Como comprovar: em `token-commands-tests.txt`, ver os testes `brief_prints_*`, `brief_refuses_*` e `brief_without_persona_*` com `ok`.

- [x] O briefing fica abaixo de 40% da carga atual (RF-04, CT-02).

  > Como comprovar: ver `brief_is_smaller_than_the_current_load ... ok`.

- [x] `vostok task check` verifica sem mexer na tarefa e recusa a bloqueada (RF-08, CT-05, CT-06).

  > Como comprovar: ver os dois testes `check_*` com `ok`.

- [x] `max_subagents` aparece no `vostok task status` e avisa nos valores inválidos (RF-02, CT-07).

  > Como comprovar: ver `max_subagents_is_shown_and_invalid_values_warn ... ok` e abrir `packages/harness/config/tasks.yml` com `max_subagents: 3`.

- [x] A suíte inteira, o fmt e o clippy passam.

  > Como comprovar: rodar a Verificação e ver todos os comandos sem erro.

## Descrição

### Resumo executivo

Dá ao harness os comandos que deixam o trabalho dos subagentes mais barato: um resumo curto do que cada um precisa saber, uma verificação que mostra só o resultado e um limite configurável de quantos rodam ao mesmo tempo. Perfil padrão, risco baixo; libera a reescrita das skills (TASK-03).

### Detalhes técnicos

- **Abordagem:** `brief` monta regras (limites, segurança, idioma e as regras de projeto do mapa da `execute-task`), persona, roteiro fixo e a tarefa; `check` reaproveita a execução do `tests` sem gravar estado; `status` mostra o limite lido de `max_subagents`.
- **Decisões:** inferências B (comando de briefing), G (verificação como comando) e H (limite de 40%); limite padrão 3 e teto 6 (decisão 3 da spec). O roteiro fica como constante na CLI, para não criar template novo fora dos componentes da spec.
- **Descartado:** reaproveitar `context load` com filtros (continuaria trazendo o catálogo de skills e os templates de resposta).
- **Riscos:** testes antigos de `status` podem conferir a saída inteira e precisar da linha nova.
- **Referências:** RF-02, RF-03, RF-04, RF-08, CT-01 a CT-07, CB-01, CB-02, CB-04 a CB-06.
- **Arquivos em comum:** nenhum com a TASK-02 da mesma onda.

### Card

- **Título:** Briefing, check e limite de subagentes na CLI
- **Tipo:** tarefa
- **Descrição:** Comandos `vostok task brief` e `vostok task check` e o limite `max_subagents` no `vostok task status`.
- **Critérios de aceite:**
  - [ ] Briefing curto com regras, persona, roteiro e tarefa
  - [ ] Verificação resumida sem mexer na tarefa
  - [ ] Limite de subagentes configurável
- **Rótulos:** SPEC-019, padrao
- **Estimativa:** padrao
- **Bloqueada por:** -

## Registro de execução

| # | Data | Etapa | Resultado | Detalhe |
|---|---|---|---|---|
| 1 | 2026-10-31 | início | in_progress | aprovada em 2026-10-31T10:32:31Z; modelo: sonnet |
| 2 | 2026-10-31 | baseline | coletado | 2 comando(s); códigos de saída: 0, 0; 1 evidência(s) automática(s) gravada(s) para comparação |
| 3 | 2026-10-31 | ajuste | in_progress | Mudar inclui apps/cli/src/commands/context.rs, alterado no passo 3 para tornar skill_entries pub(crate) |
| 4 | 2026-10-31 | testes | passed | Verificação sem falhas; saída em validation/tests.out |
| 5 | 2026-10-31 | regressão | passed | 2 comando(s) iguais ao baseline |
| 6 | 2026-10-31 | evidências | passed | 1 evidência(s) em evidence/TASK-01/ |
| 7 | 2026-10-31 | conclusão | done | testes, regressão e evidências passed; critérios de conclusão marcados |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
| 1 | 2026-10-31 | agente | Mudar inclui apps/cli/src/commands/context.rs, alterado no passo 3 para tornar skill_entries pub(crate) |
