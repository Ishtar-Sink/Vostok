#!/usr/bin/env bash
# Cria a spec refinada a partir de um rascunho e, depois de preenchida pelo agente, a finaliza.
# O front-matter da refinada parte do rascunho, com a referência a ele, o número de ajustes
# e a data do último ajuste (ver `refined_front_matter` em config/specs.yml).
#
# Uso: create-refined-spec.sh <SPEC_ID>
#        Cria a refinada em `refined_dir` a partir do template. Só aceita rascunhos com status.draft.
#      create-refined-spec.sh --finalize <SPEC_ID>
#        Confere que não sobrou nenhuma variável `#{...}` na refinada e muda o status
#        do rascunho para status.refined.
set -euo pipefail

# Raiz do harness: primeira pasta acima do script que contém `config/main.yml`.
HARNESS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && until [ -f config/main.yml ] || [ "$PWD" = / ]; do cd ..; done && pwd)"
source "$HARNESS_DIR/scripts/lib/config.sh"
source "$HARNESS_DIR/scripts/lib/text.sh"
source "$HARNESS_DIR/scripts/lib/frontmatter.sh"

MAIN_CONFIG="$HARNESS_DIR/config/main.yml"
SPECS_CONFIG="$HARNESS_DIR/config/specs.yml"

USAGE="Uso: create-refined-spec.sh [--finalize] <SPEC_ID>"

die() { echo "Erro: $*" >&2; exit 1; }

specs_dir="$(harness_path "$(config_get "$MAIN_CONFIG" paths.specs)")"
draft_dir="$specs_dir/$(config_get "$SPECS_CONFIG" draft_dir)"
refined_dir="$specs_dir/$(config_get "$SPECS_CONFIG" refined_dir)"
draft_status="$(config_get "$SPECS_CONFIG" status.draft)"
refined_status="$(config_get "$SPECS_CONFIG" status.refined)"

find_draft() {
  spec_find_by_id "$draft_dir" "$1" || die "spec $1 não encontrada em $draft_dir"
}

create_refined() {
  local spec_id="$1" draft status template target now_date now_datetime overrides front_matter

  draft="$(find_draft "$spec_id")"
  status="$(frontmatter_get "$draft" status)"
  [ "$status" = "$draft_status" ] ||
    die "a spec $spec_id está com status '$status'; só specs com status '$draft_status' podem ser refinadas"

  target="$refined_dir/$(basename "$draft")"
  [ ! -e "$target" ] ||
    die "a spec refinada já existe: $target. Preencha-a e rode: create-refined-spec.sh --finalize $spec_id"

  template="$(harness_path "$(config_get "$MAIN_CONFIG" paths.templates)")/$(config_get "$SPECS_CONFIG" refined_template)"
  [ -f "$template" ] || die "template não encontrado: $template"

  now_date="$(date -u +%Y-%m-%d)"
  now_datetime="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

  overrides="$(
    config_block "$SPECS_CONFIG" refined_front_matter |
      TPL_SPEC_STATUS="$refined_status" \
      TPL_SOURCE_PATH="$(yaml_escape "${draft#"$specs_dir"/}")" \
      TPL_DATE="$now_date" \
      TPL_DATETIME="$now_datetime" \
      render
  )"
  front_matter="$(frontmatter_merge "$(frontmatter_print "$draft")" "$overrides")"

  mkdir -p "$refined_dir"
  TPL_FRONT_MATTER="$(printf -- '---\n%s\n---' "$front_matter")" \
  TPL_SPEC_ID="$spec_id" \
  TPL_SPEC_TITLE="$(frontmatter_get "$draft" title)" \
    render "$template" > "$target"

  echo "Spec refinada $spec_id criada em $target"
}

finalize_refined() {
  local spec_id="$1" refined leftovers draft

  refined="$(spec_find_by_id "$refined_dir" "$spec_id")" ||
    die "spec refinada $spec_id não encontrada em $refined_dir. Crie-a com: create-refined-spec.sh $spec_id"

  if leftovers="$(template_leftovers "$refined")"; then
    die "ainda há variáveis sem preencher em $refined:"$'\n'"$leftovers"
  fi

  draft="$(find_draft "$spec_id")"
  frontmatter_set "$draft" status "$refined_status"

  echo "Spec $spec_id finalizada: $refined"
  echo "Status do rascunho alterado para '$refined_status': $draft"
}

case "${1:-}" in
  --finalize) [ -n "${2:-}" ] || die "$USAGE"; finalize_refined "$2" ;;
  -*|"") die "$USAGE" ;;
  *) create_refined "$1" ;;
esac
