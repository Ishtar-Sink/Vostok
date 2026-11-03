### Concluída quando

- [x] A rota `/atividades` e o item "Atividades" na sidebar existem (RF-06, CT-08).

  > Como comprovar: no print `atividades.png`, o item na sidebar e a página aberta.

- [x] A página lista todas as atividades e filtra por tipo e por spec, com mensagem de lista vazia (RF-06, CB-05).

  > Como comprovar: em `atividades-pagina-testes.txt`, os quatro testes da página passam.

- [x] O registro de execução grava a hora, e as atividades do mesmo dia saem na ordem em que aconteceram, com data e hora formatadas (RF-06).

  > Como comprovar: em `atividades-pagina-testes.txt`, `record_has_the_time` e os testes do `formatDateTime` passam; no print `atividades.png`, as datas têm hora e seguem da mais nova para a mais antiga.
