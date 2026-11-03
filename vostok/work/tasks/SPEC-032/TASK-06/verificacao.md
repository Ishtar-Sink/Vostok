### Baseline

- `pnpm --dir apps/painel exec vitest run src/features/dashboard --reporter=verbose 2>&1 | grep -E '✓|✗|×' | sed 's/ [0-9]*ms$//' | sort | grep -v 'atividades por padrão\|seletor\|todas as atividades'`

### Verificação

```bash
# evidência: atividade-recente-testes.txt
pnpm --dir apps/painel exec vitest run src/features/dashboard --reporter=verbose
pnpm --dir apps/painel exec tsc --noEmit
```

### Evidências

- `atividade-recente-testes.txt` (automática, da verificação)
  - Prova: 5 atividades por padrão, seletor de 10 e 15 e "Ver todas" para `/atividades` (RF-05, CT-07, CB-02, RNF-02, RNF-03).
  - Trecho: os três testes novos com ✓.
  - Falha: algum com × ou erro de tipo.
- `atividade-recente.png` (print): `PAINEL_BIN=apps/cli/target/release/vostok bash apps/painel/scripts/print.sh / "$VOSTOK_EVIDENCE/atividade-recente.png" 1280,1100`
  - Prova: a atividade recente real, sem rolagem interna, com o seletor ao lado do título e o "Ver todas" (RF-05).
  - Trecho: cinco itens, o seletor e o link no fim.
  - Falha: barra de rolagem na seção, mais de cinco itens ou sem o link.
