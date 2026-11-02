### Baseline

- `pnpm --dir apps/painel exec vitest run src/shared --reporter=verbose 2>&1 | grep -E '✓|✗|×' | sed 's/ [0-9]*ms$//' | sort`

### Verificação

```bash
pnpm --dir apps/painel exec tsc --noEmit
# evidência: pedir-commit-testes.txt
pnpm --dir apps/painel exec vitest run src/features/inbox src/shared --reporter=verbose
pnpm --dir apps/painel build
```

Resultado esperado: tipos, testes e build passam.

### Evidências

- `pedir-commit-testes.txt` (automática, da verificação)
  - Prova: o botão aparece e grava o pedido de commit com o texto (RF-11, CT-12).
  - Trecho: `mostra o botão Pedir commit` e `grava o pedido de commit com o texto` com `✓`.
  - Falha: `✗`, `×` ou `failed`.
- `pedir-commit.png` (print): `PAINEL_BIN=apps/cli/target/release/vostok bash apps/painel/scripts/print.sh /pendencias "$VOSTOK_EVIDENCE/pedir-commit.png" 1280,900`
  - Prova: a tela de pendências com o botão "Pedir commit" no cabeçalho, na marca da pasta (RF-11).
  - Trecho: o botão "Pedir commit" ao lado do título "Pendências".
  - Falha: tela sem o botão ou com erro de API.
