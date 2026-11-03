### Concluída quando

- [ ] Cada mudança de status da spec (approved, planned, implemented, archived) ganha uma linha de histórico, gravada pelo comando que já faz a mudança (RF-12, CT-13).

  > Como comprovar: em `historico-status-testes.txt`, os quatro testes `*status_history*` aparecem com `ok`.

- [ ] A linha de histórico não muda o hash do conteúdo da spec nem quebra as fixtures de equivalência (RF-12).

  > Como comprovar: no mesmo arquivo, `golden_spec`, `golden_plan`, `golden_task` e `golden_finalize` aparecem com `ok`.

- [ ] A `GET /api/atividades` mostra o histórico de status das specs e cada transição dos pedidos, também depois de arquivados (RF-12, CT-13).

  > Como comprovar: em `atividades-historico-testes.txt`, os testes `atividades_trazem_o_historico_de_status_das_specs`, `atividades_trazem_as_transicoes_dos_pedidos` e `atividades_leem_os_pedidos_arquivados` aparecem com `ok`.

- [ ] O código passa no clippy e no rustfmt (RF-12).

  > Como comprovar: a verificação termina sem aviso do clippy nem diferença do `cargo fmt --check`.
