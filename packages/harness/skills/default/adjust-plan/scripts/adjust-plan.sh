#!/usr/bin/env bash
# Registra os ajustes do plano de tarefas de uma spec: edições feitas à mão e ajustes do agente.
# Segue o registro da spec refinada (scripts/lib/adjustments.sh): contagem, datas, hash do conteúdo
# e uma linha por ajuste na seção de histórico do plano. A validação é a do plano (scripts/lib/plan.sh).
#
# Uso: adjust-plan.sh --record-manual <SPEC_ID>
#        Compara o hash gravado com o do conteúdo atual. Se forem diferentes, registra um ajuste
#        de origem manual. Plano sem hash conta como editado à mão.
#      adjust-plan.sh --validate <SPEC_ID>
#        Lista os problemas do plano, inclusive plano desatualizado, sem alterar nada.
#        Termina com erro se houver algum.
#      adjust-plan.sh --record <SPEC_ID> "resumo"
#        Recalcula os níveis, valida o plano e registra o ajuste do agente. Com problemas, termina
#        com erro sem alterar nada. Vários ajustes da mesma mensagem vão no mesmo resumo, separados por "; ".
set -euo pipefail

# Raiz do harness: primeira pasta acima do script que contém `config/main.yml`.
HARNESS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && until [ -f config/main.yml ] || [ "$PWD" = / ]; do cd ..; done && pwd)"
source "$HARNESS_DIR/scripts/lib/config.sh"
source "$HARNESS_DIR/scripts/lib/text.sh"
source "$HARNESS_DIR/scripts/lib/frontmatter.sh"
source "$HARNESS_DIR/scripts/lib/markdown.sh"
source "$HARNESS_DIR/scripts/lib/spec-validation.sh"
source "$HARNESS_DIR/scripts/lib/adjustments.sh"
source "$HARNESS_DIR/scripts/lib/plan.sh"

USAGE='Uso: adjust-plan.sh --record-manual <SPEC_ID> | --validate <SPEC_ID> | --record <SPEC_ID> "resumo"'

die() { echo "Erro: $*" >&2; exit 1; }

find_plan() {
  local plan
  plan="$(plan_path "$1")"
  [ -f "$plan" ] || die "a spec $1 não tem plano de tarefas para ajustar"
  printf '%s' "$plan"
}

find_spec() {
  plan_spec_file "$1" || die "spec refinada $1 não encontrada"
}

# Problemas do plano, com o aviso de plano desatualizado primeiro.
# all_errors <SPEC_ID> <plano> <spec>
all_errors() {
  ! plan_is_outdated "$2" "$3" ||
    echo "o plano foi feito para outra versão da spec $1; recrie-o com: plan-tasks.sh --create $1"
  plan_errors "$2" "$3"
}

record_manual() {
  local spec_id="$1" plan summary count

  plan="$(find_plan "$spec_id")"
  if [ "$(adjustment_hash_state "$plan")" = same ]; then
    echo "Nenhuma edição manual."
    return
  fi

  summary="$(adjustment_manual_summary "$plan")"
  count="$(adjustment_record "$plan" "$(adjustment_origin manual)" "$summary")"
  echo "Ajuste manual $count registrado no plano de $spec_id: $summary"
}

validate() {
  local spec_id="$1" plan spec errors

  plan="$(find_plan "$spec_id")"
  spec="$(find_spec "$spec_id")"
  errors="$(all_errors "$spec_id" "$plan" "$spec")"
  [ -z "$errors" ] || die "o plano de $spec_id tem problemas:"$'\n'"$errors"
  echo "Plano de $spec_id válido: $plan"
}

record() {
  local spec_id="$1" summary plan spec tmp problems errors count

  summary="$(adjustment_clean_summary "$2" 2>&1)" || die "$summary"
  plan="$(find_plan "$spec_id")"
  spec="$(find_spec "$spec_id")"

  # Níveis e validação numa cópia: o plano só muda se tudo estiver certo.
  tmp="$(mktemp)"
  cp "$plan" "$tmp"
  if ! problems="$(plan_score "$tmp")"; then
    rm -f "$tmp"
    die "o plano de $spec_id tem problemas; nenhum ajuste registrado:"$'\n'"$problems"
  fi
  errors="$(all_errors "$spec_id" "$tmp" "$spec")"
  if [ -n "$errors" ]; then
    rm -f "$tmp"
    die "o plano de $spec_id tem problemas; nenhum ajuste registrado:"$'\n'"$errors"
  fi
  cat "$tmp" > "$plan"
  rm -f "$tmp"

  count="$(adjustment_record "$plan" "$(adjustment_origin agent)" "$summary")"
  echo "Ajuste $count registrado no plano de $spec_id: $plan"
}

case "${1:-}" in
  --record-manual) [ -n "${2:-}" ] || die "$USAGE"; record_manual "$2" ;;
  --validate) [ -n "${2:-}" ] || die "$USAGE"; validate "$2" ;;
  --record) [ -n "${2:-}" ] && [ $# -ge 3 ] || die "$USAGE"; record "$2" "$3" ;;
  *) die "$USAGE" ;;
esac
