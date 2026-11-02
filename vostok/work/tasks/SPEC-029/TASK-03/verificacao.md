### Baseline

- `ls apps` (muda): a pasta `painel` passa a existir

### Verificação

```bash
pnpm --dir apps/painel exec tsc --noEmit
# evidência: frontend-base.txt
pnpm --dir apps/painel exec vitest run src/shared src/ui
pnpm --dir apps/painel build
```

Resultado esperado: sem erro de tipos; os testes do roteador, das pendências e dos componentes passam; o build gera `dist/`.

### Evidências

- `frontend-base.txt` (automática, da verificação)
  - Prova: o roteador leva cada rota à página certa, as pendências geram as ações certas e os componentes renderizam (RF-03, CT-03).
  - Trecho: `router.test.ts`, `attention.test.ts` e os testes de `ui/` com ✓, e o resumo sem falhas.
  - Falha: `FAIL` ou `×` em algum teste.
- `base-claro.png` (print): `PAINEL_BIN=/home/victor/projetos/ai-hub-painel/apps/cli/target/release/vostok bash apps/painel/scripts/print.sh / "$VOSTOK_EVIDENCE/base-claro.png" 1280,900`
  - Prova: a base com a marca da pasta, o menu lateral, o topo e os tokens do Atria (RF-03, RF-04, RNF-05).
  - Trecho: o nome e o ícone da marca no menu, as specs em andamento e a página provisória do painel.
  - Falha: página em branco, sem marca ou sem menu.
  - Requer: `apps/painel/scripts/print.sh`, `apps/painel/package.json`
