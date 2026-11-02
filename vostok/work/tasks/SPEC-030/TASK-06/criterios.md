### Concluída quando

- [ ] Sem a trava, concluir funciona como antes; com ela, `task finish` só conclui com o aval `finalizar-tarefa` do hash aprovado, e a recusa explica como obtê-lo (RF-03, CT-01, CT-03, CT-04, CB-01, CB-07).

  > Como comprovar: em `trava-tarefa-spec.txt`, ver os testes de finish ok.

- [ ] Com a trava, as aprovações de spec e tarefa pelo chat exigem o aval `aprovar` (RF-08, CT-09).

  > Como comprovar: em `trava-tarefa-spec.txt`, ver `chat_approvals_need_the_aval_with_the_lock` ok.

- [ ] A validação acusa `approved_by: painel` sem aval, e os golden continuam passando (RF-09, CT-10).

  > Como comprovar: em `trava-tarefa-spec.txt`, ver `panel_approval_without_aval_is_reported` e os golden ok.
