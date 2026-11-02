<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Muda o texto das regras em `packages/harness/rules/agent/boundaries.md` e nas seis skills, no pacote distribuído; o `vostok update` leva a mudança aos projetos. Cobre RF-16 e CT-17; atende RN-01 e RN-05.

**Decisões.** Uma regra nova em `boundaries.md` concentra o princípio; as skills só citam o pedido do painel na regra que já exige o pedido do usuário. Reescrever cada regra por inteiro foi descartado para não mexer no que já funciona. A conferência do print continua do usuário, porque o pedido do painel não substitui olhar a imagem.

**Riscos.** As cópias em `vostok/` deste repositório só mudam no próximo `vostok update`; os testes usam o pacote.

**Arquivos em comum na onda.** Nenhum.


**O que foi feito.**
- `boundaries.md` ganhou `[agent/boundaries-05]`: o pedido do painel vale como pedido do usuário; com a trava, as ações só passam com o aval; o agente nunca grava `avais.log` nem `approved_by`, nem chama as rotas do painel.
- `[agent/boundaries-01]` passou a aceitar o pedido `fazer-commit`.
- As regras `-01` de approve-spec, approve-plan, approve-task, finalize-spec e approve-commit, e as `[finish-task-08]` e `[finish-task-09]`, citam o pedido do painel.
- approve-commit passou a usar `--pedido PED-NNN` com a trava ligada.

**Verificação.** As suítes `context_memory_rules` (22) e `commit_skill` (26) passaram, e a regressão ficou igual ao baseline. As evidências são `regras-testes.txt` e os diffs de `boundaries.md`, finish-task e approve-commit.
