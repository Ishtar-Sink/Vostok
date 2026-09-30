#!/usr/bin/env bash
# Registra os ajustes de uma spec refinada: edições feitas à mão e ajustes feitos pelo agente.
# Cada ajuste incrementa `adjustments`, atualiza `adjusted_at`, `updated_at` e o hash do conteúdo
# (`content_hash_field`) e acrescenta uma linha à seção de histórico (`adjustment_history_section`).
# A validação é a mesma do refinamento (scripts/lib/spec-validation.sh).
# Specs aprovadas também são ajustadas: todo ajuste registrado derruba a aprovação, volta o status
# para status.refined e grava `null` nos campos da aprovação.
#
# Uso: adjust-spec.sh --record-manual <SPEC_ID>
#        Compara o hash gravado com o do conteúdo atual. Se forem diferentes, registra um ajuste
#        de origem manual e lista os problemas de validação, se houver. Spec sem hash conta como
#        editada à mão.
#      adjust-spec.sh --validate <SPEC_ID>
#        Lista os problemas da refinada, sem alterar nada. Termina com erro se houver problemas.
#      adjust-spec.sh --record <SPEC_ID> "resumo"
#        Valida a refinada e registra o ajuste do agente. Com problemas, termina com erro sem
#        alterar nada. Vários ajustes da mesma mensagem vão no mesmo resumo, separados por "; ".
set -euo pipefail

# Raiz do harness: primeira pasta acima do script que contém `config/main.yml`.
HARNESS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && until [ -f config/main.yml ] || [ "$PWD" = / ]; do cd ..; done && pwd)"
source "$HARNESS_DIR/scripts/lib/config.sh"
source "$HARNESS_DIR/scripts/lib/text.sh"
source "$HARNESS_DIR/scripts/lib/frontmatter.sh"
source "$HARNESS_DIR/scripts/lib/markdown.sh"
source "$HARNESS_DIR/scripts/lib/spec-validation.sh"
source "$HARNESS_DIR/scripts/lib/adjustments.sh"

MAIN_CONFIG="$HARNESS_DIR/config/main.yml"
SPECS_CONFIG="$HARNESS_DIR/config/specs.yml"

USAGE='Uso: adjust-spec.sh --record-manual <SPEC_ID> | --validate <SPEC_ID> | --record <SPEC_ID> "resumo"'

die() { echo "Erro: $*" >&2; exit 1; }

specs_dir="$(harness_path "$(config_get "$MAIN_CONFIG" paths.specs)")"
draft_dir="$specs_dir/$(config_get "$SPECS_CONFIG" draft_dir)"
refined_dir="$specs_dir/$(config_get "$SPECS_CONFIG" refined_dir)"
refined_status="$(config_get "$SPECS_CONFIG" status.refined)"
approved_status="$(config_get "$SPECS_CONFIG" status.approved)"
approved_at_field="$(config_get "$SPECS_CONFIG" approved_at_field)"
approved_hash_field="$(config_get "$SPECS_CONFIG" approved_hash_field)"
planned_status="$(config_get "$SPECS_CONFIG" status.planned)"

# Caminho da refinada. Falha se ela não existir ou não estiver com status.refined ou status.approved.
find_refined() {
  local spec_id="$1" refined status

  if ! refined="$(spec_find_by_id "$refined_dir" "$spec_id")"; then
    spec_find_by_id "$draft_dir" "$spec_id" > /dev/null &&
      die "a spec $spec_id ainda não foi refinada; só specs refinadas são ajustadas"
    die "spec refinada $spec_id não encontrada em $refined_dir"
  fi

  status="$(frontmatter_get "$refined" status 2>/dev/null || true)"
  [ "$status" != "$planned_status" ] ||
    die "a spec $spec_id está com status '$status': as tarefas já foram criadas, e mudá-la exige um novo ciclo"
  [ "$status" = "$refined_status" ] || [ "$status" = "$approved_status" ] ||
    die "a spec $spec_id está com status '$status'; só specs com status '$refined_status' ou '$approved_status' são ajustadas"
  printf '%s' "$refined"
}

# Derruba a aprovação de uma spec aprovada e avisa. Não faz nada numa spec só refinada.
# drop_approval <arquivo> <SPEC_ID>
drop_approval() {
  [ "$(frontmatter_get "$1" status 2>/dev/null || true)" = "$approved_status" ] || return 0
  frontmatter_set "$1" status "$refined_status"
  frontmatter_put "$1" "$approved_at_field" null
  frontmatter_put "$1" "$approved_hash_field" null
  echo "Aprovação de $2 derrubada: o status voltou para '$refined_status'."
}

record_manual() {
  local spec_id="$1" refined summary count errors

  refined="$(find_refined "$spec_id")"
  if [ "$(adjustment_hash_state "$refined")" = same ]; then
    echo "Nenhuma edição manual."
    return
  fi

  summary="$(adjustment_manual_summary "$refined")"
  count="$(adjustment_record "$refined" "$(adjustment_origin manual)" "$summary")"
  echo "Ajuste manual $count registrado em $spec_id: $summary"
  drop_approval "$refined" "$spec_id"

  errors="$(refined_errors "$refined")"
  [ -z "$errors" ] || printf 'Problemas de validação:\n%s\n' "$errors"
}

validate() {
  local spec_id="$1" refined errors

  refined="$(find_refined "$spec_id")"
  errors="$(refined_errors "$refined")"
  [ -z "$errors" ] || die "a spec refinada $spec_id tem problemas:"$'\n'"$errors"
  echo "Spec refinada $spec_id válida: $refined"
}

record() {
  local spec_id="$1" summary refined errors count

  summary="$(adjustment_clean_summary "$2" 2>&1)" || die "$summary"

  refined="$(find_refined "$spec_id")"
  errors="$(refined_errors "$refined")"
  [ -z "$errors" ] || die "a spec refinada $spec_id tem problemas; nenhum ajuste registrado:"$'\n'"$errors"

  count="$(adjustment_record "$refined" "$(adjustment_origin agent)" "$summary")"
  echo "Ajuste $count registrado em $spec_id: $refined"
  drop_approval "$refined" "$spec_id"
}

case "${1:-}" in
  --record-manual) [ -n "${2:-}" ] || die "$USAGE"; record_manual "$2" ;;
  --validate) [ -n "${2:-}" ] || die "$USAGE"; validate "$2" ;;
  --record) [ -n "${2:-}" ] && [ $# -ge 3 ] || die "$USAGE"; record "$2" "$3" ;;
  *) die "$USAGE" ;;
esac
