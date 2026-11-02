# TASK-07: Travar plano, arquivamento e commit sem aval

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

- **Título:** Travar plano, arquivamento e commit sem aval
- **Tipo:** tarefa
- **Descrição:** Aval exigido no `mark-planned`, no `finalize archive` e no `commit commit` e `commit approve` com `--pedido`.
- **Critérios de aceite:**
  - [ ] Com a trava, `plan approve mark-planned` e `finalize archive` exigem o aval do painel do hash atual (RF-04, RF-05, CT-05, CT-06).
  - [ ] Com a trava, `commit commit` e `commit approve` exigem `--pedido` de um pedido `fazer-commit` em andamento com aval (RF-06, RF-07, CT-07, CT-08, CB-04).
  - [ ] Sem a trava, os testes atuais de plano, finalização e commit continuam passando (RNF-01).
- **Rótulos:** SPEC-030, julgamento
- **Estimativa:** julgamento
- **Bloqueada por:** TASK-01
