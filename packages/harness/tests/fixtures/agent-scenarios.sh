#!/usr/bin/env bash
# Monta as cópias descartáveis dos cenários com agente de tests/agent-scenarios.md.
# Cada cenário recebe uma cópia do projeto (AGENTS.md, CLAUDE.md e packages/harness) com:
#   SPEC-001 Exportar memória em JSON (refinada), SPEC-002 Listar tarefas (refinada)
#   e SPEC-003 Painel de métricas (rascunho). Alguns cenários removem specs (ver `prune`).
#
# Uso: tests/fixtures/agent-scenarios.sh <pasta de destino>
#      Cria <destino>/<cenário> para cada cenário e imprime os caminhos.
set -euo pipefail

[ -n "${1:-}" ] || { echo "Uso: agent-scenarios.sh <pasta de destino>" >&2; exit 1; }
SOURCE_HARNESS="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SOURCE_ROOT="$(cd "$SOURCE_HARNESS/../.." && pwd)"
DEST="$(mkdir -p "$1" && cd "$1" && pwd)"
B="$DEST/base"

rm -rf "$B" && mkdir -p "$B/packages"
cp "$SOURCE_ROOT/AGENTS.md" "$SOURCE_ROOT/CLAUDE.md" "$B/"
cp -R "$SOURCE_HARNESS" "$B/packages/"
H="$B/packages/harness"
rm -rf "$H/work" "$H"/memory/*.md
for key in specs decisions issues knowledge; do
  sed -i.bak "s/^$key: .*/$key: 1/" "$H/config/sequences.yml"
done
rm -f "$H/config/sequences.yml.bak"
SP="$H/scripts/default/specs"
cd /

F() { python3 "$SOURCE_HARNESS/tests/fixtures/fill-refined.py" "$@"; }
"$SP"/create-new-spec.sh 'Exportar memória em JSON' >/dev/null; "$SP"/create-refined-spec.sh SPEC-001 feature >/dev/null; f="$(ls "$H"/work/specs/refined/SPEC-001-*.md)"
F "$f" 'SPEC_SUMMARY=Cria o script `export-memory.sh`, que exporta as entradas da memória do agente em JSON.' \
 'OBJECTIVE=Hoje a memória do agente só pode ser lida em Markdown. O painel web precisa dela em JSON para exibir decisões, issues e conhecimentos.' \
 'SCOPE_IN=- Script `export-memory.sh` em `packages/harness/scripts/default/memory/`.\n- Exportação por tipo e de todos os tipos.' \
 'SCOPE_OUT=- Importação de JSON para a memória.\n- Envio do JSON ao servidor HTTP.' \
 'FUNCTIONAL_REQUIREMENTS=| RF-01 | O script deve exportar as entradas de um tipo de memória em JSON, com ID, título, status e tags. |\n| RF-02 | O script deve aceitar o tipo `all` para exportar todos os tipos num único arquivo. |\n| RF-03 | O script deve gravar a saída em `memory/export.json`. |' \
 'NON_FUNCTIONAL_REQUIREMENTS=| RNF-01 | Roda em bash 3.2 com awk e sed do BSD. | Suíte executada no macOS. |\n| RNF-02 | Usa `jq` para montar o JSON. | Revisão do script. |' \
 'BUSINESS_RULES=| RN-01 | Entradas com status `obsoleto` não são exportadas. |' \
 'MAIN_FLOW=1. O usuário executa `export-memory.sh <tipo>`.\n2. O script lê as entradas do tipo.\n3. O script grava o JSON em `memory/export.json`.' \
 'ALTERNATIVE_FLOWS=- **FA-01 (passo 1):** tipo inválido. O script termina com erro e lista os tipos.' \
 'EDGE_CASES=| CB-01 | Memória vazia | Grava `[]` | RF-01 |\n| CB-02 | Título com aspas | Aspas escapadas no JSON | RF-01 |' \
 'CONTRACTS=```bash\nexport-memory.sh <tipo>   # decision, issue, knowledge ou all\n```' \
 'AFFECTED_COMPONENTS=| `packages/harness/scripts/default/memory/export-memory.sh` | criar | Script de exportação. |' \
 'ASSUMPTIONS=- `jq` está instalado.' \
 'RISKS=| `jq` ausente na máquina | médio | Verificar no início e terminar com erro claro. |' \
 'TEST_CASES=| CT-01 | Exportar decisões | Executar com `decision` | JSON com as decisões | RF-01, RF-03 |\n| CT-02 | Exportar tudo | Executar com `all` | JSON com os três tipos | RF-02 |' \
 'ACCEPTANCE_CRITERIA=- [ ] CA-01: `export-memory.sh decision` gera JSON com as decisões ativas (RF-01, RF-03).\n- [ ] CA-02: `export-memory.sh all` gera um único arquivo com os três tipos (RF-02).' \
 'REFERENCES=| arquivo | `packages/harness/config/memory.yml` | Tipos e status da memória. |' \
 'WHAT_CHANGED=| 1 | Formato | Não definido | JSON com ID, título, status e tags |' \
 'DECISIONS=| 1 | Saída em arquivo fixo | Saída padrão | inferência A |' \
 'PENDING=| - | Nenhuma | - |'
"$SP"/create-refined-spec.sh --mark-refined SPEC-001 >/dev/null
"$SP"/create-new-spec.sh 'Listar tarefas' >/dev/null; "$SP"/create-refined-spec.sh SPEC-002 feature >/dev/null; g="$(ls "$H"/work/specs/refined/SPEC-002-*.md)"
F "$g" 'SPEC_SUMMARY=Cria o script `list-tasks.sh`, que lista as tarefas de uma spec.' 'OBJECTIVE=O agente precisa listar as tarefas geradas de uma spec para acompanhar a execução.' \
 'FUNCTIONAL_REQUIREMENTS=| RF-01 | O script deve listar as tarefas de uma spec em tabela, com ID, título e status. |\n| RF-02 | O script deve aceitar o ID da spec como argumento obrigatório. |' \
 'AFFECTED_COMPONENTS=| `packages/harness/scripts/default/tasks/list-tasks.sh` | criar | Script de listagem. |' \
 'TEST_CASES=| CT-01 | Listar | Executar com SPEC-001 | Tabela com as tarefas | RF-01, RF-02 |' 'ACCEPTANCE_CRITERIA=- [ ] CA-01: `list-tasks.sh SPEC-001` lista as tarefas (RF-01, RF-02).'
"$SP"/create-refined-spec.sh --mark-refined SPEC-002 >/dev/null
"$SP"/create-new-spec.sh 'Painel de métricas' >/dev/null

# Remove as specs informadas (rascunho e refinada) de um cenário.
prune() {
  local dir="$1" id
  shift
  for id in "$@"; do rm -f "$dir"/packages/harness/work/specs/*/"$id"-*.md; done
}

for scenario in ct01 ct02 ct03 ct04 ct05 ct06 ct09 ct13 ct14 ct15 ct16; do
  rm -rf "${DEST:?}/$scenario"
  cp -R "$B" "$DEST/$scenario"
  case "$scenario" in
    ct04|ct09) prune "$DEST/$scenario" SPEC-002 ;;
    ct05) prune "$DEST/$scenario" SPEC-001 SPEC-002 ;;
  esac
  echo "$DEST/$scenario"
done
rm -rf "$B"
