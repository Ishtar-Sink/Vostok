---
id: TASK-04
spec: SPEC-021
title: "Orientação da regressão nas skills"
profile: padrao
persona: desenvolvedor
depends_on: []
wave: 1
covers: [RF-07, CT-09]
type: tarefa
labels: [SPEC-021, padrao]
status: done
content_hash: d16050bc906b72aa3d4346db0ea2e6fe4d6e6d758bc5eab31b6c9915a01b1005
adjustments: 0
adjusted_at: null
approved_at: 2026-10-31T14:09:07Z
approved_hash: 2cf75ec204f169856a88eb71fc23563969706ce0691d27dada46a19c8e506f3b
attempts: 0
tests_result: passed
regression_result: passed
evidence_result: passed
model: opus
---

# TASK-04: Orientação da regressão nas skills

## Execução

### Ler

- `vostok/work/specs/refined/SPEC-021-regressao-que-nao-e-regressao.md` (RF-07, "Contratos e interfaces")
- `packages/harness/skills/default/approve-plan/SKILL.md` (variável do Baseline, linha 110)
- `packages/harness/skills/default/finish-task/SKILL.md` (passo 4, verificar regressões, e a retomada)
- `packages/harness/personas/qa-lead.md`
- `apps/cli/tests/base.rs` (testes que conferem trechos das skills)

### Mudar

- `packages/harness/skills/default/approve-plan/SKILL.md` (alterar): critério novo e item `(muda)` na orientação do Baseline
- `packages/harness/skills/default/finish-task/SKILL.md` (alterar): acréscimo, mudança esperada e aceite que roda só a regressão
- `packages/harness/personas/qa-lead.md` (alterar): regra sobre comando que a tarefa muda de propósito

### Passos

1. `approve-plan`, orientação do Baseline: diga que linhas novas com o mesmo código de saída não bloqueiam (acréscimo), salvo as que contêm um texto de `regression_alert`, e que um comando que a tarefa muda de propósito usa `` - `comando` (muda): motivo ``, com o exemplo da spec. Mantenha a regra de medir o que a tarefa não pode quebrar.
2. `finish-task`, passo de regressão: explique que acréscimos e mudanças esperadas aparecem na saída sem bloquear e que, num bloqueio aceito pelo usuário, depois do `vostok task resume … --accept-regression N` basta rodar `vostok task regression` e `vostok task finish`.
3. `qa-lead`: acrescente uma regra numerada na sequência: comando de Baseline que a tarefa muda de propósito leva `(muda)` com o motivo; nunca usá-lo para esconder uma regressão.
4. Rode `cargo test --manifest-path apps/cli/Cargo.toml --test base`; se um teste conferir um trecho que mudou, ajuste só o trecho, sem mudar o teste.

### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test base 2>&1 | grep -E '^test ' | sort`

### Verificação

```bash
cargo test --manifest-path apps/cli/Cargo.toml --test base
```

Resultado esperado: os testes de `base` passam.

### Evidências

- `approve-plan.diff` (diff): `packages/harness/skills/default/approve-plan/SKILL.md`
  - Prova: a orientação do Baseline com acréscimo, alerta e `(muda)` (RF-07, CT-09).
  - Trecho: as linhas acrescentadas com `(muda)` e `regression_alert`.
  - Falha: diff sem essas linhas.
  - Espera: `(muda)`
- `finish-task.diff` (diff): `packages/harness/skills/default/finish-task/SKILL.md`
  - Prova: a finalização descreve acréscimo, mudança esperada e o aceite que roda só a regressão (RF-07, CT-09).
  - Trecho: as linhas acrescentadas no passo de regressão.
  - Falha: diff sem o aceite novo.
  - Espera: `--accept-regression`
- `qa-lead.diff` (diff): `packages/harness/personas/qa-lead.md`
  - Prova: a regra nova sobre `(muda)` (RF-07, CT-09).
  - Trecho: a regra numerada acrescentada.
  - Falha: diff sem a regra.
  - Espera: `(muda)`

### Concluída quando

- [x] A `approve-plan`, a `finish-task` e a `qa-lead` descrevem acréscimo, alerta, `(muda)` e o aceite (RF-07, CT-09).

  > Como comprovar: abrir os três diffs e ver os trechos novos.

- [x] Os testes de `base` continuam passando.

  > Como comprovar: a Verificação termina com código 0.

## Descrição

### Resumo executivo

Ensina o planejamento e a finalização a usar o critério novo da regressão: acréscimos não travam, mudanças esperadas são declaradas e o aceite fica mais barato. Perfil padrão, risco baixo; roda em paralelo com a TASK-01.

### Detalhes técnicos

Só texto, escrito a partir dos contratos da spec. As evidências usam o tipo `diff` da 0.2.5. A SPEC-022 TASK-05 também muda a `approve-plan` e a `qa-lead`: executar depois desta. Referências: RF-07, CT-09, RN-03.

### Card

- **Título:** Orientação da regressão nas skills
- **Tipo:** tarefa
- **Descrição:** Orientação do critério novo da regressão nas skills e na `qa-lead`.
- **Critérios de aceite:**
  - [ ] approve-plan, finish-task e qa-lead atualizadas
- **Rótulos:** SPEC-021, padrao
- **Estimativa:** padrao
- **Bloqueada por:** -

## Registro de execução

| # | Data | Etapa | Resultado | Detalhe |
|---|---|---|---|---|
| 1 | 2026-10-31 | início | in_progress | aprovada em 2026-10-31T14:09:07Z; modelo: opus |
| 2 | 2026-10-31 | baseline | coletado | 1 comando(s); códigos de saída: 0 |
| 3 | 2026-10-31 | testes | passed | Verificação sem falhas; saída em validation/tests.out |
| 4 | 2026-10-31 | regressão | passed | 1 comando(s) iguais ao baseline |
| 5 | 2026-10-31 | evidências | passed | 3 evidência(s) em evidence/TASK-04/ |
| 6 | 2026-10-31 | conclusão | done | testes, regressão e evidências passed; critérios de conclusão marcados |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
