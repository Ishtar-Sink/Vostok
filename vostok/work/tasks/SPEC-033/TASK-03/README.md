# TASK-03: Avais por spec

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-033
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

- **Título:** Avais por spec
- **Tipo:** tarefa
- **Descrição:** Um log por spec, só de acréscimo, migração do `avais.log` e o `vostok avais <ID>`.
- **Critérios de aceite:**
  - [ ] Um aval de uma spec ou de uma tarefa dela fica no arquivo da spec na pasta de avais, e um sem spec no arquivo à parte, sempre por acréscimo (RF-09, CT-10).
  - [ ] `vostok avais <ID>` mostra os avais de uma spec, de uma tarefa ou de um pedido, do mais novo para o mais antigo, e diz quando não há nenhum (RF-09, CT-10, RNF-02).
  - [ ] O `avais.log` antigo é dividido por spec uma vez, sem mudar as linhas, e as malformadas vão para o arquivo sem spec sem quebrar a migração (RF-09, CT-17, CB-03).
  - [ ] Com a pasta de avais e o arquivo sem spec configurados com outros nomes, os avais vão para eles e o `vostok avais` os lê (RF-09, CT-19).
  - [ ] A trava de aval, as aprovações do painel e as atividades continuam funcionando com os avais por spec (RF-09).
- **Rótulos:** SPEC-033, julgamento
- **Estimativa:** julgamento
- **Bloqueada por:** TASK-01
