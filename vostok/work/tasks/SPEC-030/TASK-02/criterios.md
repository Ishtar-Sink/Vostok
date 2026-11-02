### Concluída quando

- [ ] A aprovação pelo painel grava o aval com a quinta coluna `aprovar` (RF-12, CT-13).

  > Como comprovar: em `painel-aval.txt`, ver `approval_aval_has_the_action_column` ok.

- [ ] Os pedidos de finalizar tarefa, aprovar plano, finalizar spec e fazer commit gravam o aval com o hash do alvo, e os outros pedidos não gravam (RF-10, CT-11).

  > Como comprovar: em `painel-aval.txt`, ver os testes de pedidos travados e `unlocked_requests_record_no_aval` ok.

- [ ] O painel aceita a ação `fazer-commit` (RF-11, CT-12).

  > Como comprovar: em `painel-aval.txt`, ver `request_actions_are_eleven` e `commit_request_records_the_aval` ok.
