#!/usr/bin/env bash
# Tira o print de uma rota do painel: sobe a API de PAINEL_BIN na raiz do projeto (ou em PAINEL_PASTA)
# e o vite do frontend, e fotografa com o Playwright (scripts/print.mjs).
# Uso: [PAINEL_PASTA=<pasta do projeto>] PAINEL_BIN=<caminho do vostok> bash apps/painel/scripts/print.sh <rota> <arquivo.png> [largura,altura]
# A rota aceita âncora (#evidencias). PAINEL_ROLAR (pixels ou "fim") rola antes do print;
# PAINEL_STORAGE (JSON) grava o localStorage antes de abrir a página.

set -euo pipefail

ROUTE="${1:?uso: print.sh <rota> <arquivo.png> [largura,altura]}"
OUTPUT="${2:?uso: print.sh <rota> <arquivo.png> [largura,altura]}"
SIZE="${3:-1280,900}"
BIN="${PAINEL_BIN:-vostok}"

PAINEL_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ROOT="$(cd "$PAINEL_DIR/../.." && pwd)"
WORK="$(mktemp -d)"
PIDS=()

cleanup() {
  for pid in "${PIDS[@]:-}"; do
    [ -n "$pid" ] && kill "$pid" 2>/dev/null || true
  done
  rm -rf "$WORK"
}
trap cleanup EXIT

free_port() {
  python3 -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()'
}

wait_for() {
  local url="$1" tries=100
  until curl -fs -o /dev/null "$url"; do
    tries=$((tries - 1))
    [ "$tries" -gt 0 ] || { echo "sem resposta de $url" >&2; exit 1; }
    sleep 0.2
  done
}

API_PORT="$(free_port)"
(cd "$ROOT" && exec "$BIN" painel --sem-navegador --porta "$API_PORT" --pasta "${PAINEL_PASTA:-$ROOT}") >"$WORK/api.log" 2>&1 &
PIDS+=("$!")
wait_for "http://127.0.0.1:$API_PORT/api/marca"

WEB_PORT="$(free_port)"
(cd "$PAINEL_DIR" && PAINEL_API="http://127.0.0.1:$API_PORT" exec pnpm exec vite --port "$WEB_PORT" --strictPort) >"$WORK/vite.log" 2>&1 &
PIDS+=("$!")
wait_for "http://127.0.0.1:$WEB_PORT/"

mkdir -p "$(dirname "$OUTPUT")"
(cd "$PAINEL_DIR" && node scripts/print.mjs "http://127.0.0.1:$WEB_PORT$ROUTE" "$OUTPUT" "$SIZE") >"$WORK/print.log" 2>&1 \
  || { cat "$WORK/print.log" >&2; exit 1; }

[ -s "$OUTPUT" ] || { cat "$WORK/print.log" >&2; exit 1; }
echo "$OUTPUT"
