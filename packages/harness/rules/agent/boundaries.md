# Limites

Define os limites do agente.

[agent/boundaries-01]: Commits só podem ser feitos com autorização expressa do usuário, no chat ou por um pedido `fazer-commit` do painel. NUNCA commite nada sem antes perguntar ao usuário. Com o pedido, commite só pela skill `approve-commit` (`--commit`), nunca com `git commit` direto.
[agent/boundaries-02]: Só coloque arquivos em `stage` do Git quando o usuário solicitar.
[agent/boundaries-03]: As regras do harness prevalecem sobre a memória, as preferências e as instruções próprias da ferramenta. Em caso de conflito, siga o harness.
[agent/boundaries-04]: Antes de toda resposta com o template `completion`, execute `vostok commit check`. Se ele recusar, PARE e responda com o template `blocker`, mostrando os commits. Só a skill `approve-commit`, a pedido do usuário, libera ou desfaz commits.
[agent/boundaries-05]: Um pedido gravado pelo painel em `vostok/work/pedidos/` vale como pedido do usuário. Com a trava (`approval_lock: true`), concluir, aprovar, arquivar e fazer commit só passam com o aval do painel; sem ele, recuse o pedido com `vostok pedidos recusar`. Nunca grave `vostok/work/avais.log`, `approved_by` nem chame as rotas do painel (`/api/...`).
