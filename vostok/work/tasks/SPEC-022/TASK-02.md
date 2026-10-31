---
id: TASK-02
spec: SPEC-022
title: "Requer nas evidências"
profile: padrao
persona: desenvolvedor
depends_on: [TASK-01]
wave: 2
covers: [RF-01, RF-02, RF-03, CT-01, CT-02, CT-03, CT-04, CB-01, CB-02]
type: tarefa
labels: [SPEC-022, padrao]
status: done
content_hash: 9fa186e7cc7d18e23555d1f974db13c18c09a4386d2d47eb11986d6d417326cb
adjustments: 0
adjusted_at: null
approved_at: 2026-10-31T14:09:07Z
approved_hash: 5187bd39e1a4a0d9cfd7a64208833005a1088ba7b1e6b2be3d72501683f90039
attempts: 0
tests_result: passed
regression_result: passed
evidence_result: passed
model: opus
---

# TASK-02: Requer nas evidências

## Execução

### Ler

- `vostok/work/specs/refined/SPEC-022-baseline-sem-rodar-evidencias-que-so-podem-falhar-antes-da-tarefa.md` (RF-01 a RF-03, RN-02, CB-01, CB-02)
- `apps/cli/src/commands/task.rs` (fn collect_baseline, fn automatic_evidences, fn evidence, fn evidence_expectations, fn evidence_fields, fn evidence_rule_errors, fn created_paths)
- `apps/cli/src/commands/plan.rs` (fn evidence_errors, fn evidence_fields)
- `apps/cli/tests/task.rs` (testes evidence_baseline_*)
- `apps/cli/tests/plan.rs` (fn plan_task_with)

### Mudar

- `apps/cli/src/commands/task.rs` (alterar): subitem `Requer:` na coleta, na comparação e na validação
- `apps/cli/tests/task.rs` (alterar): testes `requires_*`
- `apps/cli/src/commands/plan.rs` (alterar): `Requer:` aceito e validado no plano
- `apps/cli/tests/plan.rs` (alterar): teste `requires_*` do plano

### Passos

1. Escreva primeiro os testes `requires_*`: em `tests/task.rs`, evidência com `` Requer: `novo.txt` `` (caminho `criar`) ausente: o baseline grava `$ comando`, `exit: 1` e `não executada: novo.txt ainda não existe`, e o comando não roda (contador inalterado) (CT-01); com o caminho já existente, roda (CT-02, CB-02); o `vostok task evidence` não avisa "saída igual" para ela (CT-04); a validação recusa `Requer:` com caminho que não é `criar` (CT-03, CB-01). Em `tests/plan.rs`, o plano aceita `Requer:` com caminho `criar` e recusa os demais.
2. Leia o subitem `Requer:` como o `Espera:` (função de subitens por evidência), com a lista de caminhos entre crases separados por vírgula; aceite-o como campo opcional mesmo fora de `evidence_optional_fields` (tarefa e plano).
3. Em `collect_baseline`, antes de rodar uma evidência, confira os caminhos de `Requer:`; com algum ausente, grave a linha de não executada e não rode.
4. No `vostok task evidence`, uma evidência cujo baseline começa a saída por `não executada:` não entra na comparação de "saída igual".
5. Em `evidence_rule_errors`, recuse caminho de `Requer:` fora dos caminhos `criar`: `'Evidências': a evidência <arquivo> requer '<caminho>', que não é criado pela tarefa`.
6. Rode `cargo fmt` e `cargo clippy --manifest-path apps/cli/Cargo.toml -- -D warnings`.

### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test task -- evidence_baseline_ 2>&1 | grep -E '^test ' | sort`
- `cargo test --manifest-path apps/cli/Cargo.toml --test plan -- plan_evidence_rules_ 2>&1 | grep -E '^test ' | sort`

### Verificação

```bash
# evidência: requires-tests.txt
cargo test --manifest-path apps/cli/Cargo.toml --test task --test plan -- requires_
cargo test --manifest-path apps/cli/Cargo.toml
cargo clippy --manifest-path apps/cli/Cargo.toml -- -D warnings
```

Resultado esperado: toda a suíte passa e o clippy termina sem avisos.

### Evidências

- `requires-tests.txt` (automática, da verificação)
  - Prova: evidência com `Requer:` ausente não roda no baseline nem é comparada, e só caminhos `criar` são aceitos (RF-01 a RF-03, CT-01 a CT-04, CB-01, CB-02).
  - Trecho: as linhas `test ... ok` dos testes novos.
  - Falha: `FAILED` ou ausência dos testes `requires_`.
  - Espera: `requires_`

### Concluída quando

- [x] Evidência com `Requer:` ausente não roda no baseline e grava o motivo (RF-01, CT-01, CT-02).

  > Como comprovar: em `requires-tests.txt`, ver aprovados os testes do caminho ausente e presente.

- [x] Ela não gera aviso de "saída igual" (RF-03, CT-04).

  > Como comprovar: em `requires-tests.txt`, ver aprovado o teste sem aviso.

- [x] `Requer:` só aceita caminhos `criar`, na tarefa e no plano (RF-02, CT-03, CB-01).

  > Como comprovar: em `requires-tests.txt`, ver aprovados os testes de recusa.

## Descrição

### Resumo executivo

Permite declarar que uma evidência depende de algo que a própria tarefa cria; antes da tarefa, ela não roda, e o início deixa de gastar dezenas de minutos com testes que só podem falhar. Perfil padrão; depende da TASK-01.

### Detalhes técnicos

Abordagem: subitem opcional lido como o `Espera:`; a coleta pula a evidência e grava o motivo; a validação compartilhada (`evidence_rule_errors`) recusa caminhos que não são `criar`. `plan.rs` entra porque o plano aceita e valida subitens em separado. Componentes compartilhados com TASK-01 e TASK-03 (cadeia). Referências: RF-01 a RF-03, RN-02, decisão 1.

### Card

- **Título:** Requer nas evidências
- **Tipo:** tarefa
- **Descrição:** Subitem `Requer:` para evidências que dependem do que a tarefa cria.
- **Critérios de aceite:**
  - [ ] Não roda no baseline sem os caminhos
  - [ ] Sem aviso de saída igual
  - [ ] Só caminhos criar
- **Rótulos:** SPEC-022, padrao
- **Estimativa:** padrao
- **Bloqueada por:** TASK-01

## Registro de execução

| # | Data | Etapa | Resultado | Detalhe |
|---|---|---|---|---|
| 1 | 2026-10-31 | início | in_progress | aprovada em 2026-10-31T14:09:07Z; modelo: opus |
| 2 | 2026-10-31 | baseline | coletado | 2 comando(s); códigos de saída: 0, 0 |
| 3 | 2026-10-31 | testes | passed | Verificação sem falhas; saída em validation/tests.out |
| 4 | 2026-10-31 | regressão | passed | 2 comando(s) iguais ao baseline |
| 5 | 2026-10-31 | evidências | passed | 1 evidência(s) em evidence/TASK-02/ |
| 6 | 2026-10-31 | conclusão | done | testes, regressão e evidências passed; critérios de conclusão marcados |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
