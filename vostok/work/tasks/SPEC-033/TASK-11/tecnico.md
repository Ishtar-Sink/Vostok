<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Foi só texto de skill, no pacote distribuído `packages/harness/skills/default/`, embutido no binário pelo `cargo build`. Foram alterados oito `SKILL.md`, cada um com a versão menor subida. A regra central ficou na `process-requests` (risco "Skills de conversa com dois destinos de pergunta" da spec):
- a `process-requests` avisa a skill de que ela roda por um pedido (regra [process-requests-09]);
- a skill de conversa grava a rodada pelo `vostok perguntas abrir PED-NNN`, da TASK-04, e para;
- a `process-requests` conclui o pedido com "aguardando a pergunta P-NNN" (regra [process-requests-04], RF-02).

O pedido de resposta, criado pela ação `responder-pergunta` da TASK-08, roda de novo a skill do pedido de origem, com o `## Pedido` original seguido das perguntas e da `## Resposta` (RF-03). Quando o pedido de origem foi recusado, o de resposta também é recusado com o motivo `o pedido de origem PED-NNN foi recusado` (CB-01).

O que mudou em cada arquivo:
- `process-requests/SKILL.md` (versão 1.2.0): termos (de "uma das 11" para "uma das 16"), com os termos **Pergunta** e **Pedido de resposta**; regras [process-requests-04] e [process-requests-05] trocadas; regra [process-requests-09] nova; mapa do passo 3 com `criar-spec`, `refinar-spec`, `executar-onda`, `liberar-commits` e `desfazer-commits`, e `responder-pergunta` apontando para a skill do pedido de origem; passo 3 dividido em dois parágrafos (pergunta gravada ou recusa, e pedido de resposta).
- `create-spec/SKILL.md`: regra [create-spec-07] nova e, nos passos 2 e 3, a gravação da pergunta com `vostok perguntas abrir` quando o pedido vem do painel.
- `refine-spec/SKILL.md`: regra [refine-spec-09] nova, com a rodada gravada pelo comando e contada pelas perguntas já respondidas da mesma spec na cadeia de pedidos (o limite de [refine-spec-03] vale igual); no passo 5, a rodada gravada e, na 2ª rodada respondida pelo painel, o que continuar em aberto vai para as pendências (RN-03).
- `adjust-spec/SKILL.md`: regra [adjust-spec-09] nova e gravação da rodada no passo da rodada de perguntas.
- `adjust-plan/SKILL.md`: regra [adjust-plan-09] nova e gravação da rodada, no mesmo formato do `adjust-spec`.
- `adjust-task/SKILL.md`: regra [adjust-task-13] nova e gravação da rodada no passo "6. Rodada de perguntas".
- `finish-task/SKILL.md` (regra [finish-task-09] trocada): a conferência de print feita no painel vale como a do usuário (RN-01), e nunca se executa `--reviewed` por um pedido do painel. Nos prints sem conferência, o pedido é concluído como "aguardando a conferência dos prints no painel", com a lista dos prints; depois da conferência, um novo `finalizar-tarefa` segue daqui. Print rejeitado no painel deixa a tarefa bloqueada, com a retomada da tarefa bloqueada (FA-02).
- `approve-commit/SKILL.md` (versão 1.1.0): regra [approve-commit-01] cobre também os pedidos `liberar-commits` e `desfazer-commits`; no passo 3, esses pedidos vão direto ao passo 4, sem pergunta, porque a ação já é a decisão; no passo 4, com a trava de aprovação ligada, `--pedido PED-NNN` vai no `vostok commit approve` (pedidos `liberar-commits` e `fazer-commit`) e no `vostok commit revert` (pedido `desfazer-commits`) (RF-07, RN-02).

A cópia em `vostok/skills/default/` não foi alterada: ela é atualizada pelo `vostok update` ([project/architecture-08]), como nas specs anteriores.

**Decisões.**
- **Regra por skill, com o mesmo texto.** Cada skill de conversa ganhou uma regra própria, com o próximo número, e o acréscimo no passo da rodada. Alternativa descartada: só a regra na `process-requests`. Quem segue a skill de conversa leria "PARE e pergunte" sem saber do outro destino.
- **Rodadas da refine-spec pelo painel.** São contadas pelas perguntas já respondidas da mesma spec na cadeia de pedidos (RN-03, FA-03). Alternativa descartada: um contador no pedido, que mudaria o formato do arquivo.
- **Prints num pedido de finalização.** O pedido não fica recusado nem pergunta no chat: é concluído como "aguardando a conferência dos prints no painel". Um novo `finalizar-tarefa` segue depois da conferência (RF-06, RN-01). Print rejeitado no painel bloqueia a tarefa, como na FA-02.
- **Commits por pedido.** `liberar-commits` e `desfazer-commits` dispensam a pergunta do passo 3 da approve-commit, porque a ação já é a decisão, e passam `--pedido` para a trava da CLI da TASK-08 (RF-07, RN-02, decisão 12).
- **Nomes de pasta.** Os textos não citam nomes de pasta fixos: escrevem "a pasta de perguntas respondidas" e "a pasta de pedidos", porque os nomes vêm da configuração (RF-16, RN-06).

**Verificação.**
- Testes: suítes base com 36 testes ok, incluindo `ct13_skills_and_skill_templates_call_vostok` e `init_update` (saída em `validation/tests.out`). O `ct13` confere o `context load`, o `allowed-tools` e a ausência de scripts; não há outro validador de conteúdo por skill.
- Regressão: 2 comandos iguais ao baseline.
- Evidências: 11 passaram, em `vostok/work/tasks/SPEC-033/TASK-11/evidencias/`. Oito diffs automáticos `skill-<nome>.diff` (um por skill alterada, com `git diff 96cdadb 7b71467 -- <skill>` e a regra nova com `Espera:`), `skill-instalada.txt` (automático) e `pedido-que-pergunta.txt` (manual).
- Comportamento: na demonstração manual, feita pelo usuário num projeto de demonstração com o binário novo, o PED-001 (refine-spec) gravou a pergunta P-001 com `vostok perguntas abrir` e terminou `feito` aguardando a P-001, sem perguntar no chat. A resposta no painel gerou o PED-002 (responder-pergunta), que rodou a refine-spec e deixou a spec refined. O usuário seguiu a demonstração até finalizar a spec pelo painel.

**Ajuste 1 (aprovado pelo usuário).** A primeira evidência automática de skills falhou porque o diff contra o HEAD saía vazio: as skills já estavam no commit 7b71467. Os oito diffs passaram a comparar `96cdadb..7b71467`, com `Espera:` na regra nova. A `skill-instalada.txt` deixou de procurar no `vostok context load --skill`, que devolve só regras e configuração, e passou a fazer grep no `vostok/skills/default/process-requests/SKILL.md` instalado num projeto novo.

**Riscos.** Eram dois, ambos do planejamento:
- O texto exato do `vostok perguntas abrir` (ID impresso e pastas) vinha da TASK-04.
- A ação `responder-pergunta` e o texto `Resposta à pergunta P-NNN do pedido PED-NNN.` vinham da TASK-08, da mesma onda, e os dois lados precisavam usar o mesmo texto. A demonstração mostrou o encadeamento de PED-001 a PED-002 funcionando.

**Feito.** Oito `SKILL.md` alterados, com a versão menor subida em cada um, e a entrega está no commit 7b71467 (onda 3), que entrou antes de a tarefa ser finalizada. Testes, regressão e as 11 evidências passaram, com o ajuste 1 aplicado: os oito diffs passaram a ser `git diff 96cdadb 7b71467 -- <skill>`, com `Espera:` na regra nova, e o `skill-instalada.txt` passou a ler o `SKILL.md` instalado num projeto novo, porque o `vostok context load --skill` não traz o texto da skill. A evidência manual `pedido-que-pergunta.txt` foi feita pelo usuário num projeto de demonstração.

**Pontos de atenção.** Os achados de experiência da demonstração (estado visível, plano desatualizado sem o botão "Ajustar plano", tarefa em validação sem botão para concluir) vão para a SPEC-034 (correções, antes da TASK-13) e para a spec do guia do painel; não são defeitos desta tarefa.

**Arquivos em comum na onda 3.** Nenhum arquivo em comum com as outras tarefas da onda.

**Referências da spec.** RF-02, RF-03, RF-04, RF-06, RF-07, RF-16, RN-01, RN-02, RN-03, RN-06, CT-02, CT-04, CB-01, FA-02, FA-03; decisões 9 e 12.
