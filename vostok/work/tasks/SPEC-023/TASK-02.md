---
id: TASK-02
spec: SPEC-023
title: "Camada da tarefa com o formato atual"
profile: julgamento
persona: desenvolvedor
depends_on: [TASK-01]
wave: 2
covers: [RF-11, CT-11]
type: tarefa
labels: [SPEC-023, julgamento]
status: approved
content_hash: 79ac2ae0db3570a0d068ae67827faad16c3a44ed7c6ec114cd80fbc066834278
adjustments: 0
adjusted_at: null
approved_at: 2026-11-01T12:01:26Z
approved_hash: 79ac2ae0db3570a0d068ae67827faad16c3a44ed7c6ec114cd80fbc066834278
attempts: 0
tests_result: null
regression_result: null
evidence_result: null
---

# TASK-02: Camada da tarefa com o formato atual

## Execução

### Ler

- `vostok/work/specs/refined/SPEC-023-tarefa-como-pasta-com-arquivos-dedicados.md` (RF-11, "Riscos", "Contratos e interfaces")
- `docs/estudos/tarefa-como-pasta.md` (seção "Contras", item 1)
- `apps/cli/src/commands/task.rs` (fn find_task, fn existing_task, fn task_file, fn evidence_dir, fn subsection, fn record_step, fn status_of, fn content_hash, fn markdown_files e as chamadas de frontmatter_set e frontmatter_get)
- `apps/cli/src/commands/plan.rs` (fn task_profile)
- `apps/cli/tests/scenarios.rs` (cenários da TASK-01)

### Mudar

- `apps/cli/src/commands/task.rs` (alterar): camada única da tarefa
- `apps/cli/tests/task.rs` (alterar): testes `task_layer_*`

### Passos

1. `apps/cli/src/commands/task.rs` (alterar): crie a struct `TaskDoc` (aberta por `TaskDoc::open(file)`) com: `text(key)` para a subseção de execução pela chave de configuração; `description(key)`; `get(field)` e `set(field, value)` do estado; `record(step, result, detail)`; `history()`; `evidence_dir()`; `hash_content()` (o texto usado nos hashes). Nesta tarefa ela só conhece o arquivo único e devolve exatamente o que os comandos leem hoje. Confira: `cargo build --manifest-path apps/cli/Cargo.toml`.
2. `apps/cli/src/commands/task.rs`: troque, comando por comando, as leituras por subseção, as chamadas de `frontmatter_set`/`frontmatter_get` sobre a tarefa, o `record_step` e o `evidence_dir` por chamadas à `TaskDoc`. Os comandos não leem mais o arquivo da tarefa direto. Confira após cada comando: `cargo test --test scenarios --test task`.
3. `apps/cli/src/commands/task.rs`: a listagem das tarefas de uma spec (`markdown_files` sobre a pasta da spec) passa por uma função `task_entries(dir)`, que devolve os IDs e os caminhos; a TASK-03 acrescenta as pastas nela. Confira: `vostok task status` nos testes de status.
4. `apps/cli/tests/task.rs` (alterar): testes `task_layer_*` da `TaskDoc` pelo comportamento: ler um campo, gravar um campo sem tocar no resto do arquivo, acrescentar uma linha ao registro e devolver a pasta de evidências atual. Confira: `cargo test --test task -- task_layer_`.
5. Não atualize fixture de cenário nem caso golden: qualquer diferença neles é mudança de comportamento. Diferença que não se resolve: **PARE** com o template `blocker`.

### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test task --test plan --test finalize 2>&1 | grep -E '^test ' | sort`
- `cargo test --manifest-path apps/cli/Cargo.toml --test scenarios 2>&1 | grep -E '^test ' | sort`

### Verificação

```bash
# evidência: task-layer-tests.txt
cargo test --manifest-path apps/cli/Cargo.toml --test scenarios --test task -- scenario_ task_layer_
cargo test --manifest-path apps/cli/Cargo.toml
cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings
```

Resultado esperado: toda a suíte passa, inclusive os goldens e os cenários, e o clippy termina sem avisos.

### Evidências

- `task-layer-tests.txt` (automática, da verificação)
  - Prova: a camada única mantém o comportamento: os cenários de referência e os testes da camada passam sem mudar fixture (RF-11, CT-11, RNF-04).
  - Trecho: as linhas `test ... ok` dos testes novos.
  - Falha: `FAILED` em algum `scenario_` ou ausência dos testes `task_layer_`.
  - Espera: `scenario_`

### Concluída quando

- [ ] Os comandos leem e gravam a tarefa só pela `TaskDoc` (RF-11).

  > Como comprovar: em `task-layer-tests.txt`, ver aprovados os testes `task_layer_`.

- [ ] Cenários e goldens passam sem atualização de fixture (CT-11).

  > Como comprovar: em `task-layer-tests.txt`, ver os dez `scenario_` aprovados, e a Verificação termina com código 0.

## Descrição

### Resumo executivo

Reorganiza o código que lê e grava as tarefas num único ponto, sem mudar o que o harness faz. É a base para o formato novo e a parte de maior risco, conferida pelos cenários gravados antes.

### Detalhes técnicos

Abordagem: struct `TaskDoc` dentro de `task.rs`, para não criar módulo fora dos componentes da spec. A troca é feita comando por comando, com os cenários e os 126 goldens rodando a cada passo. Alternativa descartada: implementar o formato de pasta direto nos comandos, o que espalharia dois formatos por dezenas de pontos. Referências: RF-11, Riscos da spec, estudo.

### Card

- **Título:** Camada da tarefa com o formato atual
- **Tipo:** tarefa
- **Descrição:** Interface única de leitura e gravação da tarefa, sem mudança de comportamento.
- **Critérios de aceite:**
  - [ ] Todos os comandos usam a camada
  - [ ] Cenários e goldens sem mudança
- **Rótulos:** SPEC-023, julgamento
- **Estimativa:** julgamento
- **Bloqueada por:** TASK-01

## Registro de execução

| # | Data | Etapa | Resultado | Detalhe |
|---|---|---|---|---|

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
