# TASK-04: Pendência de finalização só com a conferência

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-032
- **Status:** approved
- **Onda:** 3
- **Dependências:** TASK-03

## Arquivos

- [Execução](execucao.md): o que ler, mudar e os passos.
- [Verificação](verificacao.md): baseline, comandos e evidências.
- [Critérios](criterios.md): quando a tarefa está concluída.
- [Resumo executivo](executivo.md)
- [Detalhes técnicos](tecnico.md)
- [Registro](registro.md): etapas da execução e histórico de ajustes.

## Card

- **Título:** Pendência de finalização só com a conferência
- **Tipo:** tarefa
- **Descrição:** Mostra a tarefa em execução nas pendências só com `check_result: passed`, mantendo o botão na tela da tarefa.
- **Critérios de aceite:**
  - [ ] Tarefa em execução sem conferência passada não aparece nas pendências (RF-04, CT-06, CB-01).
  - [ ] A tela da tarefa continua oferecendo "Pedir finalização" (RF-04).
- **Rótulos:** SPEC-032, mecanica
- **Estimativa:** mecanica
- **Bloqueada por:** TASK-03
