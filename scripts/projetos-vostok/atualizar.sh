#!/usr/bin/env bash
# Roda `vostok update` em cada projeto de projetos-vostok.yml, ou só nos nomes informados.
# Uso: scripts/projetos-vostok/atualizar.sh [nome...]
# Um nome de pasta (ex.: exemplos) vale para todos os projetos dela.
# Não faz commit, stage nem push. Código de saída 1 quando algum projeto falha.

set -u
. "$(dirname "$0")/comum.sh"
carregar_env

cli="$(versao_cli)"
[ -n "$cli" ] || { echo "vostok não encontrado no PATH" >&2; exit 1; }

linhas="$(listar_projetos)" || exit 1
falhas=0
detalhes=""

echo "CLI no PATH: vostok $cli"
echo
echo "| Projeto | Antes | Depois | Resultado | Pendências no git |"
echo "|---|---|---|---|---|"
while IFS='|' read -r nome caminho erro; do
  [ -n "$nome" ] || continue
  selecionado "$nome" "$@" || continue
  if [ -n "$erro" ]; then
    echo "| $nome | - | - | erro: $erro | - |"
    falhas=$((falhas + 1))
    continue
  fi
  antes="$(versao_harness "$caminho")"
  saida="$(cd "$caminho" && vostok update 2>&1 </dev/null)"
  codigo=$?
  depois="$(versao_harness "$caminho")"
  if [ "$codigo" -ne 0 ]; then
    resultado="erro (código $codigo)"
    falhas=$((falhas + 1))
  elif [ "$antes" = "$depois" ]; then
    resultado="já atualizado"
  else
    resultado="atualizado"
  fi
  echo "| $nome | $antes | $depois | $resultado | $(pendencias_git "$caminho") |"
  acrescimos="$(printf '%s\n' "$saida" | grep -E '^(Configuração acrescentada|Erro|erro|Aviso)' || true)"
  if [ "$codigo" -ne 0 ]; then
    acrescimos="$(printf '%s\n' "$saida" | tail -5)"
  fi
  if [ -n "$acrescimos" ]; then
    detalhes="$detalhes
### $nome
$acrescimos
"
  fi
done <<EOF
$linhas
EOF

if [ -n "$detalhes" ]; then
  echo
  echo "## Detalhes"
  printf '%s\n' "$detalhes" | sed "s#$HOME#~#g"
fi

[ "$falhas" -eq 0 ]
