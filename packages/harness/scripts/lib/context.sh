# Impressão de arquivos de contexto para o agente.
# Uso: source "$HARNESS_DIR/scripts/lib/context.sh"   (requer HARNESS_DIR definido)

# Cabeçalho da saída, com a raiz usada para resolver os caminhos citados nos arquivos.
# print_context_header <título>
print_context_header() {
  echo "# $1"
  echo
  echo "Raiz do harness: \`$HARNESS_DIR\`"
  echo "Caminhos relativos citados em cada arquivo partem da pasta daquele arquivo."
}

# Imprime um arquivo precedido de um cabeçalho que identifica sua origem.
# print_context_file <arquivo>
print_context_file() {
  printf '\n--- arquivo: %s ---\n\n' "${1#"$HARNESS_DIR"/}"
  cat "$1"
  echo
}

# Imprime um arquivo ou, se for uma pasta, todos os arquivos dela em ordem alfabética.
# Arquivos ocultos (ex.: .gitkeep) são ignorados. Falha se o caminho não existir.
# print_context_path <arquivo|pasta>
print_context_path() {
  if [ -f "$1" ]; then
    print_context_file "$1"
  elif [ -d "$1" ]; then
    while IFS= read -r file; do
      print_context_file "$file"
    done < <(find "$1" -type f ! -name '.*' | LC_ALL=C sort)
  else
    return 1
  fi
}
