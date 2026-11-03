### Concluída quando

- [x] Arquivar a spec leva os pedidos concluídos, as perguntas e o log de avais dela para o arquivo morto, e deixa os de outras specs (RF-10, CT-11).

  > Como comprovar: em `arquivar-testes.txt`, o teste `archive_leva_pedidos_perguntas_e_avais_da_spec` aparece com `ok`.

- [x] O log de avais vai para o arquivo morto com o mesmo conteúdo, sem linha reescrita (RF-10, RN-05).

  > Como comprovar: o mesmo teste compara os bytes do log arquivado com os do original.

- [x] Uma spec sem pedidos é arquivada com a mesma saída de antes (RF-10).

  > Como comprovar: `golden_finalize` aparece com `ok` em `arquivar-testes.txt`.

- [x] O `prune` sem `--confirmar` só lista e não apaga nada; com ele, apaga os listados e nunca os avais (RF-11, CT-12).

  > Como comprovar: em `prune-testes.txt`, `prune_sem_confirmar_so_lista` e `prune_com_confirmar_apaga_e_mantem_os_avais` aparecem com `ok`.

- [x] Pedidos sem spec só são limpos depois de 30 dias concluídos, e o `prune` sem nada a apagar diz que não há o que limpar (RF-11, CB-02, CB-05).

  > Como comprovar: o `prune_sem_confirmar_so_lista` deixa de fora o pedido recente e o pendente, e o `prune_sem_nada_diz_que_nao_ha_o_que_limpar` passa.
