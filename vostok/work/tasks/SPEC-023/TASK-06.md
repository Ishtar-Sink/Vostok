---
id: TASK-06
spec: SPEC-023
title: "Briefing dos documentos e exigência no finish"
profile: padrao
persona: desenvolvedor
depends_on: [TASK-04, TASK-05]
wave: 5
covers: [RF-09, RF-10, CT-09, CT-10]
type: tarefa
labels: [SPEC-023, padrao]
status: done
content_hash: 6e02c0cc82cd0b28547800b973b5f8c01fc3e971c3aea49730d7a8cd58492ff3
adjustments: 0
adjusted_at: null
approved_at: 2026-11-01T12:01:26Z
approved_hash: 0cdc671e5341abd84c8c95e55c3777d2dd8cef1aa0d94de14cda34dde1a59fef
attempts: 0
tests_result: passed
regression_result: passed
evidence_result: passed
model: claude-opus-5-5
---

# TASK-06: Briefing dos documentos e exigência no finish

## Execução

### Ler

- `vostok/work/specs/refined/SPEC-023-tarefa-como-pasta-com-arquivos-dedicados.md` (RF-09, RF-10, CB-04, CB-08)
- `apps/cli/src/commands/task.rs` (fn finish, fn brief, fn git_text, fn path_diff, struct TaskDoc)
- `apps/cli/tests/task.rs` (testes folder_format_* e part_hash_*)

### Mudar

- `apps/cli/src/commands/task.rs` (alterar): `vostok task describe` e exigência dos documentos no `finish`
- `apps/cli/tests/task.rs` (alterar): testes `describe_*`

### Passos

1. `apps/cli/src/commands/task.rs`: subcomando `vostok task describe <SPEC> <TASK>`, para tarefa `validating` ou `in_progress`. Imprime: uma instrução curta (escrever `executivo.md` para quem não é técnico e `tecnico.md` para quem mantém o código, descrevendo o que foi feito); título e descrição; `execucao.md`; os caminhos de "Mudar" com `git diff --stat` de cada um; a lista das evidências com o resultado; e o `registro.md`. Confira: `cargo test --test task -- describe_`.
2. `apps/cli/src/commands/task.rs`: no formato de pasta, o `vostok task finish` recusa enquanto `executivo.md` ou `tecnico.md` tiverem variável do template sem preencher, com `o documento <arquivo> de <TASK> ainda está no template; escreva-o a partir de vostok task describe` (RF-10, CB-08). No formato atual, nada muda.
3. `apps/cli/tests/task.rs` (alterar): testes `describe_*`: a saída traz os passos, os caminhos com o diff resumido, as evidências e o registro (CT-09); o `finish` recusa com o `tecnico.md` no template e conclui depois de escrito (CT-10, CB-08); tarefa migrada com os documentos preenchidos conclui sem reescrever (CB-04). Confira: `cargo test --test task -- describe_`.

### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test task --test plan --test finalize 2>&1 | grep -E '^test ' | sort`
- `cargo test --manifest-path apps/cli/Cargo.toml --test scenarios 2>&1 | grep -E '^test ' | sort`

### Verificação

```bash
# evidência: describe-tests.txt
cargo test --manifest-path apps/cli/Cargo.toml --test task -- describe_
cargo test --manifest-path apps/cli/Cargo.toml
cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings
```

Resultado esperado: toda a suíte passa, inclusive os goldens e os cenários, e o clippy termina sem avisos.

### Evidências

- `describe-tests.txt` (automática, da verificação)
  - Prova: o briefing dos documentos e a exigência deles no `finish` (RF-09, RF-10, CT-09, CT-10, CB-04, CB-08).
  - Trecho: as linhas `test ... ok` dos testes novos.
  - Falha: `FAILED` ou ausência dos testes `describe_`.
  - Espera: `describe_`

### Concluída quando

- [x] O `describe` traz passos, diff, evidências e registro (RF-09, CT-09).

  > Como comprovar: em `describe-tests.txt`, ver aprovado o teste da saída.

- [x] O `finish` exige o executivo e o técnico escritos (RF-10, CT-10).

  > Como comprovar: em `describe-tests.txt`, ver aprovado o teste da recusa.

## Descrição

### Resumo executivo

Prepara o resumo que um modelo mais barato usa para escrever o executivo e o técnico de cada tarefa depois que ela está pronta, e impede a conclusão sem esses dois documentos.

### Detalhes técnicos

Abordagem: briefing montado pelo `vostok` com os fatos (diff, evidências, registro), para o subagente não precisar ler a pasta (inferência G). A exigência vale só no formato de pasta. Divide `task.rs` e `tests/task.rs` com a TASK-07 na onda 5: executar em sequência. Referências: RF-09, RF-10, CB-04, CB-08, estudo E2 e E3.

### Card

- **Título:** Briefing dos documentos e exigência no finish
- **Tipo:** tarefa
- **Descrição:** Briefing para o executivo e o técnico e finish que exige os dois.
- **Critérios de aceite:**
  - [ ] describe com passos, diff, evidências e registro
  - [ ] finish recusa documento no template
- **Rótulos:** SPEC-023, padrao
- **Estimativa:** padrao
- **Bloqueada por:** TASK-04, TASK-05

## Registro de execução

| # | Data | Etapa | Resultado | Detalhe |
|---|---|---|---|---|
| 1 | 2026-11-01 | início | in_progress | aprovada em 2026-11-01T12:01:26Z; modelo: claude-opus-5-5 |
| 2 | 2026-11-01 | baseline | coletado | 2 comando(s); códigos de saída: 0, 0 |
| 3 | 2026-11-01 | testes | passed | Verificação sem falhas; saída em validation/tests.out |
| 4 | 2026-11-01 | regressão | passed | 1 comando(s) iguais ao baseline; 1 com acréscimo |
| 5 | 2026-11-01 | evidências | passed | 1 evidência(s) em evidence/TASK-06/ |
| 6 | 2026-11-01 | conclusão | done | testes, regressão e evidências passed; critérios de conclusão marcados |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
