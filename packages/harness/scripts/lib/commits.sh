# Commits liberados pelo usuário ([agent/boundaries-04]). A referência é a foto das refs locais
# (HEAD, branches e tags, sem `refs/remotes/` nem `refs/stash`) no momento da última liberação, gravada
# em `paths.state`/commit-ack, fora do git. Commit novo é o alcançável hoje e não alcançável pela foto;
# commit removido, o contrário. Branch trocada ou criada sobre commits existentes não conta.
# Uso: source "$HARNESS_DIR/scripts/lib/commits.sh"
#      (requer HARNESS_DIR e as libs config.sh e text.sh)

_commits_root() {
  (cd "$(harness_path "$(config_get "$HARNESS_DIR/config/main.yml" paths.root)")" && pwd)
}

_commits_git() {
  git -C "$(_commits_root)" "$@"
}

# Caminho do arquivo com a foto das refs liberadas.
commits_ack_file() {
  printf '%s/commit-ack' "$(harness_path "$(config_get "$HARNESS_DIR/config/main.yml" paths.state)")"
}

# Sucesso se a raiz do projeto estiver num repositório git.
commits_has_git() {
  command -v git > /dev/null 2>&1 && _commits_git rev-parse --git-dir > /dev/null 2>&1
}

# Foto atual: uma linha "hash ref" por ref que aponta (direto ou por tag) para um commit, ordenada pela ref.
commits_snapshot() {
  {
    _commits_git rev-parse -q --verify 'HEAD^{commit}' 2>/dev/null | sed 's/$/ HEAD/' || true
    _commits_git for-each-ref --format='%(objecttype) %(objectname) %(*objecttype) %(*objectname) %(refname)' |
      awk '$NF ~ /^refs\/remotes\// || $NF == "refs/stash" { next }
           $1 == "commit" { print $2, $NF; next }
           $1 == "tag" && $3 == "commit" { print $4, $NF }'
  } | LC_ALL=C sort -k2,2
}

# Código curto que identifica a foto atual; a liberação exige o código mostrado na verificação.
commits_token() {
  commits_snapshot | sha256_hash | cut -c1-12
}

# Problemas do arquivo de referência, um por linha: linha fora do formato ou commit inexistente.
# commits_ack_errors <arquivo>
commits_ack_errors() {
  local line hash
  while IFS= read -r line; do
    [ -n "$line" ] || continue
    if ! [[ "$line" =~ ^([0-9a-f]{40}|[0-9a-f]{64})\ [^[:space:]]+$ ]]; then
      echo "linha fora do formato 'hash ref': $line"
      continue
    fi
    hash="${line%% *}"
    _commits_git cat-file -e "$hash^{commit}" 2>/dev/null || echo "commit liberado não existe mais no repositório: $line"
  done < "$1"
}

_commits_tips() {
  cut -d' ' -f1 | LC_ALL=C sort -u
}

# Commits alcançáveis pelas pontas de <incluir> e não pelas de <excluir>, do mais novo ao mais antigo.
# _commits_between <arquivo com as pontas a incluir> <arquivo com as pontas a excluir>
_commits_between() {
  local include exclude
  include="$(_commits_tips < "$1")"
  [ -n "$include" ] || return 0
  exclude="$(_commits_tips < "$2")"
  # shellcheck disable=SC2086
  _commits_git rev-list --topo-order $include --not $exclude
}

# Commits novos e removidos em relação à referência, uma linha cada: "novo <hash>" ou "removido <hash>".
# commits_changes <arquivo de referência>
commits_changes() {
  local current
  current="$(mktemp)"
  commits_snapshot > "$current"
  _commits_between "$current" "$1" | sed 's/^/novo /'
  _commits_between "$1" "$current" | sed 's/^/removido /'
  rm -f "$current"
}

# Descrição de um commit: hash curto, assunto, arquivos e se já está num ref remoto.
# commits_describe <hash>
commits_describe() {
  local files remote=""
  files="$(_commits_git log -1 --format= --name-only -m --first-parent "$1" | grep -v '^$' | paste -sd ',' - | sed 's/,/, /g')"
  [ -z "$(_commits_git branch -r --contains "$1" 2>/dev/null)" ] || remote=" [já enviado ao remoto]"
  printf -- '- %s %s (arquivos: %s)%s\n' "$(_commits_git log -1 --format=%h "$1")" \
    "$(_commits_git log -1 --format=%s "$1")" "${files:--}" "$remote"
}

# Relatório dos commits não liberados. Imprime nada se não houver nenhum.
# commits_report <arquivo de referência>
commits_report() {
  local changes kind hash
  changes="$(commits_changes "$1")"
  [ -n "$changes" ] || return 0
  if grep -q '^novo ' <<< "$changes"; then
    echo "Commits novos:"
    while read -r kind hash; do [ "$kind" = novo ] && commits_describe "$hash"; done <<< "$changes"
  fi
  if grep -q '^removido ' <<< "$changes"; then
    echo "Commits removidos (estavam liberados e saíram do histórico):"
    while read -r kind hash; do [ "$kind" = removido ] && commits_describe "$hash"; done <<< "$changes"
  fi
  return 0
}

# Grava a foto atual como liberada.
commits_store() {
  local file
  file="$(commits_ack_file)"
  mkdir -p "$(dirname "$file")"
  commits_snapshot > "$file.tmp"
  mv "$file.tmp" "$file"
}
