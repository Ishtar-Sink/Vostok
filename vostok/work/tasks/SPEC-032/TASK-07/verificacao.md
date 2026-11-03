### Baseline

- `pnpm --dir apps/painel exec vitest run src/shared/router.test.ts src/layout --reporter=verbose 2>&1 | grep -E '✓|✗|×' | sed 's/ [0-9]*ms$//' | sort | grep -v 'atividades'`

### Verificação

```bash
# evidência: atividades-pagina-testes.txt
pnpm --dir apps/painel exec vitest run src/features/activity src/shared/router.test.ts src/shared/api.test.ts --reporter=verbose
pnpm --dir apps/painel exec tsc --noEmit
cargo test --manifest-path apps/cli/Cargo.toml --test task --test finalize
cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::
```

### Evidências

- `atividades-pagina-testes.txt` (automática, da verificação)
  - Prova: a rota existe e a página lista e filtra por tipo e por spec, com estado vazio (RF-06, CT-08, CB-05, RNF-03).
  - Trecho: os quatro testes da página, o da rota e os do `formatDateTime` com ✓, e `record_has_the_time ... ok`.
  - Falha: algum com × ou erro de tipo.
  - Requer: `apps/painel/src/features/activity/activity-page.test.tsx`
- `atividades.png` (print): `PAINEL_BIN=apps/cli/target/release/vostok bash apps/painel/scripts/print.sh /atividades "$VOSTOK_EVIDENCE/atividades.png" 1280,1100`
  - Requer: `apps/painel/src/features/activity/activity-page.tsx`
  - Prova: a página real, com o item "Atividades" na sidebar e os dois filtros (RF-06).
  - Trecho: o título "Atividades", os filtros "Tipo" e "Spec" e a lista com data e hora, da mais nova para a mais antiga.
  - Falha: página "não encontrada", sem filtros ou sem o item na sidebar.
