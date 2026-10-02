#!/usr/bin/env bash
# Libera ou desfaz, a pedido do usuário, os commits apontados pelo check-commits.sh ([agent/boundaries-04]).
# O código de liberação identifica o estado mostrado: se o repositório mudar depois, a ação é recusada.
#
# Uso: approve-commit.sh --show
#        Lista os commits não liberados e o código de liberação. Não altera nada.
#      approve-commit.sh --approve <código>
#        Grava o estado atual como liberado.
#      approve-commit.sh --revert <código>
#        Desfaz os commits novos da branch atual com `git reset --mixed`, mantendo as alterações na pasta
#        de trabalho, fora de stage. Recusa se algum commit já estiver num ref remoto, se houver commits
#        removidos ou fora da branch atual.
set -euo pipefail

# Raiz do harness: primeira pasta acima do script que contém `config/main.yml`.
HARNESS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && until [ -f config/main.yml ] || [ "$PWD" = / ]; do cd ..; done && pwd)"
source "$HARNESS_DIR/scripts/lib/config.sh"
source "$HARNESS_DIR/scripts/lib/text.sh"
source "$HARNESS_DIR/scripts/lib/commits.sh"

die() { echo "Erro: $*" >&2; exit 1; }

USAGE="Uso: approve-commit.sh --show | --approve <código> | --revert <código>"

commits_has_git || die "sem repositório git em $(_commits_root)"
ack="$(commits_ack_file)"
[ -f "$ack" ] || die "ainda não há referência de commits; execute check-commits.sh"

# Mudanças em relação à referência. Recusa se a referência for inválida.
checked_changes() {
  local errors
  errors="$(commits_ack_errors "$ack")"
  [ -z "$errors" ] || die "a referência de commits é inválida; ela só pode ser refeita com --approve:"$'\n'"$errors"
  commits_changes "$ack"
}

require_token() {
  [ -n "${1:-}" ] || die "$USAGE"
  [ "$1" = "$(commits_token)" ] ||
    die "o repositório mudou desde a verificação (código atual: $(commits_token)); mostre os commits ao usuário de novo"
}

show() {
  local errors report
  errors="$(commits_ack_errors "$ack")"
  if [ -n "$errors" ]; then
    echo "Referência de commits inválida:"; echo "$errors"
  else
    report="$(commits_report "$ack")"
    if [ -z "$report" ]; then echo "Nenhum commit sem liberação."; else echo "$report"; fi
  fi
  echo "Código de liberação: $(commits_token)"
}

approve() {
  local errors count=0
  require_token "${1:-}"
  errors="$(commits_ack_errors "$ack")"
  [ -n "$errors" ] || count="$(commits_changes "$ack" | grep -c . || true)"
  if [ -z "$errors" ] && [ "$count" -eq 0 ]; then echo "Nenhum commit sem liberação."; return; fi
  commits_store
  if [ -n "$errors" ]; then
    echo "Referência de commits refeita a partir do estado atual."
  else
    echo "Liberação gravada: $count commit(s) novo(s) ou removido(s)."
  fi
}

revert() {
  local changes branch base news head_news
  require_token "${1:-}"
  changes="$(checked_changes)"
  [ -n "$changes" ] || { echo "Nenhum commit sem liberação."; return; }
  ! grep -q '^removido ' <<< "$changes" ||
    die "há commits removidos; desfazer não os recupera. Peça ao usuário para decidir como seguir"
  branch="$(_commits_git symbolic-ref -q HEAD)" || die "HEAD sem branch (detached); peça ao usuário para desfazer à mão"
  base="$(awk -v r="$branch" '$2 == r { print $1 }' "$ack")"
  [ -n "$base" ] || die "a branch ${branch#refs/heads/} não existia na liberação; peça ao usuário para desfazer à mão"
  _commits_git merge-base --is-ancestor "$base" HEAD ||
    die "a branch ${branch#refs/heads/} foi reescrita; peça ao usuário para desfazer à mão"
  news="$(grep '^novo ' <<< "$changes" | cut -d' ' -f2 | LC_ALL=C sort)"
  head_news="$(_commits_git rev-list HEAD --not "$base" | LC_ALL=C sort)"
  [ "$news" = "$head_news" ] ||
    die "há commits novos fora da branch ${branch#refs/heads/}; peça ao usuário para desfazer à mão"
  # Commits da branch que nenhum ref remoto contém; se faltar algum, ele já foi enviado.
  [ "$(_commits_git rev-list HEAD --not "$base" --remotes | LC_ALL=C sort)" = "$head_news" ] ||
    die "há commits já enviados ao remoto; desfazê-los exige reescrever o histórico compartilhado. Peça ao usuário para decidir"
  _commits_git reset --mixed -q "$base"
  echo "$(grep -c . <<< "$news") commit(s) desfeito(s) em ${branch#refs/heads/}; as alterações continuam na pasta de trabalho, fora de stage."
}

case "${1:-}" in
  --show) [ "$#" -eq 1 ] || die "$USAGE"; show ;;
  --approve) [ "$#" -eq 2 ] || die "$USAGE"; approve "$2" ;;
  --revert) [ "$#" -eq 2 ] || die "$USAGE"; revert "$2" ;;
  *) die "$USAGE" ;;
esac
