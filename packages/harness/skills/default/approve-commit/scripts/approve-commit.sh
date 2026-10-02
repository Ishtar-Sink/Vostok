#!/usr/bin/env bash
# Faz commits e libera ou desfaz os commits apontados pelo check-commits.sh, sempre a pedido do usuário
# ([agent/boundaries-01], [agent/boundaries-04]). O código de liberação identifica o estado mostrado: se o
# repositório mudar depois, a ação é recusada.
#
# Uso: approve-commit.sh --show
#        Lista os commits não liberados e o código de liberação. Não altera nada.
#      approve-commit.sh --approve <código>
#        Grava o estado atual como liberado.
#      approve-commit.sh --commit [caminho...] < mensagem
#        Faz um commit com a mensagem lida da entrada e o libera. Com caminhos, commita só eles (como
#        `git commit -- caminho...`); sem caminhos, commita o que está em stage. Recusa se já houver
#        commits sem liberação. Se o commit falhar, nada é liberado.
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

USAGE="Uso: approve-commit.sh --show | --approve <código> | --commit [caminho...] < mensagem | --revert <código>"

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

commit() {
  local message before created pending
  pending="$(checked_changes)" || exit 1
  [ -z "$pending" ] ||
    die "já há commits sem liberação; mostre-os ao usuário e libere ou desfaça antes de um commit novo"
  message="$(cat)"
  [[ "$message" =~ [^[:space:]] ]] || die "mensagem de commit vazia; passe-a pela entrada padrão"
  before="$(_commits_git rev-parse -q --verify HEAD || true)"
  if [ "$#" -gt 0 ]; then
    # O `add` antes inclui arquivos novos e removidos; o commit com caminhos deixa o resto do stage como está.
    _commits_git add -A -- "$@" || die "caminho inválido; nada foi commitado"
    printf '%s\n' "$message" | _commits_git commit -q -F - -- "$@" || die "o commit falhou; nada foi liberado"
  else
    printf '%s\n' "$message" | _commits_git commit -q -F - || die "o commit falhou; nada foi liberado"
  fi
  # Libera só se o único commit novo for o que acabou de ser criado, sobre o HEAD anterior.
  created="$(commits_changes "$ack")"
  [ "$created" = "novo $(_commits_git rev-parse HEAD)" ] && [ "$(_commits_git rev-parse -q --verify HEAD^ || true)" = "$before" ] ||
    die "o estado do repositório não corresponde a um único commit novo; nada foi liberado"
  commits_store
  echo "Commit feito e liberado: $(_commits_git log -1 --format='%h %s')"
}

revert() {
  local changes branch base news head_news
  require_token "${1:-}"
  changes="$(checked_changes)" || exit 1
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
  --commit) shift; commit "$@" ;;
  --revert) [ "$#" -eq 2 ] || die "$USAGE"; revert "$2" ;;
  *) die "$USAGE" ;;
esac
