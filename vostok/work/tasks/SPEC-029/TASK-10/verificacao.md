### Baseline

- `pnpm --dir apps/painel exec tsc --noEmit`

### Verificação

```bash
pnpm --dir apps/painel exec tsc --noEmit
# evidência: task-10-testes.txt
pnpm --dir apps/painel exec vitest run src/features/settings
pnpm --dir apps/painel build
```

Resultado esperado: sem erro de tipos; os testes das telas passam; o build gera `dist/`.

### Evidências

- `task-10-testes.txt` (automática, da verificação)
  - Prova: as telas renderizam os dados do contrato, os estados de vazio e erro e as ações (RF-10, CT-11).
  - Trecho: os testes de `features/settings` com ✓ e o resumo sem falhas.
  - Falha: `FAIL` ou `×`.
- `configuracoes.png` (print): `PAINEL_BIN=/home/victor/projetos/ai-hub-painel/apps/cli/target/release/vostok bash apps/painel/scripts/print.sh /configuracoes "$VOSTOK_EVIDENCE/configuracoes.png" 1280,1100`
  - Prova: as configurações com nomes da interface e controles por tipo (RF-10, CT-03).
  - Trecho: as seções por arquivo, os controles de seleção e de marcar e a barra de salvar.
  - Falha: página provisória, vazia ou com erro.
