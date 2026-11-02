# TASK-06: Travar tarefa e spec sem aval

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-030
- **Status:** approved
- **Onda:** 2
- **Dependências:** TASK-01

## Arquivos

- [Execução](execucao.md): o que ler, mudar e os passos.
- [Verificação](verificacao.md): baseline, comandos e evidências.
- [Critérios](criterios.md): quando a tarefa está concluída.
- [Resumo executivo](executivo.md)
- [Detalhes técnicos](tecnico.md)
- [Registro](registro.md): etapas da execução e histórico de ajustes.

## Card

- **Título:** Travar tarefa e spec sem aval
- **Tipo:** tarefa
- **Descrição:** Aval exigido no `task finish` e nas aprovações de spec e tarefa, e `approved_by: painel` sem aval acusado na validação.
- **Critérios de aceite:**
  - [ ] Sem a trava, concluir funciona como antes; com ela, `task finish` só conclui com o aval `finalizar-tarefa` do hash aprovado, e a recusa explica como obtê-lo (RF-03, CT-01, CT-03, CT-04, CB-01, CB-07).
  - [ ] Com a trava, as aprovações de spec e tarefa pelo chat exigem o aval `aprovar` (RF-08, CT-09).
  - [ ] A validação acusa `approved_by: painel` sem aval, e os golden continuam passando (RF-09, CT-10).
- **Rótulos:** SPEC-030, julgamento
- **Estimativa:** julgamento
- **Bloqueada por:** TASK-01
