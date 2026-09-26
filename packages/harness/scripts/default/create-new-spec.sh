#!/usr/bin/env bash
# Cria uma nova spec a partir do template, com front-matter e ID sequencial.
# Tudo o que é configurável fica em `config/` (main.yml, specs.yml, sequences.yml).
#
# Uso: create-new-spec.sh "Título da spec"
#      Sem argumento, pergunta o título (só em terminal interativo).
set -euo pipefail

HARNESS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$HARNESS_DIR/scripts/lib/config.sh"
source "$HARNESS_DIR/scripts/lib/text.sh"

MAIN_CONFIG="$HARNESS_DIR/config/main.yml"
SPECS_CONFIG="$HARNESS_DIR/config/specs.yml"
SEQUENCES_CONFIG="$HARNESS_DIR/config/sequences.yml"

die() { echo "Erro: $*" >&2; exit 1; }

# Caminhos da configuração são relativos à raiz do harness.
harness_path() {
  case "$1" in
    /*) printf '%s' "$1" ;;
    *) printf '%s/%s' "$HARNESS_DIR" "${1#./}" ;;
  esac
}

# Título
title="${1:-}"
if [ -z "$title" ] && [ -t 0 ]; then
  read -rp "Título da spec: " title
fi
title="$(printf '%s' "$title" | sed -E 's/^[[:space:]]+//; s/[[:space:]]+$//')"
[ -n "$title" ] || die "informe o título: create-new-spec.sh \"Título da spec\""

slug="$(slugify "$title")"
[ -n "$slug" ] || die "o título precisa ter ao menos uma letra ou número"

# ID
prefix="$(config_get "$SPECS_CONFIG" id.prefix)"
digits="$(config_get "$SPECS_CONFIG" id.digits)"
sequence="$(config_get "$SEQUENCES_CONFIG" specs)"
[[ "$digits" =~ ^[0-9]+$ ]] || die "id.digits inválido em $SPECS_CONFIG: '$digits'"
[[ "$sequence" =~ ^[0-9]+$ ]] || die "specs inválido em $SEQUENCES_CONFIG: '$sequence'"
sequence=$((10#$sequence))
spec_id="$(printf '%s-%0*d' "$prefix" "$digits" "$sequence")"

# Destino
draft_dir="$(harness_path "$(config_get "$MAIN_CONFIG" paths.specs)")/$(config_get "$SPECS_CONFIG" draft_dir)"
templates_dir="$(harness_path "$(config_get "$MAIN_CONFIG" paths.templates)")"
template="$templates_dir/$(config_get "$SPECS_CONFIG" template)"
[ -f "$template" ] || die "template não encontrado: $template"

file_name="$(config_get "$SPECS_CONFIG" file_name | TPL_SPEC_ID="$spec_id" TPL_SPEC_SLUG="$slug" render)"
target="$draft_dir/$file_name"
[ ! -e "$target" ] || die "o arquivo já existe: $target"

# Front-matter: valores escapados, pois entram em YAML.
now_date="$(date -u +%Y-%m-%d)"
now_datetime="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
author="$(git -C "$HARNESS_DIR" config user.name 2>/dev/null || printf '%s' "${USER:-}")"

front_matter="$(
  config_block "$SPECS_CONFIG" front_matter |
    TPL_SPEC_ID="$spec_id" \
    TPL_SPEC_TITLE="$(yaml_escape "$title")" \
    TPL_AUTHOR="$(yaml_escape "$author")" \
    TPL_DATE="$now_date" \
    TPL_DATETIME="$now_datetime" \
    render
)"

# Arquivo
mkdir -p "$draft_dir"
TPL_FRONT_MATTER="$(printf -- '---\n%s\n---' "$front_matter")" \
TPL_SPEC_ID="$spec_id" \
TPL_SPEC_TITLE="$title" \
  render "$template" > "$target"

# A sequência só avança depois que a spec foi criada.
config_set "$SEQUENCES_CONFIG" specs "$((sequence + 1))"

echo "$spec_id criada em $target"
