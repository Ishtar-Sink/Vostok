# TASK-04: Abrir perguntas do agente

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

- **Título:** Abrir perguntas do agente
- **Tipo:** tarefa
- **Descrição:** O `vostok perguntas abrir <PEDIDO>`, que grava a pergunta aberta com o pedido, a spec e a tarefa.
- **Critérios de aceite:**
  - [ ] `vostok perguntas abrir PED-001` grava `perguntas/aberta/P-001.md` com o pedido, a spec, a tarefa e `status: aberta` (RF-01, CT-01).
  - [ ] Cada pergunta nova recebe o número seguinte ao maior já usado, inclusive entre as respondidas (RF-01).
  - [ ] Pedido inexistente, ID inválido e texto vazio são recusados sem gravar nada (RF-01).
  - [ ] Com os nomes de pasta trocados na configuração, a pergunta vai para a pasta configurada e o status continua `aberta` (RF-16).
  - [ ] O código passa no clippy e no rustfmt (RF-01).
- **Rótulos:** SPEC-033, padrao
- **Estimativa:** padrao
- **Bloqueada por:** TASK-01
