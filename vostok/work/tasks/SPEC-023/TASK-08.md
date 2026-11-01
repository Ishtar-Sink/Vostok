---
id: TASK-08
spec: SPEC-023
title: "Finalização da spec com pastas"
profile: padrao
persona: desenvolvedor
depends_on: [TASK-03]
wave: 4
covers: [RF-13, RF-14, CT-13, CT-14]
type: tarefa
labels: [SPEC-023, padrao]
status: approved
content_hash: 83808dc24a26e520584650c641f15405aec2e0e517880c530b67060b82b82a8d
adjustments: 0
adjusted_at: null
approved_at: 2026-11-01T12:01:26Z
approved_hash: 83808dc24a26e520584650c641f15405aec2e0e517880c530b67060b82b82a8d
attempts: 0
tests_result: null
regression_result: null
evidence_result: null
---

# TASK-08: Finalização da spec com pastas

## Execução

### Ler

- `vostok/work/specs/refined/SPEC-023-tarefa-como-pasta-com-arquivos-dedicados.md` (RF-13, RF-14)
- `apps/cli/src/commands/finalize.rs` (fn task_evidence_dir, fn sources, fn build, fn archive, fn before_html)
- `apps/cli/tests/finalize.rs` (testes evidence_before_after_*, before_skipped_*)
- `packages/harness/templates/finalize/` (templates do técnico)

### Mudar

- `apps/cli/src/commands/finalize.rs` (alterar): leitura e arquivamento das pastas e técnico das tarefas no documento
- `apps/cli/tests/finalize.rs` (alterar): testes `finalize_folder_*`

### Passos

1. `apps/cli/src/commands/finalize.rs`: as tarefas de uma spec são lidas nos dois formatos, pela mesma regra de pasta com `estado.yml`; as evidências de uma tarefa em pasta vêm de `TASK-NN/evidencias/`. Confira: `cargo test --test finalize -- finalize_folder_`.
2. `apps/cli/src/commands/finalize.rs`: o `vostok finalize sources` lista, por tarefa em pasta, o `executivo.md` e o `tecnico.md` com o conteúdo (RF-13).
3. `apps/cli/src/commands/finalize.rs`: o `vostok finalize build` inclui no `tecnico.html`, por tarefa, o `tecnico.md` (RF-14), escapado como os demais textos do documento.
4. `apps/cli/src/commands/finalize.rs`: o `vostok finalize archive` move as pastas das tarefas, com `evidencias/`, para `<archive>/<SPEC>/tasks/`.
5. `apps/cli/tests/finalize.rs` (alterar): testes `finalize_folder_*`: `sources` traz o executivo e o técnico das tarefas; `build` põe o técnico de cada tarefa no `tecnico.html` e as evidências de `evidencias/` no `evidencias.html`; `archive` move as pastas (CT-13, CT-14). Confira: `cargo test --test finalize -- finalize_folder_`.

### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test finalize 2>&1 | grep -E '^test ' | sort`

### Verificação

```bash
# evidência: finalize-folder-tests.txt
cargo test --manifest-path apps/cli/Cargo.toml --test finalize -- finalize_folder_
cargo test --manifest-path apps/cli/Cargo.toml
cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings
```

Resultado esperado: toda a suíte passa, inclusive os goldens e os cenários, e o clippy termina sem avisos.

### Evidências

- `finalize-folder-tests.txt` (automática, da verificação)
  - Prova: a finalização da spec lê e arquiva as pastas e inclui o técnico de cada tarefa (RF-13, RF-14, CT-13, CT-14).
  - Trecho: as linhas `test ... ok` dos testes novos.
  - Falha: `FAILED` ou ausência dos testes `finalize_folder_`.
  - Espera: `finalize_folder_`

### Concluída quando

- [ ] `sources`, `build` e `archive` funcionam com tarefas em pasta (RF-13, CT-13).

  > Como comprovar: em `finalize-folder-tests.txt`, ver aprovados os testes das fontes e do arquivo.

- [ ] O `tecnico.html` traz o técnico de cada tarefa (RF-14, CT-14).

  > Como comprovar: em `finalize-folder-tests.txt`, ver aprovado o teste do documento.

## Descrição

### Resumo executivo

Faz o fechamento de uma spec funcionar com as tarefas em pasta e leva o técnico detalhado de cada tarefa para o documento técnico final.

### Detalhes técnicos

Abordagem: a `finalize.rs` reconhece o formato de pasta pela mesma regra da `TaskDoc` (sem importar o tipo, que é privado em `task.rs`). O técnico de cada tarefa entra no documento (inferência A). Sem arquivos em comum com a onda 4. Referências: RF-13, RF-14.

### Card

- **Título:** Finalização da spec com pastas
- **Tipo:** tarefa
- **Descrição:** Finalização da spec lendo e arquivando as pastas, com o técnico de cada tarefa no documento.
- **Critérios de aceite:**
  - [ ] Fontes com os documentos das tarefas
  - [ ] tecnico.html com o técnico de cada tarefa
  - [ ] Pastas arquivadas
- **Rótulos:** SPEC-023, padrao
- **Estimativa:** padrao
- **Bloqueada por:** TASK-03

## Registro de execução

| # | Data | Etapa | Resultado | Detalhe |
|---|---|---|---|---|

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
