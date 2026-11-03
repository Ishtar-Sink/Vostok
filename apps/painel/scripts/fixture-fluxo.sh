#!/usr/bin/env bash
# Monta numa pasta vazia um projeto de demonstração para os prints da SPEC-034: a SPEC-001 aprovada
# de novo depois do plano (plano desatualizado) e a SPEC-002 com a TASK-01 em validação, com um print
# ainda sem conferência.
# Uso: [PAINEL_BIN=<caminho do vostok>] bash apps/painel/scripts/fixture-fluxo.sh <pasta>

set -euo pipefail

DIR="${1:?uso: fixture-fluxo.sh <pasta>}"
BIN="${PAINEL_BIN:-vostok}"
# Um caminho relativo deixaria de valer depois do cd para a pasta do projeto.
case "$BIN" in
  */*) BIN="$(cd "$(dirname "$BIN")" && pwd)/$(basename "$BIN")" ;;
esac

mkdir -p "$DIR"
cd "$DIR"
git init -q
"$BIN" init --yes >/dev/null

WORK=vostok/work
mkdir -p "$WORK/specs/refined" "$WORK/plans" "$WORK/tasks/SPEC-002/TASK-01"

cat >"$WORK/specs/refined/SPEC-001-exportar-a-lista-de-tarefas.md" <<'EOF'
---
id: SPEC-001
title: "Exportar a lista de tarefas"
status: approved
author: "Demonstração"
created_at: 2026-11-03T10:00:00Z
updated_at: 2026-11-03T12:00:00Z
tags: []
type: feature
refined_at: 2026-11-03T10:30:00Z
approved_at: 2026-11-03T12:00:00Z
approved_hash: 2222222222222222222222222222222222222222222222222222222222222222
adjustments: 1
adjusted_at: 2026-11-03T11:30:00Z
status_history: 2026-11-03T10:40:00Z approved; 2026-11-03T11:30:00Z refined; 2026-11-03T12:00:00Z approved
---

# Exportar a lista de tarefas - Especificação Refinada

Exporta para um arquivo CSV as tarefas visíveis na lista, com os filtros aplicados.

## Objetivo

Levar as tarefas do app para outra ferramenta.

## Requisitos funcionais

| ID | Requisito |
|---|---|
| RF-01 | O sistema deve gerar um CSV com as tarefas visíveis na lista. |
| RF-02 | O sistema deve incluir na lista um filtro por status. |
EOF

cat >"$WORK/plans/SPEC-001.md" <<'EOF'
---
spec: SPEC-001
spec_hash: 1111111111111111111111111111111111111111111111111111111111111111
content_hash: 3333333333333333333333333333333333333333333333333333333333333333
adjustments: 0
adjusted_at: null
created_at: 2026-11-03T10:50:00Z
updated_at: 2026-11-03T10:50:00Z
---

# Plano de tarefas: SPEC-001 - Exportar a lista de tarefas

## Visão geral

Plano feito antes do ajuste que acrescentou o filtro por status.

## Tarefas

| ID | Título | Descrição | Tipo | Persona | Perfil | Dependências | Onda | Cobre |
|---|---|---|---|---|---|---|---|---|
| TASK-01 | Gerar o CSV | Gera o CSV das tarefas visíveis. | tarefa | desenvolvedor | padrao | - | 1 | RF-01 |
EOF

cat >"$WORK/specs/refined/SPEC-002-criar-o-projeto-base.md" <<'EOF'
---
id: SPEC-002
title: "Criar o projeto base"
status: planned
author: "Demonstração"
created_at: 2026-11-03T13:00:00Z
updated_at: 2026-11-03T14:00:00Z
tags: []
type: feature
refined_at: 2026-11-03T13:20:00Z
approved_at: 2026-11-03T13:40:00Z
approved_hash: 4444444444444444444444444444444444444444444444444444444444444444
adjustments: 0
adjusted_at: null
status_history: 2026-11-03T13:40:00Z approved; 2026-11-03T14:00:00Z planned
---

# Criar o projeto base - Especificação Refinada

Cria o app com uma página inicial e um teste de exemplo.

## Requisitos funcionais

| ID | Requisito |
|---|---|
| RF-01 | O sistema deve abrir uma página com o título "Tarefas". |
EOF

TASK="$WORK/tasks/SPEC-002/TASK-01"
cat >"$TASK/estado.yml" <<'EOF'
id: TASK-01
spec: SPEC-002
title: "Criar o projeto base"
description: "Cria o app com a página inicial e um teste de exemplo."
profile: padrao
persona: desenvolvedor
depends_on: []
wave: 1
covers: [RF-01]
type: tarefa
labels: [SPEC-002, padrao]
status: validating
approved_at: 2026-11-03T14:10:00Z
approved_by: painel
model: claude-sonnet-5-5
attempts: 0
tests_result: passed
regression_result: passed
evidence_result: pending
updated_at: 2026-11-03T14:40:00Z
EOF

cat >"$TASK/execucao.md" <<'EOF'
### Ler

- `vostok/work/specs/refined/SPEC-002-criar-o-projeto-base.md`

### Mudar

- `app/src/app.tsx` (criar): página com o título "Tarefas"

### Passos

1. `app/src/app.tsx` (criar): componente com o título "Tarefas".
EOF

cat >"$TASK/verificacao.md" <<'EOF'
### Baseline

- `node --version`

### Verificação

```bash
npm --prefix app test
```

### Evidências

- `app-base.png` (print): `node app/scripts/take-screenshot.mjs "$VOSTOK_EVIDENCE/app-base.png"`
  - Prova: a página abre com o título "Tarefas" (RF-01).
  - Trecho: o título "Tarefas" no topo.
  - Falha: página em branco.
EOF

cat >"$TASK/criterios.md" <<'EOF'
### Concluída quando

- [x] A página abre com o título "Tarefas" (RF-01).

  > Como comprovar: abrir o print `app-base.png` e ver o título "Tarefas".
EOF

cat >"$TASK/executivo.md" <<'EOF'
<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

Cria a base do app de tarefas, com a página inicial e um teste de exemplo.
EOF

cat >"$TASK/tecnico.md" <<'EOF'
<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** App em `app/`, com a página inicial e o teste do título.
EOF

cat >"$TASK/registro.md" <<'EOF'
## Registro de execução

| # | Data | Etapa | Resultado | Detalhe |
|---|---|---|---|---|
| 1 | 2026-11-03T14:10:00Z | início | in_progress | aprovada em 2026-11-03T14:10:00Z; modelo: claude-sonnet-5-5 |
| 2 | 2026-11-03T14:30:00Z | testes | passed | Verificação sem falhas |
| 3 | 2026-11-03T14:35:00Z | regressão | passed | 1 comando(s) iguais ao baseline |
| 4 | 2026-11-03T14:40:00Z | evidências | pending | print sem conferência humana |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
EOF

echo "$DIR"
