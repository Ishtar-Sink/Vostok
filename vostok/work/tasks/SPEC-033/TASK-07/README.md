# TASK-07: Disparo sob demanda

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-033
- **Status:** done
- **Onda:** 3
- **Dependências:** TASK-02

## Arquivos

- [Execução](execucao.md): o que ler, mudar e os passos.
- [Verificação](verificacao.md): baseline, comandos e evidências.
- [Critérios](criterios.md): quando a tarefa está concluída.
- [Resumo executivo](executivo.md)
- [Detalhes técnicos](tecnico.md)
- [Registro](registro.md): etapas da execução e histórico de ajustes.

## Card

- **Título:** Disparo sob demanda
- **Tipo:** tarefa
- **Descrição:** Com `request_dispatch`, o painel chama o agente para cada pedido pendente, um por vez, reaproveitando o `vigiar`.
- **Critérios de aceite:**
  - [x] Com `request_dispatch: codex` ou `claude`, o painel chama o agente uma vez para cada pedido pendente (RF-13, CT-14).
  - [x] Nunca há dois agentes rodando ao mesmo tempo (RN-04).
  - [x] Com o disparo desligado (padrão), nenhum agente é chamado (RF-13, CT-14).
  - [x] Com o agente fora do PATH, o painel avisa e o pedido continua pendente, sem ser recusado (CB-04).
  - [x] O `vostok pedidos vigiar` continua funcionando como antes (RF-13).
- **Rótulos:** SPEC-033, julgamento
- **Estimativa:** julgamento
- **Bloqueada por:** TASK-02
