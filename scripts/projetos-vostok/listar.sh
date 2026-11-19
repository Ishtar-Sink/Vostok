#!/usr/bin/env bash
# Lista os projetos de projetos-vostok.yml com a versão do harness instalado e as pendências no git.
# Uso: scripts/projetos-vostok/listar.sh [nome...]
# Não muda nada. Código de saída 1 quando algum projeto tem erro.

set -u
. "$(dirname "$0")/comum.sh"
carregar_env

cli="$(versao_cli)"
[ -n "$cli" ] || { echo "vostok não encontrado no PATH" >&2; exit 1; }

linhas="$(listar_projetos)" || exit 1
falhas=0

echo "CLI no PATH: vostok $cli"
echo
echo "| Projeto | Harness | Situação | Pendências no git |"
echo "|---|---|---|---|"
while IFS='|' read -r nome caminho erro; do
  [ -n "$nome" ] || continue
  selecionado "$nome" "$@" || continue
  if [ -n "$erro" ]; then
    echo "| $nome | - | erro: $erro | - |"
    falhas=$((falhas + 1))
    continue
  fi
  versao="$(versao_harness "$caminho")"
  if [ "$versao" = "$cli" ]; then situacao="atualizado"; else situacao="desatualizado"; fi
  echo "| $nome | $versao | $situacao | $(pendencias_git "$caminho") |"
done <<EOF
$linhas
EOF

[ "$falhas" -eq 0 ]
