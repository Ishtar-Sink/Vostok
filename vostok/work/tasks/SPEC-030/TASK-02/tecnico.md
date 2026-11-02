<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Muda só `apps/cli/src/painel/actions.rs`: `record_aval` ganha a ação, `REQUEST_ACTIONS` ganha `fazer-commit` e `request` grava o aval dos pedidos travados. Cobre RF-10, RF-12, CT-11, CT-13 e a parte de servidor do CT-12.

**Decisões.** O hash vem dos campos que o `vostok` grava (`approved_hash`, `content_hash`), os mesmos que a CLI confere na TASK-06 e na TASK-07; recalcular o SHA-256 do arquivo foi descartado porque o arquivo muda na execução. Pedido sem aval é apagado na falha, para o loop nunca pegar um pedido travado que a CLI vai recusar. A leitura do hash fica no próprio `actions.rs`, sem usar o módulo `aval` da TASK-01, porque as duas tarefas rodam juntas.

**Riscos.** Quebrar as rotas da SPEC-029: os testes de `painel::actions` e `painel_server` continuam na verificação.

**Arquivos em comum na onda.** Nenhum.
