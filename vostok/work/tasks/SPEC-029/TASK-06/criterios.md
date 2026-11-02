### Concluída quando

- [x] A aprovação pelo painel grava `approved_by: painel` e o aval, e uma aprovação recusada não grava aval (RF-06, CT-04).

  > Como comprovar: em `acoes-testes.txt`, ver os testes de aprovação passando.

- [x] A edição e a configuração gravam com hash, respondem 409 num conflito sem tocar no arquivo e recusam os arquivos do `vostok` e os caminhos de fora (RF-09, RF-10, CT-08 a CT-11, CB-02, CB-04).

  > Como comprovar: em `acoes-testes.txt`, ver os testes de gravação, conflito e recusa passando.

- [x] Os pedidos são gravados no formato do estudo, e uma ação desconhecida ou de aprovação é recusada (RF-11, CT-12, CB-03).

  > Como comprovar: em `acoes-testes.txt`, ver os testes de pedido passando.

- [x] Gravar uma configuração devolve os mesmos problemas do `vostok rules validate`, e o servidor encaminha `/arquivos/...` e as ações com token (RF-10, RF-02, CT-11).

  > Como comprovar: em `acoes-testes.txt`, ver o teste da configuração com problema; na saída da verificação, ver os testes de `painel::server` e de `context_memory_rules` passando.
