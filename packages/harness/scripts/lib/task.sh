# Validação do arquivo de tarefa e leitura dos itens de suas subseções. Compartilhado por approve-plan.sh
# e pelos scripts de execução. Seções, subseções e rótulos vêm de config/tasks.yml.
# Uso: source "$HARNESS_DIR/scripts/lib/task.sh"
#      (requer HARNESS_DIR e as libs config.sh, text.sh, frontmatter.sh, markdown.sh, spec-validation.sh e plan.sh)

# Raiz do projeto (`paths.root` em main.yml), onde ficam os caminhos citados em "Mudar".
_task_root_dir() {
  (cd "$(harness_path "$(config_get "$HARNESS_DIR/config/main.yml" paths.root)")" && pwd)
}

# Texto de uma subseção de "Execução" da tarefa. Vazio se ela não existir.
# task_subsection <arquivo> <chave de config com o título da subseção>
task_subsection() {
  markdown_subsection "$1" "$(config_get "$(_tasks_config)" execution_section)" \
    "$(config_get "$(_tasks_config)" "$2")" 2>/dev/null || true
}

# Converte "TASK-01, TASK-02" ou "-" numa lista separada por ", " (vazia para "-").
as_list() {
  [ "$1" = "-" ] && return 0
  printf '%s' "$1" | tr ',' '\n' | sed 's/^[[:space:]]*//; s/[[:space:]]*$//' | grep -v '^$' | paste -sd ',' - | sed 's/,/, /g'
}

# Itens de "Mudar" no formato "- `caminho` (ação): motivo", um por linha: ação, tabulação e caminho.
# change_items <arquivo>
change_items() {
  local item
  while IFS= read -r item; do
    [[ "$item" =~ ^-\ \`([^\`]+)\`\ \(([a-z]+)\) ]] || continue
    printf '%s\t%s\n' "${BASH_REMATCH[2]}" "${BASH_REMATCH[1]}"
  done <<< "$(markdown_subsection "$1" "$(config_get "$(_tasks_config)" execution_section)" \
    "$(config_get "$(_tasks_config)" change_subsection)" 2>/dev/null || true)"
}

# Problemas de "Baseline", um por linha: cada item é um comando no formato "- `comando`" e há ao menos um.
# task_baseline_errors <arquivo>
task_baseline_errors() {
  local text item count=0
  text="$(task_subsection "$1" baseline_subsection)"
  [ -n "$text" ] || return 0
  while IFS= read -r item; do
    [[ "$item" =~ [^[:space:]] ]] || continue
    if [[ "$item" =~ ^-\ \`[^\`]+\`$ ]]; then
      count=$((count + 1))
    else
      echo "'$(config_get "$(_tasks_config)" baseline_subsection)' com item fora do formato: $item (use: - \`comando\`)"
    fi
  done <<< "$text"
  [ "$count" -gt 0 ] || echo "'$(config_get "$(_tasks_config)" baseline_subsection)' sem nenhum item no formato: - \`comando\`"
}

# Problemas de "Evidências", um por linha: itens "- `arquivo` (automática): `comando`" ou
# "- `arquivo` (manual): descrição", ou o texto único de `evidence_none`.
# task_evidence_errors <arquivo>
task_evidence_errors() {
  local text item count=0 cfg sub auto manual none re_auto re_manual
  cfg="$(_tasks_config)"
  text="$(task_subsection "$1" evidence_subsection)"
  [ -n "$text" ] || return 0
  sub="$(config_get "$cfg" evidence_subsection)"
  auto="$(config_get "$cfg" evidence_automatic)"
  manual="$(config_get "$cfg" evidence_manual)"
  none="$(config_get "$cfg" evidence_none)"
  re_auto="^-\\ \`[^\`]+\`\\ \\($auto\\):\\ \`[^\`]+\`$"
  re_manual="^-\\ \`[^\`]+\`\\ \\($manual\\):\\ [^[:space:]]"
  if [ "$(printf '%s' "$text" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//' | grep -v '^$')" = "$none" ]; then
    return 0
  fi
  while IFS= read -r item; do
    [[ "$item" =~ [^[:space:]] ]] || continue
    if [[ "$item" =~ $re_auto ]] || [[ "$item" =~ $re_manual ]]; then
      count=$((count + 1))
    else
      echo "'$sub' com item fora do formato: $item (use: - \`arquivo\` ($auto): \`comando\` ou - \`arquivo\` ($manual): descrição)"
    fi
  done <<< "$text"
  [ "$count" -gt 0 ] || echo "'$sub' sem nenhum item no formato e sem o texto '$none'"
}

# Problemas de uma tarefa, um por linha, prefixados pelo ID.
# task_errors <arquivo> <IDs cobertos pelo plano, separados por ", "> [caminhos criados pelas dependências]
task_errors() {
  local file="$1" plan_covers="$2" upstream="${3:-}" id section sub text field path action covers max lines cfg root_dir
  cfg="$(_tasks_config)"
  root_dir="$(_task_root_dir)"
  id="$(basename "$file" .md)"

  template_leftovers "$file" | sed "s|^|$id: variável sem preencher: |" || true

  for section in execution description; do
    while IFS= read -r sub; do
      if ! text="$(markdown_subsection "$file" "$(config_get "$cfg" "${section}_section")" "$sub")"; then
        echo "$id: subseção ausente: '### $sub'"
      elif ! grep -q '[^[:space:]]' <<< "$text"; then
        echo "$id: a subseção '$sub' está vazia"
      fi
    done <<< "$(config_list "$cfg" "${section}_subsections")"
  done

  max="$(config_get "$cfg" executive_summary_max_lines)"
  lines="$(markdown_subsection "$file" "$(config_get "$cfg" description_section)" \
    "$(config_get "$cfg" executive_summary_subsection)" 2>/dev/null | grep -c '[^[:space:]]' || true)"
  [ "${lines:-0}" -le "$max" ] || echo "$id: o resumo executivo tem $lines linhas (máximo: $max)"

  text="$(markdown_subsection "$file" "$(config_get "$cfg" description_section)" \
    "$(config_get "$cfg" card_subsection)" 2>/dev/null || true)"
  while IFS= read -r field; do
    grep -qF -- "- **$field:**" <<< "$text" || echo "$id: campo do card ausente: '$field'"
  done <<< "$(config_list "$cfg" card_fields)"

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
    echo "$id: '$(config_get "$cfg" change_subsection)' sem nenhum item no formato: - \`caminho\` (ação): motivo"

  # Critérios de conclusão: um checkbox por critério, seguido da forma de comprová-lo numa citação
  # recuada ("  > Como comprovar: ...").
  sub="$(config_get "$cfg" done_subsection)"
  field="$(config_get "$cfg" done_proof_label)"
  text="$(markdown_subsection "$file" "$(config_get "$cfg" execution_section)" "$sub" 2>/dev/null || true)"
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

  task_baseline_errors "$file" | sed "s|^|$id: |"
  task_evidence_errors "$file" | sed "s|^|$id: |"

  covers="$(frontmatter_get "$file" covers 2>/dev/null | tr -d '[]' || true)"
  [ "$(as_list "$covers" | tr -d ' ' | tr ',' '\n' | sort | paste -sd ',' -)" = \
    "$(as_list "$plan_covers" | tr -d ' ' | tr ',' '\n' | sort | paste -sd ',' -)" ] ||
    echo "$id: cobertura '$covers' diferente da do plano '$plan_covers'"
}
