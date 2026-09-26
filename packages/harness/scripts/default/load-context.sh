#!/usr/bin/env bash
# Carrega os arquivos do harness que sempre são obrigatórios, como determinadas regras, contexto, etc.
set -euo pipefail

# Raiz do harness, resolvida a partir do próprio script para funcionar de qualquer diretório.
HARNESS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$HARNESS_DIR/scripts/lib/context.sh"

# Pastas (carregadas por inteiro) e arquivos específicos, relativos à raiz do harness.
# São carregados na ordem em que aparecem aqui.
SOURCES=(
  context
  rules/agent
)

print_context_header "Contexto obrigatório do harness"

for source in "${SOURCES[@]}"; do
  print_context_path "$HARNESS_DIR/$source" ||
    echo "Aviso: '$source' não encontrado em $HARNESS_DIR" >&2
done
