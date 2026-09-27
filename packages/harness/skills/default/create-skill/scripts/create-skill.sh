#!/usr/bin/env bash
# Cria skills do harness no formato padrão: SKILL.md, scripts, redirecionamentos e catálogo.
# Tudo o que é configurável fica em `config/skills.yml`.
#
# Uso: create-skill.sh [--default] <nome>
#        Cria o SKILL.md a partir do template em `dirs.custom` (ou `dirs.default`, com --default).
#      create-skill.sh --script <nome> <nome-do-script>
#        Cria `scripts/<nome-do-script>.sh` na pasta da skill a partir do template de script.
#      create-skill.sh --finalize <nome>
#        Valida a skill, gera os redirecionamentos para as ferramentas e atualiza o catálogo.
set -euo pipefail

# Raiz do harness: primeira pasta acima do script que contém `config/main.yml`.
HARNESS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && until [ -f config/main.yml ] || [ "$PWD" = / ]; do cd ..; done && pwd)"
source "$HARNESS_DIR/scripts/lib/config.sh"
source "$HARNESS_DIR/scripts/lib/text.sh"
source "$HARNESS_DIR/scripts/lib/frontmatter.sh"

MAIN_CONFIG="$HARNESS_DIR/config/main.yml"
SKILLS_CONFIG="$HARNESS_DIR/config/skills.yml"

USAGE="Uso: create-skill.sh [--default] <nome> | --script <nome> <nome-do-script> | --finalize <nome>"

# Formato de nome exigido pela especificação de Agent Skills (agentskills.io):
# minúsculas, números e hífens simples, sem hífen no início ou no fim.
NAME_PATTERN='^[a-z0-9]+(-[a-z0-9]+)*$'
MAX_NAME_LENGTH=64
XML_TAG_PATTERN='<[A-Za-z/]'
MAX_DESCRIPTION_LENGTH=1024
# Recomendação da Anthropic para o SKILL.md continuar eficiente no contexto do agente.
MAX_SKILL_LINES=500
# Chaves de `metadata` com caminhos (relativos à raiz do harness) que precisam existir.
METADATA_PATH_KEYS=(requires scripts templates)

die() { echo "Erro: $*" >&2; exit 1; }

templates_dir="$(harness_path "$(config_get "$MAIN_CONFIG" paths.templates)")"
root_dir="$(cd "$(harness_path "$(config_get "$MAIN_CONFIG" paths.root)")" && pwd)"

skill_dir_in() {
  printf '%s/%s' "$(harness_path "$(config_get "$SKILLS_CONFIG" "dirs.$1")")" "$2"
}

# Imprime o escopo (default ou custom) em que a skill existe.
find_skill_scope() {
  local scope
  for scope in default custom; do
    if [ -f "$(skill_dir_in "$scope" "$1")/SKILL.md" ]; then
      printf '%s' "$scope"
      return 0
    fi
  done
  return 1
}

validate_name() {
  [[ "$1" =~ $NAME_PATTERN ]] ||
    die "nome inválido '$1': use minúsculas, números e hífens simples (ex.: review-code)"
  [ "${#1}" -le "$MAX_NAME_LENGTH" ] || die "o nome '$1' passa de $MAX_NAME_LENGTH caracteres"
  case "$1" in
    *anthropic*|*claude*) die "o nome não pode conter as palavras reservadas 'anthropic' ou 'claude'" ;;
  esac
}

create_skill() {
  local scope="$1" name="$2" dir existing

  validate_name "$name"
  if existing="$(find_skill_scope "$name")"; then
    die "a skill '$name' já existe em $(skill_dir_in "$existing" "$name")"
  fi

  dir="$(skill_dir_in "$scope" "$name")"
  mkdir -p "$dir"
  TPL_SKILL_NAME="$name" \
  TPL_AUTHOR="$(yaml_escape "$(git -C "$HARNESS_DIR" config user.name 2>/dev/null || printf '%s' "${USER:-}")")" \
    render "$templates_dir/$(config_get "$SKILLS_CONFIG" template)" > "$dir/SKILL.md"

  echo "Skill '$name' criada em $dir/SKILL.md"
}

create_script() {
  local name="$1" script="$2" scope target

  scope="$(find_skill_scope "$name")" || die "skill '$name' não encontrada"
  [[ "$script" =~ $NAME_PATTERN ]] || die "nome de script inválido '$script': use kebab-case (ex.: list-items)"

  target="$(skill_dir_in "$scope" "$name")/scripts/$script.sh"
  [ ! -e "$target" ] || die "o script já existe: $target"

  mkdir -p "$(dirname "$target")"
  TPL_SCRIPT_NAME="$script" render "$templates_dir/$(config_get "$SKILLS_CONFIG" script_template)" > "$target"
  chmod +x "$target"

  echo "Script criado em $target"
}

# Imprime um erro por linha para cada problema da skill. Não imprime nada se ela estiver válida.
skill_errors() {
  local name="$1" dir="$2" file="$2/SKILL.md" description key paths path lines

  template_leftovers "$dir" | sed "s#^$dir/#variável sem preencher: #" || true

  [ "$(frontmatter_get "$file" name 2>/dev/null || true)" = "$name" ] ||
    echo "o campo 'name' do front-matter precisa ser '$name', igual ao nome da pasta"

  description="$(frontmatter_get "$file" description 2>/dev/null || true)"
  [ -n "$description" ] || echo "o campo 'description' está vazio ou ausente"
  [ "${#description}" -le "$MAX_DESCRIPTION_LENGTH" ] ||
    echo "o campo 'description' tem ${#description} caracteres; o máximo é $MAX_DESCRIPTION_LENGTH"
  if [[ "$description" =~ $XML_TAG_PATTERN ]]; then
    echo "o campo 'description' não pode conter tags XML"
  fi

  lines="$(wc -l < "$file" | tr -d ' ')"
  [ "$lines" -le "$MAX_SKILL_LINES" ] ||
    echo "o SKILL.md tem $lines linhas; o máximo é $MAX_SKILL_LINES (mova detalhes para arquivos de referência)"

  for key in "${METADATA_PATH_KEYS[@]}"; do
    paths="$(frontmatter_print "$file" | config_get /dev/stdin "metadata.$key" 2>/dev/null || true)"
    for path in $paths; do
      [ -e "$HARNESS_DIR/$path" ] || echo "metadata.$key: '$path' não existe na raiz do harness"
    done
  done
}

write_redirects() {
  local name="$1" file="$2" keys redirect_dirs front_matter skill_path redirect_dir target

  keys="$(config_list "$SKILLS_CONFIG" redirect_keys)"
  redirect_dirs="$(config_list "$SKILLS_CONFIG" redirect_dirs)"
  front_matter="$(frontmatter_pick "$file" $keys)"
  skill_path="${file#"$root_dir"/}"

  while IFS= read -r redirect_dir; do
    [ -n "$redirect_dir" ] || continue
    target="$root_dir/$redirect_dir/$name/SKILL.md"
    mkdir -p "$(dirname "$target")"
    TPL_FRONT_MATTER="$(printf -- '---\n%s\n---' "$front_matter")" \
    TPL_SKILL_PATH="$skill_path" \
      render "$templates_dir/$(config_get "$SKILLS_CONFIG" redirect_template)" > "$target"
    echo "Redirecionamento: $target"
  done <<< "$redirect_dirs"
}

# Insere ou atualiza a linha da skill na tabela da seção do seu escopo no catálogo.
update_catalog() {
  local name="$1" scope="$2" file="$3" catalog section catalog_dir depth prefix row tmp i

  catalog="$(harness_path "$(config_get "$SKILLS_CONFIG" catalog)")"
  section="$(config_get "$SKILLS_CONFIG" "catalog_sections.$scope")"

  # Caminho da skill relativo à pasta do catálogo.
  catalog_dir="$(dirname "${catalog#"$HARNESS_DIR"/}")"
  prefix=""
  if [ "$catalog_dir" != "." ]; then
    depth="$(printf '%s' "$catalog_dir" | tr -cd '/' | wc -c)"
    for ((i = 0; i <= depth; i++)); do prefix+="../"; done
  fi

  row="| $name | $(frontmatter_get "$file" description | sed 's/|/\\|/g') | \`$prefix$(config_get "$SKILLS_CONFIG" "dirs.$scope")/$name\` |"

  tmp="$(mktemp)"
  if CATALOG_ROW="$row" awk -v section="## $section" -v name="$name" '
    { line[NR] = $0 }
    END {
      for (i = 1; i <= NR; i++) {
        if (line[i] ~ /^## /) inside = (line[i] == section)
        if (!inside) continue
        seen = 1
        if (line[i] ~ /^\|/) {
          last = i
          if (index(line[i], "| " name " |") == 1) found = i
        }
      }
      if (!seen || !last) exit 1
      for (i = 1; i <= NR; i++) {
        if (i == found) { print ENVIRON["CATALOG_ROW"]; continue }
        print line[i]
        if (!found && i == last) print ENVIRON["CATALOG_ROW"]
      }
    }
  ' "$catalog" > "$tmp"; then
    cat "$tmp" > "$catalog"
    rm -f "$tmp"
  else
    rm -f "$tmp"
    die "seção '## $section' com tabela não encontrada em $catalog"
  fi

  echo "Catálogo atualizado: $catalog"
}

finalize_skill() {
  local name="$1" scope dir errors

  scope="$(find_skill_scope "$name")" || die "skill '$name' não encontrada"
  dir="$(skill_dir_in "$scope" "$name")"

  errors="$(skill_errors "$name" "$dir")"
  [ -z "$errors" ] || die "a skill '$name' tem problemas:"$'\n'"$errors"

  write_redirects "$name" "$dir/SKILL.md"
  update_catalog "$name" "$scope" "$dir/SKILL.md"
  echo "Skill '$name' finalizada."
}

case "${1:-}" in
  --default) [ -n "${2:-}" ] || die "$USAGE"; create_skill default "$2" ;;
  --script) [ -n "${2:-}" ] && [ -n "${3:-}" ] || die "$USAGE"; create_script "$2" "$3" ;;
  --finalize) [ -n "${2:-}" ] || die "$USAGE"; finalize_skill "$2" ;;
  -*|"") die "$USAGE" ;;
  *) create_skill custom "$1" ;;
esac
