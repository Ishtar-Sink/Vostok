<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Chama `aval::require` (TASK-01) no `finish` e na aprovação pelo chat de `task.rs` e `spec.rs`, e acrescenta à validação o problema de `approved_by: painel` sem aval. Cobre RF-03, RF-08, RF-09, CT-01, CT-03, CT-04, CT-09, CT-10, CB-01 e CB-07; atende RN-02 e RN-05.

**Decisões.**
- **Hash do finish:** o `approved_hash` da tarefa, que não muda na execução; um ajuste depois do aval derruba a aprovação e, com ela, o aval (RN-02). O hash do arquivo foi descartado porque o registro muda a cada etapa.
- **Aprovação pelo chat com a trava:** exige o aval `aprovar` do mesmo hash (inferência D). Liberar as aprovações pelo chat foi descartado, porque deixaria o caminho que a trava quer fechar. O `approve_from_panel` não confere, porque é ele quem grava o aval.
- **Validação sem depender da trava:** o `approved_by: painel` sem aval é acusado sempre, porque indica forja mesmo com a trava desligada.
- **Onde conferir:** no início do `finish`, antes de qualquer gravação, para a recusa não deixar a tarefa pela metade.

**Riscos.** Quebrar os golden de tarefa e spec: a conferência só age com a trava ligada ou com `approved_by: painel`, que os golden não têm. `task.rs` tem mais de 5 mil linhas: mude só as funções citadas.

**Arquivos em comum na onda.** Nenhum: a TASK-07 grava os testes dela em `apps/cli/tests/trava_plano_commit.rs`, e as duas podem rodar juntas.
