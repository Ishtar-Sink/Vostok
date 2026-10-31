---
id: TASK-05
spec: SPEC-022
title: "Orientação de Requer e Tempo-limite"
profile: padrao
persona: desenvolvedor
depends_on: []
wave: 1
covers: [RF-08, CT-09]
type: tarefa
labels: [SPEC-022, padrao]
status: approved
content_hash: 52c420c327e8a37bc48edf2e92f48f82be71726c404cb3030060697bce086d56
adjustments: 0
adjusted_at: null
approved_at: 2026-10-31T14:09:07Z
approved_hash: 52c420c327e8a37bc48edf2e92f48f82be71726c404cb3030060697bce086d56
attempts: 0
tests_result: null
regression_result: null
evidence_result: null
---

# TASK-05: Orientação de Requer e Tempo-limite

## Execução

### Ler

- `vostok/work/specs/refined/SPEC-022-baseline-sem-rodar-evidencias-que-so-podem-falhar-antes-da-tarefa.md` (RF-08, RN-01, "Contratos e interfaces")
- `packages/harness/skills/default/approve-plan/SKILL.md` (variável das Evidências, linha 118)
- `packages/harness/skills/default/approve-task/SKILL.md` (passos de aprovação e resposta)
- `packages/harness/personas/qa-lead.md`
- `apps/cli/tests/base.rs` (testes que conferem trechos das skills)

### Mudar

- `packages/harness/skills/default/approve-plan/SKILL.md` (alterar): orientação de `Requer:` e `Tempo-limite:`
- `packages/harness/skills/default/approve-task/SKILL.md` (alterar): mostrar as exceções de tempo-limite ao pedir a aprovação
- `packages/harness/personas/qa-lead.md` (alterar): regra de dependência e tempo das evidências

### Passos

1. `approve-plan`, Evidências: acrescente os subitens opcionais `` Requer: `caminho` `` (caminhos `criar` de que a evidência depende; sem eles, ela não roda no baseline) e `Tempo-limite: <segundos>` (só para evidência legitimamente lenta; 0 sem limite; vira uma exceção que o usuário aprova com a tarefa), com o exemplo da spec. Diga que as evidências no baseline têm o tempo-limite de `evidence_baseline_timeout`.
2. `approve-task`: depois de validar, se a saída listar evidências com tempo-limite acima do padrão, mostre-as ao usuário na resposta antes da aprovação, dizendo que aprovar a tarefa aprova a exceção.
3. `qa-lead`: acrescente uma regra numerada na sequência: evidência que depende do que a tarefa cria leva `Requer:`; `Tempo-limite:` só com motivo, porque vira exceção aprovada pelo usuário.
4. Rode `cargo test --manifest-path apps/cli/Cargo.toml --test base`; se um teste conferir um trecho que mudou, ajuste só o trecho.

### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test base 2>&1 | grep -E '^test ' | sort`

### Verificação

```bash
cargo test --manifest-path apps/cli/Cargo.toml --test base
```

Resultado esperado: os testes de `base` passam.

### Evidências

- `approve-plan.diff` (diff): `packages/harness/skills/default/approve-plan/SKILL.md`
  - Prova: a orientação de `Requer:` e `Tempo-limite:` (RF-08, CT-09).
  - Trecho: as linhas acrescentadas com os dois subitens.
  - Falha: diff sem os subitens.
  - Espera: `Requer:`
- `approve-task.diff` (diff): `packages/harness/skills/default/approve-task/SKILL.md`
  - Prova: a aprovação mostra as exceções ao usuário (RF-08, CT-09).
  - Trecho: o passo que mostra as exceções.
  - Falha: diff sem o passo.
  - Espera: `Tempo-limite`
- `qa-lead.diff` (diff): `packages/harness/personas/qa-lead.md`
  - Prova: a regra nova de dependência e tempo (RF-08, CT-09).
  - Trecho: a regra numerada acrescentada.
  - Falha: diff sem a regra.
  - Espera: `Requer:`

### Concluída quando

- [ ] A `approve-plan`, a `approve-task` e a `qa-lead` orientam `Requer:` e `Tempo-limite:` (RF-08, CT-09).

  > Como comprovar: abrir os três diffs e ver os trechos novos.

- [ ] Os testes de `base` continuam passando.

  > Como comprovar: a Verificação termina com código 0.

## Descrição

### Resumo executivo

Ensina o planejamento a declarar quando uma evidência depende do que a tarefa cria e quando ela precisa de mais tempo, e faz a aprovação mostrar essas exceções ao usuário. Perfil padrão, risco baixo; roda em paralelo na primeira onda.

### Detalhes técnicos

Só texto, a partir dos contratos da spec. A SPEC-021 TASK-04 também muda a `approve-plan` e a `qa-lead`: executar depois dela. Referências: RF-08, CT-09, RN-01.

### Card

- **Título:** Orientação de Requer e Tempo-limite
- **Tipo:** tarefa
- **Descrição:** Orientação de `Requer:` e `Tempo-limite:` nas skills e na `qa-lead`.
- **Critérios de aceite:**
  - [ ] approve-plan, approve-task e qa-lead atualizadas
- **Rótulos:** SPEC-022, padrao
- **Estimativa:** padrao
- **Bloqueada por:** -

## Registro de execução

| # | Data | Etapa | Resultado | Detalhe |
|---|---|---|---|---|

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
