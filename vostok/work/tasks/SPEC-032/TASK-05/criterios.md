### Concluída quando

- [x] `GET /api/atividades` traz tarefas (abertas e arquivadas), specs, planos, pedidos e avais, da mais nova para a mais antiga (RF-07, CT-09, CB-04).

  > Como comprovar: em `atividades-testes.txt`, `atividades_reunem_todas_as_fontes` passa.

- [x] A rota filtra por `limite`, `tipo` e `spec` e ignora avais malformados (RF-07, CB-03).

  > Como comprovar: os testes de filtro e de avais malformados passam.

- [x] A rota responde em até 500 ms com 1.000 atividades (RNF-01, CT-10).

  > Como comprovar: `atividades_respondem_em_ate_500_ms` passa.
