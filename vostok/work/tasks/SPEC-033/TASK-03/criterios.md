### Concluída quando

- [ ] Um aval de uma spec ou de uma tarefa dela fica no arquivo da spec na pasta de avais, e um sem spec no arquivo à parte, sempre por acréscimo (RF-09, CT-10).

  > Como comprovar: em `avais-testes.txt`, `by_spec_task_approval_goes_to_the_spec_log` passa; em `avais-painel-testes.txt`, `avais_log_only_receives_appends` passa.

- [ ] `vostok avais <ID>` mostra os avais de uma spec, de uma tarefa ou de um pedido, do mais novo para o mais antigo, e diz quando não há nenhum (RF-09, CT-10, RNF-02).

  > Como comprovar: em `avais-testes.txt`, `by_spec_avais_command_lists_newest_first` e `by_spec_avais_command_without_avais_says_so` passam.

- [ ] O `avais.log` antigo é dividido por spec uma vez, sem mudar as linhas, e as malformadas vão para o arquivo sem spec sem quebrar a migração (RF-09, CT-17, CB-03).

  > Como comprovar: em `avais-testes.txt`, `by_spec_legacy_log_is_split_once` passa.

- [ ] Com a pasta de avais e o arquivo sem spec configurados com outros nomes, os avais vão para eles e o `vostok avais` os lê (RF-09, CT-19).

  > Como comprovar: em `avais-testes.txt`, `by_spec_folders_follow_the_config` passa.

- [ ] A trava de aval, as aprovações do painel e as atividades continuam funcionando com os avais por spec (RF-09).

  > Como comprovar: os testes atuais de `aval`, `trava_plano_commit` e do painel passam, sem regressão contra o baseline.
