#!/usr/bin/env bash
# Monta as cópias descartáveis dos cenários com agente de tests/agent-scenarios.md.
# Cada cenário recebe uma cópia do projeto (AGENTS.md, CLAUDE.md e packages/harness) com:
#   SPEC-001 Exportar memória em JSON (refinada), SPEC-002 Listar tarefas (refinada)
#   e SPEC-003 Painel de métricas (rascunho). Alguns cenários removem specs (ver `prune`),
#   partem de specs aprovadas (ver `approve`), de um plano de tarefas (ver `plan`), da
#   SPEC-001 já planejada (ver `planned`), das tarefas dela aprovadas (ver `tasks_approved`,
#   `failing_verify` e `independent`), da TASK-01 em execução (ver `started`, `changing_baseline`,
#   `env_verify` e `blocked_by_env`) ou da SPEC-001 implementada (ver `implemented` e `implemented_too`).
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
rm -rf "$H/work" "$H/.state" "$H"/memory/*.md
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

# Inicia a TASK-01 (aprovada), coleta o baseline e escreve o `export-memory.sh`, como a `execute-task` deixa a tarefa:
# em execução, com o código implementado e sem nenhuma validação.
started() {
  local dir="$1" ph="$1/packages/harness" s
  "$ph/skills/default/execute-task/scripts/execute-task.sh" --start SPEC-001 TASK-01 > /dev/null
  "$ph/skills/default/execute-task/scripts/execute-task.sh" --baseline SPEC-001 TASK-01 > /dev/null
  s="$ph/scripts/default/memory/export-memory.sh"
  cat > "$s" <<'SH'
#!/usr/bin/env bash
# Exporta as entradas ativas de um tipo de memória em JSON, em memory/export.json.
set -euo pipefail
case "${1:-}" in
  decision|issue|knowledge|all) ;;
  *) echo "Tipo inválido: ${1:-}. Tipos: decision, issue, knowledge, all." >&2; exit 1 ;;
esac
dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)/memory"
mkdir -p "$dir"
echo '[]' > "$dir/export.json"
SH
  chmod +x "$s"
}

# Troca o "Baseline" da TASK-01 (pendente) por um comando que muda depois da tarefa: o arquivo de exportação ainda
# não existe no baseline e passa a existir quando o script é executado. Registra o ajuste.
changing_baseline() {
  local ph="$1/packages/harness"
  python3 - "$ph/work/tasks/SPEC-001/TASK-01.md" <<'PY'
import sys; p = sys.argv[1]; s = open(p, encoding='utf-8').read()
old = '- `packages/harness/scripts/default/memory/search-memory.sh decision memória`'
assert old in s
s = s.replace(old, '- `cat packages/harness/memory/export.json 2>/dev/null || echo ausente`', 1)
open(p, 'w', encoding='utf-8').write(s)
PY
  "$ph/skills/default/adjust-task/scripts/adjust-task.sh" --record SPEC-001 TASK-01 "Baseline que muda com a tarefa" > /dev/null
}

# Troca a "Verificação" da TASK-01 (pendente) por uma que depende de um arquivo do ambiente
# (`packages/harness/memory/.ambiente`) e registra o ajuste.
env_verify() {
  local ph="$1/packages/harness"
  python3 - "$ph/work/tasks/SPEC-001/TASK-01.md" <<'PY'
import re, sys; p = sys.argv[1]; s = open(p, encoding='utf-8').read()
v = '```bash\ns=packages/harness/scripts/default/memory/export-memory.sh\ntest -x "$s"\n"$s" decision\ntest "$(cat packages/harness/memory/export.json)" = "[]"\ntest -f packages/harness/memory/.ambiente   # o ambiente de exportação precisa estar preparado\n```'
s = re.sub(r'```bash\n.*?```', lambda m: v, s, count=1, flags=re.S)
open(p, 'w', encoding='utf-8').write(s)
PY
  "$ph/skills/default/adjust-task/scripts/adjust-task.sh" --record SPEC-001 TASK-01 "Verificação que depende do ambiente" > /dev/null
}

# Bloqueia a TASK-01 (em execução) por falha nos testes, nas duas tentativas de `max_attempts`, e depois prepara o
# ambiente (cria o arquivo de que a "Verificação" depende), como o usuário faria antes de pedir a retomada.
blocked_by_env() {
  local ph="$1/packages/harness" i
  for i in 1 2; do
    "$ph/skills/default/finish-task/scripts/finish-task.sh" --tests SPEC-001 TASK-01 > /dev/null 2>&1 || true
  done
  : > "$ph/memory/.ambiente"
}

# Executa os testes da TASK-01 (em execução), para que ela tenha resultado gravado.
tested() {
  "$1/packages/harness/skills/default/finish-task/scripts/finish-task.sh" --tests SPEC-001 TASK-01 > /dev/null
}

# Leva a SPEC-001 (tarefas aprovadas) a `implemented`, como se as três tarefas tivessem sido executadas: um
# repositório git com um commit anterior à aprovação, o script `export-memory.sh` e os demais caminhos de "Mudar"
# num commit posterior, liberados para o check-commits.sh, as tarefas `done` com o registro de execução e as
# evidências gravadas.
implemented() {
  local dir="$1" ph="$1/packages/harness" task tid today
  today="$(date -u +%Y-%m-%d)"
  gi() { git -C "$dir" -c user.name=Fixture -c user.email=f@f -c commit.gpgsign=false -c core.hooksPath=/dev/null "$@"; }
  gi init -q -b main
  gi add -A
  GIT_AUTHOR_DATE=2026-01-01T12:00:00Z GIT_COMMITTER_DATE=2026-01-01T12:00:00Z gi commit -qm "chore: base do projeto"
  mkdir -p "$dir/nao/existe"
  cat > "$ph/scripts/default/memory/export-memory.sh" <<'SH'
#!/usr/bin/env bash
# Exporta as entradas ativas de um tipo de memória em JSON, em packages/harness/memory/export.json.
# Uso: export-memory.sh <decision|issue|knowledge|all>
set -euo pipefail
H="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
case "${1:-}" in
  decision) files="decisions.md" ;; issue) files="issues.md" ;; knowledge) files="knowledge.md" ;;
  all) files="decisions.md issues.md knowledge.md" ;;
  *) echo "Erro: tipo inválido '${1:-}'; use decision, issue, knowledge ou all" >&2; exit 1 ;;
esac
for f in $files; do if [ -f "$H/memory/$f" ]; then cat "$H/memory/$f"; fi; done | awk '
  function esc(s) { gsub(/\\/, "\\\\", s); gsub(/"/, "\\\"", s); return s }
  function flush() { if (id != "" && status != "obsoleto") out = out (out == "" ? "" : ",") "{\"id\":\"" esc(id) "\",\"title\":\"" esc(title) "\",\"status\":\"" esc(status) "\"}"; id = "" }
  /^## [A-Z]+-[0-9]+: / { flush(); id = $2; sub(/:$/, "", id); title = $0; sub(/^## [A-Z]+-[0-9]+: /, "", title); status = ""; next }
  /^- \*\*Status:\*\* / { status = $0; sub(/^- \*\*Status:\*\* /, "", status) }
  END { flush(); print "[" out "]" }
' > "$H/memory/export.json"
SH
  chmod +x "$ph/scripts/default/memory/export-memory.sh"
  printf '#!/usr/bin/env bash\necho ok\n' > "$dir/nao/existe/novo.sh"
  printf '\n- Exportação da memória em JSON: `packages/harness/scripts/default/memory/export-memory.sh`.\n' >> "$dir/AGENTS.md"
  gi add -A
  gi commit -qm "feat(memory): export memory entries as JSON"
  "$ph/scripts/default/memory/export-memory.sh" decision
  mkdir -p "$ph/work/tasks/SPEC-001/evidence/TASK-01" "$ph/work/tasks/SPEC-001/evidence/TASK-02" "$ph/work/tasks/SPEC-001/evidence/TASK-03"
  cp "$ph/memory/export.json" "$ph/work/tasks/SPEC-001/evidence/TASK-01/export.txt"
  echo ok > "$ph/work/tasks/SPEC-001/evidence/TASK-02/saida.txt"
  echo ok > "$ph/work/tasks/SPEC-001/evidence/TASK-03/saida.txt"
  for task in "$ph"/work/tasks/SPEC-001/TASK-*.md; do
    tid="$(basename "$task" .md)"
    python3 - "$task" "$today" <<'PY'
import re, sys
p, d = sys.argv[1], sys.argv[2]
s = open(p, encoding='utf-8').read()
for k in ('tests_result', 'regression_result', 'evidence_result'):
    s = re.sub(r'^%s: .*$' % k, '%s: passed' % k, s, count=1, flags=re.M)
s = re.sub(r'^status: .*$', 'status: done', s, count=1, flags=re.M)
rows = ['início | in_progress | aprovada', 'baseline | coletado | 1 comando(s); códigos de saída: 0',
        'testes | passed | Verificação sem falhas', 'regressão | passed | 1 comando(s) iguais ao baseline',
        'evidências | passed | evidências gravadas', 'conclusão | done | testes, regressão e evidências passed; critérios de conclusão marcados']
table = ''.join('| %d | %s | %s |\n' % (i + 1, d, r) for i, r in enumerate(rows))
s = s.replace('| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n', '| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n' + table, 1)
s = re.sub(r'^- \[ \] ', '- [x] ', s, flags=re.M)
open(p, 'w', encoding='utf-8').write(s)
PY
  done
  f="$(ls "$ph"/work/specs/refined/SPEC-001-*.md)"
  sed -i.bak 's/^status: .*/status: implemented/' "$f" && rm -f "$f.bak"
  # A primeira verificação grava os commits da fixture como referência, como se já estivessem liberados.
  (cd "$dir" && packages/harness/scripts/default/git/check-commits.sh > /dev/null)
}

# Leva também a SPEC-002 a `implemented`, só pelo status, para o agente ter duas specs para escolher.
implemented_too() {
  local f
  approve "$1" SPEC-002
  f="$(ls "$1"/packages/harness/work/specs/refined/SPEC-002-*.md)"
  sed -i.bak 's/^status: .*/status: implemented/' "$f" && rm -f "$f.bak"
}

for scenario in ct01 ct02 ct03 ct04 ct05 ct06 ct09 ct13 ct14 ct15 ct16 ap01 ap02 ap03 ap04 ap13 \
  pl01 pl03 pl06 pl06b pl07 pl11 pl13a pl13b pl13c tk01 tk19 tk04 tk09 tk15 tk16 tk17a tk17b tk20 tk21 tk22 tk23 fn01 fn02 fn03; do
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
    tk15) approve "$DEST/$scenario"; plan "$DEST/$scenario"; planned "$DEST/$scenario"; failing_verify "$DEST/$scenario"; tasks_approved "$DEST/$scenario"; started "$DEST/$scenario" ;;
    tk16) approve "$DEST/$scenario"; plan "$DEST/$scenario"; planned "$DEST/$scenario"; independent "$DEST/$scenario"; tasks_approved "$DEST/$scenario" ;;
    tk20) approve "$DEST/$scenario"; plan "$DEST/$scenario"; planned "$DEST/$scenario"; tasks_approved "$DEST/$scenario"; started "$DEST/$scenario" ;;
    tk21) approve "$DEST/$scenario"; plan "$DEST/$scenario"; planned "$DEST/$scenario"; changing_baseline "$DEST/$scenario"; tasks_approved "$DEST/$scenario"; started "$DEST/$scenario" ;;
    tk22) approve "$DEST/$scenario"; plan "$DEST/$scenario"; planned "$DEST/$scenario"; env_verify "$DEST/$scenario"; tasks_approved "$DEST/$scenario"; started "$DEST/$scenario"; blocked_by_env "$DEST/$scenario" ;;
    tk23) approve "$DEST/$scenario"; plan "$DEST/$scenario"; planned "$DEST/$scenario"; tasks_approved "$DEST/$scenario"; started "$DEST/$scenario"; tested "$DEST/$scenario" ;;
    fn01) approve "$DEST/$scenario"; plan "$DEST/$scenario"; planned "$DEST/$scenario"; tasks_approved "$DEST/$scenario"; implemented "$DEST/$scenario" ;;
    fn02) approve "$DEST/$scenario"; plan "$DEST/$scenario"; planned "$DEST/$scenario"; tasks_approved "$DEST/$scenario"; implemented "$DEST/$scenario"; implemented_too "$DEST/$scenario" ;;
    fn03) approve "$DEST/$scenario"; plan "$DEST/$scenario"; planned "$DEST/$scenario" ;;
  esac
  echo "$DEST/$scenario"
done
rm -rf "$B"
