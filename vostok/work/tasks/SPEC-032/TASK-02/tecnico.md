<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** A CLI já grava `validating` no `vostok task tests`; o quadro só juntava `in_progress` e `validating` em "Em execução". A mudança é no frontend, em `BOARD_COLUMNS`.

**Decisões.** A coluna nova fica logo depois de "Em execução", seguindo o fluxo (executar, validar, bloquear ou concluir). O print usa 1440 px de largura, porque seis colunas cortam em 1280 px (lição dos prints da SPEC-031).

**Riscos.** Outras telas que usam `BOARD_COLUMNS` (lista de tarefas) ganham a coluna também; o `tsc` e os testes de `src/shared` conferem.

**Referências da spec.** RF-02, CT-02.

**Arquivos em comum com a onda 2.** Nenhum: as TASK-03 e TASK-05 mudam a CLI e `api.ts`.
