<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Cria `apps/cli/src/commands/pedidos.rs` com os quatro subcomandos e o registra em `cli.rs` e `commands/mod.rs`. O formato do pedido é o que o painel grava (`id`, `acao`, `spec`, `tarefa`, `estado`, `origem`, `criado_em`, `atualizado_em`, `## Pedido`, `## Resultado`). Cobre RF-13, CT-14 e CB-05; atende RN-03.

**Decisões.** O estado só muda pelo comando (inferência E): a edição do front-matter pelo chat foi descartada, porque o agente erraria o formato e o painel leria lixo. As transições são estritas para o loop nunca pegar o mesmo pedido duas vezes. O resultado vem da entrada padrão, como a mensagem do `vostok commit commit`.

**Riscos.** A TASK-09 acrescenta o `vigiar` no mesmo arquivo, na onda 3.

**Arquivos em comum na onda.** Nenhum.


**O que foi feito.** `apps/cli/src/commands/pedidos.rs` tem `listar [--estado]`, `iniciar`, `concluir` e `recusar`, sobre `<raiz do harness>/work/pedidos/`. `concluir` e `recusar` leem a entrada padrão e gravam em `## Resultado`. As transições fora de ordem e o ID fora de `PED-NNN` são recusados sem mudar o arquivo. `cli.rs` e `commands/mod.rs` registram o comando.

**Verificação.** Os 8 testes de `tests/pedidos.rs` e os de `base` passaram, com clippy sem avisos; a regressão ficou igual ao baseline. A evidência é `pedidos-testes.txt`.

**Na execução.** Os testes foram escritos junto com o código, sem a etapa de vê-los falhar. O `cargo fmt` rodou no `apps/cli` inteiro, e só os arquivos das tarefas mudaram.
