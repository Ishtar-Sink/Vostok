<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Foram adicionados três tipos de ação no padrão de `attention.ts` e `action-bar.tsx`: `create-spec` (pedido `criar-spec`, com o assunto obrigatório), `refine-spec` (pedido `refinar-spec`, na spec `draft`) e `execute-wave` (pedido `executar-onda`, na spec `planned`). Os botões usam o `ActionDialog`, que já traz o diálogo acessível, o texto obrigatório, o estado de envio, a mensagem de sucesso (`role="status"`) e a de recusa (`role="alert"`). "Nova spec" ficou no cabeçalho da lista de specs, no molde do "Pedir commit" das pendências (`RequestCommit`, em `inbox-page.tsx`). "Refinar" e "Executar onda" ficaram na barra de ações da spec, montada por `specActions`.

**Próxima onda executável.** Tarefas `approved` com todas as dependências `done` são as prontas. A onda é a menor entre elas, e o pedido leva a spec e a onda (`&onda=N`). O diálogo mostra o título "Onda N: TASK-xx, ...".

**Decisões.**
- "Executar onda" substituiu o item de execução da spec que gravava `executar-tarefa` sem tarefa. Manter os dois foi descartado, porque seriam dois botões para quase a mesma coisa, e o antigo não dizia qual tarefa executar. A execução de uma tarefa continua só em `taskActions`, na página da tarefa.
- A onda é calculada pelo painel, a partir do estado das tarefas, e não escolhida pelo agente. Assim o usuário vê no diálogo o que será executado antes de confirmar.
- "Nova spec" usa o `ActionDialog` com texto obrigatório. Um formulário próprio foi descartado, porque duplicaria o estado de envio e as mensagens.
- `apps/painel/src/features/spec/spec-page.tsx` não mudou: a barra de ações já vem de `specActions`. Mexer na página só para cumprir a lista foi descartado.

**Contrato com a TASK-08.** Os pedidos são `POST /api/pedidos?acao=criar-spec` (corpo: o assunto), `?acao=refinar-spec&spec=SPEC-NNN` e `?acao=executar-onda&spec=SPEC-NNN&onda=N`, todos respondendo `{"id":"PED-NNN"}`. Os nomes dos pedidos e do parâmetro `onda` foram usados como no plano.

**Arquivos fora dos componentes do plano.** Foram alterados `apps/painel/src/shared/action-bar.tsx`, onde os tipos de ação viram pedidos, e os testes `attention.test.ts`, `specs-page.test.tsx` e `spec-page.test.tsx`.

**Arquivos em comum com a TASK-10.** `apps/painel/src/shared/attention.ts` e `apps/painel/src/shared/action-bar.tsx` são compartilhados com a TASK-10, que está na onda 4.

**Riscos.** O print `executar-onda.png` dependia do estado da SPEC-033. Na finalização, a SPEC-033 não tinha tarefa pronta para onda, e o print saiu das evidências (ver Feito).

**Feito.**
- Botões "Nova spec" (lista de specs, com diálogo do assunto), "Refinar" (spec em rascunho) e "Executar onda" (spec planejada, com o título da onda) gravam os pedidos `criar-spec`, `refinar-spec` e `executar-onda`.
- Ajuste 1, aprovado pelo usuário: o print `executar-onda.png` foi removido das evidências. Na finalização, a SPEC-033 real não tinha tarefa aprovada e pronta (TASK-09 e TASK-10 em execução, TASK-13 dependendo delas), então o botão corretamente não aparecia. O critério do "Executar onda" ficou comprovado só pelos testes. O print `nova-spec.png` foi conferido e aceito pelo usuário.
- Testes: 3 em `attention.test.ts`, 2 em `specs-page.test.tsx` e 3 em `spec-page.test.tsx`. `tsc` sem erros e binário de release compilado. Verificação, regressão (2 comandos iguais ao baseline) e evidências passaram.

**Pontos de atenção.**
- O "Executar onda" não tem print. Ele está coberto só pelos testes, como descrito no ajuste 1.
- O parâmetro `onda` confere com a TASK-08: `actions.rs` lê `onda` no `executar-onda` e responde 400 com "onda inválida" quando falta ou é 0.
- O item antigo de execução da spec saiu. Quem quiser executar uma tarefa só tem o caminho da página da tarefa.
- A TASK-13 (versão 0.4.0) depende desta tarefa e da TASK-10.

**Referências da spec.** RF-05, CT-05, CT-06, CT-18, RNF-04, CA-01; contratos "Ações novas de pedido".
