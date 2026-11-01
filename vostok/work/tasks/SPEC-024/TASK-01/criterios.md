### Concluída quando

- [x] O `finish` deixa o executivo e o técnico com títulos como foram escritos, e o registro só ganha a linha de conclusão (RF-01, CT-01).

  > Como comprovar: em `docs-tests.txt`, ver aprovado `docs_finish_keeps_structured_documents`.

- [x] Nenhum outro comando muda os documentos, com títulos de qualquer nível ou código (RF-02, CT-02).

  > Como comprovar: em `docs-tests.txt`, ver aprovado `docs_commands_keep_structured_documents`.

- [x] A aprovação recusa documento no template (RF-03, CT-03).

  > Como comprovar: em `docs-tests.txt`, ver aprovado `docs_approve_refuses_a_document_in_the_template`.

- [x] O `start` grava os hashes e o `finish` exige o enriquecimento, sem travar tarefas antigas (RF-05, RF-06, RF-07, CT-04, CT-05, CT-06).

  > Como comprovar: em `docs-tests.txt`, ver aprovados os três testes `docs_start_*` e `docs_finish_*`.

- [x] Os cenários de referência não mudaram (CT-08).

  > Como comprovar: em `scenarios-tests.txt`, ver os dez `scenario_` aprovados, e `git status` sem mudança em `apps/cli/tests/fixtures/scenarios`.
