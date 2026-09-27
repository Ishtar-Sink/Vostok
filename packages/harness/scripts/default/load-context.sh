#!/usr/bin/env bash
# Carrega os arquivos do harness que sempre são obrigatórios, como determinadas regras, contexto, etc.
# A lista de fontes fica em `required`, no `config/context.yml`.
set -euo pipefail

# Raiz do harness, resolvida a partir do próprio script para funcionar de qualquer diretório.
HARNESS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$HARNESS_DIR/scripts/lib/config.sh"
source "$HARNESS_DIR/scripts/lib/context.sh"

CONTEXT_CONFIG="$HARNESS_DIR/config/context.yml"

sources="$(config_list "$CONTEXT_CONFIG" required)"

print_context_header "Contexto obrigatório do harness"

while IFS= read -r source; do
  [ -n "$source" ] || continue
  print_context_path "$HARNESS_DIR/$source" ||
    echo "Aviso: '$source' não encontrado em $HARNESS_DIR" >&2
done <<< "$sources"
