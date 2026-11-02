### Concluída quando

- [ ] A aprovação pelo painel grava `approved_by: painel` e o aval, e uma aprovação recusada não grava aval (RF-06, CT-04).

  > Como comprovar: em `acoes-testes.txt`, ver os testes de aprovação passando.

- [ ] A edição e a configuração gravam com hash, respondem 409 num conflito sem tocar no arquivo e recusam os arquivos do `vostok` e os caminhos de fora (RF-09, RF-10, CT-08 a CT-11, CB-02, CB-04).

  > Como comprovar: em `acoes-testes.txt`, ver os testes de gravação, conflito e recusa passando.

- [ ] Os pedidos são gravados no formato do estudo, e uma ação desconhecida ou de aprovação é recusada (RF-11, CT-12, CB-03).

  > Como comprovar: em `acoes-testes.txt`, ver os testes de pedido passando.
