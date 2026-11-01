---
id: TASK-10
spec: SPEC-023
title: "Equivalência dos cenários no formato de pasta"
profile: padrao
persona: qa
depends_on: [TASK-06, TASK-07, TASK-08]
wave: 6
covers: [CT-17]
type: tarefa
labels: [SPEC-023, padrao]
status: done
content_hash: 4ab480507f4a3542f9ae1872c888f1388b50891b1cdad321b6d9e861577052c8
adjustments: 1
adjusted_at: 2026-11-01T13:46:16Z
approved_at: 2026-11-01T12:01:26Z
approved_hash: 06e5f7f69cfa18108621b98216e5082fbf86d6eec6328ea0eba76c441e6d0ab6
attempts: 0
tests_result: passed
regression_result: passed
evidence_result: passed
model: claude-opus-5-5
updated_at: 2026-11-01T13:46:16Z
---

# TASK-10: Equivalência dos cenários no formato de pasta

## Execução

### Ler

- `vostok/work/specs/refined/SPEC-023-tarefa-como-pasta-com-arquivos-dedicados.md` (RNF-04, CT-17, RF-07, RF-04)
- `apps/cli/tests/scenarios.rs` (fn write_task, fn state_of, fn record_rows, fn evidence_root e a normalização, da TASK-01)
- `apps/cli/tests/fixtures/scenarios` (transcrições da TASK-01)
- `apps/cli/src/commands/task.rs` (subcomando migrate, da TASK-07)

### Mudar

- `apps/cli/tests/scenarios.rs` (alterar): os cenários rodam também no formato de pasta e numa spec migrada
- `apps/cli/src/commands/task.rs` (alterar): correções achadas pela equivalência: a migração mantém sem hash a tarefa que nunca teve hash, e o registro de evidências cita a pasta certa no formato de pasta

### Passos

1. `apps/cli/tests/scenarios.rs`: acrescente o formato às funções de acesso: `write_task` grava a tarefa em pasta a partir do mesmo conteúdo (dividindo as seções nos arquivos do contrato e o front-matter em `estado.yml`); `state_of` lê `estado.yml`; `record_rows` lê `registro.md`; `evidence_root` aponta para `TASK-NN/evidencias`. Confira: `cargo test --test scenarios`.
2. `apps/cli/tests/scenarios.rs`: cada cenário roda em três variantes: atual (como hoje), pasta (escrita direto em pasta) e migrada (escrita no formato atual e convertida com `vostok task migrate SPEC-001` antes do primeiro comando; no `scenario_regression_block_and_accept`, a conversão acontece depois do `start`). O comando de migração não entra na transcrição.
3. `apps/cli/tests/scenarios.rs`: a normalização trata `vostok/work/tasks/SPEC-001/TASK-01/` como `<TAREFA>` e `TASK-01/evidencias` como `<EVIDENCIAS>`. As únicas outras diferenças aceitas são as que a spec pede, cada uma numa regra com o RF: a mensagem do `record-manual` citando o arquivo (RF-07), os arquivos gerados que não são evidência (RF-04), a linha da migração no registro (RF-12) e o executivo e o técnico no `finalize sources` (RF-13). Confira: as três variantes comparam com a mesma fixture.
4. Não atualize nenhuma fixture de `apps/cli/tests/fixtures/scenarios`: elas são o "antes". Uma diferença fora das regras do passo 3 é mudança de comportamento: **PARE** com o template `blocker`, citando o cenário e as linhas.

### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test scenarios 2>&1 | grep -E '^test ' | sort`

### Verificação

```bash
# evidência: equivalence-tests.txt
cargo test --manifest-path apps/cli/Cargo.toml --test scenarios
cargo test --manifest-path apps/cli/Cargo.toml
cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings
```

Resultado esperado: toda a suíte passa, inclusive os goldens e os cenários, e o clippy termina sem avisos.

### Evidências

- `equivalence-tests.txt` (automática, da verificação)
  - Prova: os dez cenários, felizes e de falha, dão a mesma transcrição no formato atual, no de pasta e numa spec migrada (RNF-04, CT-17).
  - Trecho: as linhas `test ... ok` dos testes novos.
  - Falha: `FAILED` em algum `scenario_`.
  - Espera: `scenario_`

### Concluída quando

- [x] Os cenários dão o mesmo resultado no formato de pasta e numa spec migrada (CT-17).

  > Como comprovar: em `equivalence-tests.txt`, ver os dez `scenario_` aprovados com as três variantes.

- [x] As fixtures da TASK-01 não mudaram.

  > Como comprovar: `git log` de `apps/cli/tests/fixtures/scenarios` mostra só o commit da TASK-01.

## Descrição

### Resumo executivo

Fecha a reestruturação comprovando que o harness se comporta igual no formato novo e numa spec migrada, inclusive nos casos de erro, comparando com a gravação feita antes de qualquer mudança.

### Detalhes técnicos

Abordagem: as variantes trocam só as funções de acesso ao formato; as diferenças aceitas são só as previstas na spec, uma regra por RF. As fixtures não podem mudar (o "antes" é a régua). Persona `qa`. Referências: RNF-04, CT-17, pedido do usuário no planejamento.

### Card

- **Título:** Equivalência dos cenários no formato de pasta
- **Tipo:** tarefa
- **Descrição:** Os cenários de referência dão o mesmo resultado no formato de pasta e numa spec migrada.
- **Critérios de aceite:**
  - [ ] Formato de pasta igual às fixtures
  - [ ] Spec migrada igual às fixtures
  - [ ] Diferenças só as previstas na spec
- **Rótulos:** SPEC-023, padrao
- **Estimativa:** padrao
- **Bloqueada por:** TASK-06, TASK-07, TASK-08

## Registro de execução

| # | Data | Etapa | Resultado | Detalhe |
|---|---|---|---|---|
| 1 | 2026-11-01 | início | in_progress | aprovada em 2026-11-01T12:01:26Z; modelo: claude-opus-5-5 |
| 2 | 2026-11-01 | baseline | coletado | 1 comando(s); códigos de saída: 0 |
| 3 | 2026-11-01 | ajuste | in_progress | a equivalência aceita a linha da migração (RF-12) e o executivo e o técnico no sources (RF-13), e task.rs entra em Mudar com duas correções achadas pelo teste (decisão do usuário) |
| 4 | 2026-11-01 | testes | passed | Verificação sem falhas; saída em validation/tests.out |
| 5 | 2026-11-01 | regressão | passed | 1 comando(s) iguais ao baseline |
| 6 | 2026-11-01 | evidências | passed | 1 evidência(s) em evidence/TASK-10/ |
| 7 | 2026-11-01 | conclusão | done | testes, regressão e evidências passed; critérios de conclusão marcados |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
| 1 | 2026-11-01 | agente | a equivalência aceita a linha da migração (RF-12) e o executivo e o técnico no sources (RF-13), e task.rs entra em Mudar com duas correções achadas pelo teste (decisão do usuário) |
