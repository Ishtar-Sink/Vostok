# TASK-05: Arquivar com a spec e limpar

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-033
- **Status:** approved
- **Onda:** 3
- **Dependências:** TASK-02, TASK-03, TASK-04

## Arquivos

- [Execução](execucao.md): o que ler, mudar e os passos.
- [Verificação](verificacao.md): baseline, comandos e evidências.
- [Critérios](criterios.md): quando a tarefa está concluída.
- [Resumo executivo](executivo.md)
- [Detalhes técnicos](tecnico.md)
- [Registro](registro.md): etapas da execução e histórico de ajustes.

## Card

- **Título:** Arquivar com a spec e limpar
- **Tipo:** tarefa
- **Descrição:** O `vostok finalize archive` leva pedidos, perguntas e avais da spec, e o `vostok pedidos prune` limpa com confirmação.
- **Critérios de aceite:**
  - [ ] Arquivar a spec leva os pedidos concluídos, as perguntas e o log de avais dela para o arquivo morto, e deixa os de outras specs (RF-10, CT-11).
  - [ ] O log de avais vai para o arquivo morto com o mesmo conteúdo, sem linha reescrita (RF-10, RN-05).
  - [ ] Uma spec sem pedidos é arquivada com a mesma saída de antes (RF-10).
  - [ ] O `prune` sem `--confirmar` só lista e não apaga nada; com ele, apaga os listados e nunca os avais (RF-11, CT-12).
  - [ ] Pedidos sem spec só são limpos depois de 30 dias concluídos, e o `prune` sem nada a apagar diz que não há o que limpar (RF-11, CB-02, CB-05).
- **Rótulos:** SPEC-033, julgamento
- **Estimativa:** julgamento
- **Bloqueada por:** TASK-02, TASK-03, TASK-04
