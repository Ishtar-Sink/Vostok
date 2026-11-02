<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Acrescenta o subcomando `vigiar` a `apps/cli/src/commands/pedidos.rs`, que confere `vostok/work/pedidos/` no intervalo e roda `codex exec` com a instrução da skill `process-requests` para o menor pedido pendente. Cobre RF-15, CT-16 e CB-06; atende RNF-04.

**Decisões.** Comando próprio, separado do painel (resposta 3 da spec), para funcionar com o painel fechado. Um pedido por vez e em ordem (inferência H), porque dois `codex exec` na mesma pasta disputariam os arquivos. Pedido que continua `pendente` depois da execução vira `recusado`, para o laço não repetir o mesmo pedido para sempre. Intervalo padrão de 15 s, dentro da faixa de 10 a 20 s que o estudo apontou.

**Riscos.** O teste depende de um `codex` simulado no `PATH`; o uso real com o Codex fica para depois da spec.

**Arquivos em comum na onda.** Nenhum: é a única tarefa da onda 3.
