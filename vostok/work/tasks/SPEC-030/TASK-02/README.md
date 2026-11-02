# TASK-02: Gravar o aval nos pedidos do painel

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-030
- **Status:** approved
- **Onda:** 1
- **Dependências:** -

## Arquivos

- [Execução](execucao.md): o que ler, mudar e os passos.
- [Verificação](verificacao.md): baseline, comandos e evidências.
- [Critérios](criterios.md): quando a tarefa está concluída.
- [Resumo executivo](executivo.md)
- [Detalhes técnicos](tecnico.md)
- [Registro](registro.md): etapas da execução e histórico de ajustes.

## Card

- **Título:** Gravar o aval nos pedidos do painel
- **Tipo:** tarefa
- **Descrição:** Aval de 5 colunas na aprovação, aval junto dos pedidos travados e a ação `fazer-commit` no painel.
- **Critérios de aceite:**
  - [ ] A aprovação pelo painel grava o aval com a quinta coluna `aprovar` (RF-12, CT-13).
  - [ ] Os pedidos de finalizar tarefa, aprovar plano, finalizar spec e fazer commit gravam o aval com o hash do alvo, e os outros pedidos não gravam (RF-10, CT-11).
  - [ ] O painel aceita a ação `fazer-commit` (RF-11, CT-12).
- **Rótulos:** SPEC-030, padrao
- **Estimativa:** padrao
- **Bloqueada por:** -
