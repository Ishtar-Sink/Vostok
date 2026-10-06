# Finalização de specs: conferência, fontes, conteúdo, documentos e arquivamento (finalize-spec.sh).
# Status vêm de config/specs.yml e config/tasks.yml; caminhos, de `paths` em config/main.yml.
# Uso: source "$HARNESS_DIR/scripts/lib/finalize.sh"
#      (requer HARNESS_DIR e as libs config.sh, text.sh, frontmatter.sh, markdown.sh, spec-validation.sh,
#      adjustments.sh, plan.sh, task.sh e memory.sh)

# Arquivo de conteúdo dentro da pasta da spec em `paths.finalize` e rótulos dos campos de cada evidência.
FINALIZE_CONTENT_FILE="content.md"
FINALIZE_EVIDENCE_HOW="Como chegar"
FINALIZE_EVIDENCE_READ="Como ler"

# Template do documento executivo, relativo a `paths.templates` (main.yml).
FINALIZE_EXECUTIVE_TEMPLATE="finalize/executive.html"
# Template do documento técnico, relativo a `paths.templates` (main.yml).
FINALIZE_TECHNICAL_TEMPLATE="finalize/technical.html"

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
# vazias, variáveis sem preencher, entregas do executivo incompletas, evidências listadas que não existem
# ou sem bloco e blocos sem "Como chegar" ou "Como ler". Não imprime nada se o conteúdo estiver válido.
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

  finalize_content_part "$file" Executivo | finalize_part_section Entregas | finalize_deliveries_errors

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

# Configuração dos documentos HTML (tokens, fontes, limite e extensões).
_finalize_documents_config() {
  printf '%s/config/documents.yml' "$HARNESS_DIR"
}

# Pares "chave<TAB>valor" de um grupo de primeiro nível de um arquivo de configuração, na ordem do arquivo.
# Aspas externas e comentários no fim da linha são removidos, como em config_get.
# finalize_config_group <arquivo> <grupo>
finalize_config_group() {
  awk -v group="$2" '
    /^[ \t]*(#|$)/ { next }
    /^[^ \t]/ { ingroup = ($0 ~ ("^" group ":[ \t]*(#.*)?$")); next }
    ingroup {
      line = $0; sub(/^[ \t]+/, "", line)
      key = line; sub(/[ \t]*:.*/, "", key)
      val = substr(line, index(line, ":") + 1)
      sub(/[ \t]+#.*$/, "", val); gsub(/^[ \t]+|[ \t]+$/, "", val)
      if (val ~ /^".*"$/ || val ~ /^\047.*\047$/) val = substr(val, 2, length(val) - 2)
      print key "\t" val
    }
  ' "$1"
}

# Imprime o CSS dos tokens: `:root` com o tema escuro e as fontes, e `:root[data-theme="light"]` com o
# tema claro. Cada chave vira uma variável (`color_surface_2` -> `--color-surface-2`), na ordem do arquivo.
# finalize_css_tokens
finalize_css_tokens() {
  local cfg dark light font key value
  cfg="$(_finalize_documents_config)"
  dark="$(finalize_config_group "$cfg" dark)"
  light="$(finalize_config_group "$cfg" light)"
  font="$(finalize_config_group "$cfg" font)"
  if [ -z "$dark" ] || [ -z "$light" ]; then
    echo "config/documents.yml sem os grupos 'dark' e 'light'" >&2
    return 1
  fi
  echo ':root {'
  echo '  color-scheme: dark;'
  while IFS=$'\t' read -r key value; do
    echo "  --${key//_/-}: $value;"
  done <<< "$dark"
  if [ -n "$font" ]; then
    while IFS=$'\t' read -r key value; do
      echo "  --font-${key//_/-}: $value;"
    done <<< "$font"
  fi
  echo '}'
  echo ':root[data-theme="light"] {'
  echo '  color-scheme: light;'
  while IFS=$'\t' read -r key value; do
    echo "  --${key//_/-}: $value;"
  done <<< "$light"
  echo '}'
}

# Escapa `&`, `<`, `>` e aspas do texto lido da entrada padrão.
# finalize_html_escape < texto
finalize_html_escape() {
  sed -e 's/&/\&amp;/g' -e 's/</\&lt;/g' -e 's/>/\&gt;/g' -e 's/"/\&quot;/g' -e "s/'/\&#39;/g"
}

# Converte o Markdown restrito do arquivo de conteúdo, lido da entrada padrão, em HTML: parágrafos, listas
# com `- ` ou numeradas, tabelas, `código` e **negrito**. Todo o texto sai escapado, inclusive o de blocos
# de código. Linhas que são só um comentário `<!-- ... -->` são descartadas.
# finalize_markdown_html < markdown
finalize_markdown_html() {
  awk '
    function esc(s) {
      gsub(/&/, "\\&amp;", s); gsub(/</, "\\&lt;", s); gsub(/>/, "\\&gt;", s)
      gsub(/"/, "\\&quot;", s); gsub(/\047/, "\\&#39;", s)
      return s
    }
    function trim(s) { gsub(/^[ \t]+|[ \t]+$/, "", s); return s }
    function bold(s,   out) {
      out = ""
      while (match(s, /\*\*[^*]+\*\*/)) {
        out = out substr(s, 1, RSTART - 1) "<strong>" substr(s, RSTART + 2, RLENGTH - 4) "</strong>"
        s = substr(s, RSTART + RLENGTH)
      }
      return out s
    }
    # Texto já escapado: `código` (sem negrito dentro) e **negrito**. Crase sem par fica como está.
    function inline(s,   n, parts, i, out) {
      n = split(s, parts, "`")
      if (n % 2 == 0) return bold(s)
      out = ""
      for (i = 1; i <= n; i++) out = out ((i % 2) ? bold(parts[i]) : "<code>" parts[i] "</code>")
      return out
    }
    function close_block(   i) {
      if (mode == "p") print "<p>" inline(esc(para)) "</p>"
      else if (mode == "ul" || mode == "ol") print "</" mode ">"
      else if (mode == "table") {
        print "<table>"
        if (header) { print "<thead>"; print table_row(rows[1], "th"); print "</thead>" }
        print "<tbody>"
        for (i = header ? 2 : 1; i <= nrows; i++) print table_row(rows[i], "td")
        print "</tbody>"
        print "</table>"
      }
      mode = ""; para = ""; nrows = 0; header = 0
    }
    function table_row(line, tag,   cells, n, i, out) {
      line = trim(line)
      sub(/^\|/, "", line); sub(/\|$/, "", line)
      n = split(line, cells, "|")
      out = "<tr>"
      for (i = 1; i <= n; i++) out = out "<" tag ">" inline(esc(trim(cells[i]))) "</" tag ">"
      return out "</tr>"
    }
    { sub(/\r$/, "") }
    /^<!--.*-->[ \t]*$/ { next }
    /^[ \t]*$/ { close_block(); next }
    /^\|/ {
      if (mode != "table") { close_block(); mode = "table"; nrows = 0; header = 0 }
      if ($0 ~ /^\|[ \t:|-]+\|[ \t]*$/ && $0 ~ /---/) { if (nrows == 1) header = 1; next }
      rows[++nrows] = $0
      next
    }
    /^- / {
      if (mode != "ul") { close_block(); mode = "ul"; print "<ul>" }
      print "<li>" inline(esc(substr($0, 3))) "</li>"; next
    }
    /^[0-9]+\. / {
      if (mode != "ol") { close_block(); mode = "ol"; print "<ol>" }
      line = $0; sub(/^[0-9]+\. /, "", line)
      print "<li>" inline(esc(line)) "</li>"; next
    }
    {
      if (mode != "p") { close_block(); mode = "p"; para = "" }
      para = (para == "") ? trim($0) : para " " trim($0)
    }
    END { close_block() }
  '
}

# Caminho de um documento HTML da spec (`finalize_documents.<tipo>` em specs.yml), dentro de
# `paths.finalize`/<SPEC_ID>.
# finalize_document_path <SPEC_ID> <executive|technical|evidence>
finalize_document_path() {
  printf '%s/%s' "$(finalize_content_dir "$1")" \
    "$(config_get "$HARNESS_DIR/config/specs.yml" "finalize_documents.$2")"
}

# Linhas de uma parte (`# Título`) do arquivo de conteúdo, sem o título, até a próxima parte.
# finalize_content_part <arquivo de conteúdo> <título da parte, sem "# ">
finalize_content_part() {
  PART="# $2" awk '
    { sub(/\r$/, "") }
    /^# / { inside = ($0 == ENVIRON["PART"]); next }
    inside { print }
  ' "$1"
}

# Linhas de uma seção (`## Título`) de uma parte, lida da entrada padrão, sem o título, até a próxima seção.
# finalize_part_section <título da seção, sem "## "> < parte
finalize_part_section() {
  SECTION="## $1" awk '
    /^## / { inside = ($0 == ENVIRON["SECTION"]); next }
    inside { print }
  '
}

# HTML de uma seção em Markdown restrito, lida da entrada padrão, com cada tabela dentro de um contêiner
# que rola na horizontal em telas estreitas.
# finalize_block_html < markdown
finalize_block_html() {
  finalize_markdown_html | sed -e 's#^<table>$#<div class="tw"><table>#' -e 's#^</table>$#</table></div>#'
}

# HTML de uma linha de Markdown restrito, sem o parágrafo em volta.
# _finalize_inline <texto>
_finalize_inline() {
  printf '%s\n' "$1" | finalize_markdown_html | sed -e 's#^<p>##' -e 's#</p>$##'
}

# Faixa de indicadores de valor a partir da tabela "| Valor | Indicador |" da seção, lida da entrada padrão.
# Sem linhas de dados (por exemplo, "Não se aplica."), não imprime nada.
# finalize_value_kpis_html < seção
finalize_value_kpis_html() {
  local rows value label
  rows="$(awk '
    { sub(/\r$/, "") }
    /^\|/ {
      if ($0 ~ /^\|[ \t:|-]+\|[ \t]*$/ && $0 ~ /---/) { sep = 1; next }
      if (!sep) next
      line = $0; sub(/^\|/, "", line); sub(/\|[ \t]*$/, "", line)
      n = split(line, c, "|")
      if (n < 2) next
      gsub(/^[ \t]+|[ \t]+$/, "", c[1]); gsub(/^[ \t]+|[ \t]+$/, "", c[2])
      print c[1] "\t" c[2]
    }
  ')"
  [ -n "$rows" ] || return 0
  echo '<div class="kpis">'
  while IFS=$'\t' read -r value label; do
    printf '<div class="kpi"><b>%s</b><span>%s</span></div>\n' "$(_finalize_inline "$value")" "$(_finalize_inline "$label")"
  done <<< "$rows"
  echo '</div>'
}

# Entregas da seção, lida da entrada padrão, uma linha por item: "E<TAB>título" abre uma entrega,
# "F<TAB>rótulo<TAB>valor" é um campo "- **Rótulo:** valor" e "T<TAB>linha" é texto solto da entrega.
# Linhas antes da primeira entrega e comentários são ignorados.
# finalize_deliveries < seção
finalize_deliveries() {
  awk '
    { sub(/\r$/, "") }
    /^<!--.*-->[ \t]*$/ { next }
    /^### / { open = 1; t = $0; sub(/^### +/, "", t); sub(/[ \t]+$/, "", t); print "E\t" t; next }
    !open { next }
    match($0, /^- \*\*[^*]+:\*\*/) {
      label = substr($0, 5, RLENGTH - 7)
      value = substr($0, RLENGTH + 1); gsub(/^[ \t]+|[ \t]+$/, "", value)
      print "F\t" label "\t" value; next
    }
    { print "T\t" $0 }
  '
}

# Classe do impacto de uma entrega: alto, medio ou baixo; vazia para outro valor.
# _finalize_impact_class <impacto>
_finalize_impact_class() {
  case "$1" in
    alto) printf alto ;;
    médio|medio) printf medio ;;
    baixo) printf baixo ;;
  esac
}

# Etiqueta do impacto de uma entrega, vazia sem impacto.
# _finalize_impact_tag <impacto>
_finalize_impact_tag() {
  [ -n "$1" ] || return 0
  printf '<span class="tag %s">Impacto %s</span>' "$(_finalize_impact_class "$1")" "$(printf '%s' "$1" | finalize_html_escape)"
}

# Tabela-resumo das entregas: nome, impacto e valor.
# finalize_deliveries_table_html < seção de entregas
finalize_deliveries_table_html() {
  local kind a b title="" impact="" value="" n=0
  _finalize_emit_row() {
    [ -n "$title" ] || return 0
    printf '<tr><td class="strong">%s</td><td>%s</td><td>%s</td></tr>\n' \
      "$(_finalize_inline "$title")" "$(_finalize_impact_tag "$impact")" "$(_finalize_inline "$value")"
  }
  echo '<div class="tw"><table>'
  echo '<thead><tr><th>Entrega</th><th>Impacto</th><th>Valor</th></tr></thead>'
  echo '<tbody>'
  while IFS=$'\t' read -r kind a b; do
    case "$kind" in
      E) _finalize_emit_row; title="$a"; impact=""; value=""; n=$((n + 1)) ;;
      F) case "$a" in Impacto) impact="$b" ;; Valor) value="$b" ;; esac ;;
    esac
  done <<< "$(finalize_deliveries)"
  _finalize_emit_row
  echo '</tbody>'
  echo '</table></div>'
}

# Cartões expansíveis das entregas: título e impacto no resumo; os outros campos como parágrafos rotulados,
# na ordem do conteúdo, e o texto solto pelo Markdown restrito.
# finalize_deliveries_cards_html < seção de entregas
finalize_deliveries_cards_html() {
  local kind a b title="" impact="" body="" text=""
  _finalize_flush_text() {
    [ -n "$text" ] || return 0
    body="$body$(printf '%s\n' "$text" | finalize_block_html)"$'\n'
    text=""
  }
  _finalize_emit_card() {
    [ -n "$title" ] || return 0
    _finalize_flush_text
    printf '<details class="card impact-%s">\n' "$(_finalize_impact_class "$impact")"
    printf '<summary><b>%s</b>%s</summary>\n' "$(_finalize_inline "$title")" "$(_finalize_impact_tag "$impact")"
    printf '<div class="card-body">\n%s</div>\n</details>\n' "$body"
  }
  while IFS=$'\t' read -r kind a b; do
    case "$kind" in
      E) _finalize_emit_card; title="$a"; impact=""; body=""; text="" ;;
      F)
        if [ "$a" = Impacto ]; then
          impact="$b"
        else
          _finalize_flush_text
          body="$body$(printf '<p><b>%s</b>%s</p>' "$(printf '%s' "$a" | finalize_html_escape)" "$(_finalize_inline "$b")")"$'\n'
        fi
        ;;
      T) text="$text$a"$'\n' ;;
    esac
  done <<< "$(finalize_deliveries)"
  _finalize_emit_card
}

# Problemas das entregas do executivo, um por linha: nenhuma entrega, entrega sem "O que mudou" ou "Valor"
# e impacto fora de alto, médio e baixo.
# finalize_deliveries_errors < seção de entregas
finalize_deliveries_errors() {
  local kind a b title="" changed=0 value=0 n=0
  _finalize_delivery_check() {
    [ -n "$title" ] || return 0
    [ "$changed" -eq 1 ] || echo "entrega sem \"O que mudou\": ### $title"
    [ "$value" -eq 1 ] || echo "entrega sem \"Valor\": ### $title"
  }
  while IFS=$'\t' read -r kind a b; do
    case "$kind" in
      E) _finalize_delivery_check; title="$a"; changed=0; value=0; n=$((n + 1)) ;;
      F)
        case "$a" in
          "O que mudou") [ -n "$b" ] && changed=1 ;;
          Valor) [ -n "$b" ] && value=1 ;;
          Impacto) [ -n "$(_finalize_impact_class "$b")" ] || echo "impacto inválido em ### $title: '$b' (use alto, médio ou baixo)" ;;
        esac
        ;;
    esac
  done <<< "$(finalize_deliveries)"
  _finalize_delivery_check
  [ "$n" -gt 0 ] || echo "entregas sem nenhuma entrega: use um \"### \" por entrega"
}

# Data da última linha "conclusão" do registro de execução da tarefa, vazia se ela não tiver.
# finalize_task_done_date <arquivo da tarefa>
finalize_task_done_date() {
  markdown_section "$1" "$(config_get "$HARNESS_DIR/config/tasks.yml" record_section)" 2>/dev/null |
    awk -F'|' '
      { sub(/\r$/, "") }
      /^\| *[0-9]+ *\|/ {
        step = $4; gsub(/^[ \t]+|[ \t]+$/, "", step)
        if (step == "conclusão") { date = $3; gsub(/^[ \t]+|[ \t]+$/, "", date) }
      }
      END { printf "%s", date }
    '
}

# Um indicador de execução: valor, total e rótulo.
# _finalize_kpi <valor> <total> <rótulo>
_finalize_kpi() {
  printf '<div class="kpi"><b>%s<small> / %s</small></b><span>%s</span></div>\n' "$1" "$2" "$3"
}

# Indicadores de execução do executivo: tarefas `done`, tarefas com testes `passed` e evidências listadas (o --build só
# chega aqui com todas existentes).
# finalize_indicators_html <pasta das tarefas da spec>
finalize_indicators_html() {
  local task done total=0 n_done=0 n_passed=0 n_ev=0 name kind
  done="$(config_get "$HARNESS_DIR/config/tasks.yml" status.done)"
  for task in "$1"/*.md; do
    [ -f "$task" ] || continue
    total=$((total + 1))
    [ "$(frontmatter_get "$task" status 2>/dev/null || true)" = "$done" ] && n_done=$((n_done + 1))
    [ "$(frontmatter_get "$task" tests_result 2>/dev/null || true)" = passed ] && n_passed=$((n_passed + 1))
    while IFS=$'\t' read -r name kind; do
      [ -n "$name" ] && n_ev=$((n_ev + 1))
    done <<< "$(finalize_evidence_items "$task")"
  done
  echo '<div class="kpis">'
  _finalize_kpi "$n_done" "$total" "Tarefas concluídas"
  _finalize_kpi "$n_passed" "$total" "Tarefas com testes aprovados"
  _finalize_kpi "$n_ev" "$n_ev" "Evidências geradas"
  echo '</div>'
}

# Linha do tempo do executivo: uma tarefa por item, com a data da conclusão tirada do registro.
# finalize_timeline_html <pasta das tarefas da spec>
finalize_timeline_html() {
  local task date id title class
  for task in "$1"/*.md; do
    [ -f "$task" ] || continue
    date="$(finalize_task_done_date "$task")"
    id="$(frontmatter_get "$task" id | finalize_html_escape)"
    title="$(frontmatter_get "$task" title | finalize_html_escape)"
    class=""
    [ -n "$date" ] || { class=' class="pending"'; date="sem data"; }
    printf '      <li%s><span class="date">%s</span><span class="task"><span class="id">%s</span>%s</span></li>\n' \
      "$class" "$date" "$id" "$title"
  done
}

# Gera o documento executivo a partir do conteúdo, da spec e das tarefas, e imprime o caminho. Não usa a
# data da execução: gerar de novo com as mesmas entradas dá o mesmo arquivo.
# finalize_build_executive <SPEC_ID> <arquivo da spec> <pasta das tarefas da spec>
finalize_build_executive() {
  local id="$1" spec="$2" tasks_dir="$3" out title approved last task date tokens part deliveries count
  out="$(finalize_document_path "$id" executive)"
  tokens="$(finalize_css_tokens)" || return 1
  title="$(frontmatter_get "$spec" title | finalize_html_escape)"
  approved="$(finalize_approved_at "$spec")"
  last=""
  for task in "$tasks_dir"/*.md; do
    [ -f "$task" ] || continue
    date="$(finalize_task_done_date "$task")"
    [[ "$date" > "$last" ]] && last="$date"
  done
  approved="${approved%%T*}"
  part="$(finalize_content_part "$(finalize_content_path "$id")" Executivo)"
  deliveries="$(printf '%s\n' "$part" | finalize_part_section Entregas)"
  count="$(printf '%s\n' "$deliveries" | finalize_deliveries | grep -c '^E' || true)"
  mkdir -p "$(dirname "$out")"
  TPL_DOC_TITLE="$id · Resumo executivo · $title" \
  TPL_TOKENS="$tokens" \
  TPL_SPEC_ID="$id" \
  TPL_SPEC_TITLE="$title" \
  TPL_APPROVED="${approved:-data não registrada}" \
  TPL_FINISHED="${last:-data não registrada}" \
  TPL_LEDE="$(printf '%s\n' "$part" | finalize_part_section Panorama | finalize_block_html)" \
  TPL_VALUE_KPIS="$(printf '%s\n' "$part" | finalize_part_section 'Indicadores de valor' | finalize_value_kpis_html)" \
  TPL_SCOPE="$(printf '%s\n' "$part" | finalize_part_section Recorte | finalize_block_html)" \
  TPL_DELIVERY_COUNT="$count" \
  TPL_DELIVERY_TABLE="$(printf '%s\n' "$deliveries" | finalize_deliveries_table_html)" \
  TPL_DELIVERIES="$(printf '%s\n' "$deliveries" | finalize_deliveries_cards_html)" \
  TPL_BEFORE_AFTER="$(printf '%s\n' "$part" | finalize_part_section 'Antes e depois' | finalize_block_html)" \
  TPL_NEXT="$(printf '%s\n' "$part" | finalize_part_section 'Próximos passos' | finalize_block_html)" \
  TPL_RISKS="$(printf '%s\n' "$part" | finalize_part_section 'Riscos e limitações' | finalize_block_html)" \
  TPL_INDICATORS="$(finalize_indicators_html "$tasks_dir")" \
  TPL_TIMELINE="$(finalize_timeline_html "$tasks_dir")" \
    render "$(harness_path "$(config_get "$HARNESS_DIR/config/main.yml" paths.templates)")/$FINALIZE_EXECUTIVE_TEMPLATE" > "$out"
  printf '%s' "$out"
}

# Requisitos cobertos pelas tarefas, um por linha: ID, tabulação e as tarefas que o cobrem separadas por
# vírgula. A ordem é a da primeira aparição, tarefa a tarefa.
# finalize_requirement_map <pasta das tarefas da spec>
finalize_requirement_map() {
  local task covers tid req
  for task in "$1"/*.md; do
    [ -f "$task" ] || continue
    tid="$(frontmatter_get "$task" id)"
    covers="$(frontmatter_get "$task" covers 2>/dev/null | tr -d '[]' || true)"
    while IFS= read -r req; do
      [ -n "$req" ] && printf '%s\t%s\n' "$req" "$tid"
    done <<< "$(printf '%s' "$covers" | tr ',' '\n' | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')"
  done | awk -F'\t' '
    !($1 in tasks) { order[++n] = $1 }
    { tasks[$1] = ($1 in tasks) ? tasks[$1] "," $2 : $2 }
    END { for (i = 1; i <= n; i++) print order[i] "\t" tasks[order[i]] }
  '
}

# Descrição de um requisito na spec: a segunda célula da linha de tabela que começa com o ID, vazia se não achar.
# finalize_requirement_text <arquivo da spec> <ID do requisito>
finalize_requirement_text() {
  REQ="$2" awk -F'|' '
    { sub(/\r$/, "") }
    {
      id = $2; gsub(/^[ \t]+|[ \t]+$/, "", id)
      if (id == ENVIRON["REQ"]) { text = $3; gsub(/^[ \t]+|[ \t]+$/, "", text); print text; exit }
    }
  ' "$1"
}

# Tabela de requisitos com as tarefas que os cobrem.
# finalize_requirements_html <arquivo da spec> <pasta das tarefas da spec>
finalize_requirements_html() {
  local req tasks text chips t map
  map="$(finalize_requirement_map "$2")"
  if [ -z "$map" ]; then
    echo '<p>Nenhum requisito coberto pelas tarefas.</p>'
    return 0
  fi
  echo '<div class="tw"><table>'
  echo '<thead><tr><th>Requisito</th><th>Descrição</th><th>Tarefas</th></tr></thead>'
  echo '<tbody>'
  while IFS=$'\t' read -r req tasks; do
    text="$(finalize_requirement_text "$1" "$req")"
    chips=""
    while IFS= read -r t; do
      chips="$chips<span class=\"chip\">$(printf '%s' "$t" | finalize_html_escape)</span>"
    done <<< "$(printf '%s' "$tasks" | tr ',' '\n')"
    printf '<tr><td class="mono">%s</td><td>%s</td><td>%s</td></tr>\n' \
      "$(printf '%s' "$req" | finalize_html_escape)" "$(_finalize_inline "$text")" "$chips"
  done <<< "$map"
  echo '</tbody>'
  echo '</table></div>'
}

# Tabela dos arquivos alterados, com a ação de cada um.
# finalize_files_html <pasta das tarefas da spec>
finalize_files_html() {
  local action path items
  items="$(finalize_changed_items "$1")"
  if [ -z "$items" ]; then
    echo '<p>Nenhum arquivo alterado.</p>'
    return 0
  fi
  echo '<div class="tw"><table>'
  echo '<thead><tr><th>Ação</th><th>Caminho</th></tr></thead>'
  echo '<tbody>'
  while IFS=$'\t' read -r action path; do
    printf '<tr><td><span class="tag %s">%s</span></td><td class="path">%s</td></tr>\n' \
      "$(printf '%s' "$action" | finalize_html_escape)" "$(printf '%s' "$action" | finalize_html_escape)" \
      "$(printf '%s' "$path" | finalize_html_escape)"
  done <<< "$items"
  echo '</tbody>'
  echo '</table></div>'
}

# Tabela dos commits das fontes: hash, data e mensagem. Sem commits, imprime o aviso das fontes
# ("Nenhum commit encontrado.", "Fora de um repositório git." ou "Spec sem data de aprovação.").
# finalize_commits_html <arquivo da spec> <pasta das tarefas da spec>
finalize_commits_html() {
  local lines line hash date message re='^-\ `([0-9a-f]+)`\ ([0-9-]+)\ (.*)$' found=0 rows=""
  lines="$(finalize_sources_commits "$1" "$2" | sed '1,2d')"
  while IFS= read -r line; do
    if [[ "$line" =~ $re ]]; then
      found=1
      hash="${BASH_REMATCH[1]}"; date="${BASH_REMATCH[2]}"; message="${BASH_REMATCH[3]}"
      rows="$rows$(printf '<tr><td class="mono">%s</td><td class="mono">%s</td><td>%s</td></tr>' \
        "$hash" "$date" "$(printf '%s' "$message" | finalize_html_escape)")"$'\n'
    fi
  done <<< "$lines"
  if [ "$found" -eq 0 ]; then
    printf '<p>%s</p>\n' "$(printf '%s' "$lines" | head -1 | finalize_html_escape)"
    return 0
  fi
  echo '<div class="tw"><table>'
  echo '<thead><tr><th>Commit</th><th>Data</th><th>Mensagem</th></tr></thead>'
  echo '<tbody>'
  printf '%s' "$rows"
  echo '</tbody>'
  echo '</table></div>'
}

# Último commit anterior à aprovação da spec, base do diff do documento técnico. Termina com erro, imprimindo o
# motivo na saída de erro, se não houver base: fora de um repositório git, spec sem data de aprovação ou sem
# commit anterior a ela.
# finalize_diff_base <arquivo da spec>
finalize_diff_base() {
  local root approved base
  root="$(_task_root_dir)"
  if ! git -C "$root" rev-parse --is-inside-work-tree > /dev/null 2>&1; then
    echo "fora de um repositório git" >&2
    return 1
  fi
  approved="$(finalize_approved_at "$1")"
  if [ -z "$approved" ]; then
    echo "a spec não tem data de aprovação" >&2
    return 1
  fi
  base="$(git -C "$root" rev-list -1 --before="$approved" HEAD 2>/dev/null || true)"
  if [ -z "$base" ]; then
    echo "nenhum commit anterior à aprovação da spec" >&2
    return 1
  fi
  printf '%s' "$base"
}

# Converte um diff unificado, lido da entrada padrão, em linhas de tabela de duas colunas (antes e depois),
# com o número de cada linha, o contexto e as linhas removidas e acrescentadas destacadas. Remoções e
# acréscimos seguidos ficam lado a lado, na ordem. Todo o código sai escapado.
# finalize_diff_rows < diff
finalize_diff_rows() {
  awk '
    function esc(s) {
      gsub(/&/, "\\&amp;", s); gsub(/</, "\\&lt;", s); gsub(/>/, "\\&gt;", s)
      gsub(/"/, "\\&quot;", s); gsub(/\047/, "\\&#39;", s)
      return s
    }
    function cell(num, text, cls) {
      return "<td class=\"ln\">" num "</td><td class=\"code" cls "\">" esc(text) "</td>"
    }
    function flush(   n, i, l, r) {
      n = (nd > na) ? nd : na
      for (i = 1; i <= n; i++) {
        l = (i <= nd) ? cell(dn[i], dt[i], " del") : "<td class=\"ln\"></td><td class=\"code empty\"></td>"
        r = (i <= na) ? cell(an[i], at[i], " add") : "<td class=\"ln\"></td><td class=\"code empty\"></td>"
        print "<tr>" l r "</tr>"
      }
      nd = 0; na = 0
    }
    { sub(/\r$/, "") }
    /^@@ / {
      flush(); inhunk = 1
      o = $2; sub(/^-/, "", o); sub(/,.*/, "", o); oldn = o + 0
      w = $3; sub(/^\+/, "", w); sub(/,.*/, "", w); newn = w + 0
      c = $0; sub(/^@@ [^@]*@@ ?/, "", c)
      print "<tr class=\"hunk\"><td colspan=\"4\">" esc($1 " " $2 " " $3 " " $4 (c == "" ? "" : " " c)) "</td></tr>"
      next
    }
    !inhunk { next }
    /^\\/ { next }
    /^-/ { dn[++nd] = oldn++; dt[nd] = substr($0, 2); next }
    /^\+/ { an[++na] = newn++; at[na] = substr($0, 2); next }
    {
      flush()
      t = substr($0, 2)
      print "<tr>" cell(oldn++, t, "") cell(newn++, t, "") "</tr>"
    }
    END { flush() }
  '
}

# Caminhos com ação `alterar` nas tarefas que nenhuma tarefa cria, um por linha, sem repetição.
# finalize_modified_paths <pasta das tarefas da spec>
finalize_modified_paths() {
  finalize_changed_items "$1" | awk -F'\t' '
    $1 == "criar" { created[$2] = 1; next }
    $1 == "alterar" && !($2 in seen) { seen[$2] = 1; order[++n] = $2 }
    END { for (i = 1; i <= n; i++) if (!(order[i] in created)) print order[i] }
  '
}

# Seção de diffs do técnico: um bloco expansível por arquivo alterado, com o diff lado a lado entre a base e o
# estado atual. Sem base de comparação, arquivo binário ou arquivo sem mudança, informa em vez de falhar.
# finalize_diffs_html <arquivo da spec> <pasta das tarefas da spec>
finalize_diffs_html() {
  local base reason path root stat add del paths
  root="$(_task_root_dir)"
  if ! base="$(finalize_diff_base "$1" 2>&1)"; then
    reason="$base"
    printf '<p>Sem base de comparação: %s.</p>\n' "$(printf '%s' "$reason" | finalize_html_escape)"
    return 0
  fi
  paths="$(finalize_modified_paths "$2")"
  if [ -z "$paths" ]; then
    echo '<p>Nenhum arquivo alterado, só criados ou removidos.</p>'
    return 0
  fi
  while IFS= read -r path; do
    [ -n "$path" ] || continue
    stat="$(git -C "$root" -c core.quotepath=off diff --numstat --no-color --no-ext-diff "$base" -- "$path" 2>/dev/null | head -1)"
    add="$(printf '%s' "$stat" | cut -f1)"; del="$(printf '%s' "$stat" | cut -f2)"
    echo '<details class="dfile">'
    if [ -z "$stat" ]; then
      printf '<summary><b class="path">%s</b><span class="tag">sem mudanças</span></summary>\n' "$(printf '%s' "$path" | finalize_html_escape)"
      echo '<div class="dbody"><p>Sem mudanças desde a base de comparação.</p></div>'
    elif [ "$add" = "-" ]; then
      printf '<summary><b class="path">%s</b><span class="tag">binário</span></summary>\n' "$(printf '%s' "$path" | finalize_html_escape)"
      echo '<div class="dbody"><p>Arquivo binário alterado.</p></div>'
    else
      printf '<summary><b class="path">%s</b><span class="tag criar">+%s</span><span class="tag remover">−%s</span></summary>\n' \
        "$(printf '%s' "$path" | finalize_html_escape)" "$add" "$del"
      echo '<div class="dbody"><div class="tw"><table class="diff">'
      echo '<colgroup><col class="cln"><col><col class="cln"><col></colgroup>'
      git -C "$root" -c core.quotepath=off diff --no-color --no-ext-diff --no-textconv -U3 "$base" -- "$path" 2>/dev/null | finalize_diff_rows
      echo '</table></div></div>'
    fi
    echo '</details>'
  done <<< "$paths"
}

# Gera o documento técnico a partir do conteúdo, da spec e das tarefas, e imprime o caminho. Como o
# executivo, não usa a data da execução.
# finalize_build_technical <SPEC_ID> <arquivo da spec> <pasta das tarefas da spec>
finalize_build_technical() {
  local id="$1" spec="$2" tasks_dir="$3" out title approved last task date tokens part
  out="$(finalize_document_path "$id" technical)"
  tokens="$(finalize_css_tokens)" || return 1
  title="$(frontmatter_get "$spec" title | finalize_html_escape)"
  approved="$(finalize_approved_at "$spec")"
  last=""
  for task in "$tasks_dir"/*.md; do
    [ -f "$task" ] || continue
    date="$(finalize_task_done_date "$task")"
    [[ "$date" > "$last" ]] && last="$date"
  done
  approved="${approved%%T*}"
  part="$(finalize_content_part "$(finalize_content_path "$id")" Técnico)"
  mkdir -p "$(dirname "$out")"
  TPL_DOC_TITLE="$id · Documento técnico · $title" \
  TPL_TOKENS="$tokens" \
  TPL_SPEC_ID="$id" \
  TPL_SPEC_TITLE="$title" \
  TPL_APPROVED="${approved:-data não registrada}" \
  TPL_FINISHED="${last:-data não registrada}" \
  TPL_OVERVIEW="$(printf '%s\n' "$part" | finalize_part_section 'Visão geral' | finalize_block_html)" \
  TPL_ARCHITECTURE="$(printf '%s\n' "$part" | finalize_part_section 'Arquitetura e componentes' | finalize_block_html)" \
  TPL_DONE="$(printf '%s\n' "$part" | finalize_part_section 'O que foi feito' | finalize_block_html)" \
  TPL_REQUIREMENTS="$(finalize_requirements_html "$spec" "$tasks_dir")" \
  TPL_FILES="$(finalize_files_html "$tasks_dir")" \
  TPL_DIFFS="$(finalize_diffs_html "$spec" "$tasks_dir")" \
  TPL_COMMITS="$(finalize_commits_html "$spec" "$tasks_dir")" \
  TPL_DECISIONS="$(printf '%s\n' "$part" | finalize_part_section 'Decisões' | finalize_block_html)" \
  TPL_LEARNINGS="$(printf '%s\n' "$part" | finalize_part_section 'Aprendizados' | finalize_block_html)" \
  TPL_ATTENTION="$(printf '%s\n' "$part" | finalize_part_section 'Pontos de atenção' | finalize_block_html)" \
  TPL_REFERENCES="$(printf '%s\n' "$part" | finalize_part_section 'Referências' | finalize_block_html)" \
    render "$(harness_path "$(config_get "$HARNESS_DIR/config/main.yml" paths.templates)")/$FINALIZE_TECHNICAL_TEMPLATE" > "$out"
  printf '%s' "$out"
}
