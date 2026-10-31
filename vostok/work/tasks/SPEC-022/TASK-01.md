---
id: TASK-01
spec: SPEC-022
title: "Progresso no início da tarefa"
profile: padrao
persona: desenvolvedor
depends_on: []
wave: 1
covers: [RF-07, CT-08]
type: tarefa
labels: [SPEC-022, padrao]
status: done
content_hash: b2995eb10c36f43ac35ac4de2d1f8f195d9489ea0d08690620eb88fe818adec5
adjustments: 0
adjusted_at: null
approved_at: 2026-10-31T14:09:07Z
approved_hash: 37ffa0c683c232bdd375494b5c33207d9b4ae76d26ca8a106627d4e55112a6b1
attempts: 0
tests_result: passed
regression_result: passed
evidence_result: passed
model: opus
---

# TASK-01: Progresso no início da tarefa

## Execução

### Ler

- `vostok/work/specs/refined/SPEC-022-baseline-sem-rodar-evidencias-que-so-podem-falhar-antes-da-tarefa.md` (RF-07, CT-08, FA-02, "Contratos e interfaces")
- `apps/cli/src/commands/task.rs` (fn start, fn collect_baseline, fn baseline)
- `apps/cli/tests/task.rs` (testes start_baseline_*, em especial start_baseline_failure_keeps_the_task_approved)

### Mudar

- `apps/cli/src/commands/task.rs` (alterar): `in_progress` e `início` antes do baseline e comandos impressos
- `apps/cli/tests/task.rs` (alterar): testes `start_progress_*` e o teste da falha na coleta
- `apps/cli/tests/fixtures/golden/task` (alterar): casos de `start` cuja saída muda
- `apps/cli/tests/fixtures/golden/blobs` (alterar): conteúdos novos dos casos atualizados

### Passos

1. Escreva primeiro os testes `start_progress_*`: a saída do `vostok task start` traz `Coletando o baseline:` e uma linha `- [N] <comando>` por comando de Baseline e `- [evidência <arquivo>] <comando>` por evidência, antes da linha final (CT-08); depois de uma coleta que falha, a tarefa fica `in_progress` com a linha `início` e sem a linha `baseline` (FA-02).
2. O teste `start_baseline_failure_keeps_the_task_approved` descreve o comportamento antigo: troque-o por `start_progress_failure_leaves_the_task_running`, com a expectativa nova.
3. Em `start`: depois das conferências, marque `in_progress`, grave o modelo e a linha `início`, imprima `Tarefa {task} iniciada: status '{in_progress}'. Coletando o baseline:` e só então chame `collect_baseline`, que imprime cada comando ao começá-lo. No fim, grave a linha `baseline` e imprima `Baseline de {task} coletado em …: {detalhe}`.
4. Em `collect_baseline`, imprima cada comando ao começá-lo, sem mudar o que é gravado.
5. Rode o golden da suíte `task`; atualize só os casos que diferem pela saída nova do `start` ou pela ordem do registro. Outra diferença: **PARE** com o template `blocker`.
6. Rode `cargo fmt` e `cargo clippy --manifest-path apps/cli/Cargo.toml -- -D warnings`.

### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test task -- start_ 2>&1 | grep -E '^test ' | grep -v 'start_baseline_failure_keeps_the_task_approved' | sort`

### Verificação

```bash
# evidência: start-progress-tests.txt
cargo test --manifest-path apps/cli/Cargo.toml --test task -- start_progress_
cargo test --manifest-path apps/cli/Cargo.toml
cargo clippy --manifest-path apps/cli/Cargo.toml -- -D warnings
```

Resultado esperado: toda a suíte passa, inclusive os goldens, e o clippy termina sem avisos.

### Evidências

- `start-progress-tests.txt` (automática, da verificação)
  - Prova: o início mostra a tarefa em execução e cada comando enquanto coleta (RF-07, CT-08).
  - Trecho: as linhas `test ... ok` dos testes novos.
  - Falha: `FAILED` ou ausência dos testes `start_progress_`.
  - Espera: `start_progress_`

### Concluída quando

- [x] O `vostok task start` marca `in_progress` antes do baseline e imprime cada comando (RF-07, CT-08).

  > Como comprovar: em `start-progress-tests.txt`, ver aprovados os testes da saída e da falha na coleta.

- [x] A suíte inteira passa.

  > Como comprovar: a Verificação termina com código 0.

## Descrição

### Resumo executivo

Faz o início de uma tarefa mostrar o que está acontecendo: a tarefa aparece em execução na hora, e cada comando do levantamento inicial é mostrado ao começar. Um início demorado deixa de parecer travado. Perfil padrão, risco baixo.

### Detalhes técnicos

Abordagem: inverter a ordem em `start` (status e `início` antes da coleta) e imprimir em `collect_baseline`. Risco: se a coleta falha, a tarefa fica `in_progress` sem baseline; o `vostok task baseline` coleta de novo e o `tests` já recusa tarefa sem baseline. A saída do `start` muda: goldens com `start` podem precisar de atualização (casos que diferem só pela saída nova). Componentes compartilhados com TASK-02 e TASK-03 (cadeia) e com a SPEC-021 (executar depois dela). Referências: RF-07, CT-08, FA-02.

### Card

- **Título:** Progresso no início da tarefa
- **Tipo:** tarefa
- **Descrição:** Progresso visível no início da tarefa.
- **Critérios de aceite:**
  - [ ] in_progress antes do baseline
  - [ ] Cada comando impresso
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
| 5 | 2026-10-31 | evidências | passed | 1 evidência(s) em evidence/TASK-01/ |
| 6 | 2026-10-31 | conclusão | done | testes, regressão e evidências passed; critérios de conclusão marcados |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
