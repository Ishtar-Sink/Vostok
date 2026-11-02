<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

Junta tudo o que as tarefas anteriores levantaram, mediram e testaram num único estudo: se o painel local é viável, quanto pesa, como deve ser construído e quais specs vêm a seguir.
Perfil padrao. Risco: uma conclusão sem número ou sem fonte; cada resposta cita o documento de apoio.
Depende de todas as tarefas anteriores e encerra a spec.

## O que foi feito

- **Conclusão do estudo:** o painel local sem servidor é viável.
  - Ele lê e grava tudo pelos próprios arquivos do harness.
  - As aprovações e edições feitas nele passam nas validações do `vostok`.
  - O que precisa do agente vira um pedido, que um chat em loop processou em pouco mais de um minuto.
- **Peso:** o painel completo soma 0,9 MB ao programa. O limite de tamanho foi revisto para 6,0 MB instalado e 2,3 MB para download, porque o primeiro limite tinha sido medido com uma versão mínima.
- **Próximos passos:** o estudo propõe cinco specs, em ordem:
  1. tirar a premissa de servidor das regras e dos documentos;
  2. a trava de aval;
  3. o painel de leitura;
  4. as ações e a edição pelo painel;
  5. os pedidos ao agente com o loop.
- **Na `main`:** entram só o estudo, os documentos de apoio e cinco imagens do protótipo. O código do protótipo continua fora.
