#!/usr/bin/env bash
# Lista as specs em uma tabela com ID, título, status e arquivo (relativo a `paths.specs`).
#
# Uso: list-specs.sh [status]
#      Com status, lista só as specs com aquele status (ex.: draft).
#      Sem nenhuma spec, imprime "Nenhuma spec encontrada." e termina sem erro.
set -euo pipefail

# Raiz do harness: primeira pasta acima do script que contém `config/main.yml`.
HARNESS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && until [ -f config/main.yml ] || [ "$PWD" = / ]; do cd ..; done && pwd)"
source "$HARNESS_DIR/scripts/lib/config.sh"
source "$HARNESS_DIR/scripts/lib/frontmatter.sh"

MAIN_CONFIG="$HARNESS_DIR/config/main.yml"

status_filter="${1:-}"
specs_dir="$(harness_path "$(config_get "$MAIN_CONFIG" paths.specs)")"

# Escapa `|` para não quebrar a tabela.
cell() { printf '%s' "${1//|/\\|}"; }

rows=""
while IFS= read -r file; do
  id="$(frontmatter_get "$file" id 2>/dev/null)" || continue
  status="$(frontmatter_get "$file" status 2>/dev/null || true)"
  [ -z "$status_filter" ] || [ "$status" = "$status_filter" ] || continue
  title="$(frontmatter_get "$file" title 2>/dev/null || true)"
  rows+="| $(cell "$id") | $(cell "$title") | $(cell "$status") | ${file#"$specs_dir"/} |"$'\n'
done < <([ -d "$specs_dir" ] && find "$specs_dir" -type f -name '*.md' | LC_ALL=C sort)

if [ -z "$rows" ]; then
  echo "Nenhuma spec encontrada."
  exit 0
fi

echo "| ID | Título | Status | Arquivo |"
echo "|---|---|---|---|"
printf '%s' "$rows"
