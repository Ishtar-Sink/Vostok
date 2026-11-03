### Concluída quando

- [x] Iniciar, concluir e recusar um pedido movem o arquivo para a subpasta do novo estado, e o estado no front-matter continua com os valores atuais (RF-08, CT-09).

  > Como comprovar: em `pastas-pedidos-testes.txt`, `folders_start_and_finish_move_the_file` passa.

- [x] Cada transição deixa uma linha na seção "Histórico" do pedido, com a data, o estado de origem e o de destino (RF-08).

  > Como comprovar: em `pastas-pedidos-testes.txt`, `folders_record_the_history` passa.

- [x] Listar os pendentes lê só a subpasta de pendentes (RF-08, RNF-01).

  > Como comprovar: em `pastas-pedidos-testes.txt`, `folders_list_pending_reads_only_pendente` passa, mesmo com um arquivo inválido em `feito/`.

- [x] Pedidos e perguntas soltos do formato antigo, ou em pastas de nomes antigos, vão uma vez para as subpastas do seu estado, sem mudar o conteúdo (CT-17, CB-07).

  > Como comprovar: `folders_migrate_loose_requests`, `folders_migrate_renamed_folders` e `questions_migrate_loose_files` passam.

- [x] Com as pastas configuradas em inglês, o pedido vai para elas (RF-08, CT-19).

  > Como comprovar: `folders_follow_the_config` passa.

- [x] O painel grava o pedido novo em `pendente/`, lê as atividades de todas as subpastas e as perguntas só das abertas, e o commit travado acha o pedido na subpasta (RF-08).

  > Como comprovar: em `pastas-painel-testes.txt`, os testes do painel passam, e em `pastas-pedidos-testes.txt` os de `trava_plano_commit` passam, sem regressão contra o baseline.
