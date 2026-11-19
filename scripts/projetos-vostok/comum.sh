#!/usr/bin/env bash
# Funções comuns dos scripts de projetos-vostok. Compatível com bash 3.2.

RAIZ="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ARQUIVO_PROJETOS="$RAIZ/projetos-vostok.yml"
ARQUIVO_ENV="$RAIZ/.env"

carregar_env() {
  if [ -f "$ARQUIVO_ENV" ]; then
    set -a
    # shellcheck disable=SC1090
    . "$ARQUIVO_ENV"
    set +a
  fi
}

# Imprime "nome|tipo|caminho" por item de projetos-vostok.yml, com o caminho como está no arquivo.
ler_itens() {
  if [ ! -f "$ARQUIVO_PROJETOS" ]; then
    echo "lista de projetos não encontrada: copie projetos-vostok.example.yml para projetos-vostok.yml e preencha" >&2
    return 1
  fi
  awk '
    function limpar(valor) {
      sub(/^[^:]*:[ \t]*/, "", valor)
      sub(/[ \t]+#.*$/, "", valor)
      gsub(/^["\x27]|["\x27]$/, "", valor)
      return valor
    }
    function emitir() { if (nome != "") print nome "|" tipo "|" caminho }
    /^[ \t]*-[ \t]*nome:/ { emitir(); nome = limpar($0); tipo = "projeto"; caminho = ""; next }
    /^[ \t]+tipo:/ { tipo = limpar($0); next }
    /^[ \t]+caminho:/ { caminho = limpar($0); next }
    END { emitir() }
  ' "$ARQUIVO_PROJETOS"
}

# Resolve o caminho de um item: ${VAR} pelo ambiente, relativo pela raiz deste repositório.
# Em erro, imprime o motivo em stderr e retorna 1.
resolver_caminho() {
  local bruto="$1" variavel resto valor
  case "$bruto" in
    '${'*'}'*)
      variavel="${bruto#\$\{}"
      variavel="${variavel%%\}*}"
      resto="${bruto#*\}}"
      valor="${!variavel}"
      if [ -z "$valor" ]; then
        echo "variável $variavel não definida no .env" >&2
        return 1
      fi
      bruto="$valor$resto"
      ;;
    '') echo "caminho vazio" >&2; return 1 ;;
  esac
  case "$bruto" in
    /*) ;;
    "~"*) bruto="$HOME${bruto#\~}" ;;
    *) bruto="$RAIZ/$bruto" ;;
  esac
  if [ ! -d "$bruto" ]; then
    echo "pasta não encontrada" >&2
    return 1
  fi
  (cd "$bruto" && pwd)
}

# Imprime "nome|caminho_absoluto|erro" por projeto, com as pastas já expandidas nos projetos de dentro.
listar_projetos() {
  local itens nome tipo bruto caminho erro sub
  itens="$(ler_itens)" || return 1
  while IFS='|' read -r nome tipo bruto; do
    [ -n "$nome" ] || continue
    erro=""
    caminho="$(resolver_caminho "$bruto" 2>&1)" || { erro="$caminho"; caminho=""; }
    if [ -n "$erro" ]; then
      echo "$nome||$erro"
      continue
    fi
    case "$tipo" in
      projeto)
        if [ -d "$caminho/vostok" ]; then
          echo "$nome|$caminho|"
        else
          echo "$nome|$caminho|sem a pasta vostok/"
        fi
        ;;
      pasta)
        local achou=""
        for sub in "$caminho"/*/; do
          [ -f "${sub}vostok/.manifest" ] || continue
          sub="${sub%/}"
          echo "$nome/$(basename "$sub")|$sub|"
          achou=1
        done
        [ -n "$achou" ] || echo "$nome|$caminho|nenhuma subpasta com vostok/"
        ;;
      *) echo "$nome||tipo inválido: '$tipo'; use projeto ou pasta" ;;
    esac
  done <<EOF
$itens
EOF
}

versao_harness() {
  local manifesto="$1/vostok/.manifest"
  [ -f "$manifesto" ] || { echo "-"; return; }
  awk '/^version:/ { print $2; exit }' "$manifesto"
}

pendencias_git() {
  local dir="$1"
  if git -C "$dir" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    git -C "$dir" status --porcelain -- vostok AGENTS.md CLAUDE.md .claude .codex .cursor 2>/dev/null | wc -l | tr -d ' '
  else
    echo "-"
  fi
}

versao_cli() {
  vostok --version 2>/dev/null | awk '{ print $2 }'
}

# Verdadeiro quando a lista de filtros está vazia ou contém o nome (ou o grupo antes da barra).
selecionado() {
  local nome="$1"; shift
  [ "$#" -eq 0 ] && return 0
  local filtro
  for filtro in "$@"; do
    [ "$filtro" = "$nome" ] && return 0
    [ "$filtro" = "${nome%%/*}" ] && return 0
  done
  return 1
}
