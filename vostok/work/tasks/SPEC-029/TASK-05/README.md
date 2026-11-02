# TASK-05: Implementar a atualização ao vivo

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-029
- **Status:** done
- **Onda:** 2
- **Dependências:** TASK-02

## Arquivos

- [Execução](execucao.md): o que ler, mudar e os passos.
- [Verificação](verificacao.md): baseline, comandos e evidências.
- [Critérios](criterios.md): quando a tarefa está concluída.
- [Resumo executivo](executivo.md)
- [Detalhes técnicos](tecnico.md)
- [Registro](registro.md): etapas da execução e histórico de ajustes.

## Card

- **Título:** Implementar a atualização ao vivo
- **Tipo:** tarefa
- **Descrição:** O WebSocket `/api/ao-vivo` avisa `mudou` às abas quando a versão de `vostok/work/` muda, conferida a cada 500 ms.
- **Critérios de aceite:**
  - [x] O painel avisa todas as abas abertas quando `vostok/work/` muda, em menos de 1 s (RF-05, CT-13, RNF-03, CB-06).
- **Rótulos:** SPEC-029, padrao
- **Estimativa:** padrao
- **Bloqueada por:** TASK-02
