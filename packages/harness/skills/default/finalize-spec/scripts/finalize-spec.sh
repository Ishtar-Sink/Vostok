#!/usr/bin/env bash
# Finaliza uma spec implementada: fontes da síntese, conteúdo dos documentos, validação, geração dos
# documentos HTML e arquivamento. Toda opção começa conferindo se a spec pode ser finalizada: status
# `implemented` e todas as tarefas `done`.
#
# Uso: finalize-spec.sh --sources <SPEC_ID>
#        Imprime a spec, as tarefas, as evidências, os caminhos alterados, a memória e os commits do período.
#      finalize-spec.sh --create <SPEC_ID>
#        Cria o arquivo de conteúdo a partir do template (`finalize_content_template` em specs.yml).
#      finalize-spec.sh --validate <SPEC_ID>
#        Lista os problemas do conteúdo, sem alterar nada.
#      finalize-spec.sh --build <SPEC_ID>
#        Gera os documentos HTML em `paths.finalize`/<SPEC_ID>/.
#      finalize-spec.sh --archive <SPEC_ID>
#        Move a spec, as tarefas e os documentos para `paths.archive`/<SPEC_ID>/, copia o executivo e o
#        técnico para `paths.spec_docs`/<SPEC_ID>/ e grava o status `archived`.
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
source "$HARNESS_DIR/scripts/lib/task.sh"
source "$HARNESS_DIR/scripts/lib/memory.sh"
source "$HARNESS_DIR/scripts/lib/finalize.sh"

SPECS_CONFIG="$HARNESS_DIR/config/specs.yml"

USAGE='Uso: finalize-spec.sh --sources|--create|--validate|--build|--archive <SPEC_ID>'

die() { echo "Erro: $*" >&2; exit 1; }

spec_prefix="$(config_get "$SPECS_CONFIG" id.prefix)"

# Caminho da spec que pode ser finalizada. Termina com erro, listando os problemas, se ela não puder.
# require_finalizable <SPEC_ID>
require_finalizable() {
  local spec problems
  [[ "$1" =~ ^$spec_prefix-[0-9]+$ ]] || die "ID de spec inválido: '$1'"
  spec="$(plan_spec_file "$1")" || die "spec $1 não encontrada entre as refinadas"
  problems="$(finalize_check "$spec" "$(finalize_tasks_dir "$1")")"
  if [ -n "$problems" ]; then
    echo "Erro: a spec $1 não pode ser finalizada:" >&2
    printf '%s\n' "$problems" | sed 's/^/  - /' >&2
    exit 1
  fi
  printf '%s' "$spec"
}

[ "$#" -eq 2 ] || die "$USAGE"
case "$1" in
  --sources)
    spec="$(require_finalizable "$2")"
    finalize_sources "$spec" "$(finalize_tasks_dir "$2")"
    ;;
  --create|--validate|--build|--archive)
    require_finalizable "$2" > /dev/null
    die "$1 ainda não implementado"
    ;;
  *) die "$USAGE" ;;
esac
