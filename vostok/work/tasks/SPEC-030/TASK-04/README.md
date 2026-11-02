# TASK-04: Criar o comando de pedidos

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

- **Título:** Criar o comando de pedidos
- **Tipo:** tarefa
- **Descrição:** `vostok pedidos` com `listar`, `iniciar`, `concluir` e `recusar`.
- **Critérios de aceite:**
  - [x] `vostok pedidos` lista os pedidos e os leva de `pendente` a `em_andamento` e a `feito` ou `recusado`, gravando o resultado e a data (RF-13, CT-14).
  - [x] Uma transição fora da ordem é recusada sem mudar o arquivo (CB-05).
- **Rótulos:** SPEC-030, padrao
- **Estimativa:** padrao
- **Bloqueada por:** -
