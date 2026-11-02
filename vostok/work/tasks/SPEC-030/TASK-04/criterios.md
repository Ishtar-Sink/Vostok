### Concluída quando

- [x] `vostok pedidos` lista os pedidos e os leva de `pendente` a `em_andamento` e a `feito` ou `recusado`, gravando o resultado e a data (RF-13, CT-14).

  > Como comprovar: em `pedidos-testes.txt`, ver os testes de listar, iniciar, concluir e recusar ok.

- [x] Uma transição fora da ordem é recusada sem mudar o arquivo (CB-05).

  > Como comprovar: em `pedidos-testes.txt`, ver `iniciar_refuses_a_request_not_pending` e `concluir_requires_em_andamento` ok.
