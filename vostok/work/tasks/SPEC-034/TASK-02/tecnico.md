<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Feito.** O painel passou a ler o campo `plan_outdated` (entregue pela TASK-01) em `OpenSpec` e `SpecDetail`, em `shared/api.ts`. `specActions` ganhou o parâmetro `planOutdated` (padrão `false`) e passou a escolher as ações de plano conforme a tabela abaixo. `attentionItems` deixou de listar "Aprovar plano" para plano desatualizado. `taskActions` passou a oferecer "Pedir finalização" também em `validating`. Os kinds `redo-plan` e `adjust-plan` entraram em `action-bar.tsx`, junto da mensagem de sucesso da aprovação, que cita o log de avais da spec. O aviso "Plano desatualizado" entrou em `spec-page.tsx`. Os testes de `attention.test.ts`, `spec-page.test.tsx` e `task-page.test.tsx` foram acrescentados sem mudar os existentes.

**Ajuste 1 (pedido do usuário).** Os prints `plano-desatualizado.png` e `tarefa-em-validacao.png`, antes manuais, passaram a ser automáticos. O script novo `apps/painel/scripts/fixture-fluxo.sh` montou numa pasta temporária um projeto de demonstração com a SPEC-001 aprovada de novo depois do plano e a SPEC-002/TASK-01 em validação. O `apps/painel/scripts/print.sh` ganhou a variável `PAINEL_PASTA`, que indica a pasta do projeto do print; sem ela, usa a raiz do repositório.

**Verificação.** 44 testes do painel passaram e `tsc --noEmit` ficou sem erros. A regressão repetiu os 2 comandos do baseline, com o mesmo resultado. Não há `avais.log` em `apps/painel/src`. Com esta tarefa a SPEC-034 ficou implementada.

**Abordagem.** O painel passou a ler o campo `plan_outdated` que a TASK-01 acrescentou ao lado de `plan` na visão geral e em `GET /api/specs/<ID>`, calculado pela regra da CLI (`plan_is_outdated`), sem repetir a regra no frontend (decisão 7). `specActions` ganhou o parâmetro `planOutdated` (padrão `false`, para as chamadas atuais e os testes antigos) e passou a escolher as ações de plano assim:

| Estado | Ações de plano |
|---|---|
| Plano em dia | "Aprovar plano" (primária) e "Ajustar plano" (secundária, texto obrigatório, pedido `ajustar-plano`) |
| Plano desatualizado, spec `approved` | "Refazer o plano" (primária, pedido `planejar-spec`) |
| Plano desatualizado, spec `refined` | nenhuma; ficaram só "Aprovar spec" e "Pedir ajuste" (CB-01, decisão 8) |
| Sem plano, spec `approved` | "Planejar tarefas", como antes (CB-02) |

Pendências (`attentionItems`) passou a listar "Aprovar plano" só com o plano em dia; não ganhou item novo. `taskActions` passou a oferecer `finish-task` também em `validating`; a finish-task diz o que falta quando há print sem conferência (CB-03, decisão 3). A ajuda de `approve-spec` e `approve-task` e a mensagem de sucesso passaram a citar `work/avais/<SPEC>.log` (na mensagem, com o ID real da spec), decisão 10.

**Decisões locais (UI/UX).**
- Foram criados os kinds `redo-plan` e `adjust-plan`, em vez de reaproveitar `plan-spec` para "Refazer o plano": o rótulo e a ajuda precisam dizer por que o plano é refeito, e o `plan-spec` continuou para a spec sem plano. Os dois kinds gravam pedidos que a CLI já aceita (`apps/cli/src/painel/actions.rs`).
- O aviso apareceu em três lugares: um `Badge` `warning` "Plano desatualizado" no cabeçalho (visível em todas as visões), a linha "Plano: Desatualizado: a spec mudou depois do plano." na coluna lateral do documento e, na visão do plano, o título "Plano desatualizado" com o motivo e o que fazer no lugar do bloco "Revisão". O tom `warning` seguiu a regra do `Badge` para pendência comum, sem `danger`. Um banner novo foi descartado: não há componente de aviso no `ui/`, e a persona não cria identidade nova.
- O botão de finalizar manteve o rótulo atual do `finish-task`, "Pedir finalização", em `in_progress` e em `validating`. Trocar o rótulo para "Finalizar" foi descartado: mudaria a tarefa em execução também, e os rótulos das ações estão no escopo da SPEC-035 ("uma ação principal"). O "Finalizar" da spec é a ação `finish-task`.
- "Ajustar plano" usou o rótulo padrão do campo, "O que ajustar", como "Pedir ajuste" da spec e da tarefa.
- O texto `how` é fixo por tipo de ação, por isso a ajuda citou `work/avais/<SPEC>.log` literal; só a mensagem de sucesso, que conhece a spec, usa o ID real.

**Evidências visuais.** Antes, o `print.sh` subia o painel na raiz deste repositório, onde não havia estado real e estável para os casos: nenhuma spec ficava com plano desatualizado, e uma tarefa em `validating` só existia durante a própria finalização. Pelo ajuste 1 os dois prints passaram de `(manual)` a `(print)`, gerados pelo `fixture-fluxo.sh` e tirados da pasta indicada por `PAINEL_PASTA`. O usuário conferiu e aceitou os dois prints (`plano-desatualizado.png` e `tarefa-em-validacao.png`). O comportamento também está coberto pelos testes de componente.

**Arquivos fora dos componentes do plano.** `apps/painel/src/shared/api.ts` (o tipo do campo `plan_outdated`) e `apps/painel/src/features/task/task-page.test.tsx` (CT-07 pede enviar o diálogo de finalizar, que fica na página da tarefa). `task-page.tsx` não mudou: a barra vem de `taskActions`.

**Arquivos em comum com a onda.** Nenhum com a TASK-03 (onda 1), que muda a mensagem `PANEL_WITHOUT_AVAL` em `apps/cli/src/commands/task.rs`.

**Pontos de atenção.**
- Sem a TASK-01 o campo viria `undefined` e o painel trataria o plano como em dia. A ordem das ondas evitou isso.
- O teste `shows the index, the metadata and the tasks` estourou o tempo do `findByRole` uma vez no planejamento, quando a pasta `src/features/specs` entrou junto por prefixo. Com as pastas `src/features/spec/` e `src/features/task/` (com a barra final), as três execuções seguidas deram a mesma saída.
- O rótulo "Pedir finalização" foi mantido; a troca de rótulos fica para a SPEC-035.
- Os prints dependem da pasta de demonstração do `fixture-fluxo.sh`; sem `PAINEL_PASTA`, o `print.sh` mostra as specs do repositório.

**Referências da spec.** RF-02, RF-03, RF-04, RF-05, RF-07; CT-02 a CT-07, CT-09; CB-01, CB-03; decisões 1, 2, 3, 7, 8 e 10; CA-01, CA-02, CA-03 e CA-05.
