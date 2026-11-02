# TASK-09: Vigiar os pedidos para o Codex

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-030
- **Status:** approved
- **Onda:** 3
- **Dependências:** TASK-04, TASK-08

## Arquivos

- [Execução](execucao.md): o que ler, mudar e os passos.
- [Verificação](verificacao.md): baseline, comandos e evidências.
- [Critérios](criterios.md): quando a tarefa está concluída.
- [Resumo executivo](executivo.md)
- [Detalhes técnicos](tecnico.md)
- [Registro](registro.md): etapas da execução e histórico de ajustes.

## Card

- **Título:** Vigiar os pedidos para o Codex
- **Tipo:** tarefa
- **Descrição:** `vostok pedidos vigiar --agente codex`, que dispara `codex exec` para cada pedido pendente, um por vez.
- **Critérios de aceite:**
  - [ ] `vostok pedidos vigiar --agente codex` processa os pedidos pendentes com `codex exec`, um por vez e em ordem, sem precisar do painel aberto (RF-15, CT-16, RNF-04).
  - [ ] Sem o `codex` no `PATH`, o comando para com a instrução de instalar (CB-06).
- **Rótulos:** SPEC-030, padrao
- **Estimativa:** padrao
- **Bloqueada por:** TASK-04, TASK-08
