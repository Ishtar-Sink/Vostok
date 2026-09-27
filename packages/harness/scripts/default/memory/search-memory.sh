#!/usr/bin/env bash
# Consulta a memória do agente sem carregar os arquivos inteiros.
#
# Uso: search-memory.sh <tipo|all>
#        Lista o índice: ID, título, status e tags de cada entrada.
#      search-memory.sh <tipo|all> <termo> [termo...]
#        Imprime as entradas completas que contêm qualquer um dos termos, das que contêm mais
#        termos para as que contêm menos, sem diferenciar maiúsculas nem acentos.
#        Limitado a `search_max_results` (config/memory.yml). Para erros, use um trecho estável da mensagem.
#      search-memory.sh --id <ID>
#        Imprime uma entrada.
#      Sem resultado, imprime "Nenhuma entrada encontrada." e termina sem erro.
set -euo pipefail

# Raiz do harness: primeira pasta acima do script que contém `config/main.yml`.
HARNESS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && until [ -f config/main.yml ] || [ "$PWD" = / ]; do cd ..; done && pwd)"
source "$HARNESS_DIR/scripts/lib/config.sh"
source "$HARNESS_DIR/scripts/lib/memory.sh"

USAGE="Uso: search-memory.sh <tipo|all> [termo...] | --id <ID>"

die() { echo "Erro: $*" >&2; exit 1; }

# Percorre as entradas dos arquivos. Modos:
#   index: uma linha por entrada, com ID, título, status e tags;
#   terms: entradas com ao menos um termo, ordenadas pela quantidade de termos encontrados;
#   id:    a entrada cujo cabeçalho é o ID.
# scan_entries <modo> <valor> <arquivo>...   (valor: termos separados por linha, ou o ID)
scan_entries() {
  local mode="$1" value="$2"
  shift 2

  SEARCH_VALUE="$value" awk \
    -v mode="$mode" \
    -v pattern="$MEMORY_ENTRY_PATTERN" \
    -v max_results="$(config_get "$MEMORY_CONFIG" search_max_results)" '
    function field(name,   i, prefix) {
      prefix = "- **" name ":** "
      for (i = 1; i <= n; i++) if (index(lines[i], prefix) == 1) return substr(lines[i], length(prefix) + 1)
      return ""
    }
    # Minúsculas e sem acentos, para a busca não depender do locale nem da acentuação.
    function normalize(s) {
      gsub(/Á|À|Â|Ã|Ä/, "a", s); gsub(/É|È|Ê|Ë/, "e", s); gsub(/Í|Ì|Î|Ï/, "i", s)
      gsub(/Ó|Ò|Ô|Õ|Ö/, "o", s); gsub(/Ú|Ù|Û|Ü/, "u", s); gsub(/Ç/, "c", s); gsub(/Ñ/, "n", s)
      s = tolower(s)
      gsub(/á|à|â|ã|ä/, "a", s); gsub(/é|è|ê|ë/, "e", s); gsub(/í|ì|î|ï/, "i", s)
      gsub(/ó|ò|ô|õ|ö/, "o", s); gsub(/ú|ù|û|ü/, "u", s); gsub(/ç/, "c", s); gsub(/ñ/, "n", s)
      return s
    }
    function block(   i, text) {
      text = lines[1]
      for (i = 2; i <= n; i++) text = text "\n" lines[i]
      return text
    }
    function flush(   i, text, title, score) {
      if (!n) return
      if (mode == "index") {
        title = lines[1]; sub(/^## /, "", title)
        printf "%s | %s | %s\n", title, field("Status"), field("Tags")
      } else if (mode == "id") {
        if (index(lines[1], "## " ENVIRON["SEARCH_VALUE"] ":") == 1) print block()
      } else {
        text = normalize(block())
        score = 0
        for (i = 1; i <= terms; i++) if (index(text, term[i])) score++
        if (score) { found++; scores[found] = score; blocks[found] = block() }
      }
      n = 0
    }
    BEGIN {
      if (mode == "terms") {
        count = split(normalize(ENVIRON["SEARCH_VALUE"]), raw, "\n")
        for (i = 1; i <= count; i++) if (raw[i] != "") term[++terms] = raw[i]
      }
    }
    FNR == 1 { flush(); in_fm = 0 }
    { sub(/\r$/, "") }
    FNR == 1 && $0 == "---" { in_fm = 1; next }
    in_fm { if ($0 == "---") in_fm = 0; next }
    $0 ~ pattern { flush() }
    n || $0 ~ pattern { lines[++n] = $0 }
    END {
      flush()
      if (mode != "terms" || !found) exit

      # Ordenação estável, da maior para a menor quantidade de termos encontrados.
      for (i = 1; i <= found; i++) order[i] = i
      for (i = 2; i <= found; i++) {
        current = order[i]
        for (j = i - 1; j >= 1 && scores[order[j]] < scores[current]; j--) order[j + 1] = order[j]
        order[j + 1] = current
      }

      shown = (found < max_results) ? found : max_results
      for (i = 1; i <= shown; i++) {
        if (i > 1) print ""
        printf "> Termos encontrados: %d de %d\n\n%s\n", scores[order[i]], terms, blocks[order[i]]
      }
      if (found > shown) printf "\nEntradas omitidas: %d. Use termos mais específicos para refinar a busca.\n", found - shown
    }
  ' "$@"
}

# Imprime os arquivos de memória existentes dos tipos pedidos, um por linha.
memory_files_for() {
  local types type file
  if [ "$1" = all ]; then
    types="$(config_list "$MEMORY_CONFIG" types)"
  else
    memory_require_type "$1" || return 1
    types="$1"
  fi
  while IFS= read -r type; do
    file="$(memory_file "$type")"
    [ ! -f "$file" ] || printf '%s\n' "$file"
  done <<< "$types"
}

print_or_empty() {
  if [ -n "$1" ]; then printf '%s\n' "$1"; else echo "Nenhuma entrada encontrada."; fi
}

[ $# -gt 0 ] || die "$USAGE"

case "$1" in
  --id)
    [ -n "${2:-}" ] || die "$USAGE"
    type="$(memory_type_for_id "$2")" || exit 1
    files="$(memory_files_for "$type")"
    [ -z "$files" ] || results="$(scan_entries id "$2" "$files")"
    ;;
  -*) die "$USAGE" ;;
  *)
    files="$(memory_files_for "$1")" || exit 1
    shift
    file_list=()
    while IFS= read -r file; do [ -z "$file" ] || file_list+=("$file"); done <<< "$files"
    if [ "${#file_list[@]}" -gt 0 ]; then
      if [ $# -eq 0 ]; then
        results="$(scan_entries index "" "${file_list[@]}")"
      else
        results="$(scan_entries terms "$(printf '%s\n' "$@")" "${file_list[@]}")"
      fi
    fi
    ;;
esac

print_or_empty "${results:-}"
