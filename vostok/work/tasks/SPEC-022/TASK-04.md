---
id: TASK-04
spec: SPEC-022
title: "Antes de evidência não executada"
profile: padrao
persona: desenvolvedor
depends_on: []
wave: 1
covers: [RF-09, CT-10]
type: tarefa
labels: [SPEC-022, padrao]
status: done
content_hash: fff63304754b19687ab6340aba96aa902a120a31303e0bc3e2de41c5a9d2ec36
adjustments: 0
adjusted_at: null
approved_at: 2026-10-31T14:09:07Z
approved_hash: 0e10ae47f6cff1e9f1fbb7d91014019440950e47107da8f38ec4754687ad6ee1
attempts: 0
tests_result: passed
regression_result: passed
evidence_result: passed
model: opus
---

# TASK-04: Antes de evidência não executada

## Execução

### Ler

- `vostok/work/specs/refined/SPEC-022-baseline-sem-rodar-evidencias-que-so-podem-falhar-antes-da-tarefa.md` (RF-09, CB-06)
- `apps/cli/src/commands/finalize.rs` (fn before_html, fn evidence_html)
- `apps/cli/tests/finalize.rs` (testes evidence_before_after_*)

### Mudar

- `apps/cli/src/commands/finalize.rs` (alterar): "antes" de evidência não executada ou interrompida
- `apps/cli/tests/finalize.rs` (alterar): testes `before_skipped_*`

### Passos

1. Escreva primeiro os testes `before_skipped_*`: baseline de evidência com `não executada: novo.txt ainda não existe` e outro terminando em `interrompida após 120 s`; o `evidencias.html` mostra essas linhas no bloco "Antes da tarefa" (CT-10).
2. Em `before_html`, mantenha o que já mostra a saída de antes sem as duas linhas de cabeçalho; confira que as linhas de não executada e de interrupção aparecem escapadas. Se já aparecerem sem mudança de código, os testes ficam como cobertura e o código não muda.
3. Rode `cargo fmt` e `cargo clippy --manifest-path apps/cli/Cargo.toml -- -D warnings`.

### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test finalize 2>&1 | grep -E '^test ' | sort`

### Verificação

```bash
# evidência: before-skipped-tests.txt
cargo test --manifest-path apps/cli/Cargo.toml --test finalize -- before_skipped_
cargo test --manifest-path apps/cli/Cargo.toml --test finalize
cargo clippy --manifest-path apps/cli/Cargo.toml -- -D warnings
```

Resultado esperado: os testes de finalize passam e o clippy termina sem avisos.

### Evidências

- `before-skipped-tests.txt` (automática, da verificação)
  - Prova: o documento mostra o motivo no "antes" de evidência não executada ou interrompida (RF-09, CT-10).
  - Trecho: as linhas `test ... ok` dos testes novos.
  - Falha: `FAILED` ou ausência dos testes `before_skipped_`.
  - Espera: `before_skipped_`

### Concluída quando

- [x] O `evidencias.html` mostra no "antes" a linha de não executada ou interrompida (RF-09, CT-10).

  > Como comprovar: em `before-skipped-tests.txt`, ver aprovados os dois testes.

## Descrição

### Resumo executivo

O documento de evidências passa a dizer, no "antes", por que uma evidência não rodou antes da tarefa. Perfil padrão, risco baixo; roda em paralelo na primeira onda.

### Detalhes técnicos

Abordagem: a linha gravada no baseline já é a explicação; o `before_html` só precisa mostrá-la, o que os testes confirmam. Sem arquivos em comum com TASK-01 e TASK-05. Referências: RF-09, CT-10, CB-06, inferência F.

### Card

- **Título:** Antes de evidência não executada
- **Tipo:** tarefa
- **Descrição:** Motivo no "antes" de evidência não executada ou interrompida.
- **Critérios de aceite:**
  - [ ] Antes mostra o motivo
- **Rótulos:** SPEC-022, padrao
- **Estimativa:** padrao
- **Bloqueada por:** -

## Registro de execução

| # | Data | Etapa | Resultado | Detalhe |
|---|---|---|---|---|
| 1 | 2026-10-31 | início | in_progress | aprovada em 2026-10-31T14:09:07Z; modelo: opus |
| 2 | 2026-10-31 | baseline | coletado | 1 comando(s); códigos de saída: 0 |
| 3 | 2026-10-31 | testes | passed | Verificação sem falhas; saída em validation/tests.out |
| 4 | 2026-10-31 | regressão | passed | 0 comando(s) iguais ao baseline; 1 com acréscimo |
| 5 | 2026-10-31 | evidências | passed | 1 evidência(s) em evidence/TASK-04/ |
| 6 | 2026-10-31 | conclusão | done | testes, regressão e evidências passed; critérios de conclusão marcados |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
