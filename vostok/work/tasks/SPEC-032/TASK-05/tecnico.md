<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Nova rota `GET /api/atividades` em `read.rs`, lida a cada requisição a partir dos arquivos, sem índice (RN-03). O `server.rs` já encaminha todo `GET /api/*` para `read::route`; a mudança nele é só o teste de roteamento.

**Decisões.**
- O campo `activity` do `overview` fica, para o painel inicial não quebrar até a TASK-06 usar a rota nova; removê-lo fica para a TASK-06.
- O limite padrão é 15, o maior valor do seletor do painel inicial (RNF-02).
- Pedidos guardam só o último estado, então geram duas atividades no máximo: criação e estado atual (premissa da spec).

**Riscos.** O custo de ler todas as fontes; o passo 7 manda parar se o estudo apontar uma fonte cara, e o teste de 500 ms mede.

**Referências da spec.** RF-07, RN-03, RN-04, RNF-01, CT-09, CT-10, CB-03, CB-04; decisão 10; contrato de `GET /api/atividades`.

**Arquivos em comum com a onda 2.** Nenhum. `api.ts` e `read.rs` também mudam na TASK-04, que é da onda 3.
