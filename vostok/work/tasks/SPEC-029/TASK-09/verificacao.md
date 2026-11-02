### Baseline

- `pnpm --dir apps/painel exec tsc --noEmit`

### Verificação

```bash
pnpm --dir apps/painel exec tsc --noEmit
# evidência: task-09-testes.txt
pnpm --dir apps/painel exec vitest run src/features/task src/features/documents
pnpm --dir apps/painel build
```

Resultado esperado: sem erro de tipos; os testes das telas passam; o build gera `dist/`.

### Evidências

- `task-09-testes.txt` (automática, da verificação)
  - Prova: as telas renderizam os dados do contrato, os estados de vazio e erro e as ações (RF-09, CT-03).
  - Trecho: os testes de `features/task`, `features/documents` com ✓ e o resumo sem falhas.
  - Falha: `FAIL` ou `×`.
- `tarefa.png` (print): `PAINEL_BIN=/home/victor/projetos/ai-hub-painel/apps/cli/target/release/vostok bash apps/painel/scripts/print.sh /tarefas/SPEC-027/TASK-02 "$VOSTOK_EVIDENCE/tarefa.png" 1280,1100`
  - Prova: uma tarefa com o conteúdo para pessoas primeiro e a parte do agente abaixo (RF-09, CT-03).
  - Trecho: executivo, técnico, critérios e evidências no topo e os metadados à direita.
  - Falha: página provisória, vazia ou com erro.
- `documentos.png` (print): `PAINEL_BIN=/home/victor/projetos/ai-hub-painel/apps/cli/target/release/vostok bash apps/painel/scripts/print.sh /documentos "$VOSTOK_EVIDENCE/documentos.png" 1280,1100`
  - Prova: os documentos das specs arquivadas (RF-09, CT-03).
  - Trecho: a lista de specs arquivadas com executivo, técnico e evidências.
  - Falha: página provisória, vazia ou com erro.
