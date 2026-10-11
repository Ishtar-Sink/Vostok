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

# Imprime os itens da lista `skills.<skill>` do mapa de contexto, um por linha, sem comentários nem aspas.
# Sem saída (e sucesso) se a chave `skills` ou a entrada da skill não existirem.
# context_skill_entries <arquivo> <skill>
context_skill_entries() {
  awk -v want="$2" '
    /^[ \t]*(#|$)/ { next }
    /^[^ \t]/ { inskills = ($0 ~ /^skills:[ \t]*(#.*)?$/); inentry = 0; next }
    !inskills { next }
    /^[ \t]*-[ \t]/ {
      if (!inentry) next
      item = $0
      sub(/^[ \t]*-[ \t]+/, "", item)
      sub(/[ \t]+#.*$/, "", item)
      gsub(/[ \t]+$/, "", item)
      if (item ~ /^".*"$/ || item ~ /^\047.*\047$/) item = substr(item, 2, length(item) - 2)
      print item
      next
    }
    {
      key = $0
      sub(/^[ \t]+/, "", key)
      sub(/[ \t]*:.*$/, "", key)
      inentry = (key == want)
    }
  ' "$1"
}
