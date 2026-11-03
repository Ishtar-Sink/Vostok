### Ler

- `vostok/work/specs/refined/SPEC-032-controlar-o-projeto-inteiramente-pelo-painel-v0-4-0.md`
- `apps/painel/src/shared/status.ts` (BOARD_COLUMNS)
- `apps/painel/src/features/board/board.tsx`
- `apps/painel/src/features/board/board.test.tsx`

### Mudar

- `apps/painel/src/shared/status.ts` (alterar): coluna "Em validação" em `BOARD_COLUMNS`
- `apps/painel/src/features/board/board.test.tsx` (alterar): teste da coluna nova

### Passos

1. `apps/painel/src/features/board/board.test.tsx` (alterar): acrescente o teste `separa em validação de em execução`, com uma tarefa `in_progress` e outra `validating`, conferindo que cada uma fica na sua coluna e que a ordem das colunas é Para aprovar, Aprovadas, Em execução, Em validação, Bloqueadas, Concluídas. Confira: o teste falha antes da mudança.
2. `apps/painel/src/shared/status.ts` (alterar): em `BOARD_COLUMNS`, a coluna `running` passa a ter só `statuses: ['in_progress']`; logo depois dela, acrescente `{ id: 'validating', label: 'Em validação', statuses: ['validating'] }`. Confira: `pnpm --dir apps/painel exec vitest run src/features/board --reporter=verbose 2>&1 | grep -E '✓|✗|×' | sed 's/ [0-9]*ms$//' | sort` mostra o teste novo com ✓.
3. `apps/painel` (conferir): `pnpm --dir apps/painel exec tsc --noEmit`. Confira: sem erros.
