---
id: TASK-01
spec: SPEC-021
title: "Acréscimo e alerta na regressão"
profile: julgamento
persona: desenvolvedor
depends_on: []
wave: 1
covers: [RF-01, RF-02, RF-03, RF-04, CT-01, CT-02, CT-03, CT-04, CT-05, CT-10, CB-01, CB-02, CB-03, CB-04]
type: tarefa
labels: [SPEC-021, julgamento]
status: done
content_hash: af870cd7bb72b7d5ccb60d28fae24c7728c2367fa458ed213ca0bc6da909004a
adjustments: 0
adjusted_at: null
approved_at: 2026-10-31T14:09:07Z
approved_hash: 41a7ec3d98ca64d64ae366df59b34e0dc9e3849cfd9de961f56794694f7a52a3
attempts: 0
tests_result: passed
regression_result: passed
evidence_result: passed
model: opus
---

# TASK-01: Acréscimo e alerta na regressão

## Execução

### Ler

- `vostok/work/specs/refined/SPEC-021-regressao-que-nao-e-regressao.md` (RF-01 a RF-04, RN-01, RN-02, RN-04, CB-01 a CB-04)
- `docs/estudos/regressao-falsa.md` (seção "O que é uma regressão")
- `apps/cli/src/commands/task.rs` (fn regression, fn without_ignored, fn recorded_run, fn regression_ignore, fn latest_acceptances)
- `apps/cli/tests/task.rs` (testes regression_*, accepted_difference_lets_the_task_finish, fn running_task)
- `packages/harness/config/tasks.yml` (regression_ignore)

### Mudar

- `apps/cli/src/commands/task.rs` (alterar): critério de acréscimo, linhas de alerta e contagem no registro
- `apps/cli/tests/task.rs` (alterar): testes `growth_*`
- `packages/harness/config/tasks.yml` (alterar): chave `regression_alert`

### Passos

1. Em `packages/harness/config/tasks.yml`, logo depois de `regression_ignore`, acrescente `regression_alert` com `FAILED`, `error`, `warning` e `panicked`, com um comentário no estilo do vizinho.
2. Escreva primeiro os testes `growth_*` em `apps/cli/tests/task.rs` (o prefixo não pode conter `regression_` nem `delivery_`, que o baseline filtra): lista que ganha duas linhas passa com "1 com acréscimo" (CT-01, CB-01); linha removida bloqueia (CT-02); linha nova com `FAILED` bloqueia citando a linha (CT-03, CB-02); código de saída diferente bloqueia (CT-04, CB-03); `regression_alert` trocado para `ERRO` bloqueia linha nova com `erro` (CT-05); saída reordenada bloqueia (CB-04); baseline gravado antes da mudança (formato atual) passa como acréscimo (CT-10).
3. Decisões já tomadas, para implementar em `regression`: a gravação tem `$ comando` e `exit: N` nas duas primeiras linhas. Depois de aplicar `without_ignored` dos dois lados, é acréscimo quando as duas primeiras linhas são iguais e as linhas do baseline aparecem na saída atual como subsequência, na mesma ordem. Linhas novas são as que sobram. Uma linha nova que contém (sem diferenciar maiúsculas) um texto de `regression_alert` faz o comando bloquear, com a linha citada na saída.
4. A ordem das regras no laço: entrega (ignorado), igual, aceito pelo usuário, acréscimo, bloqueio. Comandos que medem a entrega continuam ignorados antes de tudo (RN-04).
5. O detalhe do registro ganha `; N com acréscimo` e a saída lista `- [N] comando: acréscimo de K linha(s)` para cada um (RF-04). Sem a chave `regression_alert`, valem os quatro textos padrão.
6. Rode `cargo fmt` e `cargo clippy --manifest-path apps/cli/Cargo.toml -- -D warnings`.

### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test task -- regression_ delivery_ accept 2>&1 | grep -E '^test ' | sort`

### Verificação

```bash
# evidência: growth-tests.txt
cargo test --manifest-path apps/cli/Cargo.toml --test task -- growth_
cargo test --manifest-path apps/cli/Cargo.toml --test task
cargo clippy --manifest-path apps/cli/Cargo.toml -- -D warnings
```

Resultado esperado: todos os testes passam e o clippy termina sem avisos.

### Evidências

- `growth-tests.txt` (automática, da verificação)
  - Prova: acréscimo não bloqueia; perda, código diferente, alerta e reordenação bloqueiam (RF-01 a RF-04, CT-01 a CT-05, CT-10).
  - Trecho: as linhas `test ... ok` dos testes novos.
  - Falha: `FAILED` ou ausência dos testes `growth_`.
  - Espera: `growth_`

### Concluída quando

- [x] Uma lista do Baseline que ganha linhas, com o mesmo código de saída, passa como acréscimo (RF-01, RF-04, CT-01, CT-10).

  > Como comprovar: em `growth-tests.txt`, ver aprovados os testes do acréscimo e do baseline antigo.

- [x] Linha removida, código diferente, saída reordenada e linha nova de alerta bloqueiam (RF-01, RF-02, CT-02 a CT-04).

  > Como comprovar: em `growth-tests.txt`, ver aprovado um teste para cada caso.

- [x] `regression_alert` é configurável (RF-03, CT-05).

  > Como comprovar: em `growth-tests.txt`, ver aprovado o teste com `ERRO`.

## Descrição

### Resumo executivo

Faz a conferência de regressão parar de travar a tarefa quando ela só acrescenta itens a uma lista, como testes novos, que foi a causa de três dos oito bloqueios falsos. Linhas de erro novas continuam bloqueando. Perfil julgamento: é o critério central da regressão.

### Detalhes técnicos

Abordagem: subsequência das linhas do baseline na saída atual, com as duas linhas de cabeçalho (`$ comando`, `exit:`) iguais, depois do `regression_ignore`; linhas novas conferidas contra `regression_alert`. Alternativas descartadas: comparar por teste (depende do executor) e aceitar qualquer acréscimo (perde alertas, resposta 1A). Componentes: `task.rs`, `tests/task.rs`, `tasks.yml`, compartilhados com TASK-02 e TASK-03 (cadeia). A SPEC-022 também muda esses arquivos: executar depois desta spec. Referências: RF-01 a RF-04, RN-01, RN-02, RN-04, inferências C e D.

### Card

- **Título:** Acréscimo e alerta na regressão
- **Tipo:** tarefa
- **Descrição:** Acréscimo de linhas não bloqueia a regressão; perda e alertas continuam bloqueando.
- **Critérios de aceite:**
  - [ ] Acréscimo passa
  - [ ] Perda, código diferente e alerta bloqueiam
  - [ ] regression_alert configurável
- **Rótulos:** SPEC-021, julgamento
- **Estimativa:** julgamento
- **Bloqueada por:** -

## Registro de execução

| # | Data | Etapa | Resultado | Detalhe |
|---|---|---|---|---|
| 1 | 2026-10-31 | início | in_progress | aprovada em 2026-10-31T14:09:07Z; modelo: opus |
| 2 | 2026-10-31 | baseline | coletado | 1 comando(s); códigos de saída: 0 |
| 3 | 2026-10-31 | testes | passed | Verificação sem falhas; saída em validation/tests.out |
| 4 | 2026-10-31 | regressão | passed | 1 comando(s) iguais ao baseline |
| 5 | 2026-10-31 | evidências | passed | 1 evidência(s) em evidence/TASK-01/ |
| 6 | 2026-10-31 | conclusão | done | testes, regressão e evidências passed; critérios de conclusão marcados |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
