---
id: TASK-01
spec: SPEC-023
title: "Cenários de referência no formato atual"
profile: padrao
persona: qa
depends_on: []
wave: 1
covers: [CT-16]
type: tarefa
labels: [SPEC-023, padrao]
status: approved
content_hash: 4aaa05afd1adc6ed670dbe0100b7df190b79e26453ece2f77a0bb13d2e728133
adjustments: 0
adjusted_at: null
approved_at: 2026-11-01T12:01:26Z
approved_hash: 4aaa05afd1adc6ed670dbe0100b7df190b79e26453ece2f77a0bb13d2e728133
attempts: 0
tests_result: null
regression_result: null
evidence_result: null
---

# TASK-01: Cenários de referência no formato atual

## Execução

### Ler

- `vostok/work/specs/refined/SPEC-023-tarefa-como-pasta-com-arquivos-dedicados.md` (RNF-04, CT-16, CT-17)
- `apps/cli/tests/common/mod.rs`
- `apps/cli/tests/common/golden.rs` (normalização de `<ROOT>`)
- `apps/cli/tests/task.rs` (fn complete_task, fn task_before_baseline, fn running_task, fn set_config, fn regression_output, fn init_git)

### Mudar

- `apps/cli/tests/scenarios.rs` (criar): cenários de referência, felizes e de falha
- `apps/cli/tests/fixtures/scenarios` (criar): transcrição normalizada de cada cenário

### Passos

1. `apps/cli/tests/scenarios.rs` (criar): com `mod common;`, um executor de cenário. Cada cenário é uma lista de passos (gravar arquivo, editar arquivo, rodar `vostok` com argumentos) num `Project` novo. Para cada comando, acrescente à transcrição `$ vostok <args>`, `exit: N`, a saída padrão e a de erro. No fim, acrescente `## estado` (status, attempts, tests_result, regression_result, evidence_result e se approved_hash está preenchido), `## registro` (as linhas do "Registro de execução") e `## evidências` (os arquivos da pasta de evidências da tarefa, relativos e ordenados). Confira: `cargo test --test scenarios`.
2. `apps/cli/tests/scenarios.rs`: normalize a transcrição: a raiz do projeto vira `<ROOT>`; datas e horas ISO viram `<DATA>`; sequências de 64 hexadecimais viram `<HASH>`; o hash do commit no cabeçalho dos logs vira `<COMMIT>`; o caminho do arquivo da tarefa (`vostok/work/tasks/SPEC-001/TASK-01.md`) vira `<TAREFA>`; a pasta de evidências (`vostok/work/tasks/SPEC-001/evidence/TASK-01`) vira `<EVIDENCIAS>`. Confira: duas execuções seguidas dão a mesma transcrição.
3. `apps/cli/tests/scenarios.rs`: concentre o acesso ao formato em quatro funções: `write_task` (grava a tarefa), `state_of` (lê um campo do estado), `record_rows` (linhas do registro) e `evidence_root` (pasta de evidências). Nesta tarefa elas conhecem só o formato atual; a TASK-10 acrescenta o formato de pasta mudando só elas. Confira: nenhum cenário cita `TASK-01.md` ou `evidence/` fora dessas funções.
4. `apps/cli/tests/scenarios.rs`: um `#[test]` por cenário, com o prefixo `scenario_`: `scenario_full_flow` (aprovar, start, tests, regression, evidence, finish e finalize sources de uma spec de duas tarefas); `scenario_failing_tests_block_and_resume`; `scenario_regression_block_and_accept`; `scenario_regression_growth_passes`; `scenario_failing_evidence`; `scenario_requires_and_timeout`; `scenario_start_refused` (dependência pendente e tarefa mudada depois da aprovação); `scenario_finish_refused` (critério sem marcar e resultado pendente); `scenario_manual_edit_and_adjust` (edição manual sem registro e ajuste em execução); `scenario_tests_without_baseline`. Confira: cada cenário com falha termina com o código de saída e a mensagem de recusa na transcrição.
5. `apps/cli/tests/fixtures/scenarios` (criar): com `VOSTOK_SCENARIOS_RECORD=1`, o teste grava `<cenário>.txt`; sem a variável, compara com a fixture e, na diferença, mostra as linhas que diferem. Grave as fixtures com o código atual. Confira: `cargo test --test scenarios` passa sem a variável.
6. Não mude nada em `apps/cli/src`: os cenários descrevem o comportamento de hoje. Se um cenário não puder ser escrito sem mudar o código, **PARE** com o template `blocker`.

### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test task --test plan --test finalize 2>&1 | grep -E '^test ' | sort`

### Verificação

```bash
# evidência: scenarios-tests.txt
cargo test --manifest-path apps/cli/Cargo.toml --test scenarios
cargo test --manifest-path apps/cli/Cargo.toml
cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings
```

Resultado esperado: toda a suíte passa, inclusive os goldens e os cenários, e o clippy termina sem avisos.

### Evidências

- `scenarios-tests.txt` (automática, da verificação)
  - Prova: os dez cenários, felizes e de falha, conferem com as fixtures gravadas no formato atual (RNF-04, CT-16).
  - Trecho: as linhas `test ... ok` dos testes novos.
  - Falha: `FAILED` ou menos de dez testes `scenario_`.
  - Espera: `scenario_`

### Concluída quando

- [ ] Os dez cenários, felizes e de falha, têm fixture gravada com o código atual (CT-16).

  > Como comprovar: em `scenarios-tests.txt`, ver os dez testes `scenario_` aprovados.

- [ ] O acesso ao formato está só em `write_task`, `state_of`, `record_rows` e `evidence_root`.

  > Como comprovar: abrir `apps/cli/tests/scenarios.rs` e ver que os cenários não citam caminhos do formato.

## Descrição

### Resumo executivo

Grava, antes de qualquer mudança, como o harness se comporta hoje em dez situações, inclusive as de erro. Essa gravação é a régua do antes e depois que garante que a reestruturação não muda o funcionamento.

### Detalhes técnicos

Teste de integração novo com transcrição normalizada por cenário, gravada e comparada (`VOSTOK_SCENARIOS_RECORD=1`). O formato fica isolado em quatro funções para a TASK-10 trocar só elas. Persona `qa`. Sem mudança em `src`. Referências: RNF-04, CT-16; pedido do usuário no planejamento.

### Card

- **Título:** Cenários de referência no formato atual
- **Tipo:** tarefa
- **Descrição:** Cenários felizes e de falha gravados como referência do comportamento atual.
- **Critérios de aceite:**
  - [ ] Cenários gravados como fixtures
  - [ ] Execução determinística
- **Rótulos:** SPEC-023, padrao
- **Estimativa:** padrao
- **Bloqueada por:** -

## Registro de execução

| # | Data | Etapa | Resultado | Detalhe |
|---|---|---|---|---|

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
