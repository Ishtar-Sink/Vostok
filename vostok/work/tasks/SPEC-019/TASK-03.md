---
id: TASK-03
spec: SPEC-019
title: "Skills com Coordenador por padrão e briefing"
profile: padrao
persona: desenvolvedor
depends_on: [TASK-01]
wave: 2
covers: [RF-01, RF-05, RF-06, RF-09, RF-10, CT-09, CB-03]
type: tarefa
labels: [SPEC-019, padrao]
status: done
content_hash: f6a995edc32d537c7b304212823b0b85874728715803ee9c31f05454fbc40148
adjustments: 1
adjusted_at: 2026-10-31T10:43:36Z
approved_at: 2026-10-31T10:32:31Z
approved_hash: f7b0fb723264f5821e7b5ff17d5bbd241be808e3bd73117dfbcd2c7157733ea2
attempts: 0
tests_result: passed
regression_result: passed
evidence_result: passed
model: sonnet
updated_at: 2026-10-31T10:43:36Z
---

# TASK-03: Skills com Coordenador por padrão e briefing

## Execução

### Ler

- `vostok/work/specs/refined/SPEC-019-reduzir-o-consumo-de-tokens-dos-subagentes.md` (RF-01, RF-02, RF-05, RF-06, RF-09, RF-10, regras de negócio, fluxos)
- `packages/harness/skills/default/execute-task/SKILL.md`
- `packages/harness/skills/default/approve-plan/SKILL.md` (item de `TASK_READ` no passo 4)
- `packages/harness/skills/default/finalize-spec/SKILL.md`
- `apps/cli/tests/base.rs` (teste `ct11_execute_task_runs_a_task_or_a_wave`)

### Mudar

- `packages/harness/skills/default/execute-task/SKILL.md` (alterar): Coordenador por padrão, limite configurável, pedido pelo briefing e consolidação curta.
- `packages/harness/skills/default/approve-plan/SKILL.md` (alterar): "Ler" com trechos.
- `packages/harness/skills/default/finalize-spec/SKILL.md` (alterar): execução no Coordenador.
- `apps/cli/tests/base.rs` (alterar): trechos conferidos pelo `ct11_execute_task_runs_a_task_or_a_wave`.

### Passos

1. `execute-task` (suba `version` para `1.2.0` e acrescente `vostok task brief, vostok task check` em `scripts`):
   - Regra [execute-task-10]: tarefas com arquivos em comum rodam em sequência, no Coordenador; subagentes só para 2 ou mais tarefas da onda que podem rodar em paralelo, com no máximo o valor de `max_subagents` (linha `Subagentes ao mesmo tempo` do `vostok task status`) ao mesmo tempo; as excedentes esperam uma vaga.
   - Nova regra: "[execute-task-12]: O Coordenador faz o máximo possível: onda com uma tarefa e tarefas em sequência são executadas na própria sessão, seguindo o `vostok task brief` de cada uma; subagente só existe para paralelismo real (RN-01)."
   - Passo 5: depois de iniciar todas as tarefas (como hoje), separe as que rodam no Coordenador (onda de uma tarefa e grupos em sequência) das paralelas; as do Coordenador seguem o `vostok task brief`, uma por vez; as paralelas recebem o pedido novo. O FA-01 fica como está, citando o `vostok task brief`.
   - Pedido ao subagente: três itens — rodar `vostok task brief <SPEC_ID> <TASK_ID>` e seguir o que ele imprime; não rodar `vostok context load`, `vostok task start` nem `vostok task baseline`, nem ler esta skill; devolver o bloco de até 5 linhas do briefing.
   - Roteiro do subagente: troque as seções "Carregar a tarefa", "Conferir o baseline", "Implementar os testes" e "Implementar o código" por um parágrafo que diz que o roteiro vem do `vostok task brief` (ler por trechos, testes primeiro, `vostok task check`, sem commit), mantendo a orientação de interface (`vostok context load --skill execute-task --persona ui-ux-designer` só em tarefa com interface).
   - Passo 6: consolidar com `vostok task status <SPEC_ID>`, a linha `baseline` do registro e `git diff --stat`, sem reler os diffs; o bloco devolvido vai para a resposta.
   - Mantenha os trechos `vostok task status`, `vostok task start <SPEC_ID> <TASK_ID> --model <MODELO>`, `subagente`, `Mudar` e `em sequência`, e a regra [execute-task-03].
2. `apps/cli/tests/base.rs`: no `ct11_execute_task_runs_a_task_or_a_wave`, troque o trecho `no máximo 6 subagentes` por `max_subagents` e acrescente `vostok task brief`.
3. `approve-plan` (suba `version` para `1.2.0`): no item de "Ler" do passo 4, acrescente que, para arquivo com mais de 400 linhas, o item cita a função, o teste ou a faixa de linhas (`` - `caminho` (fn nome, teste nome) ``), e que quem executa lê por busca e trecho.
4. `finalize-spec` (suba `version` para a próxima menor): no passo 1 ou numa regra nova, diga que a skill é executada no Coordenador, uma spec por vez, sem disparar subagente (RF-10).
5. Rode `cargo test --manifest-path apps/cli/Cargo.toml --test base` e `cargo run --quiet --manifest-path apps/cli/Cargo.toml -- rules validate`.

### Baseline

- `cargo run --quiet --manifest-path apps/cli/Cargo.toml -- rules validate`
- `cargo test --manifest-path apps/cli/Cargo.toml --test base 2>&1 | grep -E '^test ' | grep -v '^test result' | sort`

### Verificação

```bash
cargo test --manifest-path apps/cli/Cargo.toml --test base
cargo run --quiet --manifest-path apps/cli/Cargo.toml -- rules validate
grep -c 'execute-task-12' packages/harness/skills/default/execute-task/SKILL.md
grep -c 'vostok task brief' packages/harness/skills/default/execute-task/SKILL.md
grep -c 'git diff --stat' packages/harness/skills/default/execute-task/SKILL.md
grep -c '400 linhas' packages/harness/skills/default/approve-plan/SKILL.md
grep -ci 'sem disparar subagente' packages/harness/skills/default/finalize-spec/SKILL.md
! grep -q 'no máximo 6 subagentes' packages/harness/skills/default/execute-task/SKILL.md
```

Resultado esperado: `base` passa; `rules validate` imprime `Regras e mapa válidos.`; cada `grep -c` imprime pelo menos `1`; o último não encontra o texto antigo.

### Evidências

- `skills-coordenador.txt` (automática): `grep -n 'execute-task-12\|max_subagents\|vostok task brief\|git diff --stat' packages/harness/skills/default/execute-task/SKILL.md`
  - Prova: Coordenador por padrão, limite configurável, pedido pelo briefing e consolidação curta (RF-01, RF-02, RF-05, RF-09, CT-09).
  - Trecho: a regra `[execute-task-12]`, a regra 10 com `max_subagents`, o pedido com `vostok task brief` e o passo 6 com `git diff --stat`.
  - Falha: saída vazia ou sem algum dos quatro.
- `skills-trechos-finalizacao.txt` (automática): `grep -n '400 linhas' packages/harness/skills/default/approve-plan/SKILL.md; grep -ni 'sem disparar subagente' packages/harness/skills/default/finalize-spec/SKILL.md`
  - Prova: "Ler" com trechos (RF-06) e finalização no Coordenador (RF-10), CT-09.
  - Trecho: uma linha de cada skill.
  - Falha: saída vazia ou só uma das skills.

### Concluída quando

- [x] A `execute-task` executa no Coordenador a onda de uma tarefa e as tarefas em sequência, e usa subagentes só para tarefas paralelas até `max_subagents` (RF-01, RF-02, CB-03, CT-09).

  > Como comprovar: abrir a skill e ver a regra `[execute-task-12]`, a regra 10 com `max_subagents` e o passo 5; ver `skills-coordenador.txt`.

- [x] O pedido ao subagente é o `vostok task brief`, e a consolidação usa `git diff --stat` (RF-05, RF-09, CT-09).

  > Como comprovar: abrir "Pedido ao subagente" e o passo 6 da skill.

- [x] A `approve-plan` orienta "Ler" com trechos e a `finalize-spec` roda no Coordenador (RF-06, RF-10, CT-09).

  > Como comprovar: ver `skills-trechos-finalizacao.txt`.

- [x] Os testes de `base` passam.

  > Como comprovar: rodar a Verificação e ver `test result: ok`.

## Descrição

### Resumo executivo

Muda as instruções do harness para que o coordenador faça a maior parte do trabalho, só use ajudantes quando houver tarefas em paralelo e passe a cada um só o resumo curto da tarefa. Perfil padrão, risco baixo: só textos de skills.

### Detalhes técnicos

- **Abordagem:** reescrita da `execute-task` (regras 10 e 11, passos 5 e 6, pedido e roteiro), da orientação de "Ler" na `approve-plan` e da `finalize-spec`.
- **Decisões:** usuário 2A (Coordenador para onda de uma tarefa e tarefas em sequência), 3A (`max_subagents`); inferências C e E.
- **Riscos:** o `ct11_execute_task_runs_a_task_or_a_wave` confere trechos da skill; ajustá-lo junto.
- **Referências:** RF-01, RF-02, RF-05, RF-06, RF-09, RF-10, CT-09, CB-03.
- **Arquivos em comum:** nenhum na onda 2.

### Card

- **Título:** Skills com Coordenador por padrão e briefing
- **Tipo:** tarefa
- **Descrição:** As skills passam a usar o Coordenador por padrão, o limite configurável, o briefing, a leitura por trechos e a consolidação curta.
- **Critérios de aceite:**
  - [ ] Coordenador por padrão e max_subagents na execute-task
  - [ ] Pedido ao subagente pelo vostok task brief
  - [ ] Ler com trechos e finalização no Coordenador
- **Rótulos:** SPEC-019, padrao
- **Estimativa:** padrao
- **Bloqueada por:** TASK-01

## Registro de execução

| # | Data | Etapa | Resultado | Detalhe |
|---|---|---|---|---|
| 1 | 2026-10-31 | início | in_progress | aprovada em 2026-10-31T10:32:31Z; modelo: sonnet |
| 2 | 2026-10-31 | baseline | coletado | 2 comando(s); códigos de saída: 0, 0; 2 evidência(s) automática(s) gravada(s) para comparação |
| 3 | 2026-10-31 | ajuste | in_progress | Regra nova da execute-task numerada [execute-task-12], porque a 11 já existe (FA-01) |
| 4 | 2026-10-31 | testes | passed | Verificação sem falhas; saída em validation/tests.out |
| 5 | 2026-10-31 | regressão | passed | 2 comando(s) iguais ao baseline |
| 6 | 2026-10-31 | evidências | passed | 2 evidência(s) em evidence/TASK-03/ |
| 7 | 2026-10-31 | conclusão | done | testes, regressão e evidências passed; critérios de conclusão marcados |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
| 1 | 2026-10-31 | agente | Regra nova da execute-task numerada [execute-task-12], porque a 11 já existe (FA-01) |
