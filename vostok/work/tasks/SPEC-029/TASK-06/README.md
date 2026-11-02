# TASK-06: Implementar as ações do painel

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-029
- **Status:** approved
- **Onda:** 2
- **Dependências:** TASK-01, TASK-02

## Arquivos

- [Execução](execucao.md): o que ler, mudar e os passos.
- [Verificação](verificacao.md): baseline, comandos e evidências.
- [Critérios](criterios.md): quando a tarefa está concluída.
- [Resumo executivo](executivo.md)
- [Detalhes técnicos](tecnico.md)
- [Registro](registro.md): etapas da execução e histórico de ajustes.

## Card

- **Título:** Implementar as ações do painel
- **Tipo:** tarefa
- **Descrição:** Aprovação com aval, edição com hash e registro manual, gravação de configuração e gravação de pedidos, com as recusas 400, 403 e 409.
- **Critérios de aceite:**
  - [ ] A aprovação pelo painel grava `approved_by: painel` e o aval, e uma aprovação recusada não grava aval (RF-06, CT-04).
  - [ ] A edição e a configuração gravam com hash, respondem 409 num conflito sem tocar no arquivo e recusam os arquivos do `vostok` e os caminhos de fora (RF-09, RF-10, CT-08 a CT-11, CB-02, CB-04).
  - [ ] Os pedidos são gravados no formato do estudo, e uma ação desconhecida ou de aprovação é recusada (RF-11, CT-12, CB-03).
- **Rótulos:** SPEC-029, julgamento
- **Estimativa:** julgamento
- **Bloqueada por:** TASK-01, TASK-02
