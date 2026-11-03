### Baseline

- `pnpm --dir apps/painel exec vitest run src/features/board src/shared --reporter=verbose 2>&1 | grep -E '✓|✗|×' | sed 's/ [0-9]*ms$//' | sort | grep -v 'em validação'`

### Verificação

```bash
# evidência: quadro-testes.txt
pnpm --dir apps/painel exec vitest run src/features/board src/shared --reporter=verbose
pnpm --dir apps/painel exec tsc --noEmit
```

### Evidências

- `quadro-testes.txt` (automática, da verificação)
  - Prova: o quadro separa as tarefas `validating` numa coluna própria (RF-02, CT-02).
  - Trecho: o teste `separa em validação de em execução` com ✓.
  - Falha: o teste com × ou erro do `tsc`.
- `quadro.png` (print): `PAINEL_BIN=apps/cli/target/release/vostok bash apps/painel/scripts/print.sh /specs/SPEC-032/quadro "$VOSTOK_EVIDENCE/quadro.png" 1440,900`
  - Prova: a coluna "Em validação" aparece no quadro real, entre "Em execução" e "Bloqueadas" (RF-02).
  - Trecho: o título "Em validação" na raia.
  - Falha: só cinco colunas ou a coluna fora de ordem.
