---
id: TASK-04
spec: SPEC-023
title: "Hash, edição manual e carregamento por arquivo"
profile: padrao
persona: desenvolvedor
depends_on: [TASK-03]
wave: 4
covers: [RF-06, RF-07, RF-08, CT-06, CT-07, CT-08]
type: tarefa
labels: [SPEC-023, padrao]
status: approved
content_hash: 260b60068b3856e8755b7d5960353137ab9426261c31308ba0497081faf1450e
adjustments: 0
adjusted_at: null
approved_at: 2026-11-01T12:01:26Z
approved_hash: 260b60068b3856e8755b7d5960353137ab9426261c31308ba0497081faf1450e
attempts: 0
tests_result: null
regression_result: null
evidence_result: null
---

# TASK-04: Hash, edição manual e carregamento por arquivo

## Execução

### Ler

- `vostok/work/specs/refined/SPEC-023-tarefa-como-pasta-com-arquivos-dedicados.md` (RF-06, RF-07, RF-08, CB-05, CB-06, "Contratos e interfaces")
- `apps/cli/src/commands/task.rs` (struct TaskDoc, fn content_hash, fn load_text, fn brief, fn adjust_record_manual, fn start_blockers)
- `apps/cli/tests/task.rs` (testes folder_format_* da TASK-03)

### Mudar

- `apps/cli/src/commands/task.rs` (alterar): hashes por arquivo, edição manual por arquivo e `load --part`
- `apps/cli/tests/task.rs` (alterar): testes `part_hash_*`

### Passos

1. `apps/cli/src/commands/task.rs`: no formato de pasta, `hash_content()` é a junção de `execucao.md`, `verificacao.md` e `criterios.md`, nessa ordem; `executivo.md` e `tecnico.md` ficam fora (RF-06). O `estado.yml` guarda também o hash de cada um desses arquivos e o do próprio `estado.yml` sem a linha do hash (`state_hash`). Confira: `cargo test --test task -- part_hash_`.
2. `apps/cli/src/commands/task.rs`: o `vostok task adjust record-manual` cita os arquivos que mudaram (`Edição manual: execucao.md`) e trata `estado.yml` editado à mão como edição manual (CB-06). No formato atual, a mensagem continua a de hoje.
3. `apps/cli/src/commands/task.rs`: `vostok task load` e `vostok task brief` montam o texto do agente com o estado, `execucao.md`, `verificacao.md` e `criterios.md`, sob o mesmo `## Execução` de hoje, sem os arquivos de pessoas. `--part <nome>` (`execucao`, `verificacao`, `criterios`, `executivo`, `tecnico`) imprime só esse arquivo; no formato atual, imprime a subseção equivalente.
4. `apps/cli/tests/task.rs` (alterar): testes `part_hash_*`: editar `executivo.md` de tarefa aprovada não impede o `start` (CT-06, CB-05); editar `execucao.md` faz o `record-manual` citar `execucao.md` (CT-07); editar `estado.yml` é edição manual (CB-06); `load` não traz o técnico e `--part tecnico` traz só ele (CT-08). Confira: `cargo test --test task -- part_hash_`.

### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test task --test plan --test finalize 2>&1 | grep -E '^test ' | sort`
- `cargo test --manifest-path apps/cli/Cargo.toml --test scenarios 2>&1 | grep -E '^test ' | sort`

### Verificação

```bash
# evidência: part-hash-tests.txt
cargo test --manifest-path apps/cli/Cargo.toml --test task -- part_hash_
cargo test --manifest-path apps/cli/Cargo.toml
cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings
```

Resultado esperado: toda a suíte passa, inclusive os goldens e os cenários, e o clippy termina sem avisos.

### Evidências

- `part-hash-tests.txt` (automática, da verificação)
  - Prova: hash só dos arquivos de execução, edição manual por arquivo e carregamento por arquivo (RF-06, RF-07, RF-08, CT-06 a CT-08, CB-05, CB-06).
  - Trecho: as linhas `test ... ok` dos testes novos.
  - Falha: `FAILED` ou ausência dos testes `part_hash_`.
  - Espera: `part_hash_`

### Concluída quando

- [ ] Editar o executivo ou o técnico não pede nova aprovação (RF-06, CT-06).

  > Como comprovar: em `part-hash-tests.txt`, ver aprovado o teste do executivo editado.

- [ ] O `record-manual` cita o arquivo que mudou (RF-07, CT-07).

  > Como comprovar: em `part-hash-tests.txt`, ver aprovado o teste com `execucao.md`.

- [ ] `load` traz só os arquivos do agente e `--part` um arquivo (RF-08, CT-08).

  > Como comprovar: em `part-hash-tests.txt`, ver aprovado o teste do carregamento.

## Descrição

### Resumo executivo

Faz a aprovação depender só do que o agente executa, aponta qual arquivo foi editado à mão e deixa o agente carregar só o arquivo de que precisa.

### Detalhes técnicos

Abordagem: hashes por arquivo no `estado.yml`, com o hash do próprio estado para detectar edição à mão. `--part` vale nos dois formatos. Divide o `task.rs` com a TASK-05 na onda 4: executar em sequência. Referências: RF-06 a RF-08, CB-05, CB-06.

### Card

- **Título:** Hash, edição manual e carregamento por arquivo
- **Tipo:** tarefa
- **Descrição:** Hash só dos arquivos de execução, edição manual citando o arquivo e carregamento por arquivo.
- **Critérios de aceite:**
  - [ ] Hash só da execução
  - [ ] record-manual cita o arquivo
  - [ ] load --part
- **Rótulos:** SPEC-023, padrao
- **Estimativa:** padrao
- **Bloqueada por:** TASK-03

## Registro de execução

| # | Data | Etapa | Resultado | Detalhe |
|---|---|---|---|---|

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
