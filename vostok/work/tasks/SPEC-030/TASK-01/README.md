# TASK-01: Ler o aval e a chave da trava

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-030
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

- **Título:** Ler o aval e a chave da trava
- **Tipo:** tarefa
- **Descrição:** Módulo `aval` com a leitura do `avais.log` em 4 e 5 colunas, do estado do pedido e da chave `approval_lock`.
- **Critérios de aceite:**
  - [x] O `avais.log` é lido nos formatos de 4 e 5 colunas, e uma linha malformada é apontada sem valer como aval (RF-02, CT-02, CB-02, CB-03).
  - [x] A chave `approval_lock` vem desligada e, sem ela, a trava fica desligada (RF-01).
- **Rótulos:** SPEC-030, padrao
- **Estimativa:** padrao
- **Bloqueada por:** -
