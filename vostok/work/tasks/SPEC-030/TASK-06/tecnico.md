<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Chama `aval::require` (TASK-01) no `finish` e na aprovação pelo chat de `task.rs` e `spec.rs`, e acrescenta à validação o problema de `approved_by: painel` sem aval. Cobre RF-03, RF-08, RF-09, CT-01, CT-03, CT-04, CT-09, CT-10, CB-01 e CB-07; atende RN-02 e RN-05.

**Decisões.**
- **Hash do finish:** o `approved_hash` da tarefa, que não muda na execução; um ajuste depois do aval derruba a aprovação e, com ela, o aval (RN-02). O hash do arquivo foi descartado porque o registro muda a cada etapa.
- **Aprovação pelo chat com a trava:** exige o aval `aprovar` do mesmo hash (inferência D). Liberar as aprovações pelo chat foi descartado, porque deixaria o caminho que a trava quer fechar. O `approve_from_panel` não confere, porque é ele quem grava o aval.
- **Validação sem depender da trava:** o `approved_by: painel` sem aval é acusado sempre, porque indica forja mesmo com a trava desligada.
- **Onde conferir:** no início do `finish`, antes de qualquer gravação, para a recusa não deixar a tarefa pela metade.

**Riscos.** Quebrar os golden de tarefa e spec: a conferência só age com a trava ligada ou com `approved_by: painel`, que os golden não têm. `task.rs` tem mais de 5 mil linhas: mude só as funções citadas.

**Arquivos em comum na onda.** Nenhum: a TASK-07 grava os testes dela em `apps/cli/tests/trava_plano_commit.rs`, e as duas podem rodar juntas.


**O que foi feito.**
- `task.rs`: o `finish` exige o aval `finalizar-tarefa` com o `approved_hash`, e o `approve` pelo chat exige o aval `aprovar` com o `content_hash`. O `errors_of` acusa `approved_by: painel sem aval correspondente em avais.log`.
- `spec.rs`: o `approve` pelo chat exige o aval, e o `spec validate` e o `spec adjust validate` acusam o mesmo problema.
- As aprovações pelo painel não passam pela trava.
- `tests/aval.rs` tem 6 testes.

**Ajuste 1 (com a aprovação do usuário).** O `aval::require`, da TASK-01, resolvia a pasta de trabalho pela chave `paths.work`, que não existe no `main.yml`. Com a trava ligada, todo comando recusava com erro de configuração. A correção passou a usar `harness.root.join("work")`, como o painel, e os testes do módulo deixaram de declarar a chave.

**Verificação.** Passaram os 6 testes da trava, os 8 de `aval`, os de `task` (207) e os de `spec` (20), inclusive os golden. O clippy ficou sem avisos, e a regressão ficou igual ao baseline. A evidência é `trava-tarefa-spec.txt`.
