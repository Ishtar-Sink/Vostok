#!/usr/bin/env bash
# Uso: executar.sh <cenário> <sem|com> <repetições>
# Roda cada repetição do braço numa cópia limpa fora do repositório, com
# `claude -p`, e grava os resultados em <cenário>/resultados/<braço>/<n>/.
# Variáveis: FLUXO_MODELO (padrão sonnet), FLUXO_TEMPO_LIMITE (segundos por
# etapa, padrão 3600), FLUXO_COLETOR (binário do coletor) e CLAUDE_CONFIG_DIR.
set -euo pipefail

command -v claude >/dev/null || { echo "claude não encontrado no PATH: instale e autentique o Claude Code antes de rodar o benchmark." >&2; exit 2; }
command -v git >/dev/null || { echo "git não encontrado no PATH: instale e autentique o Claude Code antes de rodar o benchmark." >&2; exit 2; }
command -v uuidgen >/dev/null || { echo "uuidgen não encontrado no PATH: instale e autentique o Claude Code antes de rodar o benchmark." >&2; exit 2; }

RAIZ=$(cd "$(dirname "$0")/../.." && pwd)
FLUXO_MODELO="${FLUXO_MODELO:-sonnet}"
FLUXO_TEMPO_LIMITE="${FLUXO_TEMPO_LIMITE:-3600}"
FLUXO_COLETOR="${FLUXO_COLETOR:-}"

if [ $# -ne 3 ]; then
  echo "uso: executar.sh <cenário> <sem|com> <repetições>" >&2
  exit 2
fi
CENARIO_NOME="$1"
BRACO="$2"
REPETICOES="$3"

case "$BRACO" in
  sem|com) ;;
  *) echo "braço inválido: $BRACO (use sem ou com)" >&2; exit 2 ;;
esac
case "$REPETICOES" in
  ''|*[!0-9]*) echo "repetições inválidas: $REPETICOES (inteiro maior que zero)" >&2; exit 2 ;;
esac
if [ "$REPETICOES" -lt 1 ]; then
  echo "repetições inválidas: $REPETICOES (inteiro maior que zero)" >&2
  exit 2
fi

CENARIO="${FLUXO_BENCHMARKS:-$RAIZ/benchmarks}/$CENARIO_NOME"
for item in projeto-referencia gabarito.yml bracos.yml prompts.md; do
  if [ ! -e "$CENARIO/$item" ]; then
    echo "falta $item em $CENARIO" >&2
    exit 2
  fi
done

PROMPTS=""
C=""
B=""
limpar() {
  rm -rf "$PROMPTS" "$C" "$B"
}
trap limpar EXIT

# Imprime `vostok<TAB>caminho` e `config<TAB>grupo.chave<TAB>valor` do braço.
ler_braco() {
  awk -v braco="$BRACO" '
    /^[ \t]*(#|$)/ { next }
    /^[^ \t]/ { ativo = ($0 ~ "^" braco ":[ \t]*$"); emcfg = 0; next }
    !ativo { next }
    {
      linha = $0
      sub(/[ \t]+$/, "", linha)
      if (linha ~ /^  vostok:/) {
        v = substr(linha, index(linha, ":") + 1)
        sub(/^[ \t]+/, "", v)
        if (v != "") print "vostok\t" v
        emcfg = 0
      } else if (linha ~ /^  config:/) {
        emcfg = 1
      } else if (emcfg && linha ~ /^    [^ ]/) {
        i = index(linha, ":")
        k = substr(linha, 5, i - 5)
        v = substr(linha, i + 1)
        sub(/^[ \t]+/, "", v)
        print "config\t" k "\t" v
      }
    }' "$CENARIO/bracos.yml"
}

# Grava cada prompt de prompts.md em NN-<etapa>.txt na pasta $PROMPTS.
ler_prompts() {
  PROMPTS=$(mktemp -d "${TMPDIR:-/tmp}/fluxo-prompts-XXXXXX")
  awk -v dir="$PROMPTS" '
    function fechar(   i, ini, fim) {
      if (out == "") return
      ini = 1
      while (ini <= n && buf[ini] ~ /^[ \t]*$/) ini++
      fim = n
      while (fim >= ini && buf[fim] ~ /^[ \t]*$/) fim--
      printf "" > out
      for (i = ini; i <= fim; i++) print buf[i] >> out
      close(out)
    }
    /^## / {
      fechar()
      et = substr($0, 4)
      sub(/[ \t]+$/, "", et)
      idx++
      out = sprintf("%s/%02d-%s.txt", dir, idx, et)
      n = 0
      next
    }
    out != "" { buf[++n] = $0 }
    END { fechar() }' "$CENARIO/prompts.md"
  if ! ls "$PROMPTS"/*-executar.txt >/dev/null 2>&1; then
    echo "prompts.md sem a etapa executar" >&2
    exit 2
  fi
}

# aplicar_config <main.yml> <grupo.chave> <valor>
aplicar_config() {
  local arq="$1" chave="$2" valor="$3" grupo sub
  case "$chave" in
    *.*.*|.*|*.) echo "chave de config fora do formato grupo.chave: $chave" >&2; exit 2 ;;
    *.*) ;;
    *) echo "chave de config fora do formato grupo.chave: $chave" >&2; exit 2 ;;
  esac
  grupo="${chave%%.*}"
  sub="${chave#*.}"
  awk -v g="$grupo" -v k="$sub" -v v="$valor" '
    function ehgrupo(l) { sub(/[ \t]+$/, "", l); return l == g ":" }
    function ehchave(l) { return index(l, "  " k ":") == 1 }
    FNR == 1 && NR != 1 { ing = 0 }
    FNR == NR {
      if (ehgrupo($0)) { ing = 1; achou = 1 }
      else if ($0 ~ /^[^ \t#]/) ing = 0
      else if (ing && ehchave($0)) tem = 1
      next
    }
    {
      if (ehgrupo($0)) {
        ing = 1
        print
        if (!tem) print "  " k ": " v
        next
      }
      if ($0 ~ /^[^ \t#]/) ing = 0
      if (ing && ehchave($0)) { print "  " k ": " v; next }
      print
    }
    END { if (!achou) { print g ":"; print "  " k ": " v } }' "$arq" "$arq" > "$arq.fluxo-tmp"
  mv "$arq.fluxo-tmp" "$arq"
}

MOTIVO=""
# rodar_etapa <NN-etapa.txt> dentro da cópia $C; preenche MOTIVO.
rodar_etapa() {
  local arq="$1" base etapa saida pid inicio codigo=0 estourou=0
  base=$(basename "$arq" .txt)
  etapa="${base#*-}"
  saida="$R/saida-$base"
  MOTIVO=""
  (
    cd "$C"
    exec claude -p "$(cat "$arq")" --output-format json \
      --permission-mode bypassPermissions --model "$FLUXO_MODELO" \
      --session-id "$(uuidgen | tr 'A-Z' 'a-z')" --setting-sources project,local \
      > "$saida.json" 2> "$saida.err"
  ) &
  pid=$!
  inicio=$SECONDS
  while kill -0 "$pid" 2>/dev/null; do
    if [ $((SECONDS - inicio)) -ge "$FLUXO_TEMPO_LIMITE" ]; then
      estourou=1
      kill "$pid" 2>/dev/null || true
      break
    fi
    sleep 1
  done
  wait "$pid" || codigo=$?
  if [ "$estourou" -eq 1 ]; then
    MOTIVO="tempo-limite de $FLUXO_TEMPO_LIMITE s na etapa $etapa"
  elif [ "$codigo" -ne 0 ]; then
    MOTIVO="código $codigo na etapa $etapa"
  elif grep -Eq '"is_error": ?true' "$saida.json" 2>/dev/null; then
    MOTIVO="is_error na etapa $etapa"
  fi
}

copiar_tarefas() {
  if [ -d "$C/vostok/work/tasks" ]; then
    cp -R "$C/vostok/work/tasks" "$1"
  else
    mkdir -p "$1"
  fi
}

commitar() {
  git -C "$C" add -A
  git -C "$C" -c user.name=fluxo -c user.email=fluxo@exemplo.invalid commit -q --allow-empty -m "$1"
}

coletar() {
  if [ -n "$FLUXO_COLETOR" ]; then
    "$FLUXO_COLETOR" coletar "$1"
  else
    cargo run --quiet --manifest-path "$RAIZ/benchmarks/fluxo/Cargo.toml" -- coletar "$1"
  fi
}

# rodar_repeticao <n>
rodar_repeticao() {
  local n="$1" binario="" configs="" tipo a b arq falha="" PATH_ORIGINAL="$PATH" dir_transc

  R="$CENARIO/resultados/$BRACO/$n"
  rm -rf "$R"
  mkdir -p "$R"

  C=$(mktemp -d "${TMPDIR:-/tmp}/fluxo-XXXXXX")
  cp -R "$CENARIO/projeto-referencia/." "$C/"
  C=$(cd "$C" && pwd -P)
  git -C "$C" init -q
  commitar referencia

  while IFS=$'\t' read -r tipo a b; do
    case "$tipo" in
      vostok) binario="$a" ;;
      config) configs="$configs$a"$'\t'"$b"$'\n' ;;
    esac
  done < <(ler_braco)

  if [ -n "$binario" ]; then
    case "$binario" in
      /*) ;;
      *) binario="$RAIZ/$binario" ;;
    esac
    if [ ! -e "$binario" ]; then
      echo "binário do braço não encontrado: $binario" >&2
      exit 2
    fi
    B=$(mktemp -d "${TMPDIR:-/tmp}/fluxo-bin-XXXXXX")
    ln -s "$binario" "$B/vostok"
    PATH="$B:$PATH"
    (cd "$C" && vostok update) > "$R/update.log" 2>&1 || true
    commitar "harness do braço"
  fi

  if [ -n "$configs" ]; then
    while IFS=$'\t' read -r a b; do
      [ -n "$a" ] || continue
      aplicar_config "$C/vostok/config/main.yml" "$a" "$b"
    done <<EOF_CFG
$configs
EOF_CFG
    commitar "configuração do braço"
  fi

  copiar_tarefas "$R/tarefas-inicio"

  for arq in "$PROMPTS"/*.txt; do
    case "$arq" in
      *-executar.txt) copiar_tarefas "$R/tarefas-antes" ;;
    esac
    rodar_etapa "$arq"
    if [ -n "$MOTIVO" ]; then
      falha="$MOTIVO"
      break
    fi
  done

  [ -n "$falha" ] || copiar_tarefas "$R/tarefas-fim"
  [ -d "$R/tarefas-antes" ] && [ -n "$falha" ] && rm -rf "$R/tarefas-antes"
  dir_transc="${CLAUDE_CONFIG_DIR:-$HOME/.claude}/projects/$(printf %s "$C" | sed 's/[^A-Za-z0-9]/-/g')"
  if [ -d "$dir_transc" ]; then
    mkdir -p "$R/transcricoes"
    cp -R "$dir_transc/." "$R/transcricoes/"
  elif [ -z "$falha" ]; then
    falha="sem transcrições"
  fi

  {
    echo "braco=$BRACO"
    echo "modelo=$FLUXO_MODELO"
    if [ -n "$falha" ]; then echo "falhou=true"; else echo "falhou=false"; fi
    echo "motivo=$falha"
  } > "$R/execucao.txt"
  coletar "$R" >/dev/null 2>"$R/coletor.err" || true

  rm -rf "$C"
  [ -z "$B" ] || rm -rf "$B"
  C=""
  B=""
  PATH="$PATH_ORIGINAL"

  if [ -n "$falha" ]; then
    echo "repetição $n de $REPETICOES: falhou ($falha)"
  else
    local m="$R/metricas.json" tk tp ac es
    tk=$(sed -n 's/^ *"total": *\([0-9]*\).*/\1/p' "$m" 2>/dev/null | head -1)
    tp=$(sed -n 's/^ *"tempo_s": *\([0-9]*\).*/\1/p' "$m" 2>/dev/null | head -1)
    ac=$(sed -n 's/^ *"achados_antes": *\([0-9]*\).*/\1/p' "$m" 2>/dev/null | head -1)
    es=$(sed -n 's/^ *"escapes": *\([0-9]*\).*/\1/p' "$m" 2>/dev/null | head -1)
    echo "repetição $n de $REPETICOES: ok (tokens=$tk, tempo_s=$tp, achados_antes=$ac, escapes=$es)"
  fi
}

ler_prompts
# Valida o braço antes de rodar qualquer repetição.
ler_braco >/dev/null

i=1
while [ "$i" -le "$REPETICOES" ]; do
  rodar_repeticao "$i"
  i=$((i + 1))
done
echo "Resultados em $CENARIO_NOME/resultados/$BRACO"
