# TASK-02: Medir servidor HTTP e frontend no binário

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-027
- **Status:** approved
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

- **Título:** Medir servidor HTTP e frontend no binário
- **Tipo:** tarefa
- **Descrição:** Spike no branch `spike/painel-local` que mede o binário com cada servidor HTTP e com o bundle em React e em Preact, e propõe o limite.
- **Critérios de aceite:**
  - [ ] O spike está na worktree `/home/victor/projetos/ai-hub-painel`, no branch `spike/painel-local`, sem nada na `main` além do documento (CA-13).
  - [ ] O tamanho do binário de cada servidor HTTP e de cada empacotamento está medido contra a base (PQ-02, PQ-03).
  - [ ] O limite do binário está proposto com a justificativa e a variante recomendada (PQ-04).
- **Rótulos:** SPEC-027, mecanica
- **Estimativa:** mecanica
- **Bloqueada por:** -
