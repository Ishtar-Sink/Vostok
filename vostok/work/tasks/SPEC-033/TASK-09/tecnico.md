<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Três tipos novos de ação no padrão de `attention.ts` e `action-bar.tsx`: `create-spec` (pedido `criar-spec`, com o assunto obrigatório), `refine-spec` (pedido `refinar-spec`, na spec `draft`) e `execute-wave` (pedido `executar-onda`, na spec `planned`). Os botões saem do `ActionDialog`, que já tem o diálogo acessível, o texto obrigatório, o estado de envio, a mensagem de sucesso (`role="status"`) e a de recusa (`role="alert"`). "Nova spec" fica no cabeçalho da lista de specs, no molde do "Pedir commit" das pendências; "Refinar" e "Executar onda" entram na barra de ações da spec, que já vem de `specActions`.

**Próxima onda executável.** Tarefas `approved` com todas as dependências `done` são as prontas; a onda é a menor entre elas, e o pedido leva a spec e a onda (`&onda=N`), com o título "Onda N: TASK-xx, ..." no diálogo.

**Decisões.**
- "Executar onda" substitui o item `execute-ready` da spec, que gravava `executar-tarefa` sem tarefa: descartado manter os dois, porque seriam dois botões para quase a mesma coisa, e o antigo não dizia qual tarefa executar. A execução de uma tarefa só continua na página da tarefa.
- A onda vem do painel, calculada pelo estado das tarefas: descartado deixar o agente escolher, porque o usuário precisa ver no diálogo o que vai ser executado antes de confirmar.
- "Nova spec" usa o `ActionDialog` com texto obrigatório: descartado um formulário próprio, que duplicaria o estado de envio e as mensagens.

**Contrato assumido da TASK-08.** `POST /api/pedidos?acao=criar-spec` (corpo: o assunto), `?acao=refinar-spec&spec=SPEC-NNN` e `?acao=executar-onda&spec=SPEC-NNN&onda=N`, respondendo `{"id":"PED-NNN"}`. **Ponto a alinhar:** o nome do parâmetro da onda (`onda`) e se a TASK-08 o valida; se ela usar outro nome, o passo 3 segue o da TASK-08.

**Riscos.** O print `executar-onda.png` depende do estado da SPEC-033: com a TASK-09 executada antes da TASK-10, a TASK-10 está aprovada e pronta na onda 4, e o botão aparece.

**Componente do plano sem mudança.** `apps/painel/src/features/spec/spec-page.tsx` não precisa mudar: a barra de ações do cabeçalho já é montada por `specActions`, e os botões "Refinar" e "Executar onda" aparecem nela pelo passo 2; os testes da página (`spec-page.test.tsx`) comprovam isso. Descartado mexer na página só para cumprir a lista, o que criaria código sem necessidade.

**Arquivos fora dos componentes do plano.** `apps/painel/src/shared/action-bar.tsx`, onde os tipos de ação viram pedidos, e os testes `attention.test.ts`, `specs-page.test.tsx` e `spec-page.test.tsx`.

**Arquivos em comum na onda 4.** `apps/painel/src/shared/attention.ts` e `apps/painel/src/shared/action-bar.tsx` com a TASK-10: o Coordenador executa a TASK-09 e depois a TASK-10.

**Referências da spec.** RF-05, CT-05, CT-06, CT-18, RNF-04, CA-01; contratos "Ações novas de pedido".
