#!/usr/bin/env bash
# Carrega sob demanda um ou mais arquivos ou pastas, além do contexto obrigatório.
#
# Uso: load-extra-context.sh <caminho> [caminho...]
#      Caminhos relativos partem da raiz do harness; se não existirem lá, do diretório atual.
#      Termina com erro se algum caminho não for encontrado, depois de carregar os demais.
set -euo pipefail

HARNESS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$HARNESS_DIR/scripts/lib/context.sh"

# Resolve um caminho: absoluto, relativo à raiz do harness ou relativo ao diretório atual.
resolve_path() {
  local path="${1#./}"

  case "$path" in
    /*) printf '%s' "$path" ;;
    *) if [ -e "$HARNESS_DIR/$path" ]; then
         printf '%s' "$HARNESS_DIR/$path"
       else
         printf '%s' "$PWD/$path"
       fi ;;
  esac
}

if [ $# -eq 0 ]; then
  echo "Uso: load-extra-context.sh <caminho> [caminho...]" >&2
  exit 1
fi

status=0
print_context_header "Contexto adicional"

for source in "$@"; do
  if ! print_context_path "$(resolve_path "$source")"; then
    echo "Aviso: '$source' não encontrado" >&2
    status=1
  fi
done

exit "$status"
