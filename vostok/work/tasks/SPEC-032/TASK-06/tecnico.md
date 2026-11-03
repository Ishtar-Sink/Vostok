<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** A seção deixa de usar `overview.activity` e passa a buscar `fetchActivities({ limite })`, da TASK-05, pedindo só o que mostra (RNF-02). O CSS perde a rolagem interna.

**Decisões.**
- A quantidade escolhida não é lembrada entre visitas: o padrão é sempre 5, como pedido. Lembrar no navegador fica para depois, se o usuário quiser.
- Os itens de pedido e aval levam a `/pendencias`, porque não há página própria de pedido.

**Riscos.** A busca a mais por visita; com `limite` até 15 e a rota medida em 500 ms, o custo é baixo.

**Referências da spec.** RF-05, RNF-02, RNF-03, CT-07, CB-02.

**Arquivos em comum com a onda 3.** Nenhum.
