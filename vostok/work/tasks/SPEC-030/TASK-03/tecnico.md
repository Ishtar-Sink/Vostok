<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Acrescenta a ação `fazer-commit` em `shared/attention.ts` e o botão no cabeçalho de `features/inbox/inbox-page.tsx`, com o diálogo de texto de `shared/action-bar.tsx`. Cobre RF-11 e CT-12.

**Decisões.** O botão fica no cabeçalho das pendências, não na lista: o commit não é um item que espera o usuário, é um pedido dele (inferência G da spec). Um botão no painel inicial foi descartado para não poluir a visão geral. A tela não manda caminhos: o agente decide o que commitar pela skill, e o texto do pedido diz o escopo.

**Riscos.** O print usa o binário de release: gere-o antes (`pnpm --dir apps/painel build` e `cargo build --release --manifest-path apps/cli/Cargo.toml --features painel`).

**Arquivos em comum na onda.** Nenhum.


**O que foi feito.**
- `attention.ts` ganhou a ação `fazer-commit`.
- `inbox-page.tsx` pôs o botão secundário no cabeçalho, que abre o diálogo de texto (texto obrigatório).
- `inbox-page.test.tsx` ganhou dois testes, que falharam antes do código.

**Fora do previsto.** `shared/action-bar.tsx` também mudou, embora estivesse fora de "Mudar": o pedido sem spec não saía sem mapear `fazer-commit`, omitir o `&spec=` e ajustar o título do diálogo.

**Verificação.** O `tsc`, os 32 testes do vitest e o build passaram, e a regressão ficou igual ao baseline. A evidência `pedir-commit-testes.txt` foi gravada, e o print `pedir-commit.png` foi conferido pelo usuário.
