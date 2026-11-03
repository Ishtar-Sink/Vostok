### Ler

- `vostok/work/specs/refined/SPEC-032-controlar-o-projeto-inteiramente-pelo-painel-v0-4-0.md`
- `apps/painel/src/features/dashboard/dashboard-page.tsx`
- `apps/painel/src/features/dashboard/dashboard-page.module.css` (.activity)
- `apps/painel/src/features/dashboard/dashboard-page.test.tsx`
- `apps/painel/src/shared/api.ts` (fetchActivities, da TASK-05)

### Mudar

- `apps/painel/src/features/dashboard/dashboard-page.tsx` (alterar): atividade recente com 5, seletor e "Ver todas"
- `apps/painel/src/features/dashboard/dashboard-page.module.css` (alterar): sem rolagem interna na atividade
- `apps/painel/src/features/dashboard/dashboard-page.test.tsx` (alterar): testes da atividade recente

### Passos

1. `apps/painel/src/features/dashboard/dashboard-page.test.tsx` (alterar): acrescente `mostra cinco atividades por padrão`, `troca a quantidade pelo seletor` (10 e 15, conferindo o `limite` pedido a `fetchActivities`) e `leva a todas as atividades` (o link "Ver todas" aponta para `/atividades`), com `fetchActivities` simulado. Confira: falham antes da mudança.
2. `apps/painel/src/features/dashboard/dashboard-page.tsx` (alterar): a seção "Atividade recente" passa a buscar com `fetchActivities({ limite })`, com `limite` inicial 5; ao lado do `h2`, um `select` com rótulo "Quantidade de atividades" e as opções 5, 10 e 15; no fim da lista, o link "Ver todas" para `/atividades` com `onLinkClick`. Cada item leva à página do alvo pelo `type` (tarefa para `/tarefas/<spec>/<target>`, spec e plano para `/specs/<spec>`, pedido e aval para `/pendencias`). Confira: os testes do passo 1 passam.
3. `apps/painel/src/features/dashboard/dashboard-page.module.css` (alterar): em `.activity`, tire `max-height` e `overflow-y`. Confira: o print `atividade-recente.png` mostra a lista sem barra de rolagem.
4. `apps/painel` (conferir): `pnpm --dir apps/painel exec tsc --noEmit`. Confira: sem erros.
5. `apps/cli` (conferir): antes do print, compile o binário de release com a rota nova da TASK-05: `cargo build --release --features painel --manifest-path apps/cli/Cargo.toml`. Confira: `apps/cli/target/release/vostok --version` responde.
