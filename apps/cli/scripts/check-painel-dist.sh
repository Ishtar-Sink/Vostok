#!/usr/bin/env bash
# Confere o build.rs do vostok com o bundle do painel (SPEC-034, RF-08): numa cópia isolada do
# repositório, sem rede e com diretório alvo temporário, roda os cenários CT-11 (dist em dia),
# CT-10 (dist velho sem pnpm), CB-05 (sem a feature e sem dist) e a reconstrução automática.
# Uso: bash apps/cli/scripts/check-painel-dist.sh

set -euo pipefail

root="$(cd "$(dirname "$0")/../../.." && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
export CARGO_TARGET_DIR="$work/target"

mkdir -p "$work/apps" "$work/packages"
rsync -a --exclude target "$root/apps/cli" "$work/apps/"
rsync -a --exclude node_modules --exclude dist "$root/apps/painel" "$work/apps/"
rsync -a "$root/packages/harness" "$work/packages/"

painel="$work/apps/painel"
mkdir -p "$painel/dist"
printf '<!doctype html><title>painel</title>\n' >"$painel/dist/index.html"

nopnpm="$(dirname "$(command -v cargo)"):$(dirname "$(command -v rustc)"):/usr/bin:/bin"
if env PATH="$nopnpm" sh -c 'command -v pnpm' >/dev/null; then
  echo "pnpm continua no PATH sem pnpm"
  exit 1
fi

old() {
  find "$@" -exec touch -d '2001-01-01 00:00:00' {} +
}

build() {
  local path="$1"
  shift
  env PATH="$path" cargo build --offline --manifest-path "$work/apps/cli/Cargo.toml" "$@" >"$work/out" 2>&1
}

pass() {
  echo "ok $1: $2"
}

fail() {
  echo "FALHOU $1: $2"
  tail -n 30 "$work/out"
  exit 1
}

scenario="CT-11 dist em dia"
old "$painel/src" "$painel/index.html" "$painel/package.json"
touch "$painel/dist/index.html"
code=0
build "$nopnpm" --features painel || code=$?
[ "$code" -eq 0 ] || fail "$scenario" "o build terminou com $code"
pass "$scenario" "o build terminou sem erro e sem pnpm"

scenario="CT-10 dist velho sem pnpm"
old "$painel/dist/index.html"
touch "$painel/src/main.tsx"
code=0
build "$nopnpm" --features painel || code=$?
[ "$code" -ne 0 ] || fail "$scenario" "o build aceitou o dist velho"
grep -qF 'pnpm --dir apps/painel build' "$work/out" || fail "$scenario" "a mensagem não cita pnpm --dir apps/painel build"
pass "$scenario" "o build terminou com $code"
grep -F -m1 'pnpm --dir apps/painel build' "$work/out"

scenario="CB-05 sem a feature"
code=0
build "$nopnpm" || code=$?
[ "$code" -eq 0 ] || fail "$scenario" "o build terminou com $code"
pass "$scenario" "o build sem a feature ignorou o dist velho"

scenario="Reconstrução com pnpm"
[ -d "$root/apps/painel/node_modules" ] || fail "$scenario" "sem node_modules; rode pnpm --dir apps/painel install"
cp -a "$root/apps/painel/node_modules" "$painel/"
export npm_config_offline=true
code=0
build "$PATH" --features painel || code=$?
[ "$code" -eq 0 ] || fail "$scenario" "o build terminou com $code"
[ "$painel/dist/index.html" -nt "$painel/src/main.tsx" ] || fail "$scenario" "o dist não foi reconstruído"
pass "$scenario" "o build reconstruiu o dist"

scenario="CB-05 sem dist"
rm -rf "$painel/dist" "$painel/node_modules"
# O dist não é vigiado pelo build.rs; o toque faz o build.rs rodar de novo, como num clone novo.
touch "$painel/src/main.tsx"
code=0
build "$nopnpm" --features painel || code=$?
[ "$code" -eq 0 ] || fail "$scenario" "o build terminou com $code"
grep -qF 'painel sem bundle' "$work/out" || fail "$scenario" "sem o aviso painel sem bundle"
pass "$scenario" "o build seguiu com o aviso painel sem bundle"

echo "todos os cenários ok"
