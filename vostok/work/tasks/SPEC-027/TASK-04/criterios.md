### Concluída quando

- [x] Uma aprovação e uma edição feitas pelo painel passam nos comandos de validação do `vostok` (PQ-06, CA-06).

  > Como comprovar: em `testes.diff`, ver as saídas de `task approve validate` e `task adjust validate` sem problemas.

- [x] A edição simultânea não perde dados e o controle recomendado está registrado (PQ-07, CA-10).

  > Como comprovar: em `testes.diff`, ver o 409 com o conteúdo atual e a atualização da tela pelo evento.

- [x] O formato do pedido está definido e um chat do Claude Code em `/loop` processou um pedido do painel (PQ-08, CA-09).

  > Como comprovar: em `testes.diff`, ver o formato e o tempo até `feito`; em `pedido-loop.txt`, ver o chat processando o pedido.
