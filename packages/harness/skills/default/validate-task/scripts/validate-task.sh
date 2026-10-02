#!/usr/bin/env bash
# Valida uma tarefa em execução: testes da "Verificação", regressão contra o baseline e retomada de uma
# tarefa bloqueada. Cada etapa ganha uma linha no "Registro de execução". As saídas ficam em
# `paths.tasks`/<SPEC_ID>/`evidence_dir`/<TASK_ID>/validation/.
#
# Uso: validate-task.sh --tests <SPEC_ID> <TASK_ID>
#        Só em `in_progress` ou `validating`. Executa o bloco `bash` de "Verificação" a partir da raiz do
#        projeto, parando no primeiro comando com falha. Em sucesso, `tests_result: passed`. Em falha,
#        `tests_result: failed`, regressão e evidências de volta a `null` e `attempts` + 1; ao atingir
#        `max_attempts`, a tarefa fica `blocked`. Termina com erro se os testes falharem.
#      validate-task.sh --regression <SPEC_ID> <TASK_ID>
#        Só em `in_progress` ou `validating`, com o baseline coletado. Executa de novo cada comando de
#        "Baseline" e compara o código de saída e a saída. Com diferença, `regression_result: failed`, a
#        tarefa fica `blocked` e os comandos são listados. Termina com erro se houver regressão.
#      validate-task.sh --resume <SPEC_ID> <TASK_ID> "decisão do usuário"
#        Só em `blocked`. Volta a `validating`, zera `attempts` e os resultados e registra a decisão.
set -euo pipefail

# Raiz do harness: primeira pasta acima do script que contém `config/main.yml`.
HARNESS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && until [ -f config/main.yml ] || [ "$PWD" = / ]; do cd ..; done && pwd)"
source "$HARNESS_DIR/scripts/lib/config.sh"
source "$HARNESS_DIR/scripts/lib/text.sh"
source "$HARNESS_DIR/scripts/lib/frontmatter.sh"
source "$HARNESS_DIR/scripts/lib/markdown.sh"
source "$HARNESS_DIR/scripts/lib/spec-validation.sh"
source "$HARNESS_DIR/scripts/lib/adjustments.sh"
source "$HARNESS_DIR/scripts/lib/plan.sh"
source "$HARNESS_DIR/scripts/lib/task.sh"

MAIN_CONFIG="$HARNESS_DIR/config/main.yml"
TASKS_CONFIG="$HARNESS_DIR/config/tasks.yml"

USAGE='Uso: validate-task.sh --tests <SPEC_ID> <TASK_ID> | --regression <SPEC_ID> <TASK_ID> | --resume <SPEC_ID> <TASK_ID> "decisão"'

die() { echo "Erro: $*" >&2; exit 1; }

tasks_root="$(harness_path "$(config_get "$MAIN_CONFIG" paths.tasks)")"
in_progress_status="$(config_get "$TASKS_CONFIG" status.in_progress)"
validating_status="$(config_get "$TASKS_CONFIG" status.validating)"
blocked_status="$(config_get "$TASKS_CONFIG" status.blocked)"
max_attempts="$(config_get "$TASKS_CONFIG" max_attempts)"
task_prefix="$(config_get "$TASKS_CONFIG" id.prefix)"

# Caminho da tarefa. Falha se os IDs forem inválidos ou a tarefa não existir.
# find_task <SPEC_ID> <TASK_ID>
find_task() {
  [[ "$1" =~ ^[A-Z]+-[0-9]+$ ]] || die "ID de spec inválido: '$1'"
  [[ "$2" =~ ^$task_prefix-[0-9]+$ ]] || die "ID de tarefa inválido: '$2'"
  [ -f "$tasks_root/$1/$2.md" ] || die "tarefa $2 não encontrada em $1"
  printf '%s' "$tasks_root/$1/$2.md"
}

status_of() {
  frontmatter_get "$1" status 2>/dev/null || true
}

# Recusa tarefa fora de in_progress e validating; a bloqueada só volta pelo --resume.
# require_running <arquivo> <TASK_ID>
require_running() {
  local status
  status="$(status_of "$1")"
  [ "$status" != "$blocked_status" ] ||
    die "a tarefa $2 está bloqueada; mostre as falhas ao usuário e, com a decisão dele, execute: validate-task.sh --resume"
  [ "$status" = "$in_progress_status" ] || [ "$status" = "$validating_status" ] ||
    die "a tarefa $2 está com status '$status'; só tarefas com status '$in_progress_status' ou '$validating_status' são validadas"
}

# Comandos do bloco `bash` de "Verificação".
# verify_block <arquivo>
verify_block() {
  task_subsection "$1" verify_subsection | awk '
    /^```bash[[:space:]]*$/ && !done { inside = 1; next }
    inside && /^```[[:space:]]*$/ { inside = 0; done = 1; next }
    inside { print }
  '
}

# Executa um comando a partir da raiz do projeto e imprime o comando, o código de saída e a saída,
# no formato do baseline.
# run_recorded <comando>
run_recorded() {
  local out code
  out="$(mktemp)"
  set +e
  (cd "$(_task_root_dir)" && bash -c "$1") > "$out" 2>&1 < /dev/null
  code=$?
  set -e
  printf '$ %s\nexit: %s\n' "$1" "$code"
  cat "$out"
  rm -f "$out"
}

tests() {
  local file block dir code attempts
  file="$(find_task "$1" "$2")"
  require_running "$file" "$2"
  block="$(verify_block "$file")"
  [[ "$block" =~ [^[:space:]] ]] || die "a tarefa $2 não tem um bloco \`bash\` em '$(config_get "$TASKS_CONFIG" verify_subsection)'"

  dir="$(task_evidence_path "$file")/validation"
  mkdir -p "$dir"
  set +e
  # O trap registra na saída o comando que falhou, já que muitos comandos de teste falham sem imprimir nada.
  (cd "$(_task_root_dir)" && bash -c "set -eo pipefail; trap 'echo \"comando com falha (código \$?): \$BASH_COMMAND\"' ERR"$'\n'"$block") \
    > "$dir/tests.out" 2>&1 < /dev/null
  code=$?
  set -e

  if [ "$code" -eq 0 ]; then
    frontmatter_set "$file" status "$validating_status"
    frontmatter_set "$file" tests_result passed
    task_record_step "$file" testes passed "Verificação sem falhas; saída em validation/tests.out"
    echo "Testes de $2 passaram. Saída: $dir/tests.out"
    return
  fi

  attempts="$(frontmatter_get "$file" attempts 2>/dev/null || true)"
  [[ "$attempts" =~ ^[0-9]+$ ]] || attempts=0
  attempts=$((attempts + 1))
  frontmatter_set "$file" tests_result failed
  frontmatter_set "$file" regression_result null
  frontmatter_set "$file" evidence_result null
  frontmatter_set "$file" attempts "$attempts"
  echo "Testes de $2 falharam (código de saída $code), tentativa $attempts de $max_attempts. Últimas linhas da saída ($dir/tests.out):"
  tail -n 30 "$dir/tests.out" | sed 's/^/    /'
  if [ "$attempts" -ge "$max_attempts" ]; then
    frontmatter_set "$file" status "$blocked_status"
    task_record_step "$file" testes failed "tentativa $attempts de $max_attempts; bloqueada"
    die "a tarefa $2 ficou bloqueada depois de $attempts tentativas. PARE: responda com o template \`blocker\`, mostrando as falhas, e aguarde a decisão do usuário"
  fi
  frontmatter_set "$file" status "$validating_status"
  task_record_step "$file" testes failed "tentativa $attempts de $max_attempts"
  die "os testes de $2 falharam; corrija e execute --tests de novo"
}

regression() {
  local file base dir commands cmd n=0 name changed=""
  file="$(find_task "$1" "$2")"
  require_running "$file" "$2"
  base="$(task_evidence_path "$file")/baseline"
  [ -d "$base" ] || die "o baseline de $2 não foi coletado; execute: execute-task.sh --baseline $1 $2"
  commands="$(task_baseline_commands "$file")"
  dir="$(task_evidence_path "$file")/validation/regression"
  rm -rf "$dir"
  mkdir -p "$dir"

  while IFS= read -r cmd; do
    [ -n "$cmd" ] || continue
    n=$((n + 1))
    name="$(printf '%02d' "$n").out"
    run_recorded "$cmd" > "$dir/$name"
    if [ ! -f "$base/$name" ]; then
      changed="$changed- $cmd: sem resultado no baseline"$'\n'
    elif ! cmp -s "$base/$name" "$dir/$name"; then
      changed="$changed- $cmd"$'\n'"$( { diff "$base/$name" "$dir/$name" || true; } | sed -n '2,12p' | sed 's/^/    /')"$'\n'
    fi
  done <<< "$commands"
  for name in "$base"/*.out; do
    [ -f "$dir/$(basename "$name")" ] || changed="$changed- $(head -n 1 "$name" | sed 's/^\$ //'): comando do baseline que não existe mais"$'\n'
  done

  if [ -z "$changed" ]; then
    frontmatter_set "$file" status "$validating_status"
    frontmatter_set "$file" regression_result passed
    task_record_step "$file" regressão passed "$n comando(s) iguais ao baseline"
    echo "Sem regressão em $2: $n comando(s) iguais ao baseline."
    return
  fi
  frontmatter_set "$file" regression_result failed
  frontmatter_set "$file" status "$blocked_status"
  task_record_step "$file" regressão failed "comandos diferentes do baseline; bloqueada"
  printf 'Comandos com resultado diferente do baseline (saídas em %s):\n%s' "$dir" "$changed"
  die "a tarefa $2 ficou bloqueada por regressão. PARE: responda com o template \`blocker\`, mostrando os comandos, e aguarde a decisão do usuário"
}

resume() {
  local file status decision
  file="$(find_task "$1" "$2")"
  status="$(status_of "$file")"
  [ "$status" = "$blocked_status" ] ||
    die "a tarefa $2 está com status '$status'; só tarefas com status '$blocked_status' são retomadas"
  decision="$(printf '%s' "$3" | sed -E 's/^[[:space:]]+//; s/[[:space:]]+$//')"
  [ -n "$decision" ] || die "informe a decisão do usuário"
  [[ "$decision" != *$'\n'* ]] || die "a decisão não pode ter quebra de linha"
  frontmatter_set "$file" status "$validating_status"
  frontmatter_set "$file" attempts 0
  frontmatter_set "$file" tests_result null
  frontmatter_set "$file" regression_result null
  frontmatter_set "$file" evidence_result null
  task_record_step "$file" retomada "$validating_status" "decisão do usuário: $decision"
  echo "Tarefa $2 retomada: status '$validating_status', tentativas e resultados zerados. Aplique a decisão e valide de novo, começando por --tests."
}

case "${1:-}" in
  --tests) [ "$#" -eq 3 ] || die "$USAGE"; tests "$2" "$3" ;;
  --regression) [ "$#" -eq 3 ] || die "$USAGE"; regression "$2" "$3" ;;
  --resume) [ "$#" -eq 4 ] || die "$USAGE"; resume "$2" "$3" "$4" ;;
  *) die "$USAGE" ;;
esac
