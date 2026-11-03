### Concluída quando

- [ ] Sem as chaves novas no `config/main.yml`, a CLI usa `work/pedidos`, `work/perguntas`, `work/avais` e as subpastas em português (RF-16).

  > Como comprovar: em `config-testes.txt`, `communication_paths_default_to_portuguese` e `communication_shipped_config_has_the_defaults` passam.

- [ ] Com os caminhos e as subpastas trocados para inglês na configuração, a CLI devolve os nomes configurados (RF-16, CT-19).

  > Como comprovar: em `config-testes.txt`, `communication_paths_follow_the_config` passa.

- [ ] Dois estados com a mesma pasta, ou um nome vazio, com `/` ou `..`, são recusados com a mensagem que cita os estados ou a chave (RF-16, CT-20).

  > Como comprovar: em `config-testes.txt`, `communication_folders_reject_repeated_names` e `communication_folders_reject_unsafe_names` passam.

- [ ] `request_dispatch` vem vazio por padrão e só aceita `codex` ou `claude` (RF-16, base do RF-13).

  > Como comprovar: em `config-testes.txt`, `communication_dispatch_accepts_only_known_agents` passa.

- [ ] A leitura da configuração em três níveis não muda as leituras de dois níveis nem a atualização do harness (RF-16).

  > Como comprovar: `config_reads_three_levels` e os demais `config_` passam, e o `init_update` passa sem falhas, sem regressão contra o baseline.
