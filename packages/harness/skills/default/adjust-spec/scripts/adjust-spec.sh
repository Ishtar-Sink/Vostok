#!/usr/bin/env bash
# Registra os ajustes de uma spec refinada: edições feitas à mão e ajustes feitos pelo agente.
# Cada ajuste incrementa `adjustments`, atualiza `adjusted_at`, `updated_at` e o hash do conteúdo
# (`content_hash_field`) e acrescenta uma linha à seção de histórico (`adjustment_history_section`).
# A validação é a mesma do refinamento (scripts/lib/spec-validation.sh).
# Specs aprovadas também são ajustadas: todo ajuste registrado derruba a aprovação, volta o status
# para status.refined e grava `null` nos campos da aprovação.
#
# Uso: adjust-spec.sh --record-manual <SPEC_ID>
#        Compara o hash gravado com o do conteúdo atual. Se forem diferentes, registra um ajuste
#        de origem manual e lista os problemas de validação, se houver. Se a spec não tiver hash,
#        só grava o hash atual.
#      adjust-spec.sh --validate <SPEC_ID>
#        Lista os problemas da refinada, sem alterar nada. Termina com erro se houver problemas.
#      adjust-spec.sh --record <SPEC_ID> "resumo"
#        Valida a refinada e registra o ajuste do agente. Com problemas, termina com erro sem
#        alterar nada. Vários ajustes da mesma mensagem vão no mesmo resumo, separados por "; ".
set -euo pipefail

# Raiz do harness: primeira pasta acima do script que contém `config/main.yml`.
HARNESS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && until [ -f config/main.yml ] || [ "$PWD" = / ]; do cd ..; done && pwd)"
source "$HARNESS_DIR/scripts/lib/config.sh"
source "$HARNESS_DIR/scripts/lib/text.sh"
source "$HARNESS_DIR/scripts/lib/frontmatter.sh"
source "$HARNESS_DIR/scripts/lib/markdown.sh"
source "$HARNESS_DIR/scripts/lib/spec-validation.sh"

MAIN_CONFIG="$HARNESS_DIR/config/main.yml"
SPECS_CONFIG="$HARNESS_DIR/config/specs.yml"

USAGE='Uso: adjust-spec.sh --record-manual <SPEC_ID> | --validate <SPEC_ID> | --record <SPEC_ID> "resumo"'

die() { echo "Erro: $*" >&2; exit 1; }

specs_dir="$(harness_path "$(config_get "$MAIN_CONFIG" paths.specs)")"
draft_dir="$specs_dir/$(config_get "$SPECS_CONFIG" draft_dir)"
refined_dir="$specs_dir/$(config_get "$SPECS_CONFIG" refined_dir)"
refined_status="$(config_get "$SPECS_CONFIG" status.refined)"
approved_status="$(config_get "$SPECS_CONFIG" status.approved)"
approved_at_field="$(config_get "$SPECS_CONFIG" approved_at_field)"
approved_hash_field="$(config_get "$SPECS_CONFIG" approved_hash_field)"
hash_field="$(config_get "$SPECS_CONFIG" content_hash_field)"
history_section="$(config_get "$SPECS_CONFIG" adjustment_history_section)"
manual_origin="$(config_get "$SPECS_CONFIG" adjustment_origins.manual)"
agent_origin="$(config_get "$SPECS_CONFIG" adjustment_origins.agent)"
manual_summary="$(config_get "$SPECS_CONFIG" manual_adjustment_summary)"

# Caminho da refinada. Falha se ela não existir ou não estiver com status.refined ou status.approved.
find_refined() {
  local spec_id="$1" refined status

  if ! refined="$(spec_find_by_id "$refined_dir" "$spec_id")"; then
    spec_find_by_id "$draft_dir" "$spec_id" > /dev/null &&
      die "a spec $spec_id ainda não foi refinada; só specs refinadas são ajustadas"
    die "spec refinada $spec_id não encontrada em $refined_dir"
  fi

  status="$(frontmatter_get "$refined" status 2>/dev/null || true)"
  [ "$status" = "$refined_status" ] || [ "$status" = "$approved_status" ] ||
    die "a spec $spec_id está com status '$status'; só specs com status '$refined_status' ou '$approved_status' são ajustadas"
  printf '%s' "$refined"
}

# Seções (títulos `## `) com linhas alteradas desde o último commit, separadas por vírgula.
# Não imprime nada se o arquivo não estiver versionado no git ou não tiver alterações.
changed_sections() {
  local dir base lines
  dir="$(dirname "$1")"
  base="$(basename "$1")"

  command -v git > /dev/null 2>&1 || return 0
  git -C "$dir" ls-files --error-unmatch -- "$base" > /dev/null 2>&1 || return 0
  lines="$(git -C "$dir" diff -U0 HEAD -- "$base" 2>/dev/null | sed -nE 's/^@@ -[0-9,]+ \+([0-9]+).*/\1/p')"
  [ -n "$lines" ] || return 0

  CHANGED_LINES="$lines" SKIP_HEADING="$(spec_section_heading "$history_section")" awk '
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

# Acrescenta uma linha à tabela da seção de histórico. Sem a seção, cria-a no fim do arquivo.
# append_history_row <arquivo> <linha>
append_history_row() {
  local file="$1" row="$2" tmp
  tmp="$(mktemp)"

  if HISTORY_HEADING="## $(spec_section_heading "$history_section")" HISTORY_ROW="$row" awk '
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
  else
    [ -z "$(tail -c 1 "$file")" ] || echo >> "$file"
    { echo; cat "$(spec_section_template "$history_section")"; echo "$row"; } >> "$file"
  fi
  rm -f "$tmp"
}

# Registra um ajuste: linha no histórico, contagem, datas e, por último, o hash do conteúdo.
# record_adjustment <arquivo> <origem> <resumo>
record_adjustment() {
  local file="$1" origin="$2" summary="$3" count now

  count="$(frontmatter_get "$file" adjustments 2>/dev/null || true)"
  [[ "$count" =~ ^[0-9]+$ ]] || count=0
  count=$((10#$count + 1))
  now="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

  # Barras verticais do resumo são escapadas para não quebrar a tabela.
  append_history_row "$file" "| $count | ${now%%T*} | $origin | $(printf '%s' "$summary" | sed 's/|/\\|/g') |"
  frontmatter_put "$file" adjustments "$count"
  frontmatter_put "$file" adjusted_at "$now"
  frontmatter_put "$file" updated_at "$now"
  frontmatter_put "$file" "$hash_field" "$(spec_content_hash "$file")"
  printf '%s' "$count"
}

# Derruba a aprovação de uma spec aprovada e avisa. Não faz nada numa spec só refinada.
# drop_approval <arquivo> <SPEC_ID>
drop_approval() {
  [ "$(frontmatter_get "$1" status 2>/dev/null || true)" = "$approved_status" ] || return 0
  frontmatter_set "$1" status "$refined_status"
  frontmatter_put "$1" "$approved_at_field" null
  frontmatter_put "$1" "$approved_hash_field" null
  echo "Aprovação de $2 derrubada: o status voltou para '$refined_status'."
}

record_manual() {
  local spec_id="$1" refined stored current sections summary count errors

  refined="$(find_refined "$spec_id")"
  stored="$(frontmatter_get "$refined" "$hash_field" 2>/dev/null || true)"
  current="$(spec_content_hash "$refined")"

  if [ -z "$stored" ] || [ "$stored" = null ]; then
    frontmatter_put "$refined" "$hash_field" "$current"
    echo "Hash do conteúdo gravado em $spec_id. Nenhum ajuste registrado."
    return
  fi
  if [ "$stored" = "$current" ]; then
    echo "Nenhuma edição manual."
    return
  fi

  sections="$(changed_sections "$refined")"
  summary="$manual_summary${sections:+: $sections}"
  count="$(record_adjustment "$refined" "$manual_origin" "$summary")"
  echo "Ajuste manual $count registrado em $spec_id: $summary"
  drop_approval "$refined" "$spec_id"

  errors="$(refined_errors "$refined")"
  [ -z "$errors" ] || printf 'Problemas de validação:\n%s\n' "$errors"
}

validate() {
  local spec_id="$1" refined errors

  refined="$(find_refined "$spec_id")"
  errors="$(refined_errors "$refined")"
  [ -z "$errors" ] || die "a spec refinada $spec_id tem problemas:"$'\n'"$errors"
  echo "Spec refinada $spec_id válida: $refined"
}

record() {
  local spec_id="$1" summary refined errors count

  summary="$(printf '%s' "$2" | sed -E 's/^[[:space:]]+//; s/[[:space:]]+$//')"
  [ -n "$summary" ] || die "o resumo do ajuste está vazio"
  [[ "$summary" != *$'\n'* ]] || die "o resumo não pode ter quebra de linha; separe os itens com '; '"

  refined="$(find_refined "$spec_id")"
  errors="$(refined_errors "$refined")"
  [ -z "$errors" ] || die "a spec refinada $spec_id tem problemas; nenhum ajuste registrado:"$'\n'"$errors"

  count="$(record_adjustment "$refined" "$agent_origin" "$summary")"
  echo "Ajuste $count registrado em $spec_id: $refined"
  drop_approval "$refined" "$spec_id"
}

case "${1:-}" in
  --record-manual) [ -n "${2:-}" ] || die "$USAGE"; record_manual "$2" ;;
  --validate) [ -n "${2:-}" ] || die "$USAGE"; validate "$2" ;;
  --record) [ -n "${2:-}" ] && [ $# -ge 3 ] || die "$USAGE"; record "$2" "$3" ;;
  *) die "$USAGE" ;;
esac
