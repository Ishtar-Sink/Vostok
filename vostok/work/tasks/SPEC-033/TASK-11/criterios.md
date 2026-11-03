### Concluída quando

- [x] A process-requests abre a pergunta e conclui o pedido como "aguardando a pergunta P-NNN" em vez de recusá-lo, e atende o pedido de resposta rodando de novo a skill de origem (RF-02, CT-02).

  > Como comprovar: no `skill-process-requests.diff`, ver as regras novas e o mapa com as ações; em `skill-instalada.txt`, ver `aguardando a pergunta`; em `pedido-que-pergunta.txt`, ver o pedido `feito` aguardando a pergunta e a nova rodada depois da resposta.

- [x] As skills de conversa, quando rodam por um pedido, gravam a rodada de perguntas pelo `vostok perguntas abrir` (RF-04, CT-04).

  > Como comprovar: nos diffs de create-spec, refine-spec, adjust-spec, adjust-plan e adjust-task, ver a regra nova; em `pedido-que-pergunta.txt`, ver a refine-spec gravando as perguntas sem perguntar no chat.

- [x] A finish-task aceita a conferência de print feita no painel, e a approve-commit atende `liberar-commits` e `desfazer-commits` com `--pedido` (RF-02).

  > Como comprovar: no `skill-finish-task.diff`, ver a nova finish-task-09; no `skill-approve-commit.diff`, ver os dois pedidos e o `--pedido` no `commit revert`.

- [x] As skills alteradas continuam válidas no pacote embutido (RF-02, RF-04).

  > Como comprovar: em `skills-testes.txt`, as suítes `base` e `init_update` passam, sem `FAILED`.
