### Concluída quando

- [x] Na página da tarefa em execução ou em validação, cada print mostra se está conferido e oferece "Aceitar" e "Rejeitar"; rejeitar pede confirmação, e as duas gravam a conferência pelo painel (RF-06, CT-07, FA-02).

  > Como comprovar: no print `conferencia-prints.png`, os prints com "a conferir" e os dois botões; em `painel-conferencia-testes.txt`, os testes de aceitar, rejeitar, já conferido, recusa e tarefa concluída passam.

- [x] As pendências mostram os commits apontados pelo `vostok commit check`, com "Liberar commits" e "Desfazer commits", e desfazer só grava o pedido depois da confirmação (RF-07, CT-08).

  > Como comprovar: em `painel-conferencia-testes.txt` e `commits-rota-testes.txt`, os testes dos commits e da rota passam; no print `pendencias.png`, a página sem erro.

- [x] Responder uma pergunta envia o id dela, e a resposta repetida mostra o aviso (RF-06, CB-06).

  > Como comprovar: em `painel-conferencia-testes.txt`, `responde a pergunta com o id dela` passa.

- [x] Nas Configurações, "Limpar pedidos arquivados" mostra a lista antes, só apaga ao confirmar e diz quando não há o que limpar (RF-11, CT-12, CB-05, RNF-03).

  > Como comprovar: no print `configuracoes.png`, o botão; em `painel-conferencia-testes.txt`, os quatro testes da limpeza passam.

- [x] Os botões, os diálogos e as confirmações novos são achados por papel e rótulo (CT-18, RNF-04).

  > Como comprovar: em `painel-conferencia-testes.txt`, os testes usam `getByRole` com nome e passam.
