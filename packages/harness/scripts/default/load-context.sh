#!/usr/bin/env bash
# Carrega os arquivos do harness que sempre são obrigatórios, como determinadas regras, contexto, etc.
set -euo pipefail

# Raiz do harness, resolvida a partir do próprio script para funcionar de qualquer diretório.
HARNESS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

# Pastas (carregadas por inteiro) e arquivos específicos, relativos à raiz do harness.
# São carregados na ordem em que aparecem aqui.
SOURCES=(
  context
  rules/agent
)

print_file() {
  printf '\n--- arquivo: %s ---\n\n' "${1#"$HARNESS_DIR"/}"
  cat "$1"
  echo
}

echo "# Contexto obrigatório do harness"
echo
echo "Raiz do harness: \`$HARNESS_DIR\`"
echo "Caminhos relativos citados em cada arquivo partem da pasta daquele arquivo."

for source in "${SOURCES[@]}"; do
  path="$HARNESS_DIR/$source"

  if [ -f "$path" ]; then
    print_file "$path"
  elif [ -d "$path" ]; then
    while IFS= read -r file; do
      print_file "$file"
    done < <(find "$path" -type f ! -name '.*' | LC_ALL=C sort)
  else
    echo "Aviso: '$source' não encontrado em $HARNESS_DIR" >&2
  fi
done
