# Leitura e escrita do front-matter de arquivos Markdown: o bloco entre as duas
# primeiras linhas `---`. Suporta chaves de primeiro nível com valor escalar.
# Uso: source "$HARNESS_DIR/scripts/lib/frontmatter.sh"

# Imprime o front-matter, sem os delimitadores. Falha se o arquivo não tiver um.
# frontmatter_print <arquivo>
frontmatter_print() {
  awk '
    NR == 1 { if ($0 != "---") exit; next }
    $0 == "---" { found = 1; exit }
    { print }
    END { exit !found }
  ' "$1"
}

# Imprime o valor de uma chave, sem aspas e com os escapes de aspas duplas resolvidos.
# frontmatter_get <arquivo> <chave>
frontmatter_get() {
  frontmatter_print "$1" | awk -v want="$2" '
    index($0, want ":") != 1 { next }
    {
      val = substr($0, length(want) + 2)
      gsub(/^[ \t]+|[ \t]+$/, "", val)
      if (val ~ /^".*"$/) {
        val = substr(val, 2, length(val) - 2)
        out = ""
        for (i = 1; i <= length(val); i++) {
          c = substr(val, i, 1)
          if (c == "\\" && i < length(val)) { i++; c = substr(val, i, 1) }
          out = out c
        }
        val = out
      } else if (val ~ /^\047.*\047$/) {
        val = substr(val, 2, length(val) - 2)
      }
      print val; found = 1; exit
    }
    END { exit !found }
  '
}

# Imprime, sem alterar, as linhas das chaves de primeiro nível pedidas
# (inclusive valores de várias linhas), na ordem em que aparecem no front-matter.
# frontmatter_pick <arquivo> <chave>...
frontmatter_pick() {
  local file="$1"
  shift
  frontmatter_print "$file" | awk -v keys="$*" '
    BEGIN { n = split(keys, list, " "); for (i = 1; i <= n; i++) wanted[list[i]] = 1 }
    match($0, /^[A-Za-z0-9_-]+:/) { picking = (substr($0, 1, RLENGTH - 1) in wanted) }
    picking { print }
  '
}

# Troca o valor de uma chave existente no front-matter, sem tocar no resto do arquivo.
# frontmatter_set <arquivo> <chave> <valor>
frontmatter_set() {
  local tmp
  tmp="$(mktemp)"
  if awk -v want="$2" -v new="$3" '
    NR == 1 && $0 == "---" { inside = 1; print; next }
    inside && $0 == "---" { inside = 0 }
    inside && index($0, want ":") == 1 { print want ": " new; found = 1; next }
    { print }
    END { exit !found }
  ' "$1" > "$tmp"; then
    cat "$tmp" > "$1"
    rm -f "$tmp"
  else
    rm -f "$tmp"
    echo "Chave '$2' não encontrada no front-matter de $1" >&2
    return 1
  fi
}

# Junta dois front-matters. Cada chave de <sobrepor> substitui a de mesmo nome em <base>,
# na mesma posição e inclusive quando o valor antigo ocupa várias linhas.
# Chaves novas entram no fim. Em <sobrepor>, cada chave ocupa uma linha.
# frontmatter_merge <base> <sobrepor>   (conteúdos, não arquivos)
frontmatter_merge() {
  FM_BASE="$1" FM_OVERRIDES="$2" awk '
    BEGIN {
      n = split(ENVIRON["FM_OVERRIDES"], over, "\n")
      for (i = 1; i <= n; i++)
        if (match(over[i], /^[A-Za-z0-9_-]+:/)) {
          key = substr(over[i], 1, RLENGTH - 1)
          order[++keys] = key
          line_of[key] = over[i]
        }

      m = split(ENVIRON["FM_BASE"], base, "\n")
      for (i = 1; i <= m; i++) {
        line = base[i]
        if (match(line, /^[A-Za-z0-9_-]+:/)) {
          key = substr(line, 1, RLENGTH - 1)
          skipping = (key in line_of)
          if (skipping) { print line_of[key]; used[key] = 1 }
        }
        if (!skipping && line != "") print line
      }

      for (i = 1; i <= keys; i++) if (!(order[i] in used)) print line_of[order[i]]
    }
  '
}

# Imprime o caminho do primeiro arquivo `.md` de uma pasta cujo front-matter tem o `id` informado.
# spec_find_by_id <pasta> <id>
spec_find_by_id() {
  local file
  for file in "$1"/*.md; do
    [ -f "$file" ] || continue
    if [ "$(frontmatter_get "$file" id 2>/dev/null)" = "$2" ]; then
      printf '%s' "$file"
      return 0
    fi
  done
  return 1
}
