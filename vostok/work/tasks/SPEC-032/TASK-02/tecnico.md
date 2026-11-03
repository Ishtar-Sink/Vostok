<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**O que foi feito.**
- **`apps/painel/src/shared/status.ts`:** a coluna `running` ("Em execução") de `BOARD_COLUMNS` passa a reunir só `in_progress`. Uma coluna nova, `{ id: 'validating', label: 'Em validação', statuses: ['validating'] }`, entra logo depois dela e antes de "Bloqueadas". O quadro passa a ter seis colunas.
- **`apps/painel/src/features/board/board.test.tsx`:** teste novo, "separa em validação de em execução", que confere a ordem das colunas e uma tarefa `validating` fora de "Em execução". O teste falhou antes da mudança e passa agora. O import passou a incluir `within`.

**Decisões.** A coluna nova é só de dados em `BOARD_COLUMNS`. O componente do quadro já monta as colunas a partir dessa lista, então nenhum componente mudou.

**Verificação.**
- Os testes do quadro passam (4 de 4) e o `tsc --noEmit` não acusa nada. A regressão é igual ao baseline.
- O print `quadro.png`, a 1440x900, foi conferido pelo usuário.

**Pontos de atenção (DT-017).**
- "Concluídas" não cabe em 1440 px.
- O título vindo do `estado.yml` aparece com as aspas escapadas.
- "Em validação" não tem a cor de status no título.

**Referências da spec.** RF-02, CT-02.
