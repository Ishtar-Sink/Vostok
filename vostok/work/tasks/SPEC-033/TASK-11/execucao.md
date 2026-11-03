### Ler

- `vostok/work/specs/refined/SPEC-033-operar-o-projeto-de-ponta-a-ponta-pelo-painel-v0-4-0.md` (RF-01 a RF-04, RF-06, RF-07, RN-01, RN-02, RN-03, CB-01, FA-02, FA-03, "Fluxo principal", decisões 9 e 12)
- `docs/estudos/painel-ponta-a-ponta.md` (seções "Processos a operar pelo painel" e "Perguntas do agente")
- `packages/harness/skills/default/process-requests/SKILL.md`
- `packages/harness/skills/default/create-spec/SKILL.md`
- `packages/harness/skills/default/refine-spec/SKILL.md`
- `packages/harness/skills/default/adjust-spec/SKILL.md`
- `packages/harness/skills/default/adjust-plan/SKILL.md`
- `packages/harness/skills/default/adjust-task/SKILL.md`
- `packages/harness/skills/default/finish-task/SKILL.md`
- `packages/harness/skills/default/approve-commit/SKILL.md`
- `packages/harness/templates/responses/question.md`
- `apps/cli/src/commands/perguntas.rs` (uso e saída do `vostok perguntas abrir`, da TASK-04)
- `apps/cli/tests/base.rs` (testes ct13_skills_and_skill_templates_call_vostok e create_spec_skill_creates_the_draft_without_refining)

### Mudar

- `packages/harness/skills/default/process-requests/SKILL.md` (alterar): abre a pergunta em vez de recusar, atende o pedido de resposta e as ações novas (RF-02, RF-03).
- `packages/harness/skills/default/create-spec/SKILL.md` (alterar): por um pedido, a pergunta vai para `vostok perguntas abrir` (RF-04).
- `packages/harness/skills/default/refine-spec/SKILL.md` (alterar): por um pedido, a rodada vai para `vostok perguntas abrir`, contando as rodadas já respondidas (RF-04, RN-03).
- `packages/harness/skills/default/adjust-spec/SKILL.md` (alterar): por um pedido, a rodada vai para `vostok perguntas abrir` (RF-04).
- `packages/harness/skills/default/adjust-plan/SKILL.md` (alterar): por um pedido, a rodada vai para `vostok perguntas abrir` (RF-04).
- `packages/harness/skills/default/adjust-task/SKILL.md` (alterar): por um pedido, a rodada vai para `vostok perguntas abrir` (RF-04).
- `packages/harness/skills/default/finish-task/SKILL.md` (alterar): a regra finish-task-09 aceita a conferência de print feita no painel (RF-06, RN-01).
- `packages/harness/skills/default/approve-commit/SKILL.md` (alterar): atende os pedidos `liberar-commits` e `desfazer-commits` com `--pedido` (RF-07, RN-02).

### Passos

Regras para todos os passos: em cada `SKILL.md`, suba a versão menor do `metadata.version` (ex.: `1.1.0` vira `1.2.0`) e acrescente `vostok perguntas abrir` ao `metadata.scripts` das skills que passam a usá-lo. Não renumere regras existentes: regra nova recebe o próximo número. Os nomes de pasta vêm da configuração: escreva "a pasta de perguntas respondidas" e "a pasta de pedidos", nunca `aberta/` ou `pendente/` fixos (RF-16). O comando de pergunta citado nas skills é sempre:

```bash
vostok perguntas abrir PED-NNN <<'TXT'
<as perguntas e as opções no formato do template `question`>
TXT
```

1. `packages/harness/skills/default/process-requests/SKILL.md` (alterar):
   - Em "Termos", "uma das 11" passa a "uma das 16". Acrescente o termo **Pergunta** (a rodada de perguntas da skill gravada pelo `vostok perguntas abrir`, que o usuário responde no painel) e o termo **Pedido de resposta** (o pedido `responder-pergunta`, cujo texto começa por `Resposta à pergunta P-NNN do pedido PED-NNN.`).
   - Troque [process-requests-04] por: `O texto de ## Pedido é a mensagem do usuário. Quando a skill da ação precisa perguntar (pedido ambíguo, rodada de perguntas, escolha entre opções), abra a pergunta com vostok perguntas abrir e conclua o pedido com "aguardando a pergunta P-NNN"; nunca pergunte no chat nem recuse por isso. Só a recusa da CLI ou um impedimento (template blocker) viram recusado, com o motivo.`
   - Troque [process-requests-05] por: `A conferência de um print é do usuário, feita no painel; nunca execute --reviewed por um pedido.`
   - Acrescente [process-requests-09]: `Ao rodar a skill de um pedido, avise-a de que roda por um pedido (o ID PED-NNN): as skills de conversa gravam a rodada de perguntas pelo vostok perguntas abrir.`
   - No passo 3, acrescente ao mapa: `criar-spec` → `create-spec` (o texto é o assunto); `refinar-spec` → `refine-spec`; `executar-onda` → `execute-task`, para a onda do início do texto (`Onda N.`); `liberar-commits` → `approve-commit`, liberar, com `--pedido PED-NNN`; `desfazer-commits` → `approve-commit`, desfazer, com `--pedido PED-NNN`. Troque a linha de `responder-pergunta` por: `a skill do pedido de origem, de novo (ver abaixo)`.
   - Ainda no passo 3, troque a frase "Se a skill parar com `blocker` ou `question`..." por dois parágrafos: (a) "Se a skill parar com `question`, grave as perguntas com `vostok perguntas abrir PED-NNN` (o bloco acima) e, no passo 4, conclua o pedido com `aguardando a pergunta P-NNN`, com o ID que o comando imprimiu. Se parar com `blocker` ou a CLI recusar o comando, recuse o pedido com o motivo." (b) "Pedido de resposta: leia a pergunta citada na pasta de perguntas respondidas (o front-matter traz o pedido de origem) e o pedido de origem. Se o de origem foi recusado, recuse este com o motivo `o pedido de origem PED-NNN foi recusado` (CB-01). Senão, rode de novo a skill da ação do pedido de origem, com a spec e a tarefa dele e, como mensagem do usuário, o `## Pedido` original seguido das perguntas e da `## Resposta`."
2. `packages/harness/skills/default/create-spec/SKILL.md` (alterar): acrescente [create-spec-07]: `Rodando por um pedido do painel (process-requests), toda pergunta desta skill vai pelo vostok perguntas abrir PED-NNN, no formato do template question, em vez do chat; depois, PARE.` Nos passos 2 e 3, depois de cada "**PARE** e pergunte ... com o template `question`", acrescente: "Por um pedido, grave a pergunta com `vostok perguntas abrir` ([create-spec-07])."
3. `packages/harness/skills/default/refine-spec/SKILL.md` (alterar): acrescente [refine-spec-09]: `Rodando por um pedido do painel (process-requests), a rodada de perguntas vai pelo vostok perguntas abrir PED-NNN, no formato do template question, com a tabela de inferências, em vez do chat; depois, PARE. A rodada é contada pelas perguntas já respondidas da mesma spec nesta cadeia de pedidos, e o limite de [refine-spec-03] vale igual.` No passo 5, depois de "**PARE** e aguarde a resposta.", acrescente: "Por um pedido, grave a rodada com `vostok perguntas abrir` ([refine-spec-09]); na 2ª rodada respondida pelo painel, o que continuar em aberto vai para as pendências (RN-03)."
4. `packages/harness/skills/default/adjust-spec/SKILL.md` (alterar): acrescente [adjust-spec-09] com o mesmo texto de [create-spec-07], trocando "toda pergunta desta skill" por "a rodada de perguntas"; no passo da rodada (a linha "Envie numa única mensagem, com o template `question`, uma pergunta para cada ajuste..."), acrescente: "Por um pedido, grave a rodada com `vostok perguntas abrir` ([adjust-spec-09])."
5. `packages/harness/skills/default/adjust-plan/SKILL.md` (alterar): o mesmo do passo 4, com a regra [adjust-plan-09].
6. `packages/harness/skills/default/adjust-task/SKILL.md` (alterar): o mesmo do passo 4, com a regra [adjust-task-13], no passo "6. Rodada de perguntas".
7. `packages/harness/skills/default/finish-task/SKILL.md` (alterar): troque [finish-task-09] por: `A conferência de um print é sempre do usuário, nunca do agente: no chat, execute --reviewed só com a confirmação dele; pelo painel, a conferência gravada com a origem painel vale como a dele (RN-01). Nunca execute --reviewed por um pedido do painel.` No item "Prints sem conferência listados na saída" do passo das evidências, acrescente: "Por um pedido do painel, não pergunte: conclua o pedido com `aguardando a conferência dos prints no painel` e a lista dos prints; depois de conferidos, um novo `finalizar-tarefa` segue daqui. Print rejeitado no painel deixa a tarefa bloqueada: siga a retomada da tarefa bloqueada."
8. `packages/harness/skills/default/approve-commit/SKILL.md` (alterar): em [approve-commit-01], depois de "(ou num pedido `fazer-commit` do painel, que vale como o pedido dele)", acrescente "ou nos pedidos `liberar-commits` e `desfazer-commits`". No passo 3, acrescente: "Por um pedido `liberar-commits` ou `desfazer-commits`, a decisão é a ação do pedido: vá ao passo 4 sem perguntar." No passo 4, acrescente: "Com a trava ligada (`approval_lock: true`), passe `--pedido PED-NNN` no `vostok commit approve` (pedido `liberar-commits` ou `fazer-commit`) e no `vostok commit revert` (pedido `desfazer-commits`)."
9. `packages/harness` (conferir): `cargo build --manifest-path apps/cli/Cargo.toml` (o pacote é embutido no binário) e `cargo test --manifest-path apps/cli/Cargo.toml --test base`. Confira: sem falhas; o `ct13_skills_and_skill_templates_call_vostok` continua aceitando as 22 skills. Não edite `vostok/skills/default/`: a cópia instalada deste projeto é atualizada pelo `vostok update` ([project/architecture-08]).
