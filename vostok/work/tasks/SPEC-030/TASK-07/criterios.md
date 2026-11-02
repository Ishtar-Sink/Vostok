### Concluída quando

- [x] Com a trava, `plan approve mark-planned` e `finalize archive` exigem o aval do painel do hash atual (RF-04, RF-05, CT-05, CT-06).

  > Como comprovar: em `trava-plano-commit.txt`, ver os dois testes ok.

- [x] Com a trava, `commit commit` e `commit approve` exigem `--pedido` de um pedido `fazer-commit` em andamento com aval (RF-06, RF-07, CT-07, CT-08, CB-04).

  > Como comprovar: em `trava-plano-commit.txt`, ver os testes de commit ok.

- [x] Sem a trava, os testes atuais de plano, finalização e commit continuam passando (RNF-01).

  > Como comprovar: em `trava-plano-commit.txt`, ver as suítes `plan`, `finalize` e `commit_skill` ok.
