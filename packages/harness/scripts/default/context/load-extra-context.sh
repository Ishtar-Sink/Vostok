#!/usr/bin/env bash
# Carrega sob demanda um ou mais arquivos ou pastas, além do contexto obrigatório.
#
# Uso: load-extra-context.sh <caminho> [caminho...]
#      Caminhos relativos partem da raiz do harness; se não existirem lá, do diretório atual.
#      Termina com erro se algum caminho não for encontrado, depois de carregar os demais.
#      load-extra-context.sh --skill <nome>
#      Carrega os arquivos da entrada `skills.<nome>` do config/context.yml, na ordem do mapa.
#      Erro se a skill não existir; só o cabeçalho se ela não tiver entrada.
set -euo pipefail

# Raiz do harness: primeira pasta acima do script que contém `config/main.yml`.
HARNESS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && until [ -f config/main.yml ] || [ "$PWD" = / ]; do cd ..; done && pwd)"
source "$HARNESS_DIR/scripts/lib/config.sh"
source "$HARNESS_DIR/scripts/lib/context.sh"

# Resolve um caminho: absoluto, relativo à raiz do harness ou relativo ao diretório atual.
resolve_path() {
  local path="${1#./}"

  case "$path" in
    /*) printf '%s' "$path" ;;
    *) if [ -e "$HARNESS_DIR/$path" ]; then
         printf '%s' "$HARNESS_DIR/$path"
       else
         printf '%s' "$PWD/$path"
       fi ;;
  esac
}

# Modo --skill: carrega a entrada da skill no mapa de contexto.
if [ "${1:-}" = "--skill" ]; then
  skill="${2:-}"
  if [ -z "$skill" ]; then
    echo "Uso: load-extra-context.sh --skill <nome>" >&2
    exit 1
  fi
  default_dir="$(config_get "$HARNESS_DIR/config/skills.yml" dirs.default)"
  custom_dir="$(config_get "$HARNESS_DIR/config/skills.yml" dirs.custom)"
  if [ ! -f "$HARNESS_DIR/$default_dir/$skill/SKILL.md" ] && [ ! -f "$HARNESS_DIR/$custom_dir/$skill/SKILL.md" ]; then
    echo "Skill '$skill' não existe" >&2
    exit 1
  fi

  status=0
  print_context_header "Contexto adicional"
  while IFS= read -r source; do
    [ -n "$source" ] || continue
    if ! print_context_path "$(resolve_path "$source")"; then
      echo "skills.$skill: '$source' não existe" >&2
      status=1
    fi
  done < <(context_skill_entries "$HARNESS_DIR/config/context.yml" "$skill")
  exit "$status"
fi

if [ $# -eq 0 ]; then
  echo "Uso: load-extra-context.sh <caminho> [caminho...]" >&2
  exit 1
fi

status=0
print_context_header "Contexto adicional"

for source in "$@"; do
  if ! print_context_path "$(resolve_path "$source")"; then
    echo "Aviso: '$source' não encontrado" >&2
    status=1
  fi
done

exit "$status"
