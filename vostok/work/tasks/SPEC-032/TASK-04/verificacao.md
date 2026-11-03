### Baseline

- `pnpm --dir apps/painel exec vitest run src/shared --reporter=verbose 2>&1 | grep -E '✓|✗|×' | sed 's/ [0-9]*ms$//' | sort | grep -v 'conferência passada'`

### Verificação

```bash
# evidência: pendencias-testes.txt
pnpm --dir apps/painel exec vitest run src/shared --reporter=verbose
pnpm --dir apps/painel exec tsc --noEmit
cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::read
```

### Evidências

- `pendencias-testes.txt` (automática, da verificação)
  - Prova: a tarefa em execução só vira pendência com a conferência passada, e a tela da tarefa mantém o botão (RF-04, CT-06, CB-01).
  - Trecho: `só pede finalização com a conferência passada` com ✓.
  - Falha: o teste com × ou erro de tipo.
