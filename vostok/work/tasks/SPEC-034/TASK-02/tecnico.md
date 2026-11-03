<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** O painel passa a ler o campo `plan_outdated` que a TASK-01 acrescenta ao lado de `plan` na visão geral e em `GET /api/specs/<ID>`, calculado pela regra da CLI (`plan_is_outdated`), sem repetir a regra no frontend (decisão 7). `specActions` ganha o parâmetro `planOutdated` (padrão `false`, para as chamadas atuais e os testes antigos) e escolhe as ações de plano assim:

| Estado | Ações de plano |
|---|---|
| Plano em dia | "Aprovar plano" (primária) e "Ajustar plano" (secundária, texto obrigatório, pedido `ajustar-plano`) |
| Plano desatualizado, spec `approved` | "Refazer o plano" (primária, pedido `planejar-spec`) |
| Plano desatualizado, spec `refined` | nenhuma; só "Aprovar spec" e "Pedir ajuste" (CB-01, decisão 8) |
| Sem plano, spec `approved` | "Planejar tarefas", como hoje (CB-02) |

Pendências (`attentionItems`) só lista "Aprovar plano" com o plano em dia; não ganha item novo. `taskActions` oferece o `finish-task` também em `validating`; a finish-task diz o que falta quando há print sem conferência (CB-03, decisão 3). A ajuda de `approve-spec` e `approve-task` e a mensagem de sucesso citam `work/avais/<SPEC>.log` (na mensagem, com o ID real da spec), decisão 10.

**Decisões locais (UI/UX).**
- Kinds novos `redo-plan` e `adjust-plan`, em vez de reaproveitar `plan-spec` para "Refazer o plano": o rótulo e a ajuda precisam dizer por que o plano é refeito, e o `plan-spec` continua para a spec sem plano. Os dois kinds gravam pedidos que a CLI já aceita (`apps/cli/src/painel/actions.rs`).
- O aviso aparece em três lugares: um `Badge` `warning` "Plano desatualizado" no cabeçalho (visível em todas as visões), a linha "Plano: Desatualizado: a spec mudou depois do plano." na coluna lateral do documento e, na visão do plano, o título "Plano desatualizado" com o motivo e o que fazer no lugar do bloco "Revisão". O tom `warning` segue a regra do `Badge` para pendência comum, sem `danger`. Um banner novo foi descartado: não há componente de aviso no `ui/`, e a persona não cria identidade nova.
- O botão de finalizar mantém o rótulo atual do `finish-task`, "Pedir finalização", em `in_progress` e em `validating`. Trocar o rótulo para "Finalizar" foi descartado: mudaria a tarefa em execução também, e os rótulos das ações estão no escopo da SPEC-035 ("uma ação principal"). O "Finalizar" da spec é a ação `finish-task`.
- "Ajustar plano" usa o rótulo padrão do campo, "O que ajustar", como "Pedir ajuste" da spec e da tarefa.
- O texto `how` é fixo por tipo de ação, por isso a ajuda cita `work/avais/<SPEC>.log` literal; só a mensagem de sucesso, que conhece a spec, usa o ID real.

**Evidências visuais.** O print automático (`apps/painel/scripts/print.sh`) sobe o painel na raiz deste repositório, e aqui não há estado real e estável para os casos: o único plano aberto (SPEC-034) é removido na aprovação, e nenhuma spec fica com plano desatualizado; a tarefa em `validating` só existiria durante a própria finalização desta tarefa, e o print é tirado antes de ela chegar lá. Um print da SPEC-034 planejada não prova nada da entrega. Por isso as duas evidências de imagem são `(manual)`, tiradas no projeto de demonstração com o binário gerado no passo 9 (bundle novo e API da TASK-01), com o estado e o que procurar descritos em cada uma; o comportamento fica comprovado também pelos testes de componente da evidência automática.

**Arquivos fora dos componentes do plano.** `apps/painel/src/shared/api.ts` (o tipo do campo `plan_outdated`) e `apps/painel/src/features/task/task-page.test.tsx` (CT-07 pede enviar o diálogo de finalizar, que fica na página da tarefa). `task-page.tsx` não muda: a barra vem de `taskActions`.

**Riscos.**
- Sem a TASK-01 o campo vem `undefined` e o painel trata o plano como em dia (oferece "Aprovar plano", como hoje); a ordem das ondas evita isso.
- O teste `shows the index, the metadata and the tasks` estourou o tempo do `findByRole` uma vez no planejamento, quando a pasta `src/features/specs` entrou junto por prefixo. O baseline e a verificação usam `src/features/spec/` e `src/features/task/` com a barra final; nas três execuções seguidas a saída foi igual.

**Arquivos em comum com a onda.** Nenhum: a TASK-02 é a única da onda 2. A TASK-03 (onda 1) muda a mensagem `PANEL_WITHOUT_AVAL` da CLI pelo mesmo RF-07, em `apps/cli/src/commands/task.rs`, sem arquivo em comum.

**Referências da spec.** RF-02, RF-03, RF-04, RF-05, RF-07; CT-02 a CT-07, CT-09; CB-01, CB-03; decisões 1, 2, 3, 7, 8 e 10; CA-01, CA-02, CA-03 e CA-05.
