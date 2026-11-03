### Concluída quando

- [x] O painel grava os pedidos `criar-spec` (com o assunto), `refinar-spec` e `executar-onda` (com a spec e a onda) e recusa os que vêm sem esses campos (RF-05, base para a TASK-09).

  > Como comprovar: em `acoes-painel-testes.txt`, os testes `spec_requests_` passam.

- [x] Os pedidos `liberar-commits` e `desfazer-commits` gravam o aval, e a CLI só libera ou desfaz com o pedido e o aval quando a trava está ligada (RF-07, RN-02, CT-08).

  > Como comprovar: `commit_release_and_undo_requests_record_the_aval` passa em `acoes-painel-testes.txt`, e os dois testes novos passam em `commits-por-pedido-testes.txt`.

- [x] Conferir um print no painel grava a conferência com a origem `painel`, no mesmo texto do `vostok task evidence --reviewed`; rejeitar bloqueia a tarefa (RF-06, RN-01, CT-07, FA-02).

  > Como comprovar: os testes `print_review_` passam em `acoes-painel-testes.txt`.

- [x] Responder a uma pergunta no painel a move para as respondidas e cria o pedido de resposta; a segunda resposta recebe 409 e a primeira fica (RF-03, CT-03, CB-06).

  > Como comprovar: `answer_moves_the_question_and_creates_the_reply_request` e `second_answer_is_409_and_keeps_the_first` passam em `acoes-painel-testes.txt`.

- [x] A limpeza pelo painel só lista sem confirmação, só apaga com ela e nunca apaga avais (RF-11, base para a TASK-10).

  > Como comprovar: `prune_route_lists_without_deleting_and_deletes_with_confirmation` passa em `acoes-painel-testes.txt`.
