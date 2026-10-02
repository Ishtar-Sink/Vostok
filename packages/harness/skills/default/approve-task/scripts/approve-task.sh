#!/usr/bin/env bash
# Aprova tarefas de uma spec. Cada tarefa precisa estar com status `pending`, sem edição manual sem registro
# (conteúdo igual ao hash gravado) e sem problemas na validação da tarefa (scripts/lib/task.sh). Sem
# problemas, grava status `approved`, `approved_at` e `approved_hash` em todas; com qualquer problema, não
# altera nenhuma.
#
# Uso: approve-task.sh --validate <SPEC_ID> <TASK_ID ...>|--all
#        Lista os problemas das tarefas, sem alterar nada. Termina com erro se houver algum.
#      approve-task.sh --approve <SPEC_ID> <TASK_ID ...>|--all
#        Aprova as tarefas citadas, ou todas as `pending` da spec com --all (só com pedido explícito do
#        usuário de aprovar todas). Sem tarefas nem --all, recusa.
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
SPECS_CONFIG="$HARNESS_DIR/config/specs.yml"
TASKS_CONFIG="$HARNESS_DIR/config/tasks.yml"

USAGE="Uso: approve-task.sh --validate|--approve <SPEC_ID> <TASK_ID ...>|--all"

die() { echo "Erro: $*" >&2; exit 1; }

tasks_root="$(harness_path "$(config_get "$MAIN_CONFIG" paths.tasks)")"
pending_status="$(config_get "$TASKS_CONFIG" status.pending)"
approved_status="$(config_get "$TASKS_CONFIG" status.approved)"
task_prefix="$(config_get "$TASKS_CONFIG" id.prefix)"
hash_field="$(config_get "$SPECS_CONFIG" content_hash_field)"
approved_at_field="$(config_get "$SPECS_CONFIG" approved_at_field)"
approved_hash_field="$(config_get "$SPECS_CONFIG" approved_hash_field)"

# Arquivos das tarefas pedidas, um por linha, sem repetição.
# selected_tasks <SPEC_ID> <TASK_ID ...>|--all
selected_tasks() {
  local spec_id="$1" dir file id found=""
  shift
  [[ "$spec_id" =~ ^[A-Z]+-[0-9]+$ ]] || die "ID de spec inválido: '$spec_id'"
  dir="$tasks_root/$spec_id"
  [ -d "$dir" ] || die "a spec $spec_id não tem tarefas em $dir"
  [ "$#" -gt 0 ] || die "informe as tarefas a aprovar ou --all (só com pedido explícito de aprovar todas)"$'\n'"$USAGE"

  if [ "$1" = --all ]; then
    [ "$#" -eq 1 ] || die "use --all ou TASK_IDs, não os dois"
    for file in "$dir"/*.md; do
      [ -f "$file" ] || continue
      [ "$(frontmatter_get "$file" status 2>/dev/null || true)" = "$pending_status" ] || continue
      found="$found$file"$'\n'
    done
    [ -n "$found" ] || die "a spec $spec_id não tem tarefas com status '$pending_status'"
  else
    for id in "$@"; do
      [[ "$id" =~ ^$task_prefix-[0-9]+$ ]] || die "ID de tarefa inválido: '$id'"
      [ -f "$dir/$id.md" ] || die "tarefa $id não encontrada em $spec_id"
      grep -qxF "$dir/$id.md" <<< "$found" || found="$found$dir/$id.md"$'\n'
    done
  fi
  printf '%s' "$found"
}

# Problemas das tarefas, um por linha, prefixados pelo ID.
# approval_problems <SPEC_ID> <arquivos, um por linha>
approval_problems() {
  local spec_id="$1" files="$2" file id status upstream
  upstream="$(task_upstream_paths "$tasks_root/$spec_id")"
  while IFS= read -r file; do
    [ -n "$file" ] || continue
    id="$(basename "$file" .md)"
    status="$(frontmatter_get "$file" status 2>/dev/null || true)"
    if [ "$status" != "$pending_status" ]; then
      echo "$id: status '$status'; só tarefas com status '$pending_status' são aprovadas"
      continue
    fi
    [ "$(adjustment_hash_state "$file")" = same ] ||
      echo "$id: edição manual sem registro; registre-a com: adjust-task.sh --record-manual $spec_id $id"
    task_errors "$file" "$(frontmatter_get "$file" covers 2>/dev/null | tr -d '[]' || true)" \
      "$(TASK="$id" awk -F'\t' '$1 == ENVIRON["TASK"] { print $2 }' <<< "$upstream")"
  done <<< "$files"
}

ids_of() {
  sed 's|.*/||; s|\.md$||' <<< "$1" | grep -v '^$' | sed 's/^/- /'
}

validate() {
  local spec_id="$1" files problems
  shift
  files="$(selected_tasks "$spec_id" "$@")"
  problems="$(approval_problems "$spec_id" "$files")"
  [ -z "$problems" ] || die "as tarefas de $spec_id não podem ser aprovadas:"$'\n'"$problems"
  echo "Tarefas de $spec_id que podem ser aprovadas:"
  ids_of "$files"
}

approve() {
  local spec_id="$1" files problems file now
  shift
  files="$(selected_tasks "$spec_id" "$@")"
  problems="$(approval_problems "$spec_id" "$files")"
  [ -z "$problems" ] || die "as tarefas de $spec_id não podem ser aprovadas; nenhuma foi alterada:"$'\n'"$problems"

  now="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  while IFS= read -r file; do
    [ -n "$file" ] || continue
    frontmatter_set "$file" status "$approved_status"
    frontmatter_put "$file" "$approved_at_field" "$now"
    frontmatter_put "$file" "$approved_hash_field" "$(frontmatter_get "$file" "$hash_field")"
  done <<< "$files"
  echo "Tarefas de $spec_id aprovadas:"
  ids_of "$files"
}

case "${1:-}" in
  --validate) [ -n "${2:-}" ] || die "$USAGE"; shift; validate "$@" ;;
  --approve) [ -n "${2:-}" ] || die "$USAGE"; shift; approve "$@" ;;
  *) die "$USAGE" ;;
esac
