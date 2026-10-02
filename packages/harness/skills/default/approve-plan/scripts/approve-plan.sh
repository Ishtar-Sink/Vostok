#!/usr/bin/env bash
# Aprova o plano de tarefas de uma spec: cria os arquivos de tarefa a partir do plano, valida o
# detalhamento feito pelo agente, remove o plano e marca a spec como status.planned.
# Tarefas em `paths.tasks`/<SPEC_ID>/<TASK_ID>.md, a partir de `task_template` (config/tasks.yml).
#
# Uso: approve-plan.sh --create-tasks <SPEC_ID>
#        Confere o plano (válido, atualizado e sem edição manual sem registro) e cria um arquivo por
#        tarefa, com o front-matter e os campos conhecidos do card preenchidos. Recusa, sem alterar
#        nada, se a pasta de tarefas da spec já existir.
#      approve-plan.sh --validate <SPEC_ID>
#        Lista os problemas das tarefas, sem alterar nada. Termina com erro se houver algum.
#      approve-plan.sh --mark-planned <SPEC_ID>
#        Valida as tarefas, remove o plano e muda o status da spec para status.planned.
#        Com problemas, termina com erro sem alterar nada.
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

MAIN_CONFIG="$HARNESS_DIR/config/main.yml"
SPECS_CONFIG="$HARNESS_DIR/config/specs.yml"
TASKS_CONFIG="$HARNESS_DIR/config/tasks.yml"

USAGE="Uso: approve-plan.sh --create-tasks <SPEC_ID> | --validate <SPEC_ID> | --mark-planned <SPEC_ID>"

die() { echo "Erro: $*" >&2; exit 1; }

root_dir="$(cd "$(harness_path "$(config_get "$MAIN_CONFIG" paths.root)")" && pwd)"
tasks_root="$(harness_path "$(config_get "$MAIN_CONFIG" paths.tasks)")"
approved_status="$(config_get "$SPECS_CONFIG" status.approved)"
planned_status="$(config_get "$SPECS_CONFIG" status.planned)"
pending_status="$(config_get "$TASKS_CONFIG" status.pending)"

find_spec() {
  plan_spec_file "$1" || die "spec refinada $1 não encontrada"
}

find_plan() {
  local plan
  plan="$(plan_path "$1")"
  [ -f "$plan" ] || die "a spec $1 não tem plano de tarefas. Crie-o com a skill plan-tasks"
  printf '%s' "$plan"
}

# Impedimentos do plano, um por linha: spec fora de status.approved, spec alterada depois da
# aprovação, plano desatualizado, edição manual sem registro e problemas de validação.
# plan_blockers <SPEC_ID> <plano> <spec>
plan_blockers() {
  local spec_id="$1" plan="$2" spec="$3" status

  status="$(frontmatter_get "$spec" status 2>/dev/null || true)"
  if [ "$status" != "$approved_status" ]; then
    echo "a spec $spec_id está com status '$status'; só planos de specs com status '$approved_status' são aprovados"
    return
  fi
  [ "$(adjustment_hash_state "$spec")" = same ] ||
    echo "a spec $spec_id mudou depois da aprovação; aprove-a de novo e recrie o plano"
  ! plan_is_outdated "$plan" "$spec" ||
    echo "o plano foi feito para outra versão da spec $spec_id; recrie-o com: plan-tasks.sh --create $spec_id"
  [ "$(adjustment_hash_state "$plan")" = same ] ||
    echo "edição manual no plano sem registro; registre-a com: adjust-plan.sh --record-manual $spec_id"
  plan_errors "$plan" "$spec"
}

# Converte "TASK-01, TASK-02" ou "-" numa lista separada por ", " (vazia para "-").
as_list() {
  [ "$1" = "-" ] && return 0
  printf '%s' "$1" | tr ',' '\n' | sed 's/^[[:space:]]*//; s/[[:space:]]*$//' | grep -v '^$' | paste -sd ',' - | sed 's/,/, /g'
}

create_tasks() {
  local spec_id="$1" plan spec blockers target template staging id title desc type effort deps covers

  plan="$(find_plan "$spec_id")"
  spec="$(find_spec "$spec_id")"
  blockers="$(plan_blockers "$spec_id" "$plan" "$spec")"
  [ -z "$blockers" ] || die "o plano de $spec_id não pode ser aprovado; nada foi alterado:"$'\n'"$blockers"

  target="$tasks_root/$spec_id"
  [ ! -e "$target" ] || die "a pasta de tarefas de $spec_id já existe: $target"
  template="$(harness_path "$(config_get "$MAIN_CONFIG" paths.templates)")/$(config_get "$TASKS_CONFIG" task_template)"
  [ -f "$template" ] || die "template não encontrado: $template"

  # As tarefas são montadas numa pasta temporária e movidas juntas: ou todas existem, ou nenhuma.
  staging="$(mktemp -d)"
  while IFS=$'\t' read -r id title desc type effort deps covers; do
    deps="$(as_list "$deps")"
    TPL_TASK_ID="$id" \
    TPL_SPEC_ID="$spec_id" \
    TPL_TASK_TITLE="$title" \
    TPL_TASK_TITLE_YAML="$(yaml_escape "$title")" \
    TPL_TASK_EFFORT="$effort" \
    TPL_TASK_DEPENDS="$deps" \
    TPL_TASK_COVERS="$(as_list "$covers")" \
    TPL_TASK_TYPE="$type" \
    TPL_TASK_STATUS="$pending_status" \
    TPL_TASK_BLOCKED_BY="${deps:--}" \
      render "$template" > "$staging/$id.md"
  done <<< "$(plan_tasks "$plan")"

  mkdir -p "$target"
  mv "$staging"/*.md "$target"/
  rmdir "$staging"
  echo "Tarefas de $spec_id criadas em $target:"
  ls "$target" | sed 's/^/- /'
}

# Itens de "Mudar" no formato "- `caminho` (ação): motivo", um por linha: ação, tabulação e caminho.
# change_items <arquivo>
change_items() {
  local item
  while IFS= read -r item; do
    [[ "$item" =~ ^-\ \`([^\`]+)\`\ \(([a-z]+)\) ]] || continue
    printf '%s\t%s\n' "${BASH_REMATCH[2]}" "${BASH_REMATCH[1]}"
  done <<< "$(markdown_subsection "$1" "$(config_get "$TASKS_CONFIG" execution_section)" \
    "$(config_get "$TASKS_CONFIG" change_subsection)" 2>/dev/null || true)"
}

# Caminhos criados pelas tarefas de que cada tarefa depende, direta ou indiretamente, um por linha:
# ID, tabulação e caminho. Lê da entrada linhas "D<tab>ID<tab>dependência" e "C<tab>ID<tab>caminho".
dependency_created_paths() {
  awk -F'\t' '
    $1 == "D" { deps[$2] = deps[$2] " " $3 }
    $1 == "C" { created[$2] = created[$2] "\t" $3 }
    function collect(id, dep,   n, i, list) {
      if ((id SUBSEP dep) in seen) return
      seen[id, dep] = 1
      n = split(created[dep], list, "\t")
      for (i = 2; i <= n; i++) print id "\t" list[i]
      n = split(deps[dep], list, " ")
      for (i = 1; i <= n; i++) collect(id, list[i])
    }
    END {
      for (id in deps) {
        n = split(deps[id], direct, " ")
        for (i = 1; i <= n; i++) collect(id, direct[i])
      }
    }
  '
}

# Problemas de uma tarefa, um por linha, prefixados pelo ID.
# task_errors <arquivo> <IDs cobertos pelo plano, separados por ", "> [caminhos criados pelas dependências]
task_errors() {
  local file="$1" plan_covers="$2" upstream="${3:-}" id section sub text field path action covers max lines
  id="$(basename "$file" .md)"

  template_leftovers "$file" | sed "s|^|$id: variável sem preencher: |" || true

  for section in execution description; do
    while IFS= read -r sub; do
      if ! text="$(markdown_subsection "$file" "$(config_get "$TASKS_CONFIG" "${section}_section")" "$sub")"; then
        echo "$id: subseção ausente: '### $sub'"
      elif ! grep -q '[^[:space:]]' <<< "$text"; then
        echo "$id: a subseção '$sub' está vazia"
      fi
    done <<< "$(config_list "$TASKS_CONFIG" "${section}_subsections")"
  done

  max="$(config_get "$TASKS_CONFIG" executive_summary_max_lines)"
  lines="$(markdown_subsection "$file" "$(config_get "$TASKS_CONFIG" description_section)" \
    "$(config_get "$TASKS_CONFIG" executive_summary_subsection)" 2>/dev/null | grep -c '[^[:space:]]' || true)"
  [ "${lines:-0}" -le "$max" ] || echo "$id: o resumo executivo tem $lines linhas (máximo: $max)"

  text="$(markdown_subsection "$file" "$(config_get "$TASKS_CONFIG" description_section)" \
    "$(config_get "$TASKS_CONFIG" card_subsection)" 2>/dev/null || true)"
  while IFS= read -r field; do
    grep -qF -- "- **$field:**" <<< "$text" || echo "$id: campo do card ausente: '$field'"
  done <<< "$(config_list "$TASKS_CONFIG" card_fields)"

  # Arquivos a mudar, com a ação coerente com a existência do caminho. Um caminho criado por uma
  # tarefa de que esta depende conta como existente.
  text="$(change_items "$file")"
  while IFS=$'\t' read -r action path; do
    [ -n "$path" ] || continue
    case " $SPEC_COMPONENT_ACTIONS " in
      *" $action "*) ;;
      *) echo "$id: ação inválida para '$path': '$action' (use: $SPEC_COMPONENT_ACTIONS)"; continue ;;
    esac
    if [ "$action" = criar ] && [ -e "$root_dir/$path" ]; then
      echo "$id: '$path' já existe; use a ação 'alterar'"
    elif [ "$action" != criar ] && [ ! -e "$root_dir/$path" ] && ! grep -qxF -- "$path" <<< "$upstream"; then
      echo "$id: '$path' não existe nem é criado por uma tarefa de que ela depende; use a ação 'criar' ou corrija o caminho"
    fi
  done <<< "$text"
  [ -n "$text" ] ||
    echo "$id: '$(config_get "$TASKS_CONFIG" change_subsection)' sem nenhum item no formato: - \`caminho\` (ação): motivo"

  # Critérios de conclusão: um checkbox por critério, seguido da forma de comprová-lo numa citação
  # recuada ("  > Como comprovar: ...").
  sub="$(config_get "$TASKS_CONFIG" done_subsection)"
  field="$(config_get "$TASKS_CONFIG" done_proof_label)"
  text="$(markdown_subsection "$file" "$(config_get "$TASKS_CONFIG" execution_section)" "$sub" 2>/dev/null || true)"
  local criterion="" proven=1 count=0
  while IFS= read -r item; do
    [[ "$item" =~ [^[:space:]] ]] || continue
    if [[ "$item" =~ ^-\ \[[\ x]\]\ [^[:space:]] ]]; then
      [ "$proven" -eq 1 ] || echo "$id: '$sub' com critério sem '$field': $criterion"
      criterion="$item" proven=0 count=$((count + 1))
    elif [[ "$item" =~ ^[[:space:]]+\>\ (.*)$ ]] && [[ "${BASH_REMATCH[1]}" == "$field:"* ]] && [ -n "$criterion" ]; then
      if [[ "${BASH_REMATCH[1]#"$field:"}" =~ [^[:space:]] ]]; then proven=1; fi
    else
      echo "$id: '$sub' com linha fora do formato: $item"
    fi
  done <<< "$text"
  [ "$proven" -eq 1 ] || echo "$id: '$sub' com critério sem '$field': $criterion"
  [ "$count" -gt 0 ] || [ -z "$text" ] ||
    echo "$id: '$sub' sem nenhum critério no formato de checkbox: - [ ] critério"

  covers="$(frontmatter_get "$file" covers 2>/dev/null | tr -d '[]' || true)"
  [ "$(as_list "$covers" | tr -d ' ' | tr ',' '\n' | sort | paste -sd ',' -)" = \
    "$(as_list "$plan_covers" | tr -d ' ' | tr ',' '\n' | sort | paste -sd ',' -)" ] ||
    echo "$id: cobertura '$covers' diferente da do plano '$plan_covers'"
}

# Problemas de todas as tarefas da spec, um por linha.
# tasks_errors <SPEC_ID> <plano>
tasks_errors() {
  local spec_id="$1" plan="$2" dir id title desc type effort deps covers file ids upstream
  dir="$tasks_root/$spec_id"
  if [ ! -d "$dir" ]; then
    echo "as tarefas de $spec_id ainda não foram criadas; execute: approve-plan.sh --create-tasks $spec_id"
    return
  fi
  upstream="$(
    plan_tasks "$plan" | while IFS=$'\t' read -r id title desc type effort deps covers; do
      as_list "$deps" | TASK="$id" awk -F', ' '{ for (i = 1; i <= NF; i++) print "D\t" ENVIRON["TASK"] "\t" $i }'
      if [ -f "$dir/$id.md" ]; then
        change_items "$dir/$id.md" | TASK="$id" awk -F'\t' '$1 == "criar" { print "C\t" ENVIRON["TASK"] "\t" $2 }'
      fi
    done | dependency_created_paths
  )"
  while IFS=$'\t' read -r id title desc type effort deps covers; do
    [ -f "$dir/$id.md" ] || { echo "$id: arquivo da tarefa ausente em $dir"; continue; }
    task_errors "$dir/$id.md" "$covers" "$(TASK="$id" awk -F'\t' '$1 == ENVIRON["TASK"] { print $2 }' <<< "$upstream")"
  done <<< "$(plan_tasks "$plan")"
  ids="$(plan_tasks "$plan" | cut -f1)"
  for file in "$dir"/*.md; do
    [ -f "$file" ] || continue
    grep -qxF "$(basename "$file" .md)" <<< "$ids" ||
      echo "$(basename "$file" .md): tarefa que não existe no plano"
  done
}

validate() {
  local spec_id="$1" plan errors

  plan="$(find_plan "$spec_id")"
  errors="$(tasks_errors "$spec_id" "$plan")"
  [ -z "$errors" ] || die "as tarefas de $spec_id têm problemas:"$'\n'"$errors"
  echo "Tarefas de $spec_id válidas: $tasks_root/$spec_id"
}

mark_planned() {
  local spec_id="$1" plan spec errors

  plan="$(find_plan "$spec_id")"
  spec="$(find_spec "$spec_id")"
  errors="$(plan_blockers "$spec_id" "$plan" "$spec"; tasks_errors "$spec_id" "$plan")"
  [ -z "$errors" ] || die "o plano de $spec_id não pode ser concluído; nada foi alterado:"$'\n'"$errors"

  frontmatter_set "$spec" status "$planned_status"
  frontmatter_put "$spec" updated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  rm -f "$plan"
  echo "Spec $spec_id marcada como '$planned_status': $spec"
  echo "Plano removido: $plan"
}

case "${1:-}" in
  --create-tasks) [ -n "${2:-}" ] || die "$USAGE"; create_tasks "$2" ;;
  --validate) [ -n "${2:-}" ] || die "$USAGE"; validate "$2" ;;
  --mark-planned) [ -n "${2:-}" ] || die "$USAGE"; mark_planned "$2" ;;
  *) die "$USAGE" ;;
esac
