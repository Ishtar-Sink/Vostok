---
id: TASK-07
spec: SPEC-023
title: "Migração das tarefas para pastas"
profile: julgamento
persona: desenvolvedor
depends_on: [TASK-04, TASK-05]
wave: 5
covers: [RF-12, CT-12]
type: tarefa
labels: [SPEC-023, julgamento]
status: approved
content_hash: 4a7baddd7ece25ef983f9739e753e9da0df3759a999ffd4e97b60f0c9d06b0eb
adjustments: 0
adjusted_at: null
approved_at: 2026-11-01T12:01:26Z
approved_hash: 4a7baddd7ece25ef983f9739e753e9da0df3759a999ffd4e97b60f0c9d06b0eb
attempts: 0
tests_result: null
regression_result: null
evidence_result: null
---

# TASK-07: Migração das tarefas para pastas

## Execução

### Ler

- `vostok/work/specs/refined/SPEC-023-tarefa-como-pasta-com-arquivos-dedicados.md` (RF-12, RN-01, RN-04, CB-01 a CB-04, "Contratos e interfaces")
- `apps/cli/src/commands/task.rs` (struct TaskDoc, fn task_entries, fn content_hash, fn start)
- `packages/harness/templates/task/readme.md` (da TASK-05)
- `apps/cli/tests/task.rs` (testes folder_format_* e part_hash_*)

### Mudar

- `apps/cli/src/commands/task.rs` (alterar): subcomando `vostok task migrate`
- `apps/cli/tests/task.rs` (alterar): testes `migrate_*`

### Passos

1. `apps/cli/src/commands/task.rs`: subcomando `vostok task migrate <SPEC>`. Recusa spec sem pasta em `paths.tasks` (arquivada ou inexistente). Spec já no formato de pasta: informa e não muda nada (CB-02). Confira: `cargo test --test task -- migrate_`.
2. `apps/cli/src/commands/task.rs`: para cada `TASK-NN.md`, crie `TASK-NN/`: o front-matter vai para `estado.yml`; Ler, Mudar e Passos para `execucao.md`; Baseline, Verificação e Evidências para `verificacao.md`; Concluída quando para `criterios.md`; Resumo executivo para `executivo.md` e Detalhes técnicos para `tecnico.md`, com o texto atual (CB-04); registro e histórico para `registro.md`. O card não é copiado: o `README.md` é gerado. Mova `evidence/TASK-NN/` para `TASK-NN/evidencias/` e remova `evidence/` vazia.
3. `apps/cli/src/commands/task.rs`: hashes: recalcule `content_hash` e, se o `approved_hash` antigo batia com o conteúdo antigo, recalcule também o `approved_hash`, para a aprovação continuar válida. Tarefa em execução mantém status, baseline e resultados (CB-03). Acrescente a cada `registro.md` a linha `migração | pasta | formato antigo convertido`. Tudo ou nada: em erro, nada fica migrado.
4. `apps/cli/tests/task.rs` (alterar): testes `migrate_*`: spec com uma tarefa concluída e uma em execução migra com evidências movidas, estado e registro preservados e aprovação válida, e a tarefa em execução segue até o `finish` (CT-12, CB-03); spec já migrada não muda (CB-02); documentos do formato antigo chegam a `executivo.md` e `tecnico.md` (CB-04); spec inexistente é recusada. Confira: `cargo test --test task -- migrate_`.

### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test task --test plan --test finalize 2>&1 | grep -E '^test ' | sort`
- `cargo test --manifest-path apps/cli/Cargo.toml --test scenarios 2>&1 | grep -E '^test ' | sort`

### Verificação

```bash
# evidência: migrate-tests.txt
cargo test --manifest-path apps/cli/Cargo.toml --test task -- migrate_
cargo test --manifest-path apps/cli/Cargo.toml
cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings
```

Resultado esperado: toda a suíte passa, inclusive os goldens e os cenários, e o clippy termina sem avisos.

### Evidências

- `migrate-tests.txt` (automática, da verificação)
  - Prova: a migração preserva estado, registro, aprovação e evidências e não muda uma spec já migrada (RF-12, CT-12, CB-02 a CB-04).
  - Trecho: as linhas `test ... ok` dos testes novos.
  - Falha: `FAILED` ou ausência dos testes `migrate_`.
  - Espera: `migrate_`

### Concluída quando

- [ ] Uma spec em andamento migra com estado, registro, aprovação e evidências preservados (RF-12, CT-12, CB-03).

  > Como comprovar: em `migrate-tests.txt`, ver aprovado o teste da spec com duas tarefas.

- [ ] Spec já migrada não muda (CB-02).

  > Como comprovar: em `migrate-tests.txt`, ver aprovado o teste da segunda migração.

## Descrição

### Resumo executivo

Dá aos projetos que já usam o harness um comando para levar as tarefas em andamento para o formato novo, sem perder aprovação, histórico nem evidências.

### Detalhes técnicos

Abordagem: conversão por spec inteira (inferência E), tudo ou nada, com recálculo dos hashes para manter a aprovação. Specs arquivadas ficam como estão (RN-04). Divide `task.rs` e `tests/task.rs` com a TASK-06 na onda 5: executar em sequência. Referências: RF-12, RN-01, RN-04, CB-01 a CB-04.

### Card

- **Título:** Migração das tarefas para pastas
- **Tipo:** tarefa
- **Descrição:** vostok task migrate converte uma spec não arquivada para pastas.
- **Critérios de aceite:**
  - [ ] Estado, registro e aprovação preservados
  - [ ] Evidências movidas
  - [ ] Spec já migrada sem mudança
- **Rótulos:** SPEC-023, julgamento
- **Estimativa:** julgamento
- **Bloqueada por:** TASK-04, TASK-05

## Registro de execução

| # | Data | Etapa | Resultado | Detalhe |
|---|---|---|---|---|

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
