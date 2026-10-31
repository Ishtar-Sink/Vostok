---
id: TASK-03
spec: SPEC-022
title: "Tempo-limite das evidências no baseline"
profile: julgamento
persona: desenvolvedor
depends_on: [TASK-02]
wave: 3
covers: [RF-04, RF-05, RF-06, CT-05, CT-06, CT-07, CT-11, CB-03, CB-04, CB-05, CB-06, CB-07]
type: tarefa
labels: [SPEC-022, julgamento]
status: done
content_hash: dad5b159dfa00894992928ab9cc088d85198be3de698da0a9011b5d0c92aeda3
adjustments: 0
adjusted_at: null
approved_at: 2026-10-31T14:09:07Z
approved_hash: d9d553bc16a184cae2cccf179ea41596d7909d14d2b931b5a4315c838fa7732e
attempts: 0
tests_result: passed
regression_result: passed
evidence_result: passed
model: opus
---

# TASK-03: Tempo-limite das evidências no baseline

## Execução

### Ler

- `vostok/work/specs/refined/SPEC-022-baseline-sem-rodar-evidencias-que-so-podem-falhar-antes-da-tarefa.md` (RF-04 a RF-06, RN-01, RN-03, CB-03 a CB-07, RNF-01, RNF-02)
- `apps/cli/src/commands/task.rs` (fn collect_baseline, fn recorded_run, fn shell_output, fn run_approve, fn evidence_rule_errors)
- `apps/cli/src/commands/plan.rs` (fn evidence_errors)
- `apps/cli/tests/task.rs` (testes requires_* da TASK-02)
- `packages/harness/config/tasks.yml` (evidence_baseline_dir)

### Mudar

- `apps/cli/src/commands/task.rs` (alterar): tempo-limite das evidências no baseline, `Tempo-limite:` e a lista das exceções na aprovação
- `apps/cli/tests/task.rs` (alterar): testes `evidence_timeout_*`
- `apps/cli/src/commands/plan.rs` (alterar): `Tempo-limite:` aceito e validado no plano
- `apps/cli/tests/plan.rs` (alterar): teste `evidence_timeout_*` do plano
- `packages/harness/config/tasks.yml` (alterar): chave `evidence_baseline_timeout`

### Passos

1. Em `packages/harness/config/tasks.yml`, acrescente `evidence_baseline_timeout: 120` com um comentário: vale só para as evidências no baseline; uma evidência pode ter `Tempo-limite:`, aprovado pelo usuário com a tarefa.
2. Escreva primeiro os testes `evidence_timeout_*`: com `evidence_baseline_timeout: 1`, evidência `sleep 5; echo fim` é interrompida em menos de 4 s e a gravação termina com `interrompida após 1 s` (CT-05); `Tempo-limite: 0` numa evidência `sleep 2` roda inteira (CT-06, CB-03); `Tempo-limite: abc` é recusado (CB-04); `vostok task approve validate` lista `- TASK-01: tela.txt com tempo-limite de 900 s (padrão 120 s)` para uma exceção acima do padrão e não lista uma abaixo (CT-07, CB-07); comando que abre um filho (`sh -c 'sleep 30 & wait'`) é encerrado com o filho (CB-05); tarefa sem os subitens coleta como antes (CT-11). No plano, `Tempo-limite:` numérico é aceito e o não numérico é recusado.
3. Decisão de desenho: rode a evidência num grupo de processos próprio (`std::os::unix::process::CommandExt::process_group(0)` em Unix) e, ao estourar o tempo, encerre o grupo com `kill -TERM -<pgid>` e depois `-KILL`; leia a saída numa thread para não travar o pipe. Alternativa descartada: crate de tempo-limite (RNF-02). Em Windows, encerre só o processo.
4. A saída gravada de uma interrupção é a produzida até ali, seguida de `interrompida após <N> s`, com `exit: 124`. No `vostok task evidence`, uma evidência interrompida no baseline não entra na comparação de "saída igual".
5. Leia `Tempo-limite:` como o `Requer:` e aceite-o como campo opcional (tarefa e plano); recuse valor não numérico.
6. Em `run_approve` (validate e approve), depois da lista de tarefas, imprima as evidências com `Tempo-limite:` acima de `evidence_baseline_timeout` (ou 0), uma por linha.
7. Rode `cargo fmt` e `cargo clippy --manifest-path apps/cli/Cargo.toml -- -D warnings`.

### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test task -- evidence_baseline_ requires_ approve 2>&1 | grep -E '^test ' | sort`

### Verificação

```bash
# evidência: evidence-timeout-tests.txt
cargo test --manifest-path apps/cli/Cargo.toml --test task --test plan -- evidence_timeout_
cargo test --manifest-path apps/cli/Cargo.toml
cargo clippy --manifest-path apps/cli/Cargo.toml -- -D warnings
```

Resultado esperado: toda a suíte passa e o clippy termina sem avisos.

### Evidências

- `evidence-timeout-tests.txt` (automática, da verificação)
  - Prova: tempo-limite padrão, exceção por evidência, lista das exceções na aprovação e encerramento dos filhos (RF-04 a RF-06, CT-05 a CT-07, CT-11, CB-03 a CB-07).
  - Trecho: as linhas `test ... ok` dos testes novos.
  - Falha: `FAILED` ou ausência dos testes `evidence_timeout_`.
  - Espera: `evidence_timeout_`

### Concluída quando

- [x] Uma evidência que passa do tempo no baseline é interrompida e registrada (RF-04, CT-05, CB-05).

  > Como comprovar: em `evidence-timeout-tests.txt`, ver aprovados os testes do tempo-limite e do processo filho.

- [x] `Tempo-limite:` substitui o padrão e aparece na aprovação (RF-05, RF-06, CT-06, CT-07).

  > Como comprovar: em `evidence-timeout-tests.txt`, ver aprovados os testes da exceção e da lista.

- [x] Tarefas sem os subitens coletam como antes (RNF-01, CT-11).

  > Como comprovar: em `evidence-timeout-tests.txt`, ver aprovado o teste da tarefa antiga.

## Descrição

### Resumo executivo

Coloca um limite de tempo nas evidências rodadas antes da tarefa, com exceção por evidência que o usuário aprova junto com a tarefa. Protege contra o início de tarefa que demora dezenas de minutos. Perfil julgamento: encerrar comandos com processos filhos exige cuidado.

### Detalhes técnicos

Abordagem: grupo de processos próprio e encerramento do grupo; leitura da saída numa thread. A exceção é escrita no planejamento e listada na aprovação (RN-01, resposta do usuário). O tempo-limite não vale na finalização nem nos comandos de Baseline (RN-03, inferência E). Componentes compartilhados com TASK-01 e TASK-02 (cadeia). Referências: RF-04 a RF-06, CB-03 a CB-07, decisões 2 e 4.

### Card

- **Título:** Tempo-limite das evidências no baseline
- **Tipo:** tarefa
- **Descrição:** Tempo-limite das evidências no baseline, com exceção aprovada pelo usuário.
- **Critérios de aceite:**
  - [ ] Interrompe e registra
  - [ ] Exceção por evidência listada na aprovação
  - [ ] Tarefas antigas como antes
- **Rótulos:** SPEC-022, julgamento
- **Estimativa:** julgamento
- **Bloqueada por:** TASK-02

## Registro de execução

| # | Data | Etapa | Resultado | Detalhe |
|---|---|---|---|---|
| 1 | 2026-10-31 | início | in_progress | aprovada em 2026-10-31T14:09:07Z; modelo: opus |
| 2 | 2026-10-31 | baseline | coletado | 1 comando(s); códigos de saída: 0 |
| 3 | 2026-10-31 | testes | passed | Verificação sem falhas; saída em validation/tests.out |
| 4 | 2026-10-31 | regressão | passed | 1 comando(s) iguais ao baseline |
| 5 | 2026-10-31 | evidências | passed | 1 evidência(s) em evidence/TASK-03/ |
| 6 | 2026-10-31 | conclusão | done | testes, regressão e evidências passed; critérios de conclusão marcados |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
