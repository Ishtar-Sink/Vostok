### Concluída quando

- [ ] Com `request_dispatch: codex` ou `claude`, o painel chama o agente uma vez para cada pedido pendente (RF-13, CT-14).

  > Como comprovar: em `disparo-testes.txt`, `disparo_com_codex_chama_o_agente_uma_vez_por_pedido_em_ordem` e `disparo_com_claude_usa_o_modo_nao_interativo` aparecem com `ok`.

- [ ] Nunca há dois agentes rodando ao mesmo tempo (RN-04).

  > Como comprovar: o log do teste com dois pedidos mostra o fim do primeiro antes do início do segundo.

- [ ] Com o disparo desligado (padrão), nenhum agente é chamado (RF-13, CT-14).

  > Como comprovar: `disparo_desligado_nao_chama_o_agente` aparece com `ok`.

- [ ] Com o agente fora do PATH, o painel avisa e o pedido continua pendente, sem ser recusado (CB-04).

  > Como comprovar: `disparo_sem_o_agente_no_path_avisa_e_nao_recusa` aparece com `ok`.

- [ ] O `vostok pedidos vigiar` continua funcionando como antes (RF-13).

  > Como comprovar: em `vigiar-testes.txt`, os testes `vigiar_*` aparecem com `ok`.
