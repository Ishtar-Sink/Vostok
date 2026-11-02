<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Muda só `apps/cli/src/painel/actions.rs`: `record_aval` ganha a ação, `REQUEST_ACTIONS` ganha `fazer-commit` e `request` grava o aval dos pedidos travados. Cobre RF-10, RF-12, CT-11, CT-13 e a parte de servidor do CT-12.

**Decisões.** O hash vem dos campos que o `vostok` grava (`approved_hash`, `content_hash`), os mesmos que a CLI confere na TASK-06 e na TASK-07; recalcular o SHA-256 do arquivo foi descartado porque o arquivo muda na execução. Pedido sem aval é apagado na falha, para o loop nunca pegar um pedido travado que a CLI vai recusar. A leitura do hash fica no próprio `actions.rs`, sem usar o módulo `aval` da TASK-01, porque as duas tarefas rodam juntas.

**Riscos.** Quebrar as rotas da SPEC-029: os testes de `painel::actions` e `painel_server` continuam na verificação.

**Arquivos em comum na onda.** Nenhum.


**O que foi feito.** Em `apps/cli/src/painel/actions.rs`:
- `record_aval` grava a quinta coluna (`aprovar` na aprovação);
- `REQUEST_ACTIONS` tem 11 ações, com `fazer-commit`;
- `request` grava o aval de `finalizar-tarefa` (`approved_hash`), `aprovar-plano` e `finalizar-spec` (`content_hash`) e `fazer-commit` (SHA-256 do texto do pedido).

Sem o hash do alvo, a resposta é 422 sem criar nada. Uma falha ao gravar o aval responde 500 e apaga o pedido.

**Fora do previsto.** O 422 vale também para plano e spec sem `content_hash`; a tarefa só previa o caso da tarefa sem aprovação. O teste `every_accepted_action_is_written_with_the_next_number` ganhou o ajudante `approve_task_and_write_targets`, porque os pedidos travados passaram a exigir os alvos.

**Verificação.** Passaram os 33 testes de `painel::actions` e os 10 de `painel_server`, com clippy sem avisos. A regressão mostrou só o acréscimo dos 6 testes novos. A evidência é `painel-aval.txt`.
