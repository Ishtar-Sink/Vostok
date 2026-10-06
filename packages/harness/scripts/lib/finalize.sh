# Finalização de specs: conferência, fontes, conteúdo, documentos e arquivamento (finalize-spec.sh).
# Status vêm de config/specs.yml e config/tasks.yml; caminhos, de `paths` em config/main.yml.
# Uso: source "$HARNESS_DIR/scripts/lib/finalize.sh"
#      (requer HARNESS_DIR e as libs config.sh, text.sh, frontmatter.sh, markdown.sh, spec-validation.sh,
#      adjustments.sh, plan.sh, task.sh e memory.sh)

# Arquivo de conteúdo dentro da pasta da spec em `paths.finalize` e rótulos dos campos de cada evidência.
FINALIZE_CONTENT_FILE="content.md"
FINALIZE_EVIDENCE_HOW="Como chegar"
FINALIZE_EVIDENCE_READ="Como ler"

# Pasta das tarefas de uma spec (`paths.tasks`/<SPEC_ID>).
# finalize_tasks_dir <SPEC_ID>
finalize_tasks_dir() {
  printf '%s/%s' "$(harness_path "$(config_get "$HARNESS_DIR/config/main.yml" paths.tasks)")" "$1"
}

# Imprime um problema por linha se a spec não puder ser finalizada: status diferente de
# `status.implemented` ou tarefa fora de `status.done`. Não imprime nada se ela puder.
# finalize_check <arquivo da spec> <pasta das tarefas da spec>
finalize_check() {
  local spec="$1" tasks_dir="$2" implemented done status task found=0
  implemented="$(config_get "$HARNESS_DIR/config/specs.yml" status.implemented)"
  done="$(config_get "$HARNESS_DIR/config/tasks.yml" status.done)"

  status="$(frontmatter_get "$spec" status 2>/dev/null || true)"
  [ "$status" = "$implemented" ] ||
    echo "a spec está com status '$status'; só specs com status '$implemented' são finalizadas"

  for task in "$tasks_dir"/*.md; do
    [ -f "$task" ] || continue
    found=1
    status="$(frontmatter_get "$task" status 2>/dev/null || true)"
    [ "$status" = "$done" ] ||
      echo "$(frontmatter_get "$task" id 2>/dev/null || basename "$task" .md): status '$status', não '$done'"
  done
  [ "$found" -eq 1 ] || echo "nenhuma tarefa encontrada em $tasks_dir"
}

# Caminho relativo à raiz do projeto, quando o caminho está dentro dela.
# finalize_rel <caminho>
finalize_rel() {
  local root
  root="$(_task_root_dir)"
  printf '%s' "${1#"$root"/}"
}

# Data de aprovação da spec (`approved_at`), vazia se ela não tiver.
# finalize_approved_at <arquivo da spec>
finalize_approved_at() {
  local value
  value="$(frontmatter_get "$1" "$(config_get "$HARNESS_DIR/config/specs.yml" approved_at_field)" 2>/dev/null || true)"
  [ "$value" = null ] && value=""
  printf '%s' "$value"
}

# Itens de "Mudar" de todas as tarefas, sem repetição, um por linha: ação, tabulação e caminho.
# finalize_changed_items <pasta das tarefas da spec>
finalize_changed_items() {
  local task
  for task in "$1"/*.md; do
    [ -f "$task" ] || continue
    change_items "$task"
  done | awk '!seen[$0]++'
}

# Evidências listadas em "Evidências" de uma tarefa, um item por linha: nome, tabulação e tipo
# (automática ou manual). Nada para "Não se aplica.".
# finalize_evidence_items <arquivo da tarefa>
finalize_evidence_items() {
  local item re="^-\\ \`([^\`]+)\`\\ \\(([^)]+)\\)"
  while IFS= read -r item; do
    if [[ "$item" =~ $re ]]; then
      printf '%s\t%s\n' "${BASH_REMATCH[1]}" "${BASH_REMATCH[2]}"
    fi
  done <<< "$(task_subsection "$1" evidence_subsection)"
  return 0
}

# Bloco "## Spec" das fontes.
# finalize_sources_spec <arquivo da spec>
finalize_sources_spec() {
  local approved
  approved="$(finalize_approved_at "$1")"
  echo "## Spec"
  echo
  echo "- **ID:** $(frontmatter_get "$1" id)"
  echo "- **Título:** $(frontmatter_get "$1" title)"
  echo "- **Arquivo:** \`$(finalize_rel "$1")\`"
  echo "- **Aprovada em:** ${approved:-sem data de aprovação}"
}

# Bloco "## Tarefas" das fontes: por tarefa, o status, o registro de execução, o histórico de ajustes e
# as evidências listadas, com o caminho e se o arquivo existe.
# finalize_sources_tasks <pasta das tarefas da spec>
finalize_sources_tasks() {
  local task cfg record history name kind evidence_dir found
  cfg="$HARNESS_DIR/config/tasks.yml"
  record="$(config_get "$cfg" record_section)"
  history="$(spec_section_heading "$(config_get "$HARNESS_DIR/config/specs.yml" adjustment_history_section)")"
  echo "## Tarefas"
  for task in "$1"/*.md; do
    [ -f "$task" ] || continue
    echo
    echo "### $(frontmatter_get "$task" id): $(frontmatter_get "$task" title)"
    echo
    echo "- **Status:** $(frontmatter_get "$task" status)"
    echo
    echo "**$record**"
    echo
    markdown_section "$task" "$record" 2>/dev/null | grep '^|' || true
    echo
    echo "**$history**"
    echo
    markdown_section "$task" "$history" 2>/dev/null | grep '^|' || true
    echo
    echo "**Evidências**"
    echo
    evidence_dir="$(task_evidence_path "$task")"
    found=0
    while IFS=$'\t' read -r name kind; do
      [ -n "$name" ] || continue
      found=1
      if [ -f "$evidence_dir/$name" ]; then
        echo "- \`$(finalize_rel "$evidence_dir/$name")\` ($kind): existe"
      else
        echo "- \`$(finalize_rel "$evidence_dir/$name")\` ($kind): ausente"
      fi
    done <<< "$(finalize_evidence_items "$task")"
    [ "$found" -eq 1 ] || echo "Não se aplica."
  done
}

# Bloco "## Caminhos alterados" das fontes.
# finalize_sources_paths <pasta das tarefas da spec>
finalize_sources_paths() {
  local action path
  echo "## Caminhos alterados"
  echo
  finalize_changed_items "$1" | while IFS=$'\t' read -r action path; do
    echo "- \`$path\` ($action)"
  done
}

# Bloco "## Memória do período" das fontes: entradas de memória com a data entre a aprovação da spec e hoje.
# finalize_sources_memory <arquivo da spec>
finalize_sources_memory() {
  local approved start today type file found=0 lines
  approved="$(finalize_approved_at "$1")"
  echo "## Memória do período"
  echo
  if [ -z "$approved" ]; then
    echo "Spec sem data de aprovação."
    return 0
  fi
  start="${approved%%T*}"
  today="$(date -u +%Y-%m-%d)"
  while IFS= read -r type; do
    file="$(memory_file "$type")"
    [ -f "$file" ] || continue
    lines="$(START="$start" END="$today" awk '
      function flush() {
        if (id != "" && date >= ENVIRON["START"] && date <= ENVIRON["END"])
          print "- `" id "` " title " (" status ", " date ")"
        id = ""; date = ""; status = ""
      }
      { sub(/\r$/, "") }
      /^## [A-Z]+-[0-9]+: / {
        flush()
        id = $2; sub(/:$/, "", id)
        title = $0; sub(/^## [A-Z]+-[0-9]+: /, "", title)
        next
      }
      /^- \*\*Data:\*\* / { date = $0; sub(/^- \*\*Data:\*\* /, "", date) }
      /^- \*\*Status:\*\* / { status = $0; sub(/^- \*\*Status:\*\* /, "", status) }
      END { flush() }
    ' "$file")"
    if [ -n "$lines" ]; then
      printf '%s\n' "$lines"
      found=1
    fi
  done <<< "$(config_list "$MEMORY_CONFIG" types)"
  [ "$found" -eq 1 ] || echo "Nenhuma entrada de memória no período."
}

# Bloco "## Commits" das fontes: commits desde a aprovação da spec que alteram os caminhos das tarefas.
# finalize_sources_commits <arquivo da spec> <pasta das tarefas da spec>
finalize_sources_commits() {
  local approved root action path paths=() log
  echo "## Commits"
  echo
  root="$(_task_root_dir)"
  if ! git -C "$root" rev-parse --is-inside-work-tree > /dev/null 2>&1; then
    echo "Fora de um repositório git."
    return 0
  fi
  approved="$(finalize_approved_at "$1")"
  if [ -z "$approved" ]; then
    echo "Spec sem data de aprovação."
    return 0
  fi
  while IFS=$'\t' read -r action path; do
    [ -n "$path" ] && paths+=("$path")
  done <<< "$(finalize_changed_items "$2" | awk -F'\t' '!seen[$2]++')"
  if [ "${#paths[@]}" -eq 0 ]; then
    echo "Nenhum commit encontrado."
    return 0
  fi
  log="$(git -C "$root" log --since="$approved" --format='- `%h` %ad %s' --date=short -- "${paths[@]}" 2>/dev/null || true)"
  if [ -n "$log" ]; then
    printf '%s\n' "$log"
  else
    echo "Nenhum commit encontrado."
  fi
}

# Imprime as fontes da síntese de uma spec, em Markdown.
# finalize_sources <arquivo da spec> <pasta das tarefas da spec>
finalize_sources() {
  finalize_sources_spec "$1"
  echo
  finalize_sources_tasks "$2"
  echo
  finalize_sources_paths "$2"
  echo
  finalize_sources_memory "$1"
  echo
  finalize_sources_commits "$1" "$2"
}

# Pasta do conteúdo e dos documentos de uma spec (`paths.finalize`/<SPEC_ID>).
# finalize_content_dir <SPEC_ID>
finalize_content_dir() {
  printf '%s/%s' "$(harness_path "$(config_get "$HARNESS_DIR/config/main.yml" paths.finalize)")" "$1"
}

# Arquivo de conteúdo de uma spec.
# finalize_content_path <SPEC_ID>
finalize_content_path() {
  printf '%s/%s' "$(finalize_content_dir "$1")" "$FINALIZE_CONTENT_FILE"
}

# Template do arquivo de conteúdo (`finalize_content_template` em specs.yml).
finalize_content_template() {
  printf '%s/%s' "$(harness_path "$(config_get "$HARNESS_DIR/config/main.yml" paths.templates)")" \
    "$(config_get "$HARNESS_DIR/config/specs.yml" finalize_content_template)"
}

# Cria o arquivo de conteúdo a partir do template, com um bloco por evidência listada nas tarefas, e imprime
# o caminho. Falha, sem alterar nada, se o arquivo já existir. Tarefas sem evidência não geram bloco.
# finalize_create <SPEC_ID> <pasta das tarefas da spec>
finalize_create() {
  local id="$1" file task name kind n=0 tid
  file="$(finalize_content_path "$id")"
  if [ -e "$file" ]; then
    echo "o conteúdo de $id já existe: $(finalize_rel "$file")" >&2
    return 1
  fi
  mkdir -p "$(dirname "$file")"
  {
    TPL_SPEC_ID="$id" render "$(finalize_content_template)"
    for task in "$2"/*.md; do
      [ -f "$task" ] || continue
      tid="$(frontmatter_get "$task" id)"
      while IFS=$'\t' read -r name kind; do
        [ -n "$name" ] || continue
        n=$((n + 1))
        printf '\n## %s / %s\n\n- **%s:** #{EVIDENCE_%02d_HOW}\n- **%s:** #{EVIDENCE_%02d_READ}\n' \
          "$tid" "$name" "$FINALIZE_EVIDENCE_HOW" "$n" "$FINALIZE_EVIDENCE_READ" "$n"
      done <<< "$(finalize_evidence_items "$task")"
    done
  } > "$file"
  echo "Conteúdo de $id criado em $(finalize_rel "$file"): $n bloco(s) de evidência."
}

# Imprime um problema por linha do arquivo de conteúdo, sem alterar nada: seções do template ausentes ou
# vazias, variáveis sem preencher, evidências listadas que não existem ou sem bloco e blocos sem
# "Como chegar" ou "Como ler". Não imprime nada se o conteúdo estiver válido.
# finalize_content_errors <SPEC_ID> <pasta das tarefas da spec>
finalize_content_errors() {
  local id="$1" file heading task tid name kind evidence_dir prefix
  file="$(finalize_content_path "$id")"
  if [ ! -f "$file" ]; then
    echo "conteúdo não encontrado: $(finalize_rel "$file"); crie com --create"
    return 0
  fi
  prefix="$(config_get "$HARNESS_DIR/config/tasks.yml" id.prefix)"

  template_leftovers "$file" /dev/null | sed 's/^[^:]*:\([0-9]*\):/linha \1: /; s/^/variável sem preencher: /' || true

  while IFS= read -r heading; do
    grep -qxF -- "$heading" "$file" || echo "seção ausente: $heading"
  done <<< "$(grep -E '^##? ' "$(finalize_content_template)")"

  HOW="$FINALIZE_EVIDENCE_HOW" READ="$FINALIZE_EVIDENCE_READ" PREFIX="$prefix" awk '
    function flush() {
      if (heading == "") return
      if (evidence) {
        if (!how) print "bloco de evidência sem \"" ENVIRON["HOW"] "\": " heading
        if (!read) print "bloco de evidência sem \"" ENVIRON["READ"] "\": " heading
      } else if (lines == 0) {
        print "seção vazia: " heading
      }
    }
    function filled(label,   rest) {
      if (index($0, "- **" label ":** ") != 1) return 0
      rest = substr($0, length("- **" label ":** ") + 1)
      return rest ~ /[^ \t]/
    }
    { sub(/\r$/, "") }
    /^<!--.*-->[ \t]*$/ { next }
    /^# / { flush(); heading = ""; next }
    /^## / {
      flush()
      heading = $0; lines = 0; how = 0; read = 0
      evidence = ($0 ~ ("^## " ENVIRON["PREFIX"] "-[0-9]+ / "))
      next
    }
    /[^ \t]/ {
      lines++
      if (filled(ENVIRON["HOW"])) how = 1
      if (filled(ENVIRON["READ"])) read = 1
    }
    END { flush() }
  ' "$file"

  for task in "$2"/*.md; do
    [ -f "$task" ] || continue
    tid="$(frontmatter_get "$task" id)"
    evidence_dir="$(task_evidence_path "$task")"
    while IFS=$'\t' read -r name kind; do
      [ -n "$name" ] || continue
      [ -f "$evidence_dir/$name" ] || echo "$tid: evidência ausente: $(finalize_rel "$evidence_dir/$name")"
      grep -qxF -- "## $tid / $name" "$file" || echo "evidência sem bloco: ## $tid / $name"
    done <<< "$(finalize_evidence_items "$task")"
  done
}
