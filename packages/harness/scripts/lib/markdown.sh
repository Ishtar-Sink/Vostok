# Leitura de seções e tabelas de arquivos Markdown.
# Uso: source "$HARNESS_DIR/scripts/lib/markdown.sh"

# Imprime o conteúdo de uma seção `## <título>` até a próxima seção `## `, sem o título.
# Títulos dentro de blocos de código (```) não contam. Falha se a seção não existir.
# markdown_section <arquivo> <título>
markdown_section() {
  awk -v heading="## $2" '
    { sub(/\r$/, "") }
    /^[ \t]*```/ { in_code = !in_code; if (inside) print; next }
    !in_code && $0 == heading { inside = 1; found = 1; next }
    !in_code && inside && /^## / { exit }
    inside { print }
    END { exit !found }
  ' "$1"
}

# Imprime as linhas de dados das tabelas recebidas na entrada, sem cabeçalho e separador.
# markdown_table_rows < conteúdo
markdown_table_rows() {
  awk '
    /^\|/ {
      # Separador: toda célula tem ao menos três traços (`---`, `:--:`). `| - |` é uma linha de dados.
      if (/^\|([ \t]*:?---+:?[ \t]*\|)+[ \t]*$/) { header_done = 1; next }
      if (!header_done) next
      print
      next
    }
    { header_done = 0 }
  '
}

# Imprime o valor de uma coluna (a partir de 1) de uma linha de tabela, sem espaços nas pontas.
# markdown_cell <linha> <coluna>
markdown_cell() {
  printf '%s\n' "$1" | awk -F'|' -v col="$(($2 + 1))" '{ v = $col; gsub(/^[ \t]+|[ \t]+$/, "", v); print v }'
}
