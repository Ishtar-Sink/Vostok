#!/usr/bin/env bash
# Monta as cópias descartáveis dos cenários com agente de apps/cli/tests/agent-scenarios.md.
# Cada cenário recebe um projeto novo (`git init` e `vostok init --yes`), com o harness instalado em `vostok/` e:
#   SPEC-001 Exportar memória em JSON (refinada), SPEC-002 Listar tarefas (refinada)
#   e SPEC-003 Painel de métricas (rascunho). Alguns cenários removem specs (ver `prune`),
#   partem de specs aprovadas (ver `approve`), de um plano de tarefas (ver `plan`), da
#   SPEC-001 já planejada (ver `planned`), das tarefas dela aprovadas (ver `tasks_approved`,
#   `failing_verify` e `independent`), da TASK-01 em execução (ver `started`, `changing_baseline`,
#   `env_verify` e `blocked_by_env`), da SPEC-001 implementada (ver `implemented` e `implemented_too`) ou de
#   um projeto com código, com banco, sem código ou já inicializado (ver `node_project`, `db_project`,
#   `empty_project` e `initialized_project`).
# O estado de cada cenário é montado pelos subcomandos do `vostok`, executados de dentro da cópia.
#
# Uso: apps/cli/tests/fixtures/agent-scenarios.sh <pasta de destino> [cenário ...]
#      Cria <destino>/<cenário> para cada cenário (todos, sem a lista) e imprime os caminhos.
set -euo pipefail

[ -n "${1:-}" ] || { echo "Uso: agent-scenarios.sh <pasta de destino> [cenário ...]" >&2; exit 1; }
command -v vostok > /dev/null || { echo "Erro: vostok não encontrado no PATH" >&2; exit 1; }
FIXTURES="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEST="$(mkdir -p "$1" && cd "$1" && pwd)"
shift
B="$DEST/base"

# Executa o vostok de dentro de uma cópia: V <pasta> <subcomando ...>.
V() { local dir="$1"; shift; (cd "$dir" && vostok "$@"); }
F() { python3 "$FIXTURES/fill-refined.py" "$@"; }
gi() { local dir="$1"; shift; git -C "$dir" -c user.name=Fixture -c user.email=f@f -c commit.gpgsign=false -c core.hooksPath=/dev/null "$@"; }

rm -rf "$B" && mkdir -p "$B"
gi "$B" init -q -b main
V "$B" init --yes > /dev/null
H="$B/vostok"

V "$B" spec new 'Exportar memória em JSON' >/dev/null; V "$B" spec refine SPEC-001 feature >/dev/null; f="$(ls "$H"/work/specs/refined/SPEC-001-*.md)"
F "$f" 'SPEC_SUMMARY=Cria o script `scripts/export-memory.sh`, que exporta as entradas da memória do agente em JSON.' \
 'OBJECTIVE=Hoje a memória do agente só pode ser lida em Markdown. O painel web precisa dela em JSON para exibir decisões, issues e conhecimentos.' \
 'SCOPE_IN=- Script `export-memory.sh` em `scripts/`.\n- Exportação por tipo e de todos os tipos.' \
 'SCOPE_OUT=- Importação de JSON para a memória.\n- Envio do JSON ao servidor HTTP.' \
 'FUNCTIONAL_REQUIREMENTS=| RF-01 | O script deve exportar as entradas de um tipo de memória em JSON, com ID, título, status e tags. |\n| RF-02 | O script deve aceitar o tipo `all` para exportar todos os tipos num único arquivo. |\n| RF-03 | O script deve gravar a saída em `vostok/memory/export.json`. |' \
 'NON_FUNCTIONAL_REQUIREMENTS=| RNF-01 | Roda em bash 3.2 com awk e sed do BSD. | Suíte executada no macOS. |\n| RNF-02 | Monta o JSON com awk, sem dependências externas. | Revisão do script. |' \
 'BUSINESS_RULES=| RN-01 | Entradas com status `obsoleto` não são exportadas. |' \
 'MAIN_FLOW=1. O usuário executa `scripts/export-memory.sh <tipo>`.\n2. O script lê as entradas do tipo.\n3. O script grava o JSON em `vostok/memory/export.json`.' \
 'ALTERNATIVE_FLOWS=- **FA-01 (passo 1):** tipo inválido. O script termina com erro e lista os tipos.' \
 'EDGE_CASES=| CB-01 | Memória vazia | Grava `[]` | RF-01 |\n| CB-02 | Título com aspas | Aspas escapadas no JSON | RF-01 |' \
 'CONTRACTS=```bash\nscripts/export-memory.sh <tipo>   # decision, issue, knowledge ou all\n```' \
 'AFFECTED_COMPONENTS=| `scripts/export-memory.sh` | criar | Script de exportação. |' \
 'ASSUMPTIONS=- As entradas seguem o formato de `vostok/config/memory.yml`.' \
 'RISKS=| Título com caracteres especiais | médio | Escapar aspas e barras invertidas. |' \
 'TEST_CASES=| CT-01 | Exportar decisões | Executar com `decision` | JSON com as decisões | RF-01, RF-03 |\n| CT-02 | Exportar tudo | Executar com `all` | JSON com os três tipos | RF-02 |' \
 'ACCEPTANCE_CRITERIA=- [ ] CA-01: `scripts/export-memory.sh decision` gera JSON com as decisões ativas (RF-01, RF-03).\n- [ ] CA-02: `scripts/export-memory.sh all` gera um único arquivo com os três tipos (RF-02).' \
 'REFERENCES=| arquivo | `vostok/config/memory.yml` | Tipos e status da memória. |' \
 'WHAT_CHANGED=| 1 | Formato | Não definido | JSON com ID, título, status e tags |' \
 'DECISIONS=| 1 | Saída em arquivo fixo | Saída padrão | inferência A |' \
 'PENDING=| - | Nenhuma | - |'
V "$B" spec mark-refined SPEC-001 >/dev/null
V "$B" spec new 'Listar tarefas' >/dev/null; V "$B" spec refine SPEC-002 feature >/dev/null; g="$(ls "$H"/work/specs/refined/SPEC-002-*.md)"
F "$g" 'SPEC_SUMMARY=Cria o script `scripts/list-tasks.sh`, que lista as tarefas de uma spec.' 'OBJECTIVE=O agente precisa listar as tarefas geradas de uma spec para acompanhar a execução.' \
 'FUNCTIONAL_REQUIREMENTS=| RF-01 | O script deve listar as tarefas de uma spec em tabela, com ID, título e status. |\n| RF-02 | O script deve aceitar o ID da spec como argumento obrigatório. |' \
 'AFFECTED_COMPONENTS=| `scripts/list-tasks.sh` | criar | Script de listagem. |' \
 'TEST_CASES=| CT-01 | Listar | Executar com SPEC-001 | Tabela com as tarefas | RF-01, RF-02 |' 'ACCEPTANCE_CRITERIA=- [ ] CA-01: `scripts/list-tasks.sh SPEC-001` lista as tarefas (RF-01, RF-02).'
V "$B" spec mark-refined SPEC-002 >/dev/null
V "$B" spec new 'Painel de métricas' >/dev/null

# Remove as specs informadas (rascunho e refinada) de um cenário.
prune() {
  local dir="$1" id
  shift
  for id in "$@"; do rm -f "$dir"/vostok/work/specs/*/"$id"-*.md; done
}

# Aprova as specs informadas de um cenário (SPEC-001 sem argumentos).
approve() {
  local dir="$1" id
  shift
  for id in "${@:-SPEC-001}"; do
    V "$dir" spec approve approve "$id" > /dev/null
  done
}

# Cria e pontua o plano de tarefas da SPEC-001 (aprovada) de um cenário:
# TASK-01 exporta um tipo, TASK-02 exporta todos e TASK-03 reúne os testes.
plan() {
  local dir="$1"
  V "$dir" plan create SPEC-001 > /dev/null
  F "$dir/vostok/work/plans/SPEC-001.md" \
   'PLAN_OVERVIEW=O trabalho começa pelo script que exporta um tipo de memória (TASK-01), base das demais. A exportação de todos os tipos (TASK-02) reaproveita esse script. Os testes (TASK-03) fecham o plano e dependem das duas tarefas.' \
   'PLAN_TASKS=| TASK-01 | Exportar um tipo de memória | Cria `scripts/export-memory.sh` com a exportação de um tipo e a gravação em `vostok/memory/export.json`. | tarefa | - | - | RF-01, RF-03 |\n| TASK-02 | Exportar todos os tipos | Acrescenta o tipo `all`, que junta os três tipos num único arquivo. | tarefa | - | TASK-01 | RF-02 |\n| TASK-03 | Testes da exportação | Cobre a exportação de um tipo e de todos na suíte de testes. | tarefa | - | TASK-01, TASK-02 | CT-01, CT-02 |' \
   'PLAN_FACTS=| TASK-01 | 1 | 0 | sim | não | não |\n| TASK-02 | 0 | 1 | não | não | não |\n| TASK-03 | 0 | 1 | não | não | não |'
  V "$dir" plan score SPEC-001 > /dev/null
}

# Aprova o plano da SPEC-001 com tarefas preenchidas pela fixture e marca a spec como planejada.
# A TASK-01 é detalhada de acordo com a spec; as demais usam os valores padrão de fill-refined.py.
planned() {
  local dir="$1" task
  V "$dir" plan approve create-tasks SPEC-001 > /dev/null
  F "$dir/vostok/work/tasks/SPEC-001/TASK-01.md" \
   'TASK_READ=- `vostok/config/memory.yml`\n- `vostok/memory/`' \
   'TASK_CHANGE=- `scripts/export-memory.sh` (criar): exporta um tipo de memória em JSON.' \
   'TASK_STEPS=1. Criar `scripts/export-memory.sh`, que lê as entradas ativas do tipo e grava em `vostok/memory/export.json` uma lista JSON (`[]` quando não há entradas).\n2. Com tipo inválido, terminar com erro e listar os tipos.' \
   'TASK_BASELINE=- `vostok memory search decision memória`' \
   'TASK_VERIFY=```bash\ns=scripts/export-memory.sh\ntest -x "$s"\n"$s" decision\ntest "$(cat vostok/memory/export.json)" = "[]"   # memória vazia (CB-01)\nif "$s" nada; then exit 1; fi   # tipo inválido (FA-01)\n```' \
   'TASK_EVIDENCE=- `export.txt` (automática): `cat vostok/memory/export.json`' \
   'TASK_DONE=- [ ] O script exporta um tipo de memória em JSON (RF-01, RF-03).\n\n  > Como comprovar: com a memória vazia, exportar as decisões grava uma lista vazia no arquivo de exportação.\n- [ ] Tipo inválido termina com erro (FA-01).\n\n  > Como comprovar: pedir a exportação de um tipo que não existe termina com erro.' \
   'TASK_EXECUTIVE_SUMMARY=Permite exportar as decisões, issues ou conhecimentos do agente em JSON para o painel web.' \
   'TASK_TECHNICAL_DETAILS=Script bash que acha o arquivo do tipo em `vostok/config/memory.yml` e monta o JSON com awk (RF-01, RF-03, FA-01).' \
   'TASK_CARD_DESCRIPTION=Cria o script que exporta um tipo de memória em JSON.' \
   'TASK_CARD_CRITERIA=  - [ ] Exporta um tipo em JSON.\n  - [ ] Tipo inválido termina com erro.'
  for task in "$dir"/vostok/work/tasks/SPEC-001/*.md; do F "$task"; done
  V "$dir" plan approve mark-planned SPEC-001 > /dev/null
}

# Aprova todas as tarefas da SPEC-001 (planejada) de um cenário.
tasks_approved() {
  V "$1" task approve approve SPEC-001 --all > /dev/null
}

# Troca a "Verificação" da TASK-01 (pendente) por uma que nunca passa e registra o ajuste.
failing_verify() {
  python3 - "$1/vostok/work/tasks/SPEC-001/TASK-01.md" <<'PY'
import re, sys; p = sys.argv[1]; s = open(p, encoding='utf-8').read()
s = re.sub(r'```bash\n.*?```', lambda m: '```bash\ntest "$(date +%Y)" = 1999   # o relógio da máquina está em 1999\n```', s, count=1, flags=re.S)
open(p, 'w', encoding='utf-8').write(s)
PY
  V "$1" task adjust record SPEC-001 TASK-01 "Verificação que nunca passa" > /dev/null
}

# Tira a dependência da TASK-02 (pendente), para que ela e a TASK-01 sejam executáveis juntas.
independent() {
  local file="$1/vostok/work/tasks/SPEC-001/TASK-02.md"
  sed -i.bak 's/^depends_on: .*/depends_on: []/; s/^- \*\*Bloqueada por:\*\* .*/- **Bloqueada por:** -/' "$file" && rm -f "$file.bak"
  V "$1" task adjust record SPEC-001 TASK-02 "Sem dependência" > /dev/null
}

# Inicia a TASK-01 (aprovada), coleta o baseline e escreve o `scripts/export-memory.sh`, como a `execute-task`
# deixa a tarefa: em execução, com o código implementado e sem nenhuma validação.
started() {
  local dir="$1" s
  V "$dir" task start SPEC-001 TASK-01 > /dev/null
  V "$dir" task baseline SPEC-001 TASK-01 > /dev/null
  mkdir -p "$dir/scripts"
  s="$dir/scripts/export-memory.sh"
  cat > "$s" <<'SH'
#!/usr/bin/env bash
# Exporta as entradas ativas de um tipo de memória em JSON, em vostok/memory/export.json.
set -euo pipefail
case "${1:-}" in
  decision|issue|knowledge|all) ;;
  *) echo "Tipo inválido: ${1:-}. Tipos: decision, issue, knowledge, all." >&2; exit 1 ;;
esac
dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/vostok/memory"
mkdir -p "$dir"
echo '[]' > "$dir/export.json"
SH
  chmod +x "$s"
}

# Troca o "Baseline" da TASK-01 (pendente) por um comando que muda depois da tarefa: o arquivo de exportação ainda
# não existe no baseline e passa a existir quando o script é executado. Registra o ajuste.
changing_baseline() {
  python3 - "$1/vostok/work/tasks/SPEC-001/TASK-01.md" <<'PY'
import sys; p = sys.argv[1]; s = open(p, encoding='utf-8').read()
old = '- `vostok memory search decision memória`'
assert old in s
s = s.replace(old, '- `cat vostok/memory/export.json 2>/dev/null || echo ausente`', 1)
open(p, 'w', encoding='utf-8').write(s)
PY
  V "$1" task adjust record SPEC-001 TASK-01 "Baseline que muda com a tarefa" > /dev/null
}

# Troca a "Verificação" da TASK-01 (pendente) por uma que depende de um arquivo do ambiente
# (`vostok/memory/.ambiente`) e registra o ajuste.
env_verify() {
  python3 - "$1/vostok/work/tasks/SPEC-001/TASK-01.md" <<'PY'
import re, sys; p = sys.argv[1]; s = open(p, encoding='utf-8').read()
v = '```bash\ns=scripts/export-memory.sh\ntest -x "$s"\n"$s" decision\ntest "$(cat vostok/memory/export.json)" = "[]"\ntest -f vostok/memory/.ambiente   # o ambiente de exportação precisa estar preparado\n```'
s = re.sub(r'```bash\n.*?```', lambda m: v, s, count=1, flags=re.S)
open(p, 'w', encoding='utf-8').write(s)
PY
  V "$1" task adjust record SPEC-001 TASK-01 "Verificação que depende do ambiente" > /dev/null
}

# Bloqueia a TASK-01 (em execução) por falha nos testes, nas duas tentativas de `max_attempts`, e depois prepara o
# ambiente (cria o arquivo de que a "Verificação" depende), como o usuário faria antes de pedir a retomada.
blocked_by_env() {
  local i
  for i in 1 2; do
    V "$1" task tests SPEC-001 TASK-01 > /dev/null 2>&1 || true
  done
  : > "$1/vostok/memory/.ambiente"
}

# Executa os testes da TASK-01 (em execução), para que ela tenha resultado gravado.
tested() {
  V "$1" task tests SPEC-001 TASK-01 > /dev/null
}

# Leva a SPEC-001 (tarefas aprovadas) a `implemented`, como se as três tarefas tivessem sido executadas: um
# commit anterior à aprovação, o script `scripts/export-memory.sh` e os demais caminhos de "Mudar" num commit
# posterior, liberados para o `vostok commit check`, as tarefas `done` com o registro de execução e as
# evidências gravadas.
implemented() {
  local dir="$1" ev="$1/vostok/work/tasks/SPEC-001/evidence" task today
  today="$(date -u +%Y-%m-%d)"
  gi "$dir" add -A
  GIT_AUTHOR_DATE=2026-01-01T12:00:00Z GIT_COMMITTER_DATE=2026-01-01T12:00:00Z gi "$dir" commit -qm "chore: base do projeto"
  mkdir -p "$dir/nao/existe" "$dir/scripts"
  cat > "$dir/scripts/export-memory.sh" <<'SH'
#!/usr/bin/env bash
# Exporta as entradas ativas de um tipo de memória em JSON, em vostok/memory/export.json.
# Uso: scripts/export-memory.sh <decision|issue|knowledge|all>
set -euo pipefail
H="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/vostok"
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
  chmod +x "$dir/scripts/export-memory.sh"
  printf '#!/usr/bin/env bash\necho ok\n' > "$dir/nao/existe/novo.sh"
  printf '\n- Exportação da memória em JSON: `scripts/export-memory.sh`.\n' >> "$dir/AGENTS.md"
  gi "$dir" add -A
  # O commit da tarefa precisa ser posterior à aprovação: a base dos diffs é o último commit até o `approved_at`,
  # que tem precisão de segundos.
  sleep 1
  gi "$dir" commit -qm "feat(memory): export memory entries as JSON"
  "$dir/scripts/export-memory.sh" decision
  mkdir -p "$ev/TASK-01" "$ev/TASK-02" "$ev/TASK-03"
  cp "$dir/vostok/memory/export.json" "$ev/TASK-01/export.txt"
  echo ok > "$ev/TASK-02/saida.txt"
  echo ok > "$ev/TASK-03/saida.txt"
  for task in "$dir"/vostok/work/tasks/SPEC-001/TASK-*.md; do
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
  f="$(ls "$dir"/vostok/work/specs/refined/SPEC-001-*.md)"
  sed -i.bak 's/^status: .*/status: implemented/' "$f" && rm -f "$f.bak"
  # A primeira verificação grava os commits da fixture como referência, como se já estivessem liberados.
  V "$dir" commit check > /dev/null
}

# Leva também a SPEC-002 a `implemented`, só pelo status, para o agente ter duas specs para escolher.
implemented_too() {
  local f
  approve "$1" SPEC-002
  f="$(ls "$1"/vostok/work/specs/refined/SPEC-002-*.md)"
  sed -i.bak 's/^status: .*/status: implemented/' "$f" && rm -f "$f.bak"
}

# Projeto Node/TypeScript com código: `package.json` (TypeScript, ESLint e Vitest), `.eslintrc.json`,
# `vitest.config.ts` e `src/`. Os sinais ficam na raiz do projeto, onde o `vostok rules detect` procura.
node_project() {
  local dir="$1"
  mkdir -p "$dir/src"
  cat > "$dir/package.json" <<'JSON'
{
  "name": "exemplo-node",
  "version": "1.0.0",
  "private": true,
  "scripts": { "lint": "eslint src", "test": "vitest run" },
  "devDependencies": { "typescript": "^5.4.0", "eslint": "^9.0.0", "vitest": "^2.0.0" }
}
JSON
  cat > "$dir/.eslintrc.json" <<'JSON'
{ "root": true, "extends": ["eslint:recommended"], "rules": { "semi": ["error", "always"], "quotes": ["error", "single"] } }
JSON
  cat > "$dir/vitest.config.ts" <<'TS'
import { defineConfig } from 'vitest/config';

export default defineConfig({ test: { include: ['tests/**/*.test.ts'] } });
TS
  printf "export function slugify(text: string): string {\n  return text.toLowerCase().trim().replace(/\\s+/g, '-');\n}\n" > "$dir/src/slugify.ts"
}

# Projeto com banco de dados: pasta `migrations/` e um ORM (TypeORM) no `package.json`.
db_project() {
  local dir="$1"
  mkdir -p "$dir/migrations" "$dir/src"
  cat > "$dir/package.json" <<'JSON'
{
  "name": "exemplo-db",
  "version": "1.0.0",
  "private": true,
  "dependencies": { "typeorm": "^0.3.20", "pg": "^8.11.0" },
  "devDependencies": { "typescript": "^5.4.0" }
}
JSON
  printf 'CREATE TABLE users (\n  id SERIAL PRIMARY KEY,\n  name TEXT NOT NULL\n);\n' > "$dir/migrations/001_create_users.sql"
}

# Projeto sem código: nenhum manifesto, configuração nem pasta de código (só o harness e as specs de exemplo).
empty_project() {
  local dir="$1"
  rm -rf "$dir/src" "$dir/migrations" "$dir/tests" "$dir/package.json" "$dir/.eslintrc.json" "$dir/vitest.config.ts"
}

# Projeto já inicializado: `rules/project/testing.md` com três regras, carregada na `execute-task` e na `finish-task`.
# Grava pelo próprio `vostok rules`, como a skill faria.
initialized_project() {
  local rules
  rules="$(mktemp)"
  cat > "$rules" <<'TXT'
titulo: Testes
proposito: Define como os testes do projeto são escritos e executados.
Testes unitários com Vitest, em arquivos `*.test.ts` dentro de `tests/`.
Todo teste novo é determinístico, sem depender de rede nem do horário.
Cada teste verifica um único comportamento.
TXT
  V "$1" rules write testing "$rules" > /dev/null
  V "$1" rules load testing execute-task,finish-task > /dev/null
  rm -f "$rules"
}

ALL="ct01 ct02 ct03 ct04 ct05 ct06 ct09 ct13 ct14 ct15 ct16 ap01 ap02 ap03 ap04 ap13
  pl01 pl03 pl06 pl06b pl07 pl11 pl13a pl13b pl13c tk01 tk19 tk04 tk09 tk15 tk16 tk17a tk17b tk20 tk21 tk22 tk23 fn01 fn02 fn03
  rl01 rl02 rl03 rl04 rl05 rl06 rl07 rl08 rl09"
for scenario in ${*:-$ALL}; do
  case " $(echo $ALL) " in *" $scenario "*) ;; *) echo "Erro: cenário desconhecido: $scenario" >&2; exit 1 ;; esac
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
    rl01|rl03|rl04) node_project "$DEST/$scenario" ;;
    rl02) db_project "$DEST/$scenario" ;;
    rl05|rl06) node_project "$DEST/$scenario"; initialized_project "$DEST/$scenario" ;;
    rl07|rl08) initialized_project "$DEST/$scenario" ;;
    rl09) empty_project "$DEST/$scenario" ;;
  esac
  echo "$DEST/$scenario"
done
rm -rf "$B"
