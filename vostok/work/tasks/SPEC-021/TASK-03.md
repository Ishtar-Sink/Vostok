---
id: TASK-03
spec: SPEC-021
title: "Aceite que mantém testes e evidências"
profile: padrao
persona: desenvolvedor
depends_on: [TASK-02]
wave: 3
covers: [RF-06, CT-08, CB-07]
type: tarefa
labels: [SPEC-021, padrao]
status: approved
content_hash: 380e3f93b839b0d63eedf55bb54ecbe0766da2ecfcf996dc211917b941959ca5
adjustments: 0
adjusted_at: null
approved_at: 2026-10-31T14:09:07Z
approved_hash: 380e3f93b839b0d63eedf55bb54ecbe0766da2ecfcf996dc211917b941959ca5
attempts: 0
tests_result: null
regression_result: null
evidence_result: null
---

# TASK-03: Aceite que mantém testes e evidências

## Execução

### Ler

- `vostok/work/specs/refined/SPEC-021-regressao-que-nao-e-regressao.md` (RF-06, CB-07)
- `apps/cli/src/commands/task.rs` (fn resume, fn reset_results, fn finish)
- `apps/cli/tests/task.rs` (testes accepted_difference_lets_the_task_finish, resume_without_acceptance_blocks_again_on_the_same_difference)

### Mudar

- `apps/cli/src/commands/task.rs` (alterar): aceite que mantém testes e evidências
- `apps/cli/tests/task.rs` (alterar): testes `cheap_accept_*`

### Passos

1. Escreva primeiro os testes `cheap_accept_*`: tarefa com testes e evidências aprovados, bloqueada por regressão; `resume … --accept-regression 1` deixa `tests_result: passed` e `evidence_result` como estavam e `regression_result: null`; `regression` e `finish` concluem sem rodar `tests` de novo (CT-08); retomada sem `--accept-regression` mantém o comportamento atual de zerar tudo; aceite com testes reprovados mantém o resultado dos testes (CB-07).
2. Em `resume`, com aceites: zere só `regression_result` e as tentativas, mantenha `tests_result` e `evidence_result`, e troque a mensagem final por `Tarefa {task} retomada: status '{validating}', diferença aceita. Execute vostok task regression e, com ela aprovada, vostok task finish.`. Sem aceites, mantenha o comportamento e a mensagem atuais.
3. Rode `cargo fmt` e `cargo clippy --manifest-path apps/cli/Cargo.toml -- -D warnings`.

### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test task -- resume accept 2>&1 | grep -E '^test ' | sort`

### Verificação

```bash
# evidência: cheap-accept-tests.txt
cargo test --manifest-path apps/cli/Cargo.toml --test task -- cheap_accept_
cargo test --manifest-path apps/cli/Cargo.toml --test task
cargo clippy --manifest-path apps/cli/Cargo.toml -- -D warnings
```

Resultado esperado: todos os testes passam e o clippy termina sem avisos.

### Evidências

- `cheap-accept-tests.txt` (automática, da verificação)
  - Prova: o aceite mantém testes e evidências e exige só a regressão (RF-06, CT-08, CB-07).
  - Trecho: as linhas `test ... ok` dos testes novos.
  - Falha: `FAILED` ou ausência dos testes `cheap_accept_`.
  - Espera: `cheap_accept_`

### Concluída quando

- [ ] O aceite mantém testes e evidências, e a tarefa conclui só com a regressão de novo (RF-06, CT-08).

  > Como comprovar: em `cheap-accept-tests.txt`, ver aprovado o teste que conclui sem rodar `tests`.

- [ ] A retomada sem aceite continua zerando tudo (RF-06).

  > Como comprovar: em `cheap-accept-tests.txt`, ver aprovado o teste da retomada sem aceite.

## Descrição

### Resumo executivo

Quando um bloqueio de regressão for aceito, a tarefa não precisa mais rodar todos os testes e evidências de novo: só a conferência de regressão. Perfil padrão, risco baixo; depende da TASK-02.

### Detalhes técnicos

Abordagem: o aceite não muda o código nem os testes, então os resultados de testes e evidências continuam válidos; só a regressão volta a rodar. A retomada sem aceite (decisão de corrigir algo) continua zerando tudo. Componentes compartilhados com TASK-01 e TASK-02 (cadeia). Referências: RF-06, CB-07, decisão 3, memória ISS-001.

### Card

- **Título:** Aceite que mantém testes e evidências
- **Tipo:** tarefa
- **Descrição:** Aceite de regressão que mantém testes e evidências.
- **Critérios de aceite:**
  - [ ] Aceite exige só a regressão
  - [ ] Retomada sem aceite como antes
- **Rótulos:** SPEC-021, padrao
- **Estimativa:** padrao
- **Bloqueada por:** TASK-02

## Registro de execução

| # | Data | Etapa | Resultado | Detalhe |
|---|---|---|---|---|

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
