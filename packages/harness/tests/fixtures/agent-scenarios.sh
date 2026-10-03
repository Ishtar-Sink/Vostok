#!/usr/bin/env bash
# Monta as cópias descartáveis dos cenários com agente de tests/agent-scenarios.md.
# Cada cenário recebe uma cópia do projeto (AGENTS.md, CLAUDE.md e packages/harness) com:
#   SPEC-001 Exportar memória em JSON (refinada), SPEC-002 Listar tarefas (refinada)
#   e SPEC-003 Painel de métricas (rascunho). Alguns cenários removem specs (ver `prune`),
#   partem de specs aprovadas (ver `approve`), de um plano de tarefas (ver `plan`), da
#   SPEC-001 já planejada (ver `planned`) ou das tarefas dela aprovadas (ver `tasks_approved`,
#   `failing_verify` e `independent`).
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
 'NON_FUNCTIONAL_REQUIREMENTS=| RNF-01 | Roda em bash 3.2 com awk e sed do BSD. | Suíte executada no macOS. |\n| RNF-02 | Monta o JSON com awk, sem dependências externas. | Revisão do script. |' \
 'BUSINESS_RULES=| RN-01 | Entradas com status `obsoleto` não são exportadas. |' \
 'MAIN_FLOW=1. O usuário executa `export-memory.sh <tipo>`.\n2. O script lê as entradas do tipo.\n3. O script grava o JSON em `memory/export.json`.' \
 'ALTERNATIVE_FLOWS=- **FA-01 (passo 1):** tipo inválido. O script termina com erro e lista os tipos.' \
 'EDGE_CASES=| CB-01 | Memória vazia | Grava `[]` | RF-01 |\n| CB-02 | Título com aspas | Aspas escapadas no JSON | RF-01 |' \
 'CONTRACTS=```bash\nexport-memory.sh <tipo>   # decision, issue, knowledge ou all\n```' \
 'AFFECTED_COMPONENTS=| `packages/harness/scripts/default/memory/export-memory.sh` | criar | Script de exportação. |' \
 'ASSUMPTIONS=- As entradas seguem o formato de `config/memory.yml`.' \
 'RISKS=| Título com caracteres especiais | médio | Escapar aspas e barras invertidas. |' \
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
# A TASK-01 é detalhada de acordo com a spec; as demais usam os valores padrão de fill-refined.py.
planned() {
  local ph="$1/packages/harness" task
  "$ph/skills/default/approve-plan/scripts/approve-plan.sh" --create-tasks SPEC-001 > /dev/null
  F "$ph/work/tasks/SPEC-001/TASK-01.md" \
   'TASK_READ=- `packages/harness/config/memory.yml`\n- `packages/harness/scripts/default/memory/search-memory.sh`' \
   'TASK_CHANGE=- `packages/harness/scripts/default/memory/export-memory.sh` (criar): exporta um tipo de memória em JSON.' \
   'TASK_STEPS=1. Criar `export-memory.sh`, que lê as entradas ativas do tipo e grava em `packages/harness/memory/export.json` uma lista JSON (`[]` quando não há entradas).\n2. Com tipo inválido, terminar com erro e listar os tipos.' \
   'TASK_BASELINE=- `packages/harness/scripts/default/memory/search-memory.sh decision memória`' \
   'TASK_VERIFY=```bash\ns=packages/harness/scripts/default/memory/export-memory.sh\ntest -x "$s"\n"$s" decision\ntest "$(cat packages/harness/memory/export.json)" = "[]"   # memória vazia (CB-01)\nif "$s" nada; then exit 1; fi   # tipo inválido (FA-01)\n```' \
   'TASK_EVIDENCE=- `export.txt` (automática): `cat packages/harness/memory/export.json`' \
   'TASK_DONE=- [ ] O script exporta um tipo de memória em JSON (RF-01, RF-03).\n\n  > Como comprovar: com a memória vazia, exportar as decisões grava uma lista vazia no arquivo de exportação.\n- [ ] Tipo inválido termina com erro (FA-01).\n\n  > Como comprovar: pedir a exportação de um tipo que não existe termina com erro.' \
   'TASK_EXECUTIVE_SUMMARY=Permite exportar as decisões, issues ou conhecimentos do agente em JSON para o painel web.' \
   'TASK_TECHNICAL_DETAILS=Script bash que acha o arquivo do tipo em `config/memory.yml` e monta o JSON com awk (RF-01, RF-03, FA-01).' \
   'TASK_CARD_DESCRIPTION=Cria o script que exporta um tipo de memória em JSON.' \
   'TASK_CARD_CRITERIA=  - [ ] Exporta um tipo em JSON.\n  - [ ] Tipo inválido termina com erro.'
  for task in "$ph"/work/tasks/SPEC-001/*.md; do F "$task"; done
  "$ph/skills/default/approve-plan/scripts/approve-plan.sh" --mark-planned SPEC-001 > /dev/null
}

# Aprova todas as tarefas da SPEC-001 (planejada) de um cenário.
tasks_approved() {
  "$1/packages/harness/skills/default/approve-task/scripts/approve-task.sh" --approve SPEC-001 --all > /dev/null
}

# Troca a "Verificação" da TASK-01 (pendente) por uma que nunca passa e registra o ajuste.
failing_verify() {
  local ph="$1/packages/harness"
  python3 - "$ph/work/tasks/SPEC-001/TASK-01.md" <<'PY'
import re, sys; p = sys.argv[1]; s = open(p, encoding='utf-8').read()
s = re.sub(r'```bash\n.*?```', lambda m: '```bash\ntest "$(date +%Y)" = 1999   # o relógio da máquina está em 1999\n```', s, count=1, flags=re.S)
open(p, 'w', encoding='utf-8').write(s)
PY
  "$ph/skills/default/adjust-task/scripts/adjust-task.sh" --record SPEC-001 TASK-01 "Verificação que nunca passa" > /dev/null
}

# Tira a dependência da TASK-02 (pendente), para que ela e a TASK-01 sejam executáveis juntas.
independent() {
  local file="$1/packages/harness/work/tasks/SPEC-001/TASK-02.md"
  sed -i.bak 's/^depends_on: .*/depends_on: []/; s/^- \*\*Bloqueada por:\*\* .*/- **Bloqueada por:** -/' "$file" && rm -f "$file.bak"
  "$1/packages/harness/skills/default/adjust-task/scripts/adjust-task.sh" --record SPEC-001 TASK-02 "Sem dependência" > /dev/null
}

for scenario in ct01 ct02 ct03 ct04 ct05 ct06 ct09 ct13 ct14 ct15 ct16 ap01 ap02 ap03 ap04 ap13 \
  pl01 pl03 pl06 pl06b pl07 pl11 pl13a pl13b pl13c tk01 tk19 tk04 tk09 tk15 tk16 tk17a tk17b; do
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
    tk01|tk19|tk17a) approve "$DEST/$scenario"; plan "$DEST/$scenario"; planned "$DEST/$scenario" ;;
    tk04|tk09|tk17b) approve "$DEST/$scenario"; plan "$DEST/$scenario"; planned "$DEST/$scenario"; tasks_approved "$DEST/$scenario" ;;
    tk15) approve "$DEST/$scenario"; plan "$DEST/$scenario"; planned "$DEST/$scenario"; failing_verify "$DEST/$scenario"; tasks_approved "$DEST/$scenario" ;;
    tk16) approve "$DEST/$scenario"; plan "$DEST/$scenario"; planned "$DEST/$scenario"; independent "$DEST/$scenario"; tasks_approved "$DEST/$scenario" ;;
  esac
  echo "$DEST/$scenario"
done
rm -rf "$B"
