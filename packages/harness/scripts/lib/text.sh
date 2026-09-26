# Utilitários de texto e templates.
# Uso: source "$HARNESS_DIR/scripts/lib/text.sh"

# Substitui cada `#{NOME}` pelo valor da variável de ambiente `TPL_NOME`, literalmente.
# Variáveis sem valor definido ficam como estão, para serem preenchidas depois.
# render [arquivo]   (sem arquivo, lê da entrada padrão)
render() {
  awk '
    {
      out = ""; rest = $0
      while (match(rest, /#[{][A-Z0-9_]+[}]/)) {
        name = "TPL_" substr(rest, RSTART + 2, RLENGTH - 3)
        out = out substr(rest, 1, RSTART - 1) \
          ((name in ENVIRON) ? ENVIRON[name] : substr(rest, RSTART, RLENGTH))
        rest = substr(rest, RSTART + RLENGTH)
      }
      print out rest
    }
  ' "$@"
}

# Escapa um texto para uso dentro de aspas duplas em YAML.
yaml_escape() {
  local s="${1//\\/\\\\}"
  printf '%s' "${s//\"/\\\"}"
}

# Converte um texto em slug: minúsculas, sem acentos, palavras separadas por hífen.
slugify() {
  printf '%s' "$1" | LC_ALL=C sed -E '
    s/á|à|â|ã|ä|Á|À|Â|Ã|Ä/a/g
    s/é|è|ê|ë|É|È|Ê|Ë/e/g
    s/í|ì|î|ï|Í|Ì|Î|Ï/i/g
    s/ó|ò|ô|õ|ö|Ó|Ò|Ô|Õ|Ö/o/g
    s/ú|ù|û|ü|Ú|Ù|Û|Ü/u/g
    s/ç|Ç/c/g
    s/ñ|Ñ/n/g
  ' | LC_ALL=C tr '[:upper:]' '[:lower:]' | LC_ALL=C sed -E 's/[^a-z0-9]+/-/g; s/^-+//; s/-+$//'
}
