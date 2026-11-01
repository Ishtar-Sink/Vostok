---
id: TASK-09
spec: SPEC-023
title: "Orientação das skills para a tarefa em pasta"
profile: padrao
persona: desenvolvedor
depends_on: []
wave: 1
covers: [RF-15, RF-16, RF-17, CT-15]
type: tarefa
labels: [SPEC-023, padrao]
status: done
content_hash: 10c53c3ce3e8a3b185487adf6a76ba6ceab2e0fcc0baa4c304d2793fb8ee86d4
adjustments: 0
adjusted_at: null
approved_at: 2026-11-01T12:01:26Z
approved_hash: c824fcfbe2c395ef7a851b6814dde76b78a260353e391006c345b143fef04f27
attempts: 0
tests_result: passed
regression_result: passed
evidence_result: passed
model: sonnet
---

# TASK-09: Orientação das skills para a tarefa em pasta

## Execução

### Ler

- `vostok/work/specs/refined/SPEC-023-tarefa-como-pasta-com-arquivos-dedicados.md` (RF-15, RF-16, RF-17, "Contratos e interfaces")
- `packages/harness/skills/default/approve-plan/SKILL.md` (passos de detalhamento e as variáveis da tarefa)
- `packages/harness/skills/default/finish-task/SKILL.md` (passos de evidências e conclusão)
- `packages/harness/skills/default/adjust-task/SKILL.md`, `packages/harness/skills/default/approve-task/SKILL.md`, `packages/harness/skills/default/execute-task/SKILL.md`, `packages/harness/skills/default/finalize-spec/SKILL.md`
- `packages/harness/config/models.yml` (perfis)
- `apps/cli/tests/base.rs` (testes que conferem trechos das skills)

### Mudar

- `packages/harness/skills/default/approve-plan/SKILL.md` (alterar): detalhar só os arquivos do agente e passos em formato fixo
- `packages/harness/skills/default/finish-task/SKILL.md` (alterar): subagente de perfil `mecanica` para o executivo e o técnico
- `packages/harness/skills/default/adjust-task/SKILL.md` (alterar): ajuste por arquivo
- `packages/harness/skills/default/approve-task/SKILL.md` (alterar): arquivos novos na aprovação
- `packages/harness/skills/default/execute-task/SKILL.md` (alterar): arquivos novos na execução
- `packages/harness/skills/default/finalize-spec/SKILL.md` (alterar): documentos de cada tarefa entre as fontes

### Passos

1. `packages/harness/skills/default/approve-plan/SKILL.md`: o detalhamento preenche `execucao.md`, `verificacao.md` e `criterios.md`; o executivo e o técnico ficam para a finalização da tarefa; o card e o README são gerados. Passos em formato fixo, um por item: caminho, ação e conferência, como no contrato da spec. Cite `vostok task migrate` para specs no formato antigo.
2. `packages/harness/skills/default/finish-task/SKILL.md`: depois das evidências e antes do `vostok task finish`, dispare um subagente de perfil `mecanica` (o mais simples em `config/models.yml`) com a saída de `vostok task describe`, para escrever `executivo.md` e `tecnico.md`; o Coordenador confere o texto. Sem subagente, o Coordenador escreve a partir do mesmo `describe`.
3. `packages/harness/skills/default/adjust-task/SKILL.md`: o ajuste lê só o arquivo ajustado com `vostok task load --part`; o `record-manual` cita o arquivo.
4. `packages/harness/skills/default/approve-task/SKILL.md`, `packages/harness/skills/default/execute-task/SKILL.md` e `packages/harness/skills/default/finalize-spec/SKILL.md`: troque as seções do arquivo único pelos arquivos da pasta e `evidence/<TASK>` por `<TASK>/evidencias`, mantendo as instruções para o formato antigo onde elas valem.
5. Rode `cargo test --manifest-path apps/cli/Cargo.toml --test base`; se um teste conferir um trecho que mudou, ajuste só o trecho.

### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test base 2>&1 | grep -E '^test ' | sort`

### Verificação

```bash
cargo test --manifest-path apps/cli/Cargo.toml --test base
```

Resultado esperado: os testes de `base` passam.

### Evidências

- `approve-plan.diff` (diff): `packages/harness/skills/default/approve-plan/SKILL.md`
  - Prova: o detalhamento só dos arquivos do agente e os passos em formato fixo (RF-15 a RF-17, CT-15).
  - Trecho: as linhas com `execucao.md` e o formato do passo.
  - Falha: diff sem esse trecho.
  - Espera: `execucao.md`
- `finish-task.diff` (diff): `packages/harness/skills/default/finish-task/SKILL.md`
  - Prova: o subagente `mecanica` com o `describe` antes do `finish` (RF-15 a RF-17, CT-15).
  - Trecho: o passo novo.
  - Falha: diff sem esse trecho.
  - Espera: `vostok task describe`
- `adjust-task.diff` (diff): `packages/harness/skills/default/adjust-task/SKILL.md`
  - Prova: o ajuste por arquivo (RF-15 a RF-17, CT-15).
  - Trecho: as linhas com `--part`.
  - Falha: diff sem esse trecho.
  - Espera: `--part`
- `approve-task.diff` (diff): `packages/harness/skills/default/approve-task/SKILL.md`
  - Prova: os arquivos novos na aprovação (RF-15 a RF-17, CT-15).
  - Trecho: as linhas com os arquivos da pasta.
  - Falha: diff sem esse trecho.
  - Espera: `criterios.md`
- `execute-task.diff` (diff): `packages/harness/skills/default/execute-task/SKILL.md`
  - Prova: os arquivos novos na execução (RF-15 a RF-17, CT-15).
  - Trecho: as linhas com a pasta da tarefa.
  - Falha: diff sem esse trecho.
  - Espera: `evidencias`
- `finalize-spec.diff` (diff): `packages/harness/skills/default/finalize-spec/SKILL.md`
  - Prova: os documentos de cada tarefa entre as fontes (RF-15 a RF-17, CT-15).
  - Trecho: as linhas com `tecnico.md`.
  - Falha: diff sem esse trecho.
  - Espera: `tecnico.md`

### Concluída quando

- [x] As seis skills descrevem a tarefa em pasta, o formato fixo dos passos e o subagente da `finish-task` (RF-15 a RF-17, CT-15).

  > Como comprovar: abrir os seis diffs e ver os trechos novos.

- [x] Os testes de `base` continuam passando.

  > Como comprovar: a Verificação termina com código 0.

## Descrição

### Resumo executivo

Atualiza as instruções do harness para o formato novo: o que o planejamento escreve, quem escreve os documentos de cada tarefa e onde ficam as evidências.

### Detalhes técnicos

Só texto, a partir dos contratos da spec. O perfil "simples" da spec é o `mecanica`, o mais simples de `config/models.yml` (não há perfil `simples`). Roda na onda 1, em paralelo com a TASK-01, e descreve comandos que só existem depois das tarefas de código. Referências: RF-15 a RF-17, CT-15.

### Card

- **Título:** Orientação das skills para a tarefa em pasta
- **Tipo:** tarefa
- **Descrição:** Skills descrevem a tarefa em pasta, os passos em formato fixo e o subagente mecanica da finish-task.
- **Critérios de aceite:**
  - [ ] Seis skills atualizadas
- **Rótulos:** SPEC-023, padrao
- **Estimativa:** padrao
- **Bloqueada por:** -

## Registro de execução

| # | Data | Etapa | Resultado | Detalhe |
|---|---|---|---|---|
| 1 | 2026-11-01 | início | in_progress | aprovada em 2026-11-01T12:01:26Z; modelo: sonnet |
| 2 | 2026-11-01 | baseline | coletado | 1 comando(s); códigos de saída: 0 |
| 3 | 2026-11-01 | testes | passed | Verificação sem falhas; saída em validation/tests.out |
| 4 | 2026-11-01 | regressão | passed | 1 comando(s) iguais ao baseline |
| 5 | 2026-11-01 | evidências | passed | 6 evidência(s) em evidence/TASK-09/ |
| 6 | 2026-11-01 | conclusão | done | testes, regressão e evidências passed; critérios de conclusão marcados |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
