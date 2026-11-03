# TASK-11: Skills perguntando pelo painel

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-033
- **Status:** approved
- **Onda:** 3
- **Dependências:** TASK-04

## Arquivos

- [Execução](execucao.md): o que ler, mudar e os passos.
- [Verificação](verificacao.md): baseline, comandos e evidências.
- [Critérios](criterios.md): quando a tarefa está concluída.
- [Resumo executivo](executivo.md)
- [Detalhes técnicos](tecnico.md)
- [Registro](registro.md): etapas da execução e histórico de ajustes.

## Card

- **Título:** Skills perguntando pelo painel
- **Tipo:** tarefa
- **Descrição:** A `process-requests` abre pergunta em vez de recusar, as skills de conversa gravam a rodada por pedido, e a finish-task e a approve-commit aceitam o painel.
- **Critérios de aceite:**
  - [ ] A process-requests abre a pergunta e conclui o pedido como "aguardando a pergunta P-NNN" em vez de recusá-lo, e atende o pedido de resposta rodando de novo a skill de origem (RF-02, CT-02).
  - [ ] As skills de conversa, quando rodam por um pedido, gravam a rodada de perguntas pelo `vostok perguntas abrir` (RF-04, CT-04).
  - [ ] A finish-task aceita a conferência de print feita no painel, e a approve-commit atende `liberar-commits` e `desfazer-commits` com `--pedido` (RF-02).
  - [ ] As skills alteradas continuam válidas no pacote embutido (RF-02, RF-04).
- **Rótulos:** SPEC-033, padrao
- **Estimativa:** padrao
- **Bloqueada por:** TASK-04
