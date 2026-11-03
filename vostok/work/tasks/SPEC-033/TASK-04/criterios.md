### Concluída quando

- [ ] `vostok perguntas abrir PED-001` grava `perguntas/aberta/P-001.md` com o pedido, a spec, a tarefa e `status: aberta` (RF-01, CT-01).

  > Como comprovar: em `perguntas-testes.txt`, o teste `abrir_grava_a_pergunta_aberta_com_o_pedido` aparece com `ok`.

- [ ] Cada pergunta nova recebe o número seguinte ao maior já usado, inclusive entre as respondidas (RF-01).

  > Como comprovar: o teste `abrir_numera_depois_da_maior_pergunta` passa.

- [ ] Pedido inexistente, ID inválido e texto vazio são recusados sem gravar nada (RF-01).

  > Como comprovar: os três testes `abrir_recusa_*` passam.

- [ ] Com os nomes de pasta trocados na configuração, a pergunta vai para a pasta configurada e o status continua `aberta` (RF-16).

  > Como comprovar: o teste `abrir_usa_as_pastas_configuradas` passa.

- [ ] O código passa no clippy e no rustfmt (RF-01).

  > Como comprovar: a verificação termina sem aviso do clippy nem diferença do `cargo fmt --check`.
