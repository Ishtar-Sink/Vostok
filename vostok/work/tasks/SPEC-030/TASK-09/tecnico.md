<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Acrescenta o subcomando `vigiar` a `apps/cli/src/commands/pedidos.rs`, que confere `vostok/work/pedidos/` no intervalo e roda `codex exec` com a instrução da skill `process-requests` para o menor pedido pendente. Cobre RF-15, CT-16 e CB-06; atende RNF-04.

**Decisões.** Comando próprio, separado do painel (resposta 3 da spec), para funcionar com o painel fechado. Um pedido por vez e em ordem (inferência H), porque dois `codex exec` na mesma pasta disputariam os arquivos. Pedido que continua `pendente` depois da execução vira `recusado`, para o laço não repetir o mesmo pedido para sempre. Intervalo padrão de 15 s, dentro da faixa de 10 a 20 s que o estudo apontou.

**Riscos.** O teste depende de um `codex` simulado no `PATH`; o uso real com o Codex fica para depois da spec.

**Arquivos em comum na onda.** Nenhum: é a única tarefa da onda 3.

**O que foi feito.**
- `apps/cli/src/commands/pedidos.rs` ganhou `Vigiar { agente, intervalo }` (padrão de 15 s):
  - `watch` recusa um agente diferente de `codex` e para se o `codex` não está no `PATH` (`in_path`);
  - `next_pending` acha o menor pedido `pendente`;
  - `process_with_codex` roda `codex exec --full-auto "<instrução>"` na raiz do projeto (`paths.root`) e espera terminar. Se o pedido continua `pendente`, marca `recusado` com o motivo em `## Resultado`;
  - uma linha por pedido traz o estado e o código de saída do Codex.
- `apps/cli/tests/pedidos.rs` ganhou 4 testes com um `codex` simulado, escritos antes do código: ordem e um por vez, pedido deixado pendente, `codex` ausente e agente desconhecido.

**Decisões na execução.**
- **`--full-auto`:** o `codex exec` é chamado com essa opção, porque o estudo mostrou que, sem liberar a escrita na pasta, o pedido para esperando permissão.
- **`cli.rs` sem mudança:** a ajuda de `pedidos` já lista o `vigiar` pelo próprio subcomando.
- **Teste a mais:** o do pedido deixado pendente, que cobre a regra que evita repetir o mesmo pedido para sempre.

**Verificação.** Passaram os 12 testes de `pedidos`, e o clippy ficou sem avisos. A regressão mostrou só o acréscimo dos 4 testes novos. A evidência é `vigiar-testes.txt`. O uso com o Codex real fica para depois da spec.
