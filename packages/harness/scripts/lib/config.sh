# Leitura e escrita dos arquivos em `config/`.
# Suporta o subconjunto de YAML usado ali: `chave: valor` em até dois níveis
# (acessados como `pai.filho`) e blocos literais de primeiro nível (`chave: |`).
# Uso: source "$HARNESS_DIR/scripts/lib/config.sh"

# Caminho percorrido pelo awk para cada linha: define `path`, `line` e `indent`.
_CONFIG_AWK_PATH='
  function parse(   key) {
    match($0, /^ */); indent = RLENGTH
    line = substr($0, indent + 1)
    key = line; sub(/[ \t]*:.*/, "", key)
    if (indent == 0) top = key
    path = (indent == 0) ? key : top "." key
  }
'

# Imprime o valor escalar de uma chave. Falha se a chave não existir.
# config_get <arquivo> <chave.subchave>
config_get() {
  awk -v want="$2" "$_CONFIG_AWK_PATH"'
    /^[ \t]*(#|$)/ { next }
    {
      parse()
      if (path != want || index(line, ":") == 0) next
      val = substr(line, index(line, ":") + 1)
      sub(/[ \t]+#.*$/, "", val)
      gsub(/^[ \t]+|[ \t]+$/, "", val)
      if (val ~ /^".*"$/ || val ~ /^\047.*\047$/) val = substr(val, 2, length(val) - 2)
      print val; found = 1; exit
    }
    END { exit !found }
  ' "$1" || { echo "Configuração '$2' não encontrada em $1" >&2; return 1; }
}

# Imprime o conteúdo de um bloco literal de primeiro nível, sem a indentação.
# config_block <arquivo> <chave>
config_block() {
  awk -v want="$2" '
    !inblock && $0 ~ ("^" want ":[ \t]*\\|") { inblock = 1; found = 1; next }
    inblock && /^[^ \t]/ { exit }
    inblock {
      if (!ind && $0 ~ /[^ \t]/) { match($0, /^ */); ind = RLENGTH }
      print substr($0, ind + 1)
    }
    END { exit !found }
  ' "$1" || { echo "Bloco '$2' não encontrado em $1" >&2; return 1; }
}

# Troca o valor escalar de uma chave existente, preservando o resto do arquivo.
# config_set <arquivo> <chave.subchave> <valor>
config_set() {
  local tmp
  tmp="$(mktemp)"
  if awk -v want="$2" -v new="$3" "$_CONFIG_AWK_PATH"'
    /^[ \t]*(#|$)/ { print; next }
    {
      parse()
      if (path == want && index(line, ":")) {
        printf "%*s%s: %s\n", indent, "", substr(line, 1, index(line, ":") - 1), new
        found = 1; next
      }
      print
    }
    END { exit !found }
  ' "$1" > "$tmp"; then
    cat "$tmp" > "$1"
    rm -f "$tmp"
  else
    rm -f "$tmp"
    echo "Configuração '$2' não encontrada em $1" >&2
    return 1
  fi
}
