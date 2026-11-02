<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Acrescenta a ação `fazer-commit` em `shared/attention.ts` e o botão no cabeçalho de `features/inbox/inbox-page.tsx`, com o diálogo de texto de `shared/action-bar.tsx`. Cobre RF-11 e CT-12.

**Decisões.** O botão fica no cabeçalho das pendências, não na lista: o commit não é um item que espera o usuário, é um pedido dele (inferência G da spec). Um botão no painel inicial foi descartado para não poluir a visão geral. A tela não manda caminhos: o agente decide o que commitar pela skill, e o texto do pedido diz o escopo.

**Riscos.** O print usa o binário de release: gere-o antes (`pnpm --dir apps/painel build` e `cargo build --release --manifest-path apps/cli/Cargo.toml --features painel`).

**Arquivos em comum na onda.** Nenhum.
