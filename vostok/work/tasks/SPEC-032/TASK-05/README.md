# TASK-05: Servir as atividades de todas as fontes

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-032
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

- **Título:** Servir as atividades de todas as fontes
- **Tipo:** tarefa
- **Descrição:** Monta as atividades do registro, dos históricos de ajuste, dos pedidos e do `avais.log` em `GET /api/atividades`, com a chamada no cliente.
- **Critérios de aceite:**
  - [ ] `GET /api/atividades` traz tarefas (abertas e arquivadas), specs, planos, pedidos e avais, da mais nova para a mais antiga (RF-07, CT-09, CB-04).
  - [ ] A rota filtra por `limite`, `tipo` e `spec` e ignora avais malformados (RF-07, CB-03).
  - [ ] A rota responde em até 500 ms com 1.000 atividades (RNF-01, CT-10).
- **Rótulos:** SPEC-032, padrao
- **Estimativa:** padrao
- **Bloqueada por:** TASK-01
