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
status: done
content_hash: fe1d2451eceb186113a94fa7ca1f6d9150b5c55c8127800120bb1dc6d99d7267
adjustments: 2
adjusted_at: 2026-10-31T15:19:44Z
approved_at: 2026-10-31T14:09:07Z
approved_hash: 380e3f93b839b0d63eedf55bb54ecbe0766da2ecfcf996dc211917b941959ca5
attempts: 0
tests_result: passed
regression_result: passed
evidence_result: passed
model: opus
updated_at: 2026-10-31T15:19:44Z
---

# TASK-03: Aceite que mantém testes e evidências

## Execução

### Ler

- `vostok/work/specs/refined/SPEC-021-regressao-que-nao-e-regressao.md` (RF-06, CB-07)
- `apps/cli/src/commands/task.rs` (fn resume, fn reset_results, fn finish)
- `apps/cli/tests/task.rs` (testes accepted_difference_lets_the_task_finish, resume_without_acceptance_blocks_again_on_the_same_difference)

### Mudar

- `apps/cli/src/commands/task.rs` (alterar): aceite que mantém testes e evidências
- `apps/cli/tests/task.rs` (alterar): testes `kept_results_*`

### Passos

1. Escreva primeiro os testes `kept_results_*`: tarefa com testes e evidências aprovados, bloqueada por regressão; `resume … --accept-regression 1` deixa `tests_result: passed` e `evidence_result` como estavam e `regression_result: null`; `regression` e `finish` concluem sem rodar `tests` de novo (CT-08); retomada sem `--accept-regression` mantém o comportamento atual de zerar tudo; aceite com testes reprovados mantém o resultado dos testes (CB-07).
2. Em `resume`, com aceites: zere só `regression_result` e as tentativas, mantenha `tests_result` e `evidence_result`, e troque a mensagem final por `Tarefa {task} retomada: status '{validating}', diferença aceita. Execute vostok task regression e, com ela aprovada, vostok task finish.`. Sem aceites, mantenha o comportamento e a mensagem atuais.
3. Rode `cargo fmt` e `cargo clippy --manifest-path apps/cli/Cargo.toml -- -D warnings`.

### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test task -- resume accept 2>&1 | grep -E '^test ' | sort`

### Verificação

```bash
# evidência: cheap-accept-tests.txt
cargo test --manifest-path apps/cli/Cargo.toml --test task -- kept_results_
cargo test --manifest-path apps/cli/Cargo.toml --test task
cargo clippy --manifest-path apps/cli/Cargo.toml -- -D warnings
```

Resultado esperado: todos os testes passam e o clippy termina sem avisos.

### Evidências

- `cheap-accept-tests.txt` (automática, da verificação)
  - Prova: o aceite mantém testes e evidências e exige só a regressão (RF-06, CT-08, CB-07).
  - Trecho: as linhas `test ... ok` dos testes novos.
  - Falha: `FAILED` ou ausência dos testes `kept_results_`.
  - Espera: `kept_results_`

### Concluída quando

- [x] O aceite mantém testes e evidências, e a tarefa conclui só com a regressão de novo (RF-06, CT-08).

  > Como comprovar: em `cheap-accept-tests.txt`, ver aprovado o teste que conclui sem rodar `tests`.

- [x] A retomada sem aceite continua zerando tudo (RF-06).

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
| 1 | 2026-10-31 | início | in_progress | aprovada em 2026-10-31T14:09:07Z; modelo: opus |
| 2 | 2026-10-31 | baseline | coletado | 1 comando(s); códigos de saída: 0 |
| 3 | 2026-10-31 | ajuste | in_progress | Edição manual: Execução, Registro de execução |
| 4 | 2026-10-31 | ajuste | in_progress | prefixo dos testes novos trocado de cheap_accept_ para kept_results_, que não cai no filtro accept do Baseline |
| 5 | 2026-10-31 | testes | passed | Verificação sem falhas; saída em validation/tests.out |
| 6 | 2026-10-31 | regressão | passed | 1 comando(s) iguais ao baseline |
| 7 | 2026-10-31 | evidências | passed | 1 evidência(s) em evidence/TASK-03/ |
| 8 | 2026-10-31 | conclusão | done | testes, regressão e evidências passed; critérios de conclusão marcados |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
| 1 | 2026-10-31 | manual | Edição manual: Execução, Registro de execução |
| 2 | 2026-10-31 | agente | prefixo dos testes novos trocado de cheap_accept_ para kept_results_, que não cai no filtro accept do Baseline |
