#!/usr/bin/env bash
# Cria a spec refinada a partir de um rascunho e, depois de preenchida pelo agente, a valida e finaliza.
# As seções da refinada dependem do tipo da spec (`types` e `sections_<tipo>` em config/specs.yml).
# O front-matter parte do rascunho, com o tipo, a referência ao rascunho, o número de ajustes
# e a data do último ajuste (ver `refined_front_matter`).
#
# Uso: create-refined-spec.sh <SPEC_ID> <tipo>
#        Cria a refinada em `refined_dir` com as seções do tipo. Só aceita rascunhos com status.draft.
#      create-refined-spec.sh --finalize <SPEC_ID>
#        Valida a refinada e, se estiver tudo certo, muda o status do rascunho para status.refined.
#        Lista todos os problemas encontrados: variáveis sem preencher, seções ausentes ou vazias,
#        requisitos sem caso de teste ou critério de aceite, argumentos da tese ou da antítese
#        fora da síntese e componentes afetados inconsistentes.
set -euo pipefail

# Raiz do harness: primeira pasta acima do script que contém `config/main.yml`.
HARNESS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && until [ -f config/main.yml ] || [ "$PWD" = / ]; do cd ..; done && pwd)"
source "$HARNESS_DIR/scripts/lib/config.sh"
source "$HARNESS_DIR/scripts/lib/text.sh"
source "$HARNESS_DIR/scripts/lib/frontmatter.sh"
source "$HARNESS_DIR/scripts/lib/markdown.sh"

MAIN_CONFIG="$HARNESS_DIR/config/main.yml"
SPECS_CONFIG="$HARNESS_DIR/config/specs.yml"

USAGE="Uso: create-refined-spec.sh <SPEC_ID> <tipo> | --finalize <SPEC_ID>"

# Ações aceitas na tabela de componentes afetados. `criar` exige que o caminho ainda não exista;
# `alterar` e `remover` exigem que exista. Caminhos são relativos a `paths.root`.
COMPONENT_ACTIONS="criar alterar remover"

die() { echo "Erro: $*" >&2; exit 1; }

root_dir="$(cd "$(harness_path "$(config_get "$MAIN_CONFIG" paths.root)")" && pwd)"
templates_dir="$(harness_path "$(config_get "$MAIN_CONFIG" paths.templates)")"
sections_dir="$templates_dir/$(config_get "$SPECS_CONFIG" sections_dir)"
specs_dir="$(harness_path "$(config_get "$MAIN_CONFIG" paths.specs)")"
draft_dir="$specs_dir/$(config_get "$SPECS_CONFIG" draft_dir)"
refined_dir="$specs_dir/$(config_get "$SPECS_CONFIG" refined_dir)"
draft_status="$(config_get "$SPECS_CONFIG" status.draft)"
refined_status="$(config_get "$SPECS_CONFIG" status.refined)"

find_draft() {
  spec_find_by_id "$draft_dir" "$1" || die "spec $1 não encontrada em $draft_dir"
}

# Seções do tipo, uma por linha. Falha se o tipo não existir.
sections_for_type() {
  config_list "$SPECS_CONFIG" types | grep -qxF "$1" ||
    die "tipo inválido '$1'. Tipos: $(config_list "$SPECS_CONFIG" types | tr '\n' ' ')"
  config_list "$SPECS_CONFIG" "sections_$1"
}

section_template() {
  printf '%s/%s.md' "$sections_dir" "$1"
}

# Título da seção, lido da primeira linha do template (`## Título`).
section_heading() {
  head -n 1 "$(section_template "$1")" | sed 's/^## //'
}

create_refined() {
  local spec_id="$1" type="$2" sections draft status template target now_datetime overrides front_matter
  local section content=""

  sections="$(sections_for_type "$type")"
  draft="$(find_draft "$spec_id")"
  status="$(frontmatter_get "$draft" status)"
  [ "$status" = "$draft_status" ] ||
    die "a spec $spec_id está com status '$status'; só specs com status '$draft_status' podem ser refinadas"

  target="$refined_dir/$(basename "$draft")"
  [ ! -e "$target" ] ||
    die "a spec refinada já existe: $target. Preencha-a e rode: create-refined-spec.sh --finalize $spec_id"

  template="$templates_dir/$(config_get "$SPECS_CONFIG" refined_template)"
  [ -f "$template" ] || die "template não encontrado: $template"

  while IFS= read -r section; do
    [ -f "$(section_template "$section")" ] || die "template de seção não encontrado: $(section_template "$section")"
    content+="$(cat "$(section_template "$section")")"$'\n\n'
  done <<< "$sections"

  now_datetime="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  overrides="$(
    config_block "$SPECS_CONFIG" refined_front_matter |
      TPL_SPEC_TYPE="$type" \
      TPL_SPEC_STATUS="$refined_status" \
      TPL_SOURCE_PATH="$(yaml_escape "${draft#"$specs_dir"/}")" \
      TPL_DATE="$(date -u +%Y-%m-%d)" \
      TPL_DATETIME="$now_datetime" \
      render
  )"
  front_matter="$(frontmatter_merge "$(frontmatter_print "$draft")" "$overrides")"

  mkdir -p "$refined_dir"
  TPL_FRONT_MATTER="$(printf -- '---\n%s\n---' "$front_matter")" \
  TPL_SPEC_ID="$spec_id" \
  TPL_SPEC_TITLE="$(frontmatter_get "$draft" title)" \
  TPL_SECTIONS="${content%$'\n\n'}" \
    render "$template" > "$target"

  echo "Spec refinada $spec_id ($type) criada em $target"
}

# Imprime um erro por linha para cada problema da refinada. Não imprime nada se ela estiver válida.
refined_errors() {
  local file="$1" type sections section heading text requirement row path action

  template_leftovers "$file" | sed 's/^/variável sem preencher: /' || true

  type="$(frontmatter_get "$file" type 2>/dev/null || true)"
  if ! config_list "$SPECS_CONFIG" types | grep -qxF "$type"; then
    echo "o campo 'type' do front-matter é inválido: '$type'"
    return
  fi
  sections="$(config_list "$SPECS_CONFIG" "sections_$type")"

  # Toda seção do tipo existe e tem conteúdo; seções com tabela têm ao menos uma linha.
  while IFS= read -r section; do
    heading="$(section_heading "$section")"
    if ! text="$(markdown_section "$file" "$heading")"; then
      echo "seção ausente: '## $heading'"
    elif grep -q '^|' "$(section_template "$section")"; then
      [ -n "$(markdown_table_rows <<< "$text")" ] ||
        echo "a tabela da seção '$heading' está vazia (use uma linha com '-' se não se aplica)"
    elif ! grep -q '[^[:space:]]' <<< "$text"; then
      echo "a seção '$heading' está vazia"
    fi
  done <<< "$sections"

  # Todo requisito funcional aparece em algum caso de teste e em algum critério de aceite.
  if grep -qxF functional-requirements <<< "$sections"; then
    while IFS= read -r row; do
      requirement="$(markdown_cell "$row" 1)"
      [ "$requirement" != "-" ] || continue
      if [[ ! "$requirement" =~ ^RF-[0-9]+$ ]]; then
        echo "ID de requisito funcional inválido: '$requirement' (use RF-01, RF-02...)"
        continue
      fi
      for section in test-cases acceptance-criteria; do
        grep -qxF "$section" <<< "$sections" || continue
        heading="$(section_heading "$section")"
        markdown_section "$file" "$heading" 2>/dev/null | grep -qE "(^|[^A-Z0-9-])$requirement([^0-9]|\$)" ||
          echo "$requirement não aparece em '$heading'"
      done
    done <<< "$(markdown_section "$file" "$(section_heading functional-requirements)" 2>/dev/null | markdown_table_rows)"
  fi

  # Análise: todo argumento da tese e da antítese é considerado em alguma conclusão da síntese.
  if grep -qxF synthesis <<< "$sections"; then
    text="$(markdown_section "$file" "$(section_heading synthesis)" 2>/dev/null | markdown_table_rows)"
    for section in thesis antithesis; do
      grep -qxF "$section" <<< "$sections" || continue
      while IFS= read -r row; do
        [ -n "$row" ] || continue
        requirement="$(markdown_cell "$row" 1)"
        [ "$requirement" != "-" ] || continue
        grep -qE "(^|[^A-Z0-9-])$requirement([^0-9]|\$)" <<< "$text" ||
          echo "$requirement ('$(section_heading "$section")') não é considerado em nenhuma conclusão da síntese"
      done <<< "$(markdown_section "$file" "$(section_heading "$section")" 2>/dev/null | markdown_table_rows)"
    done
  fi

  # Componentes afetados: ação conhecida e caminho coerente com ela.
  if grep -qxF affected-components <<< "$sections"; then
    while IFS= read -r row; do
      [ -n "$row" ] || continue
      path="$(markdown_cell "$row" 1 | tr -d '`')"
      action="$(markdown_cell "$row" 2)"
      [ "$path" != "-" ] || continue
      case " $COMPONENT_ACTIONS " in
        *" $action "*) ;;
        *) echo "ação inválida para '$path': '$action' (use: $COMPONENT_ACTIONS)"; continue ;;
      esac
      if [ "$action" = criar ] && [ -e "$root_dir/$path" ]; then
        echo "'$path' já existe; use a ação 'alterar'"
      elif [ "$action" != criar ] && [ ! -e "$root_dir/$path" ]; then
        echo "'$path' não existe em $root_dir; use a ação 'criar' ou corrija o caminho"
      fi
    done <<< "$(markdown_section "$file" "$(section_heading affected-components)" 2>/dev/null | markdown_table_rows)"
  fi
}

finalize_refined() {
  local spec_id="$1" refined errors draft

  refined="$(spec_find_by_id "$refined_dir" "$spec_id")" ||
    die "spec refinada $spec_id não encontrada em $refined_dir. Crie-a com: create-refined-spec.sh $spec_id <tipo>"

  errors="$(refined_errors "$refined")"
  [ -z "$errors" ] || die "a spec refinada $spec_id tem problemas:"$'\n'"$errors"

  draft="$(find_draft "$spec_id")"
  frontmatter_set "$draft" status "$refined_status"

  echo "Spec $spec_id finalizada: $refined"
  echo "Status do rascunho alterado para '$refined_status': $draft"
}

case "${1:-}" in
  --finalize) [ -n "${2:-}" ] || die "$USAGE"; finalize_refined "$2" ;;
  -*|"") die "$USAGE" ;;
  *) [ -n "${2:-}" ] || die "informe o tipo. $USAGE"; create_refined "$1" "$2" ;;
esac
