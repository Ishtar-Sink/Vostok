---
id: TASK-05
spec: SPEC-023
title: "Tarefas criadas em pastas com README gerado"
profile: padrao
persona: desenvolvedor
depends_on: [TASK-03]
wave: 4
covers: [RF-01, RF-04, CT-01, CT-04]
type: tarefa
labels: [SPEC-023, padrao]
status: done
content_hash: fadcff081d8bdb98a332e2b4dd8c90dbb8dfb59361cf5378bcc1a86a63806309
adjustments: 1
adjusted_at: 2026-11-01T13:06:57Z
approved_at: 2026-11-01T12:01:26Z
approved_hash: 704bb93feeb88ff2e9e7c9bfea652cb48fa6087ad4f6faa0d409060228d9d440
attempts: 0
tests_result: passed
regression_result: passed
evidence_result: passed
model: sonnet
updated_at: 2026-11-01T13:06:57Z
---

# TASK-05: Tarefas criadas em pastas com README gerado

## Execução

### Ler

- `vostok/work/specs/refined/SPEC-023-tarefa-como-pasta-com-arquivos-dedicados.md` (RF-01, RF-04, "Contratos e interfaces")
- `packages/harness/templates/task/task.md` (seções e variáveis atuais)
- `apps/cli/src/commands/plan.rs` (fn create_tasks, fn approve_validate, fn task_profile)
- `apps/cli/src/commands/task.rs` (struct TaskDoc, fn set)
- `apps/cli/tests/plan.rs` (fn plan_with_evidence, fn spec_and_plan)

### Mudar

- `apps/cli/src/commands/plan.rs` (alterar): `create-tasks` em pastas e validação do plano com os arquivos
- `apps/cli/tests/plan.rs` (alterar): testes `create_folder_*`
- `apps/cli/src/commands/task.rs` (alterar): geração do `README.md` a cada gravação de estado
- `packages/harness/templates/task/execucao.md` (criar): Ler, Mudar e Passos
- `packages/harness/templates/task/verificacao.md` (criar): Baseline, Verificação e Evidências
- `packages/harness/templates/task/criterios.md` (criar): Concluída quando
- `packages/harness/templates/task/executivo.md` (criar): executivo da tarefa
- `packages/harness/templates/task/tecnico.md` (criar): técnico da tarefa
- `packages/harness/templates/task/readme.md` (criar): README gerado
- `packages/harness/templates/task/estado.yml` (criar): estado inicial da tarefa em pasta
- `packages/harness/templates/task/registro.md` (criar): tabelas vazias de registro e histórico
- `packages/harness/config/tasks.yml` (alterar): chave `task_folder_template_dir`
- `apps/cli/tests/fixtures/golden/plan` (alterar): casos do `create-tasks` com as pastas
- `apps/cli/tests/fixtures/golden/blobs` (alterar): conteúdos novos dos casos atualizados

### Passos

1. `packages/harness/templates/task/` (criar os seis): divida o `task.md` atual nos arquivos do contrato, com as mesmas variáveis de hoje (leitura, mudanças, passos e as demais). `executivo.md` e `tecnico.md` têm uma variável cada (`TASK_EXECUTIVE` e `TASK_TECHNICAL`, no formato de variável dos templates), com comentário dizendo que são escritos na finalização da tarefa. `readme.md` tem título, status, onda, dependências, links para os arquivos e o card (título, tipo, descrição do plano, critérios de `criterios.md`, rótulos, estimativa e bloqueios). Confira: `cargo build`.
2. `apps/cli/src/commands/plan.rs`: `vostok plan approve create-tasks` cria `TASK-NN/` com os arquivos, o `estado.yml` (campos do front-matter de hoje) e o `registro.md` vazio. O `plan approve validate` e o `mark-planned` leem as pastas pela mesma regra da `TaskDoc`. Confira: `cargo test --test plan -- create_folder_`.
3. `apps/cli/src/commands/task.rs`: a cada gravação de estado no formato de pasta, refaça o `README.md` pelo template `readme.md` (RF-04). O card vem do plano gravado no `estado.yml` na criação (descrição) e do `criterios.md`.
4. `apps/cli/tests/plan.rs` (alterar): testes `create_folder_*`: o `create-tasks` cria as pastas com os sete arquivos e o `README.md` (CT-01); depois de `vostok task approve`, o `README.md` mostra o status novo e os critérios do `criterios.md` (CT-04). Confira: `cargo test --test plan -- create_folder_`.
5. Os goldens do grupo `plan` que criam tarefas mudam de saída e de arquivos: atualize só os casos cuja diferença vem das pastas novas. Outra diferença: **PARE** com o template `blocker`.

### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test plan --test finalize 2>&1 | grep -E '^test ' | sort`
- `cargo test --manifest-path apps/cli/Cargo.toml --test scenarios 2>&1 | grep -E '^test ' | sort`

### Verificação

```bash
# evidência: create-folder-tests.txt
cargo test --manifest-path apps/cli/Cargo.toml --test plan -- create_folder_
cargo test --manifest-path apps/cli/Cargo.toml
cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings
```

Resultado esperado: toda a suíte passa, inclusive os goldens e os cenários, e o clippy termina sem avisos.

### Evidências

- `create-folder-tests.txt` (automática, da verificação)
  - Prova: tarefas criadas em pastas pelos templates novos e README com o card refeito a cada mudança (RF-01, RF-04, CT-01, CT-04).
  - Trecho: as linhas `test ... ok` dos testes novos.
  - Falha: `FAILED` ou ausência dos testes `create_folder_`.
  - Espera: `create_folder_`

### Concluída quando

- [x] O `create-tasks` cria as tarefas como pastas com os arquivos do contrato (RF-01, CT-01).

  > Como comprovar: em `create-folder-tests.txt`, ver aprovado o teste da criação.

- [x] O `README.md` é refeito com o status e os critérios (RF-04, CT-04).

  > Como comprovar: em `create-folder-tests.txt`, ver aprovado o teste do README.

## Descrição

### Resumo executivo

Faz o planejamento criar as tarefas já no formato novo, com uma página inicial (README) gerada pelo harness, que mostra o status e o card sem ninguém precisar escrevê-lo.

### Detalhes técnicos

Abordagem: templates por arquivo, criação pela `plan.rs` e README refeito na gravação de estado da `TaskDoc`. O card deixa de ser escrito pelo agente (E1 do estudo). Os goldens do grupo `plan` podem mudar e entram na verificação. Divide o `task.rs` com a TASK-04 na onda 4. Referências: RF-01, RF-04, estudo E1.

### Card

- **Título:** Tarefas criadas em pastas com README gerado
- **Tipo:** tarefa
- **Descrição:** create-tasks cria pastas pelos templates novos e o vostok gera o README com o card.
- **Critérios de aceite:**
  - [ ] Pastas criadas pelos templates
  - [ ] README gerado a cada mudança
- **Rótulos:** SPEC-023, padrao
- **Estimativa:** padrao
- **Bloqueada por:** TASK-03

## Registro de execução

| # | Data | Etapa | Resultado | Detalhe |
|---|---|---|---|---|
| 1 | 2026-11-01 | início | in_progress | aprovada em 2026-11-01T12:01:26Z; modelo: sonnet |
| 2 | 2026-11-01 | baseline | coletado | 2 comando(s); códigos de saída: 0, 0 |
| 3 | 2026-11-01 | ajuste | in_progress | templates próprios de estado.yml e registro.md e a chave task_folder_template_dir entram em Mudar; o task.md volta ao original (decisão do usuário) |
| 4 | 2026-11-01 | testes | passed | Verificação sem falhas; saída em validation/tests.out |
| 5 | 2026-11-01 | regressão | passed | 1 comando(s) iguais ao baseline; 1 com acréscimo |
| 6 | 2026-11-01 | evidências | passed | 1 evidência(s) em evidence/TASK-05/ |
| 7 | 2026-11-01 | conclusão | done | testes, regressão e evidências passed; critérios de conclusão marcados |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
| 1 | 2026-11-01 | agente | templates próprios de estado.yml e registro.md e a chave task_folder_template_dir entram em Mudar; o task.md volta ao original (decisão do usuário) |
