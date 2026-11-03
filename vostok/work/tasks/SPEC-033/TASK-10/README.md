# TASK-10: Prints, perguntas, commits e limpeza no painel

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-033
- **Status:** approved
- **Onda:** 4
- **Dependências:** TASK-05, TASK-08

## Arquivos

- [Execução](execucao.md): o que ler, mudar e os passos.
- [Verificação](verificacao.md): baseline, comandos e evidências.
- [Critérios](criterios.md): quando a tarefa está concluída.
- [Resumo executivo](executivo.md)
- [Detalhes técnicos](tecnico.md)
- [Registro](registro.md): etapas da execução e histórico de ajustes.

## Card

- **Título:** Prints, perguntas, commits e limpeza no painel
- **Tipo:** historia
- **Descrição:** Conferência de prints na tarefa, perguntas e commits apontados nas pendências e "Limpar pedidos arquivados" nas Configurações.
- **Critérios de aceite:**
  - [ ] Na página da tarefa em execução ou em validação, cada print mostra se está conferido e oferece "Aceitar" e "Rejeitar"; rejeitar pede confirmação, e as duas gravam a conferência pelo painel (RF-06, CT-07, FA-02).
  - [ ] As pendências mostram os commits apontados pelo `vostok commit check`, com "Liberar commits" e "Desfazer commits", e desfazer só grava o pedido depois da confirmação (RF-07, CT-08).
  - [ ] Responder uma pergunta envia o id dela, e a resposta repetida mostra o aviso (RF-06, CB-06).
  - [ ] Nas Configurações, "Limpar pedidos arquivados" mostra a lista antes, só apaga ao confirmar e diz quando não há o que limpar (RF-11, CT-12, CB-05, RNF-03).
  - [ ] Os botões, os diálogos e as confirmações novos são achados por papel e rótulo (CT-18, RNF-04).
- **Rótulos:** SPEC-033, padrao
- **Estimativa:** padrao
- **Bloqueada por:** TASK-05, TASK-08
