# Registro de ajustes de documentos com histórico: specs refinadas (adjust-spec) e planos de tarefas
# (adjust-plan). Cada ajuste incrementa `adjustments`, atualiza `adjusted_at`, `updated_at` e o hash
# do conteúdo (`content_hash_field`) e acrescenta uma linha à seção `adjustment_history_section`.
# Origens, resumo do ajuste manual e seção de histórico vêm de config/specs.yml.
# Uso: source "$HARNESS_DIR/scripts/lib/adjustments.sh"
#      (requer HARNESS_DIR e as libs config.sh, text.sh, frontmatter.sh, markdown.sh e spec-validation.sh)

_adjustment_config() {
  config_get "$HARNESS_DIR/config/specs.yml" "$1"
}

# Origem registrada no histórico: `manual` ou `agent` (chaves de `adjustment_origins`).
# adjustment_origin <manual|agent>
adjustment_origin() {
  _adjustment_config "adjustment_origins.$1"
}

# Limpa o resumo de um ajuste do agente e o imprime. Falha, com mensagem, se ele estiver vazio
# ou tiver quebra de linha.
# adjustment_clean_summary <resumo>
adjustment_clean_summary() {
  local summary
  summary="$(printf '%s' "$1" | sed -E 's/^[[:space:]]+//; s/[[:space:]]+$//')"
  if [ -z "$summary" ]; then
    echo "o resumo do ajuste está vazio" >&2; return 1
  elif [[ "$summary" == *$'\n'* ]]; then
    echo "o resumo não pode ter quebra de linha; separe os itens com '; '" >&2; return 1
  fi
  printf '%s' "$summary"
}

# Estado do hash do conteúdo: `same` ou `changed`. Sem hash gravado, o estado é `changed`.
# adjustment_hash_state <arquivo>
adjustment_hash_state() {
  local stored
  stored="$(frontmatter_get "$1" "$(_adjustment_config content_hash_field)" 2>/dev/null || true)"
  if [ "$stored" = "$(spec_content_hash "$1")" ]; then
    echo same
  else
    echo changed
  fi
}

# Grava o hash do conteúdo atual no front-matter.
# adjustment_store_hash <arquivo>
adjustment_store_hash() {
  frontmatter_put "$1" "$(_adjustment_config content_hash_field)" "$(spec_content_hash "$1")"
}

# Seções (títulos `## `) com linhas alteradas desde o último commit, separadas por vírgula.
# Não imprime nada se o arquivo não estiver versionado no git ou não tiver alterações.
# adjustment_changed_sections <arquivo>
adjustment_changed_sections() {
  local dir base lines
  dir="$(dirname "$1")"
  base="$(basename "$1")"

  command -v git > /dev/null 2>&1 || return 0
  git -C "$dir" ls-files --error-unmatch -- "$base" > /dev/null 2>&1 || return 0
  lines="$(git -C "$dir" diff -U0 HEAD -- "$base" 2>/dev/null | sed -nE 's/^@@ -[0-9,]+ \+([0-9]+).*/\1/p')"
  [ -n "$lines" ] || return 0

  CHANGED_LINES="$lines" SKIP_HEADING="$(spec_section_heading "$(_adjustment_config adjustment_history_section)")" awk '
    BEGIN { n = split(ENVIRON["CHANGED_LINES"], list, "\n"); for (i = 1; i <= n; i++) changed[list[i]] = 1 }
    { sub(/\r$/, "") }
    NR == 1 && $0 == "---" { in_fm = 1; next }
    in_fm { if ($0 == "---") in_fm = 0; next }
    /^[ \t]*```/ { in_code = !in_code }
    !in_code && /^## / { heading = substr($0, 4) }
    (NR in changed) && heading != "" && heading != ENVIRON["SKIP_HEADING"] && !(heading in seen) {
      seen[heading] = 1
      out = (out == "" ? heading : out ", " heading)
    }
    END { if (out != "") print out }
  ' "$1"
}

# Resumo de um ajuste manual: `manual_adjustment_summary`, com as seções alteradas quando houver.
# adjustment_manual_summary <arquivo>
adjustment_manual_summary() {
  local sections
  sections="$(adjustment_changed_sections "$1")"
  printf '%s%s' "$(_adjustment_config manual_adjustment_summary)" "${sections:+: $sections}"
}

# Acrescenta uma linha à tabela da seção de histórico. Sem a seção, falha sem alterar o arquivo.
# adjustment_append_history_row <arquivo> <linha>
adjustment_append_history_row() {
  local file="$1" row="$2" section tmp
  section="$(_adjustment_config adjustment_history_section)"
  tmp="$(mktemp)"

  if HISTORY_HEADING="## $(spec_section_heading "$section")" HISTORY_ROW="$row" awk '
    FNR == 1 { pass++ }
    { plain = $0; sub(/\r$/, "", plain) }
    pass == 1 {
      if (plain ~ /^[ \t]*```/) { in_code = !in_code; next }
      if (in_code) next
      if (plain == ENVIRON["HISTORY_HEADING"]) { inside = 1; at = FNR; next }
      if (inside && plain ~ /^## /) inside = 0
      if (inside && plain ~ /^\|/) at = FNR
      next
    }
    { print }
    FNR == at { print ENVIRON["HISTORY_ROW"] }
    END { exit !at }
  ' "$file" "$file" > "$tmp"; then
    cat "$tmp" > "$file"
    rm -f "$tmp"
  else
    rm -f "$tmp"
    echo "seção ausente: '## $(spec_section_heading "$section")'; nenhum ajuste registrado" >&2
    return 1
  fi
}

# Registra um ajuste: linha no histórico, contagem, datas e, por último, o hash do conteúdo.
# Imprime o número do ajuste.
# adjustment_record <arquivo> <origem> <resumo>
adjustment_record() {
  local file="$1" origin="$2" summary="$3" count now

  count="$(frontmatter_get "$file" adjustments 2>/dev/null || true)"
  [[ "$count" =~ ^[0-9]+$ ]] || count=0
  count=$((10#$count + 1))
  now="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

  # Barras verticais do resumo são escapadas para não quebrar a tabela.
  adjustment_append_history_row "$file" "| $count | ${now%%T*} | $origin | $(printf '%s' "$summary" | sed 's/|/\\|/g') |" || return 1
  frontmatter_put "$file" adjustments "$count"
  frontmatter_put "$file" adjusted_at "$now"
  frontmatter_put "$file" updated_at "$now"
  adjustment_store_hash "$file"
  printf '%s' "$count"
}
