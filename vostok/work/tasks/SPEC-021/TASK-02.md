---
id: TASK-02
spec: SPEC-021
title: "Mudança esperada no Baseline"
profile: padrao
persona: desenvolvedor
depends_on: [TASK-01]
wave: 2
covers: [RF-05, CT-06, CT-07, CB-05, CB-06]
type: tarefa
labels: [SPEC-021, padrao]
status: done
content_hash: 09bb8365b287795fd45640b0d3b781f1300cefa1152f11191ed078ed73135890
adjustments: 0
adjusted_at: null
approved_at: 2026-10-31T14:09:07Z
approved_hash: 883a89e89a756135b1e39e6da19aa28063651b7babf7bb0faf6d4553fb92f9e6
attempts: 0
tests_result: passed
regression_result: passed
evidence_result: passed
model: opus
---

# TASK-02: Mudança esperada no Baseline

## Execução

### Ler

- `vostok/work/specs/refined/SPEC-021-regressao-que-nao-e-regressao.md` (RF-05, RN-03, CB-05, CB-06, "Contratos e interfaces")
- `apps/cli/src/commands/task.rs` (fn regression, fn baseline_commands, fn baseline_command, fn is_baseline_item, fn baseline_errors, fn collect_baseline)
- `apps/cli/src/commands/plan.rs` (fn baseline_errors)
- `apps/cli/tests/task.rs` (testes growth_* da TASK-01)
- `apps/cli/tests/plan.rs` (fn plan_with_evidence)

### Mudar

- `apps/cli/src/commands/task.rs` (alterar): item `(muda): motivo` no Baseline, na coleta, na regressão e na validação
- `apps/cli/tests/task.rs` (alterar): testes `expected_change_*`
- `apps/cli/src/commands/plan.rs` (alterar): validação do item `(muda)` no plano
- `apps/cli/tests/plan.rs` (alterar): teste `expected_change_*` do plano

### Passos

1. Escreva primeiro os testes `expected_change_*`: em `tests/task.rs`, Baseline `` - `cat valor.txt` (muda): o valor passa de a para b `` com o arquivo mudado; a regressão passa, grava `NN.diff`, e o registro e a saída citam "mudança esperada" e o motivo (CT-06); item `(muda)` sem motivo é recusado pela validação (CT-07, CB-05); `(muda)` num comando que cita um caminho `criar` continua ignorado como entrega (CB-06). Em `tests/plan.rs`, o `vostok plan approve validate` aceita `(muda): motivo` e recusa sem motivo.
2. Formato: `` - `comando` (muda): motivo ``. Crie uma função que devolve `(comando, Option<motivo>)` de um item de Baseline e use-a em `baseline_commands`, `is_baseline_item`/`baseline_command` (validação) e no `plan.rs`, para que o comando rode na coleta e na regressão.
3. Em `regression`, depois da regra de entrega (RN-04): comando com motivo grava `NN.diff` quando difere, não bloqueia, conta como "mudança esperada" e entra na saída como `- [N] comando: muda (motivo)`; o detalhe do registro ganha `; N mudança(s) esperada(s)`.
4. Na validação de tarefa e do plano, `(muda)` sem texto depois de `: ` é recusado com `'Baseline' com item (muda) sem motivo: <item>`.
5. Rode `cargo fmt` e `cargo clippy --manifest-path apps/cli/Cargo.toml -- -D warnings`.

### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test task -- regression_ delivery_ growth_ 2>&1 | grep -E '^test ' | sort`
- `cargo test --manifest-path apps/cli/Cargo.toml --test plan -- baseline 2>&1 | grep -E '^test ' | sort`

### Verificação

```bash
# evidência: expected-change-tests.txt
cargo test --manifest-path apps/cli/Cargo.toml --test task --test plan -- expected_change_
cargo test --manifest-path apps/cli/Cargo.toml --test task --test plan
cargo clippy --manifest-path apps/cli/Cargo.toml -- -D warnings
```

Resultado esperado: todos os testes passam e o clippy termina sem avisos.

### Evidências

- `expected-change-tests.txt` (automática, da verificação)
  - Prova: a mudança esperada passa com o motivo registrado, e o item sem motivo é recusado na tarefa e no plano (RF-05, CT-06, CT-07, CB-05, CB-06).
  - Trecho: as linhas `test ... ok` dos testes novos.
  - Falha: `FAILED` ou ausência dos testes `expected_change_`.
  - Espera: `expected_change_`

### Concluída quando

- [x] Um item `(muda): motivo` mostra a diferença sem bloquear e registra o motivo (RF-05, CT-06).

  > Como comprovar: em `expected-change-tests.txt`, ver aprovado o teste da mudança esperada.

- [x] Item `(muda)` sem motivo é recusado na tarefa e no plano (CT-07, CB-05).

  > Como comprovar: em `expected-change-tests.txt`, ver aprovados os testes de recusa.

## Descrição

### Resumo executivo

Dá ao planejamento um jeito de dizer "este número muda de propósito", como a contagem de regras que a tarefa acrescenta. A conferência mostra a mudança com o motivo, sem travar. Perfil padrão; depende da TASK-01.

### Detalhes técnicos

Abordagem: o motivo vem do item do Baseline e é aprovado com a tarefa (RN-03); a regressão grava o diff e não bloqueia. A leitura do item muda em `task.rs` e em `plan.rs`, que validam o Baseline em separado; por isso `plan.rs` e `tests/plan.rs` entram em "Mudar" além dos componentes da spec. Componentes compartilhados com TASK-01 e TASK-03 (cadeia). Referências: RF-05, RN-03, CB-05, CB-06, decisão 2.

### Card

- **Título:** Mudança esperada no Baseline
- **Tipo:** tarefa
- **Descrição:** Item de Baseline `(muda): motivo` para mudanças esperadas.
- **Critérios de aceite:**
  - [ ] Mudança esperada não bloqueia e registra o motivo
  - [ ] Item sem motivo recusado
- **Rótulos:** SPEC-021, padrao
- **Estimativa:** padrao
- **Bloqueada por:** TASK-01

## Registro de execução

| # | Data | Etapa | Resultado | Detalhe |
|---|---|---|---|---|
| 1 | 2026-10-31 | início | in_progress | aprovada em 2026-10-31T14:09:07Z; modelo: opus |
| 2 | 2026-10-31 | baseline | coletado | 2 comando(s); códigos de saída: 0, 0 |
| 3 | 2026-10-31 | testes | passed | Verificação sem falhas; saída em validation/tests.out |
| 4 | 2026-10-31 | regressão | failed | comandos diferentes do baseline; bloqueada |
| 5 | 2026-10-31 | retomada | validating | decisão do usuário: usuário aceitou: teste novo expected_change_on_a_delivery_command_* caiu no filtro delivery_ do baseline (acréscimo, não regressão); aceito(s): 1 |
| 6 | 2026-10-31 | testes | passed | Verificação sem falhas; saída em validation/tests.out |
| 7 | 2026-10-31 | regressão | passed | 1 igual(is) ao baseline, 1 aceito(s) pelo usuário |
| 8 | 2026-10-31 | evidências | passed | 1 evidência(s) em evidence/TASK-02/ |
| 9 | 2026-10-31 | conclusão | done | testes, regressão e evidências passed; critérios de conclusão marcados |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
