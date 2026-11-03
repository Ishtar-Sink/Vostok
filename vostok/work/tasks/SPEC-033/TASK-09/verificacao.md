### Baseline

- `pnpm --dir apps/painel exec vitest run src/features/specs src/features/spec src/shared/attention.test.ts --reporter=verbose 2>&1 | grep -E '✓|✗|×' | sed 's/ [0-9]*ms$//' | sort | grep -v 'nova spec\|refin\|onda'`
- `pnpm --dir apps/painel exec tsc --noEmit`

### Verificação

```bash
# evidência: botoes-spec-testes.txt
pnpm --dir apps/painel exec vitest run src/features/specs src/features/spec src/shared/attention.test.ts --reporter=verbose
pnpm --dir apps/painel exec tsc --noEmit
```

### Evidências

- `botoes-spec-testes.txt` (automática, da verificação)
  - Prova: "Nova spec" grava `criar-spec` com o assunto, "Refinar" grava `refinar-spec` só no rascunho e "Executar onda" grava `executar-onda` com a próxima onda executável, todos achados por papel e rótulo (RF-05, CT-05, CT-06, CT-18).
  - Trecho: `grava o pedido de nova spec com o assunto`, `mostra a recusa do pedido de nova spec`, `pede o refinamento da spec em rascunho`, `pede a execução da próxima onda`, `não oferece refinar fora do rascunho`, `oferece refinar para a spec em rascunho`, `oferece executar a próxima onda executável` e `não oferece executar onda sem tarefa pronta` com ✓.
  - Falha: algum desses com ×, um teste antigo quebrado ou erro do `tsc`.
- `nova-spec.png` (print): `PAINEL_BIN=apps/cli/target/release/vostok bash apps/painel/scripts/print.sh /specs "$VOSTOK_EVIDENCE/nova-spec.png" 1280,900`
  - Prova: o botão "Nova spec" na página real das specs, no estilo dos botões do painel (RF-05, CT-05).
  - Trecho: o botão primário "Nova spec" à direita do título "Especificações", acima do filtro Abertas, Arquivadas e Todas.
  - Falha: sem o botão, botão fora do cabeçalho ou com estilo diferente dos botões do painel.
