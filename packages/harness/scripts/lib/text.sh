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

# Lista, com arquivo e linha, as variáveis `#{NOME}` que ainda não foram preenchidas.
# Variáveis entre crases são menções em documentação e não contam.
# Termina com sucesso só se encontrar alguma.
# template_leftovers <arquivo|pasta>...
template_leftovers() {
  grep -rnE '(^|[^`])#[{][A-Z0-9_]+[}]' "$@"
}

# Remove espaços nas pontas e falha, com mensagem, se o título não puder virar
# cabeçalho: vazio, com quebra de linha ou com `#{`, que seria lido como variável de template.
# clean_title <título>
clean_title() {
  local title
  title="$(printf '%s' "$1" | sed -E 's/^[[:space:]]+//; s/[[:space:]]+$//')"
  if [ -z "$title" ]; then
    echo "O título está vazio" >&2; return 1
  elif [[ "$title" == *$'\n'* ]]; then
    echo "O título não pode ter quebra de linha" >&2; return 1
  elif [[ "$title" == *'#{'* ]]; then
    echo "O título não pode conter '#{', reservado para variáveis de template" >&2; return 1
  fi
  printf '%s' "$title"
}

# Escapa um texto para uso dentro de aspas duplas em YAML.
yaml_escape() {
  local s="${1//\\/\\\\}"
  printf '%s' "${s//\"/\\\"}"
}

# Imprime o SHA-256 da entrada padrão, com `sha256sum` (Linux) ou `shasum -a 256` (macOS).
# sha256_hash < conteúdo
sha256_hash() {
  if command -v sha256sum > /dev/null 2>&1; then
    sha256sum | cut -d ' ' -f 1
  else
    shasum -a 256 | cut -d ' ' -f 1
  fi
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
