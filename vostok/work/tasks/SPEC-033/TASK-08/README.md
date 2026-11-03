# TASK-08: Ações novas do painel na CLI

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

- **Título:** Ações novas do painel na CLI
- **Tipo:** tarefa
- **Descrição:** Pedidos `criar-spec`, `refinar-spec`, `executar-onda`, `liberar-commits` e `desfazer-commits`, e as ações diretas `conferir-print`, `limpar-pedidos` e a resposta nas pastas novas.
- **Critérios de aceite:**
  - [ ] O painel grava os pedidos `criar-spec` (com o assunto), `refinar-spec` e `executar-onda` (com a spec e a onda) e recusa os que vêm sem esses campos (RF-05, base para a TASK-09).
  - [ ] Os pedidos `liberar-commits` e `desfazer-commits` gravam o aval, e a CLI só libera ou desfaz com o pedido e o aval quando a trava está ligada (RF-07, RN-02, CT-08).
  - [ ] Conferir um print no painel grava a conferência com a origem `painel`, no mesmo texto do `vostok task evidence --reviewed`; rejeitar bloqueia a tarefa (RF-06, RN-01, CT-07, FA-02).
  - [ ] Responder a uma pergunta no painel a move para as respondidas e cria o pedido de resposta; a segunda resposta recebe 409 e a primeira fica (RF-03, CT-03, CB-06).
  - [ ] A limpeza pelo painel só lista sem confirmação, só apaga com ela e nunca apaga avais (RF-11, base para a TASK-10).
- **Rótulos:** SPEC-033, padrao
- **Estimativa:** padrao
- **Bloqueada por:** TASK-02, TASK-03, TASK-04
