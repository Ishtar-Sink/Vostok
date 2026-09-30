#!/usr/bin/env bash
# Monta as cópias descartáveis dos cenários com agente de tests/agent-scenarios.md.
# Cada cenário recebe uma cópia do projeto (AGENTS.md, CLAUDE.md e packages/harness) com:
#   SPEC-001 Exportar memória em JSON (refinada), SPEC-002 Listar tarefas (refinada)
#   e SPEC-003 Painel de métricas (rascunho). Alguns cenários removem specs (ver `prune`),
#   partem de specs aprovadas (ver `approve`), de um plano de tarefas (ver `plan`) ou da
#   SPEC-001 já planejada (ver `planned`).
#
# Uso: tests/fixtures/agent-scenarios.sh <pasta de destino>
#      Cria <destino>/<cenário> para cada cenário e imprime os caminhos.
set -euo pipefail

[ -n "${1:-}" ] || { echo "Uso: agent-scenarios.sh <pasta de destino>" >&2; exit 1; }
SOURCE_HARNESS="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SOURCE_ROOT="$(cd "$SOURCE_HARNESS/../.." && pwd)"
DEST="$(mkdir -p "$1" && cd "$1" && pwd)"
B="$DEST/base"

rm -rf "$B" && mkdir -p "$B/packages"
cp "$SOURCE_ROOT/AGENTS.md" "$SOURCE_ROOT/CLAUDE.md" "$B/"
cp -R "$SOURCE_HARNESS" "$B/packages/"
H="$B/packages/harness"
rm -rf "$H/work" "$H"/memory/*.md
for key in specs decisions issues knowledge; do
  sed -i.bak "s/^$key: .*/$key: 1/" "$H/config/sequences.yml"
done
rm -f "$H/config/sequences.yml.bak"
SP="$H/scripts/default/specs"
cd /

F() { python3 "$SOURCE_HARNESS/tests/fixtures/fill-refined.py" "$@"; }
"$SP"/create-new-spec.sh 'Exportar memória em JSON' >/dev/null; "$SP"/create-refined-spec.sh SPEC-001 feature >/dev/null; f="$(ls "$H"/work/specs/refined/SPEC-001-*.md)"
F "$f" 'SPEC_SUMMARY=Cria o script `export-memory.sh`, que exporta as entradas da memória do agente em JSON.' \
 'OBJECTIVE=Hoje a memória do agente só pode ser lida em Markdown. O painel web precisa dela em JSON para exibir decisões, issues e conhecimentos.' \
 'SCOPE_IN=- Script `export-memory.sh` em `packages/harness/scripts/default/memory/`.\n- Exportação por tipo e de todos os tipos.' \
 'SCOPE_OUT=- Importação de JSON para a memória.\n- Envio do JSON ao servidor HTTP.' \
 'FUNCTIONAL_REQUIREMENTS=| RF-01 | O script deve exportar as entradas de um tipo de memória em JSON, com ID, título, status e tags. |\n| RF-02 | O script deve aceitar o tipo `all` para exportar todos os tipos num único arquivo. |\n| RF-03 | O script deve gravar a saída em `memory/export.json`. |' \
 'NON_FUNCTIONAL_REQUIREMENTS=| RNF-01 | Roda em bash 3.2 com awk e sed do BSD. | Suíte executada no macOS. |\n| RNF-02 | Usa `jq` para montar o JSON. | Revisão do script. |' \
 'BUSINESS_RULES=| RN-01 | Entradas com status `obsoleto` não são exportadas. |' \
 'MAIN_FLOW=1. O usuário executa `export-memory.sh <tipo>`.\n2. O script lê as entradas do tipo.\n3. O script grava o JSON em `memory/export.json`.' \
 'ALTERNATIVE_FLOWS=- **FA-01 (passo 1):** tipo inválido. O script termina com erro e lista os tipos.' \
 'EDGE_CASES=| CB-01 | Memória vazia | Grava `[]` | RF-01 |\n| CB-02 | Título com aspas | Aspas escapadas no JSON | RF-01 |' \
 'CONTRACTS=```bash\nexport-memory.sh <tipo>   # decision, issue, knowledge ou all\n```' \
 'AFFECTED_COMPONENTS=| `packages/harness/scripts/default/memory/export-memory.sh` | criar | Script de exportação. |' \
 'ASSUMPTIONS=- `jq` está instalado.' \
 'RISKS=| `jq` ausente na máquina | médio | Verificar no início e terminar com erro claro. |' \
 'TEST_CASES=| CT-01 | Exportar decisões | Executar com `decision` | JSON com as decisões | RF-01, RF-03 |\n| CT-02 | Exportar tudo | Executar com `all` | JSON com os três tipos | RF-02 |' \
 'ACCEPTANCE_CRITERIA=- [ ] CA-01: `export-memory.sh decision` gera JSON com as decisões ativas (RF-01, RF-03).\n- [ ] CA-02: `export-memory.sh all` gera um único arquivo com os três tipos (RF-02).' \
 'REFERENCES=| arquivo | `packages/harness/config/memory.yml` | Tipos e status da memória. |' \
 'WHAT_CHANGED=| 1 | Formato | Não definido | JSON com ID, título, status e tags |' \
 'DECISIONS=| 1 | Saída em arquivo fixo | Saída padrão | inferência A |' \
 'PENDING=| - | Nenhuma | - |'
"$SP"/create-refined-spec.sh --mark-refined SPEC-001 >/dev/null
"$SP"/create-new-spec.sh 'Listar tarefas' >/dev/null; "$SP"/create-refined-spec.sh SPEC-002 feature >/dev/null; g="$(ls "$H"/work/specs/refined/SPEC-002-*.md)"
F "$g" 'SPEC_SUMMARY=Cria o script `list-tasks.sh`, que lista as tarefas de uma spec.' 'OBJECTIVE=O agente precisa listar as tarefas geradas de uma spec para acompanhar a execução.' \
 'FUNCTIONAL_REQUIREMENTS=| RF-01 | O script deve listar as tarefas de uma spec em tabela, com ID, título e status. |\n| RF-02 | O script deve aceitar o ID da spec como argumento obrigatório. |' \
 'AFFECTED_COMPONENTS=| `packages/harness/scripts/default/tasks/list-tasks.sh` | criar | Script de listagem. |' \
 'TEST_CASES=| CT-01 | Listar | Executar com SPEC-001 | Tabela com as tarefas | RF-01, RF-02 |' 'ACCEPTANCE_CRITERIA=- [ ] CA-01: `list-tasks.sh SPEC-001` lista as tarefas (RF-01, RF-02).'
"$SP"/create-refined-spec.sh --mark-refined SPEC-002 >/dev/null
"$SP"/create-new-spec.sh 'Painel de métricas' >/dev/null

# Remove as specs informadas (rascunho e refinada) de um cenário.
prune() {
  local dir="$1" id
  shift
  for id in "$@"; do rm -f "$dir"/packages/harness/work/specs/*/"$id"-*.md; done
}

# Aprova as specs informadas de um cenário (SPEC-001 sem argumentos).
approve() {
  local dir="$1" id
  shift
  for id in "${@:-SPEC-001}"; do
    "$dir/packages/harness/skills/default/approve-spec/scripts/approve-spec.sh" --approve "$id" > /dev/null
  done
}

# Cria e pontua o plano de tarefas da SPEC-001 (aprovada) de um cenário:
# TASK-01 exporta um tipo, TASK-02 exporta todos e TASK-03 reúne os testes.
plan() {
  local ph="$1/packages/harness"
  "$ph/skills/default/plan-tasks/scripts/plan-tasks.sh" --create SPEC-001 > /dev/null
  F "$ph/work/plans/SPEC-001.md" \
   'PLAN_OVERVIEW=O trabalho começa pelo script que exporta um tipo de memória (TASK-01), base das demais. A exportação de todos os tipos (TASK-02) reaproveita esse script. Os testes (TASK-03) fecham o plano e dependem das duas tarefas.' \
   'PLAN_TASKS=| TASK-01 | Exportar um tipo de memória | Cria `export-memory.sh` com a exportação de um tipo e a gravação em `memory/export.json`. | tarefa | - | - | RF-01, RF-03 |\n| TASK-02 | Exportar todos os tipos | Acrescenta o tipo `all`, que junta os três tipos num único arquivo. | tarefa | - | TASK-01 | RF-02 |\n| TASK-03 | Testes da exportação | Cobre a exportação de um tipo e de todos na suíte de testes. | tarefa | - | TASK-01, TASK-02 | CT-01, CT-02 |' \
   'PLAN_FACTS=| TASK-01 | 1 | 0 | sim | não | não |\n| TASK-02 | 0 | 1 | não | não | não |\n| TASK-03 | 0 | 1 | não | não | não |'
  "$ph/skills/default/plan-tasks/scripts/plan-tasks.sh" --score SPEC-001 > /dev/null
}

# Aprova o plano da SPEC-001 com tarefas preenchidas pela fixture e marca a spec como planejada.
planned() {
  local ph="$1/packages/harness" task
  "$ph/skills/default/approve-plan/scripts/approve-plan.sh" --create-tasks SPEC-001 > /dev/null
  for task in "$ph"/work/tasks/SPEC-001/*.md; do F "$task"; done
  "$ph/skills/default/approve-plan/scripts/approve-plan.sh" --mark-planned SPEC-001 > /dev/null
}

for scenario in ct01 ct02 ct03 ct04 ct05 ct06 ct09 ct13 ct14 ct15 ct16 ap01 ap02 ap03 ap04 ap13 \
  pl01 pl03 pl06 pl06b pl07 pl11 pl13a pl13b pl13c; do
  rm -rf "${DEST:?}/$scenario"
  cp -R "$B" "$DEST/$scenario"
  case "$scenario" in
    ct04|ct09) prune "$DEST/$scenario" SPEC-002 ;;
    ct05|ap04) prune "$DEST/$scenario" SPEC-001 SPEC-002 ;;
    ap02) prune "$DEST/$scenario" SPEC-002 ;;
    ap13|pl01) approve "$DEST/$scenario" ;;
    pl03) approve "$DEST/$scenario" SPEC-001 SPEC-002 ;;
    pl06|pl06b|pl07) approve "$DEST/$scenario"; plan "$DEST/$scenario" ;;
    pl11) approve "$DEST/$scenario"; plan "$DEST/$scenario"; planned "$DEST/$scenario" ;;
    pl13a|pl13b|pl13c) approve "$DEST/$scenario" SPEC-001 SPEC-002; plan "$DEST/$scenario" ;;
  esac
  echo "$DEST/$scenario"
done
rm -rf "$B"
