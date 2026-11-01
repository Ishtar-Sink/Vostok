---
id: TASK-03
spec: SPEC-023
title: "Formato de pasta na camada da tarefa"
profile: julgamento
persona: desenvolvedor
depends_on: [TASK-02]
wave: 3
covers: [RF-02, RF-03, RF-05, CT-02, CT-03, CT-05]
type: tarefa
labels: [SPEC-023, julgamento]
status: approved
content_hash: 3b2f2f6a2d87066782211b353a0fbb29103f9fc679db62448dcde4e966fd1c98
adjustments: 0
adjusted_at: null
approved_at: 2026-11-01T12:01:26Z
approved_hash: 3b2f2f6a2d87066782211b353a0fbb29103f9fc679db62448dcde4e966fd1c98
attempts: 0
tests_result: null
regression_result: null
evidence_result: null
---

# TASK-03: Formato de pasta na camada da tarefa

## Execução

### Ler

- `vostok/work/specs/refined/SPEC-023-tarefa-como-pasta-com-arquivos-dedicados.md` (RF-02, RF-03, RF-05, RN-01, RN-02, CB-01, CB-06, CB-07, "Contratos e interfaces")
- `apps/cli/src/commands/task.rs` (struct TaskDoc e fn task_entries, da TASK-02)
- `apps/cli/src/support.rs` (fn config_get, fn config_list, fn frontmatter_set)
- `packages/harness/config/tasks.yml` (evidence_dir, execution_section e as chaves de subseção)

### Mudar

- `apps/cli/src/commands/task.rs` (alterar): formato de pasta na `TaskDoc`
- `apps/cli/tests/task.rs` (alterar): testes `folder_format_*`
- `packages/harness/config/tasks.yml` (alterar): chaves `task_files` e `task_evidence_dir`

### Passos

1. `packages/harness/config/tasks.yml` (alterar): acrescente `task_files` (state, readme, execution, verification, criteria, executive, technical, record, com os nomes do contrato) e `task_evidence_dir: evidencias`, com comentário. Sem as chaves (projetos antigos), valem os nomes do contrato. Confira: `cargo test --test init_update`.
2. `apps/cli/src/commands/task.rs`: `TaskDoc::open` reconhece a pasta `TASK-NN/` com `estado.yml` como formato de pasta. Nela: o estado é lido e gravado em `estado.yml` (uma chave por linha, como o front-matter de hoje); as subseções vêm de `execucao.md` (Ler, Mudar, Passos), `verificacao.md` (Baseline, Verificação, Evidências) e `criterios.md` (Concluída quando), com os mesmos títulos `###`; o registro e o histórico ficam em `registro.md`; as evidências, em `TASK-NN/evidencias/`. Confira: `cargo test --test task -- folder_format_`.
3. `apps/cli/src/commands/task.rs`: `task_entries` lista arquivos `TASK-NN.md` e pastas `TASK-NN/`. Spec com os dois formatos: os comandos recusam com `a spec <SPEC> tem tarefas nos dois formatos; execute: vostok task migrate <SPEC>` (RN-01, CB-01). Arquivo de execução ausente na pasta: recusa citando o arquivo (CB-07).
4. `apps/cli/tests/task.rs` (alterar): testes `folder_format_*` com uma tarefa escrita à mão em pasta: aprovar e iniciar grava status e hashes em `estado.yml`, e os markdown ficam sem front-matter (CT-02); `start` e `finish` acrescentam linhas a `registro.md` (CT-03); `start`, `tests` e `evidence` gravam em `TASK-01/evidencias/` sem criar `evidence/` (CT-05); spec mista recusada (CB-01); arquivo ausente recusado (CB-07). Confira: `cargo test --test task -- folder_format_`.
5. Os cenários e os goldens do formato atual continuam passando sem atualização.

### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test task --test plan --test finalize 2>&1 | grep -E '^test ' | sort`
- `cargo test --manifest-path apps/cli/Cargo.toml --test scenarios 2>&1 | grep -E '^test ' | sort`

### Verificação

```bash
# evidência: folder-format-tests.txt
cargo test --manifest-path apps/cli/Cargo.toml --test task -- folder_format_
cargo test --manifest-path apps/cli/Cargo.toml
cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings
```

Resultado esperado: toda a suíte passa, inclusive os goldens e os cenários, e o clippy termina sem avisos.

### Evidências

- `folder-format-tests.txt` (automática, da verificação)
  - Prova: estado em `estado.yml`, registro em `registro.md`, evidências em `evidencias/` e recusa de spec mista (RF-02, RF-03, RF-05, CT-02, CT-03, CT-05, CB-01, CB-07).
  - Trecho: as linhas `test ... ok` dos testes novos.
  - Falha: `FAILED` ou ausência dos testes `folder_format_`.
  - Espera: `folder_format_`

### Concluída quando

- [ ] Tarefa em pasta guarda o estado em `estado.yml` e o registro em `registro.md` (RF-02, RF-03, CT-02, CT-03).

  > Como comprovar: em `folder-format-tests.txt`, ver aprovados os testes de estado e registro.

- [ ] As evidências da tarefa em pasta ficam em `evidencias/` (RF-05, CT-05).

  > Como comprovar: em `folder-format-tests.txt`, ver aprovado o teste das evidências.

- [ ] Spec com os dois formatos é recusada (CB-01).

  > Como comprovar: em `folder-format-tests.txt`, ver aprovado o teste da spec mista.

## Descrição

### Resumo executivo

Ensina o harness a trabalhar com a tarefa em pasta: estado, registro e evidências no lugar novo, sem mudar nada para as tarefas no formato atual.

### Detalhes técnicos

Abordagem: a `TaskDoc` da TASK-02 ganha o segundo formato, escolhido pela presença da pasta com `estado.yml`. Os títulos `###` ficam dentro dos arquivos (inferência C), o que reaproveita a leitura por subseção. O `estado.yml` usa uma chave por linha, lida e gravada como o front-matter, sem crate nova (RNF-03). Referências: RF-02, RF-03, RF-05, RN-01, inferências C e D.

### Card

- **Título:** Formato de pasta na camada da tarefa
- **Tipo:** tarefa
- **Descrição:** Tarefa em pasta: estado em estado.yml, registro em registro.md e evidências em evidencias/.
- **Critérios de aceite:**
  - [ ] Estado e registro fora do markdown
  - [ ] Evidências na pasta
  - [ ] Spec com dois formatos recusada
- **Rótulos:** SPEC-023, julgamento
- **Estimativa:** julgamento
- **Bloqueada por:** TASK-02

## Registro de execução

| # | Data | Etapa | Resultado | Detalhe |
|---|---|---|---|---|

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
