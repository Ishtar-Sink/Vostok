### Ler

- `vostok/work/specs/refined/SPEC-032-controlar-o-projeto-inteiramente-pelo-painel-v0-4-0.md`
- `apps/painel/src/shared/attention.ts` (fn attentionItems, fn taskActions)
- `apps/painel/src/shared/attention.test.ts`
- `apps/painel/src/shared/api.ts` (TaskSummary)
- `apps/cli/src/painel/read.rs` (fn task_summary)

### Mudar

- `apps/painel/src/shared/attention.ts` (alterar): pendência de finalização só com `check_result: passed`
- `apps/painel/src/shared/attention.test.ts` (alterar): testes da regra nova
- `apps/painel/src/shared/api.ts` (alterar): campo `check_result` em `TaskSummary`
- `apps/cli/src/painel/read.rs` (alterar): `check_result` no resumo da tarefa

### Passos

1. `apps/painel/src/shared/attention.test.ts` (alterar): acrescente `só pede finalização com a conferência passada`: uma tarefa `in_progress` sem `check_result` não gera item `finish-task`; uma com `check_result: 'passed'` gera; uma com `failed` não gera. Confira: o teste falha antes da mudança.
2. `apps/cli/src/painel/read.rs` (alterar): em `fn task_summary`, inclua `check_result` na lista de campos lidos do `estado.yml`. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::read`.
3. `apps/painel/src/shared/api.ts` (alterar): acrescente `check_result?: string` a `TaskSummary`. Confira: `pnpm --dir apps/painel exec tsc --noEmit`.
4. `apps/painel/src/shared/attention.ts` (alterar): em `attentionItems`, a condição do item `finish-task` passa a exigir também `task.check_result === 'passed'`. Não mude `taskActions`: a tela da tarefa continua oferecendo "Pedir finalização" para `in_progress` e `validating`. Confira: `pnpm --dir apps/painel exec vitest run src/shared --reporter=verbose 2>&1 | grep -E '✓|✗|×' | sed 's/ [0-9]*ms$//' | sort` com o teste novo em ✓.
