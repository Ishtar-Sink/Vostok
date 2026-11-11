#!/usr/bin/env bash
# Cria um projeto de exemplo com uma tarefa cujas evidências têm imagem, texto e vídeo,
# para os prints do visualizador, do botão de copiar e do player.
# Uso: bash apps/painel/scripts/fixture-midia.sh <pasta>
# Depois: PAINEL_PASTA=<pasta> PAINEL_BIN=<vostok> bash apps/painel/scripts/print.sh '/tarefas/SPEC-001/TASK-01#evidencias' <arquivo.png>

set -euo pipefail

TARGET="${1:?uso: fixture-midia.sh <pasta>}"
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
MEDIA="$HERE/fixtures/midia"

TASK="$TARGET/vostok/work/tasks/SPEC-001/TASK-01"
mkdir -p "$TARGET/vostok/work/specs/refined" "$TASK/evidencias"
cp -R "$ROOT/vostok/config" "$TARGET/vostok/config"

cat > "$TARGET/vostok/work/specs/refined/SPEC-001-exemplo-de-midia.md" <<'SPEC'
---
id: SPEC-001
title: "Exemplo de midia"
status: planned
author: "Exemplo"
created_at: 2026-11-01T10:00:00Z
updated_at: 2026-11-01T10:00:00Z
tags: []
type: improvement
---

# Exemplo de midia

Projeto de exemplo para os prints do painel.
SPEC

cat > "$TASK/estado.yml" <<'STATE'
id: TASK-01
spec: SPEC-001
title: "Evidências com mídia"
description: "Tarefa de exemplo com imagem, texto e vídeo nas evidências."
profile: padrao
persona: ui-ux-designer
depends_on: []
closes_after: []
wave: 1
covers: []
type: tarefa
labels: [SPEC-001]
status: executed
adjustments: 0
attempts: 0
tests_result: passed
regression_result: passed
evidence_result: passed
model: exemplo
STATE

printf '### Ler\n\n- `exemplo.txt`: arquivo de exemplo.\n\n### Mudar\n\n- `exemplo.txt` (alterar): exemplo.\n\n### Passos\n\n1. Exemplo.\n' > "$TASK/execucao.md"
printf '### Concluída quando\n\n- [x] A evidência mostra imagem, texto e vídeo.\n\n  > Como comprovar: abrir a aba Evidências.\n' > "$TASK/criterios.md"

cat > "$TASK/verificacao.md" <<'VERIFICACAO'
### Verificação

```bash
true
```

### Evidências

- `tela.png` (print): `true`
  - Título: Tela de exemplo
  - Prova: a imagem de exemplo aparece como miniatura e abre no visualizador.
  - Trecho: o quadro de teste colorido.
  - Falha: imagem ausente ou cortada.
- `log.txt` (automática): `true`
  - Título: Log de exemplo
  - Prova: o log traz o cabeçalho de quatro linhas e a saída do comando.
  - Trecho: a linha com `0 passed`, e não a de `10 passed`, e a linha `✓ ... esconde a raia`.
  - Falha: a palavra `FAILED` no log.
- `demo.mp4` (manual): vídeo de exemplo anexado.
  - Título: Vídeo de exemplo
  - Prova: o vídeo toca no player do painel.
  - Trecho: o vídeo de teste e a barra de controles.
  - Falha: controles nativos do navegador.
VERIFICACAO

cp "$MEDIA/demo.png" "$TASK/evidencias/tela.png"
cp "$MEDIA/demo.mp4" "$TASK/evidencias/demo.mp4"
printf '$ pnpm exec vitest run\n# data: 2026-11-11T12:00:00Z\n# commit: 5a43c5a\n# código de saída: 0\n\n Test Files  3 passed (3)\n      Tests  10 passed (10)\n  Snapshots  0 passed (0)\n✓ src/board.test.tsx > Board > esconde a raia desmarcada\n' > "$TASK/evidencias/log.txt"

echo "$TARGET"
