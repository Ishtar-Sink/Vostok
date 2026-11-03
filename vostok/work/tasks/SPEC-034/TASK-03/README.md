# TASK-03: Histórico do ajuste e aval na CLI

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-034
- **Status:** done
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

- **Título:** Histórico do ajuste e aval na CLI
- **Tipo:** bug
- **Descrição:** O ajuste que derruba a aprovação grava o `status_history`, e a mensagem de validação cita o log da spec.
- **Critérios de aceite:**
  - [x] O ajuste que derruba a aprovação de uma spec, pelo agente ou manual, grava a volta para refined no histórico de status, depois da linha da aprovação (RF-06, CT-08).
  - [x] O ajuste de uma spec só refinada não acrescenta linha ao histórico de status (CB-04).
  - [x] A mensagem de validação da CLI para uma aprovação do painel sem aval cita o log de avais da spec, `work/avais/<SPEC>.log`, e nenhuma mensagem da CLI cita `avais.log` como destino (RF-07, CT-09).
  - [x] A aprovação, a validação das specs e das tarefas e a trava de aval continuam funcionando como antes.
- **Rótulos:** SPEC-034, padrao
- **Estimativa:** padrao
- **Bloqueada por:** -
