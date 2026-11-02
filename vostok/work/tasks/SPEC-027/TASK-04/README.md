# TASK-04: Testar aprovação, edição simultânea e loop

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-027
- **Status:** approved
- **Onda:** 3
- **Dependências:** TASK-01, TASK-03

## Arquivos

- [Execução](execucao.md): o que ler, mudar e os passos.
- [Verificação](verificacao.md): baseline, comandos e evidências.
- [Critérios](criterios.md): quando a tarefa está concluída.
- [Resumo executivo](executivo.md)
- [Detalhes técnicos](tecnico.md)
- [Registro](registro.md): etapas da execução e histórico de ajustes.

## Card

- **Título:** Testar aprovação, edição simultânea e loop
- **Tipo:** tarefa
- **Descrição:** Teste no protótipo da aprovação e da edição pelo painel, da edição simultânea com o agente e de um chat em `/loop` processando um pedido.
- **Critérios de aceite:**
  - [ ] Uma aprovação e uma edição feitas pelo painel passam nos comandos de validação do `vostok` (PQ-06, CA-06).
  - [ ] A edição simultânea não perde dados e o controle recomendado está registrado (PQ-07, CA-10).
  - [ ] O formato do pedido está definido e um chat do Claude Code em `/loop` processou um pedido do painel (PQ-08, CA-09).
- **Rótulos:** SPEC-027, julgamento
- **Estimativa:** julgamento
- **Bloqueada por:** TASK-01, TASK-03
