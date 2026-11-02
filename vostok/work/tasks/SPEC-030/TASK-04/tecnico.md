<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Cria `apps/cli/src/commands/pedidos.rs` com os quatro subcomandos e o registra em `cli.rs` e `commands/mod.rs`. O formato do pedido é o que o painel grava (`id`, `acao`, `spec`, `tarefa`, `estado`, `origem`, `criado_em`, `atualizado_em`, `## Pedido`, `## Resultado`). Cobre RF-13, CT-14 e CB-05; atende RN-03.

**Decisões.** O estado só muda pelo comando (inferência E): a edição do front-matter pelo chat foi descartada, porque o agente erraria o formato e o painel leria lixo. As transições são estritas para o loop nunca pegar o mesmo pedido duas vezes. O resultado vem da entrada padrão, como a mensagem do `vostok commit commit`.

**Riscos.** A TASK-09 acrescenta o `vigiar` no mesmo arquivo, na onda 3.

**Arquivos em comum na onda.** Nenhum.
