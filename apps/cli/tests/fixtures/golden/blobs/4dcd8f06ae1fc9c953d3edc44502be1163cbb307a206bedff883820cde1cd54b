#!/usr/bin/env bash
# #{SCRIPT_DESCRIPTION}
#
# Uso: #{SCRIPT_NAME}.sh #{SCRIPT_USAGE}
set -euo pipefail

# Raiz do harness: primeira pasta acima do script que contém `config/main.yml`.
HARNESS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && until [ -f config/main.yml ] || [ "$PWD" = / ]; do cd ..; done && pwd)"
source "$HARNESS_DIR/scripts/lib/config.sh"

die() { echo "Erro: $*" >&2; exit 1; }

#{SCRIPT_BODY}
