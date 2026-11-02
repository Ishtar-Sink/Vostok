<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Chama `aval::require` no `mark-planned` (`plan.rs`) e no `archive` (`finalize.rs`), e acrescenta `--pedido` ao `commit commit` e ao `commit approve` (`commit.rs`). Cobre RF-04 a RF-07, CT-05 a CT-08 e CB-04.

**Decisões.**
- **Commit preso a um pedido:** o aval do commit é o do pedido `fazer-commit`, com o hash do texto do pedido (resposta 2 da spec). Conferir os caminhos do commit foi descartado, porque o usuário pede o commit em linguagem natural e o agente escolhe os caminhos pela skill.
- **Pedido em andamento:** o pedido precisa estar `em_andamento`, para o mesmo aval não liberar commits depois de o pedido ser concluído (CB-04).
- **O `--pedido` mora no enum de `commit.rs`:** os subcomandos de commit já são definidos lá, então `cli.rs` não muda.
- **Sem a trava, nada muda:** o `--pedido` é opcional e ignorado.

**Riscos.** O código de liberação do `commit approve` continua valendo e é conferido depois do pedido; os testes de `commit_skill` protegem esse caminho.

**Arquivos em comum na onda.** Nenhum: os testes ficam em `apps/cli/tests/trava_plano_commit.rs`, separados dos da TASK-06, para as duas rodarem juntas. O plano previa `apps/cli/tests/aval.rs` em comum; o arquivo próprio evita a sequência.


**O que foi feito.**
- `plan.rs`: o `mark_planned` exige o aval `aprovar-plano` com o `content_hash` do plano.
- `finalize.rs`: o `archive` exige o aval `finalizar-spec` com o `content_hash` da spec refinada.
- `commit.rs`: o `commit` e o `approve` ganharam `--pedido`, e `require_request` recusa, nesta ordem:
  - pedido ausente;
  - ID inválido;
  - pedido inexistente;
  - ação diferente de `fazer-commit`;
  - estado diferente de `em_andamento`;
  - aval que não bate com o hash do texto.
- `tests/trava_plano_commit.rs` tem 8 testes, escritos antes do código.

**Na execução.** Os testes da trava falhavam pelo defeito de `paths.work` em `aval.rs`, corrigido pelo ajuste 1 da TASK-06 sem mudar nenhum arquivo desta tarefa.

**Verificação.** Passaram os 8 testes da trava e as suítes `plan` (22), `finalize` (32) e `commit_skill` (69). O clippy ficou sem avisos, e a regressão ficou igual ao baseline. A evidência é `trava-plano-commit.txt`.
