<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Só texto de skill, no pacote distribuído `packages/harness/skills/default/`, embutido no binário pelo `cargo build`. São oito `SKILL.md`. A regra central fica na `process-requests` (risco "Skills de conversa com dois destinos de pergunta" da spec):
- a `process-requests` avisa a skill de que roda por um pedido;
- a skill de conversa grava a rodada pelo `vostok perguntas abrir PED-NNN`, da TASK-04, e para;
- a `process-requests` conclui o pedido com "aguardando a pergunta P-NNN" (RF-02).

O pedido de resposta, criado pela ação `responder-pergunta` da TASK-08, roda de novo a skill do pedido de origem, com o texto original e as respostas (RF-03). Quando o pedido de origem foi recusado, o de resposta também é recusado (CB-01).

**Decisões.**
- **Regra por skill, com o mesmo texto.** Cada skill de conversa ganha uma regra própria, com o próximo número, e o acréscimo no passo da rodada. Alternativa descartada: só a regra na `process-requests`. Quem segue a skill de conversa leria "PARE e pergunte" sem saber do outro destino.
- **Rodadas da refine-spec pelo painel.** São contadas pelas perguntas já respondidas da mesma spec na cadeia de pedidos (RN-03, FA-03). Alternativa descartada: um contador no pedido, que mudaria o formato do arquivo.
- **Prints num pedido de finalização.** O pedido não fica recusado nem pergunta no chat: é concluído como "aguardando a conferência dos prints no painel". Um novo `finalizar-tarefa` segue depois da conferência (RF-06, RN-01). Print rejeitado no painel bloqueia a tarefa, como na FA-02.
- **Commits por pedido.** `liberar-commits` e `desfazer-commits` dispensam a pergunta do passo 3 da approve-commit, porque a ação já é a decisão, e passam `--pedido` para a trava da CLI da TASK-08 (RF-07, RN-02, decisão 12).
- **Nomes de pasta.** Os textos não citam nomes de pasta fixos, porque eles vêm da configuração (RF-16, RN-06).
- **Cópia instalada.** A cópia em `vostok/skills/default/` não entra em "Mudar": ela é atualizada pelo `vostok update` ([project/architecture-08]), como nas specs anteriores.

**Verificação.**
- Não há validador de conteúdo por skill além do `ct13_skills_and_skill_templates_call_vostok` de `apps/cli/tests/base.rs`, que confere o `context load`, o `allowed-tools` e a ausência de scripts.
- O texto entregue é provado por diff, e o comportamento por um cenário manual com o agente (CT-02, CT-04), como o `loop-pedido.txt` da SPEC-030.

**Riscos.**
- O texto exato do `vostok perguntas abrir` (ID impresso e pastas) vem da TASK-04: confira a saída do comando antes de escrever o passo 1.
- A ação `responder-pergunta` e o texto `Resposta à pergunta P-NNN do pedido PED-NNN.` vêm da TASK-08, da mesma onda. Os dois lados precisam usar o mesmo texto.

**Arquivos em comum na onda 3.** Nenhum: as outras tarefas da onda mudam só código da CLI.

**Referências da spec.** RF-02, RF-03, RF-04, RF-06, RF-07, RF-16, RN-01, RN-02, RN-03, RN-06, CT-02, CT-04, CB-01, FA-02, FA-03; decisões 9 e 12.
