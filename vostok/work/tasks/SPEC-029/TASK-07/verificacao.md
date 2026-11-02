### Baseline

- `pnpm --dir apps/painel exec tsc --noEmit`

### Verificação

```bash
pnpm --dir apps/painel exec tsc --noEmit
# evidência: task-07-testes.txt
pnpm --dir apps/painel exec vitest run src/features/dashboard src/features/inbox
pnpm --dir apps/painel build
```

Resultado esperado: sem erro de tipos; os testes das telas passam; o build gera `dist/`.

### Evidências

- `task-07-testes.txt` (automática, da verificação)
  - Prova: as telas renderizam os dados do contrato, os estados de vazio e erro e as ações (RF-06, RF-11, CT-03).
  - Trecho: os testes de `features/dashboard`, `features/inbox` com ✓ e o resumo sem falhas.
  - Falha: `FAIL` ou `×`.
- `painel.png` (print): `PAINEL_BIN=/home/victor/projetos/ai-hub-painel/apps/cli/target/release/vostok bash apps/painel/scripts/print.sh / "$VOSTOK_EVIDENCE/painel.png" 1280,1100`
  - Prova: o painel da pasta com os indicadores e as specs em andamento (RF-06, RF-11, CT-03).
  - Trecho: os indicadores, "Aguardando você" e a barra por status das specs em andamento.
  - Falha: página provisória, vazia ou com erro.
- `pendencias.png` (print): `PAINEL_BIN=/home/victor/projetos/ai-hub-painel/apps/cli/target/release/vostok bash apps/painel/scripts/print.sh /pendencias "$VOSTOK_EVIDENCE/pendencias.png" 1280,1100`
  - Prova: as pendências com as ações de aprovar e pedir ao agente (RF-06, RF-11, CT-03).
  - Trecho: a lista de pendências com os botões de ação.
  - Falha: página provisória, vazia ou com erro.
