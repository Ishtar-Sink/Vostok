# Plano de tarefas: leitura, cálculo do nível de esforço, validação, cobertura e dependências.
# Compartilhado por plan-tasks.sh, adjust-plan.sh e approve-plan.sh. Níveis, pesos, limites, tipos
# e seções de cobertura vêm de config/tasks.yml.
# Uso: source "$HARNESS_DIR/scripts/lib/plan.sh"
#      (requer HARNESS_DIR e as libs config.sh, text.sh, frontmatter.sh, markdown.sh e spec-validation.sh)

# Títulos das seções do plano (templates/plan/plan.md).
PLAN_TASKS_HEADING="Tarefas"
PLAN_FACTS_HEADING="Fatos de esforço"
PLAN_OVERVIEW_HEADING="Visão geral"

_tasks_config() {
  printf '%s/config/tasks.yml' "$HARNESS_DIR"
}

# Pasta dos planos (`paths.plans` em main.yml).
plan_dir() {
  harness_path "$(config_get "$HARNESS_DIR/config/main.yml" paths.plans)"
}

# Caminho do plano de uma spec (existindo ou não).
# plan_path <SPEC_ID>
plan_path() {
  printf '%s/%s.md' "$(plan_dir)" "$1"
}

# Caminho da spec refinada de um ID. Falha se ela não existir.
# plan_spec_file <SPEC_ID>
plan_spec_file() {
  local specs_dir
  specs_dir="$(harness_path "$(config_get "$HARNESS_DIR/config/main.yml" paths.specs)")"
  spec_find_by_id "$specs_dir/$(config_get "$(_spec_config)" refined_dir)" "$1"
}

# Termina com sucesso se o plano foi feito para outra versão da spec: `spec_hash` diferente do
# `approved_hash` atual, ou spec sem aprovação.
# plan_is_outdated <plano> <spec>
plan_is_outdated() {
  local approved
  approved="$(frontmatter_get "$2" "$(config_get "$(_spec_config)" approved_hash_field)" 2>/dev/null || true)"
  [ -z "$approved" ] || [ "$approved" = null ] || [ "$(frontmatter_get "$1" spec_hash 2>/dev/null || true)" != "$approved" ]
}

# IDs da spec que o plano precisa cobrir, um por linha: primeira coluna das tabelas das seções
# `coverage_<tipo>` (config/tasks.yml).
# plan_spec_ids <spec>
plan_spec_ids() {
  local type section
  type="$(frontmatter_get "$1" type 2>/dev/null || true)"
  config_list "$(_tasks_config)" "coverage_$type" 2>/dev/null | while IFS= read -r section; do
    markdown_section "$1" "$(spec_section_heading "$section")" 2>/dev/null | markdown_table_rows |
      while IFS= read -r row; do
        row="$(markdown_cell "$row" 1)"
        [ "$row" = "-" ] || printf '%s\n' "$row"
      done
  done
}

# Linhas das tabelas de tarefas (prefixo T) e de fatos (prefixo F) do plano.
_plan_table_rows() {
  markdown_section "$1" "$PLAN_TASKS_HEADING" 2>/dev/null | markdown_table_rows | sed 's/^/T/'
  markdown_section "$1" "$PLAN_FACTS_HEADING" 2>/dev/null | markdown_table_rows | sed 's/^/F/'
}

# Analisa o plano e imprime linhas separadas por tabulação:
#   level <ID> <nível calculado> <nível gravado>
#   error <problema>
# Só avalia tarefas e fatos; cobertura e variáveis ficam em plan_errors.
_plan_analyze() {
  local levels limits level digits id_re i
  levels="$(config_list "$(_tasks_config)" effort_levels | tr '\n' ' ')"
  limits=""
  # Expressão do ID sem intervalos `{n}`, que o awk antigo do macOS não suporta.
  digits="$(config_get "$(_tasks_config)" id.digits)"
  id_re="^$(config_get "$(_tasks_config)" id.prefix)-"
  for ((i = 0; i < digits; i++)); do id_re="$id_re[0-9]"; done
  id_re="$id_re\$"
  for level in $levels; do
    limits="$limits$level=$(config_get "$(_tasks_config)" "effort_limits.$level" 2>/dev/null || echo -) "
  done

  _plan_table_rows "$1" | \
  LEVELS="$levels" LIMITS="$limits" \
  TYPES="$(config_list "$(_tasks_config)" task_types | tr '\n' ' ')" \
  ID_RE="$id_re" \
  W_CREATE="$(config_get "$(_tasks_config)" effort_weights.files_create)" \
  W_CHANGE="$(config_get "$(_tasks_config)" effort_weights.files_change)" \
  W_IDS="$(config_get "$(_tasks_config)" effort_weights.ids_covered)" \
  W_CONTRACT="$(config_get "$(_tasks_config)" effort_weights.public_contract)" \
  W_INFERENCE="$(config_get "$(_tasks_config)" effort_weights.inference)" \
  W_RISK="$(config_get "$(_tasks_config)" effort_weights.high_risk)" \
  awk '
    function trim(s) { gsub(/^[ \t]+|[ \t]+$/, "", s); return s }
    function err(msg) { printf "error\t%s\n", msg }
    function yesno(v, id, name) {
      v = tolower(v)
      if (v == "sim") return 1
      if (v == "não" || v == "nao") return 0
      err(id ": fato \"" name "\" inválido: \"" v "\" (use sim ou não)"); bad[id] = 1; return 0
    }
    function number(v, id, name) {
      if (v ~ /^[0-9]+$/) return v + 0
      err(id ": fato \"" name "\" inválido: \"" v "\" (use um número inteiro)"); bad[id] = 1; return 0
    }
    # Busca em profundidade: 1 em andamento, 2 concluído. Um vértice em andamento revisitado fecha um ciclo.
    function visit(id,   n, i, list, d) {
      state[id] = 1
      n = split(deps[id], list, " ")
      for (i = 1; i <= n; i++) {
        d = list[i]
        if (!(d in task)) continue
        if (state[d] == 1) { if (!(d in cycled)) { cycled[d] = 1; err("dependência em ciclo envolvendo " d) } }
        else if (state[d] == 0) visit(d)
      }
      state[id] = 2
    }
    BEGIN {
      nl = split(ENVIRON["LEVELS"], level_names, " ")
      split(ENVIRON["LIMITS"], pairs, " ")
      for (i in pairs) { split(pairs[i], kv, "="); limit[kv[1]] = kv[2] }
      nt = split(ENVIRON["TYPES"], type_list, " ")
      for (i = 1; i <= nt; i++) valid_type[type_list[i]] = 1
    }
    {
      kind = substr($0, 1, 1); line = substr($0, 2)
      n = split(line, c, "|")
      id = trim(c[2])
      if (kind == "T") {
        if (id == "-") next
        if (n != 9) { err("linha da tabela de tarefas com " (n - 2) " colunas (esperado 7): " line); next }
        if (id !~ ENVIRON["ID_RE"]) err("ID de tarefa inválido: \"" id "\"")
        if (id in task) err("ID de tarefa repetido: " id)
        task[id] = 1; order[++count] = id
        if (!(trim(c[5]) in valid_type)) err(id ": tipo inválido \"" trim(c[5]) "\" (use: " ENVIRON["TYPES"] ")")
        stored[id] = trim(c[6])
        dep = trim(c[7]); gsub(/,/, " ", dep); if (dep == "-") dep = ""
        deps[id] = dep
        cover = trim(c[8]); gsub(/,/, " ", cover); covered[id] = (cover == "-") ? 0 : split(cover, tmp, " ")
      } else {
        if (id == "-") next
        if (n != 8) { err("linha da tabela de fatos com " (n - 2) " colunas (esperado 6): " line); next }
        if (id in fact) err("fatos repetidos para " id)
        fact[id] = 1
        score[id] = number(trim(c[3]), id, "Arquivos a criar") * ENVIRON["W_CREATE"] \
                  + number(trim(c[4]), id, "Arquivos a alterar") * ENVIRON["W_CHANGE"] \
                  + yesno(trim(c[5]), id, "Contrato público") * ENVIRON["W_CONTRACT"] \
                  + yesno(trim(c[6]), id, "Inferência") * ENVIRON["W_INFERENCE"] \
                  + yesno(trim(c[7]), id, "Risco alto") * ENVIRON["W_RISK"]
      }
    }
    END {
      if (count == 0) err("a tabela de tarefas está vazia")
      for (i = 1; i <= count; i++) {
        id = order[i]
        n = split(deps[id], list, " ")
        for (j = 1; j <= n; j++) {
          if (list[j] == id) err(id ": depende de si mesma")
          else if (!(list[j] in task)) err(id ": dependência inexistente " list[j])
        }
        if (!(id in fact)) { err(id ": sem fatos de esforço"); continue }
        if (id in bad) continue
        total = score[id] + covered[id] * ENVIRON["W_IDS"]
        chosen = level_names[nl]
        for (k = 1; k <= nl; k++)
          if (limit[level_names[k]] != "-" && limit[level_names[k]] != "" && total <= limit[level_names[k]] + 0) { chosen = level_names[k]; break }
        printf "level\t%s\t%s\t%s\n", id, chosen, stored[id]
      }
      for (id in fact) if (!(id in task)) err("fatos de esforço de uma tarefa inexistente: " id)
      for (i = 1; i <= count; i++) if (state[order[i]] == 0) visit(order[i])
    }
  '
}

# Grava na coluna "Nível" o nível calculado de cada tarefa. Com fatos inválidos ou problemas na
# tabela, imprime os problemas e falha sem alterar o plano.
# plan_score <plano>
plan_score() {
  local analysis errors tmp
  analysis="$(_plan_analyze "$1")"
  errors="$(printf '%s\n' "$analysis" | awk -F'\t' '$1 == "error" { print $2 }')"
  if [ -n "$errors" ]; then
    printf '%s\n' "$errors"
    return 1
  fi

  tmp="$(mktemp)"
  LEVELS_BY_ID="$(printf '%s\n' "$analysis" | awk -F'\t' '$1 == "level" { print $2 "=" $3 }')" \
  HEADING="## $PLAN_TASKS_HEADING" awk '
    BEGIN { n = split(ENVIRON["LEVELS_BY_ID"], pairs, "\n"); for (i = 1; i <= n; i++) { split(pairs[i], kv, "="); lvl[kv[1]] = kv[2] } }
    { plain = $0; sub(/\r$/, "", plain) }
    /^[ \t]*```/ { in_code = !in_code }
    !in_code && plain == ENVIRON["HEADING"] { inside = 1; print; next }
    !in_code && inside && plain ~ /^## / { inside = 0 }
    inside && plain ~ /^\|/ {
      if (!header) { header = 1; print; next }
      if (plain ~ /^\|[ \t:|-]+\|[ \t]*$/ && plain ~ /---/) { print; next }
      n = split(plain, c, "|"); id = c[2]; gsub(/^[ \t]+|[ \t]+$/, "", id)
      if (n == 9 && (id in lvl)) {
        c[6] = " " lvl[id] " "
        out = c[1]; for (i = 2; i <= n; i++) out = out "|" c[i]
        print out; next
      }
    }
    { print }
  ' "$1" > "$tmp"
  cat "$tmp" > "$1"
  rm -f "$tmp"
}

# Imprime um problema por linha. Não imprime nada se o plano estiver válido.
# plan_errors <plano> <spec>
plan_errors() {
  local plan="$1" spec="$2" analysis covered ids id

  template_leftovers "$plan" | sed 's/^/variável sem preencher: /' || true
  grep -q '[^[:space:]]' <<< "$(markdown_section "$plan" "$PLAN_OVERVIEW_HEADING" 2>/dev/null)" ||
    echo "a seção '$PLAN_OVERVIEW_HEADING' está vazia"

  analysis="$(_plan_analyze "$plan")"
  printf '%s\n' "$analysis" | awk -F'\t' '
    $1 == "error" { print $2 }
    $1 == "level" && $3 != $4 { print $2 ": nível \"" $4 "\" diferente do calculado \"" $3 "\" (execute --score)" }
  '

  # Cobertura: todo ID da spec em alguma tarefa, e nenhuma tarefa cobrindo ID que não existe.
  covered="$(markdown_section "$plan" "$PLAN_TASKS_HEADING" 2>/dev/null | markdown_table_rows |
    while IFS= read -r row; do markdown_cell "$row" 7; done | tr ',' '\n' | sed 's/^[[:space:]]*//; s/[[:space:]]*$//' | grep -v '^-\{0,1\}$' || true)"
  ids="$(plan_spec_ids "$spec")"
  printf '%s\n' "$ids" | while IFS= read -r id; do
    [ -n "$id" ] || continue
    grep -qxF "$id" <<< "$covered" || echo "$id não é coberto por nenhuma tarefa"
  done
  printf '%s\n' "$covered" | sort -u | while IFS= read -r id; do
    [ -n "$id" ] || continue
    grep -qxF "$id" <<< "$ids" || echo "a tarefa cobre $id, que não existe na spec"
  done
}

# Linhas da tabela de tarefas do plano, uma por tarefa, com os campos separados por tabulação:
# ID, título, descrição, tipo, nível, dependências e IDs cobertos.
# plan_tasks <plano>
plan_tasks() {
  markdown_section "$1" "$PLAN_TASKS_HEADING" 2>/dev/null | markdown_table_rows | awk -F'|' '
    function trim(s) { gsub(/^[ \t]+|[ \t]+$/, "", s); return s }
    NF == 9 && trim($2) != "-" { print trim($2) "\t" trim($3) "\t" trim($4) "\t" trim($5) "\t" trim($6) "\t" trim($7) "\t" trim($8) }
  '
}

# Linhas da tabela "Níveis de esforço": `| nível | descrição |` para cada nível configurado.
plan_effort_level_rows() {
  config_list "$(_tasks_config)" effort_levels | while IFS= read -r level; do
    printf '| %s | %s |\n' "$level" "$(config_get "$(_tasks_config)" "effort_descriptions.$level")"
  done
}
