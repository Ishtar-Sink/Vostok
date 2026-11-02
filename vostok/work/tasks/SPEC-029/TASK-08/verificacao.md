### Baseline

- `pnpm --dir apps/painel exec tsc --noEmit`

### Verificação

```bash
pnpm --dir apps/painel exec tsc --noEmit
# evidência: task-08-testes.txt
pnpm --dir apps/painel exec vitest run src/features/specs src/features/spec src/features/board
pnpm --dir apps/painel build
```

Resultado esperado: sem erro de tipos; os testes das telas passam; o build gera `dist/`.

### Evidências

- `task-08-testes.txt` (automática, da verificação)
  - Prova: as telas renderizam os dados do contrato, os estados de vazio e erro e as ações (RF-09, CT-03, CB-07).
  - Trecho: os testes de `features/specs`, `features/spec`, `features/board` com ✓ e o resumo sem falhas.
  - Falha: `FAIL` ou `×`.
- `specs.png` (print): `PAINEL_BIN=/home/victor/projetos/ai-hub-painel/apps/cli/target/release/vostok bash apps/painel/scripts/print.sh /specs "$VOSTOK_EVIDENCE/specs.png" 1280,1100`
  - Prova: a lista de specs com tipo, progresso e status (RF-09, CT-03).
  - Trecho: a tabela com as specs abertas.
  - Falha: página provisória, vazia ou com erro.
- `spec.png` (print): `PAINEL_BIN=/home/victor/projetos/ai-hub-painel/apps/cli/target/release/vostok bash apps/painel/scripts/print.sh /specs/SPEC-029 "$VOSTOK_EVIDENCE/spec.png" 1280,1100`
  - Prova: uma spec com índice, metadados, tarefas e ações (RF-09, CT-03).
  - Trecho: o índice à esquerda, as tarefas à direita e as ações no topo.
  - Falha: página provisória, vazia ou com erro.
- `quadro.png` (print): `PAINEL_BIN=/home/victor/projetos/ai-hub-painel/apps/cli/target/release/vostok bash apps/painel/scripts/print.sh /specs/SPEC-029/quadro "$VOSTOK_EVIDENCE/quadro.png" 1280,1100`
  - Prova: o quadro kanban da spec por status, com onda e dependências (RF-09, CT-03).
  - Trecho: as colunas por status com os cartões das tarefas.
  - Falha: página provisória, vazia ou com erro.
