<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Um módulo novo, `apps/cli/src/commands/perguntas.rs`, com o subcomando `vostok perguntas abrir <PEDIDO>`. O texto chega pela entrada padrão, no formato do template `question`. O comando grava `<questions>/<open>/P-NNN.md` com o front-matter do contrato da spec: `id`, `pedido`, `spec`, `task`, `status: aberta` e `created_at`. O corpo é `## Pergunta` seguido do texto. É o mesmo front-matter que `fn questions` de `painel/read.rs` já lê. As pastas vêm de `paths.questions`, `paths.requests` e da seção `folders`, lidas pelo que a TASK-01 criou em `harness.rs` (RF-16). O pedido é localizado pelos leitores `Requests` e `Questions` de `commands/pedidos.rs`, criados pela TASK-02 na mesma onda.

**Decisões.**
- **Pedido localizado pelo leitor da TASK-02, em qualquer estado.** O comando chama `Requests::open(...).find(id)` para achar o pedido nas subpastas de estado, e `Questions::open(...).folder_of(OPEN)` para achar a pasta de gravação. Não há busca própria: o planejamento previa uma, mas a TASK-02 já cobre o mesmo caso. Um pedido em qualquer estado serve: a `process-requests` abre a pergunta com o pedido `em_andamento`, mas outro estado não impede a gravação. Alternativa descartada: exigir `em_andamento`, que acoplaria o comando ao fluxo da skill sem proteger nada.
- **Visibilidade das constantes.** `OPEN` ("aberta") e `ANSWERED` ("respondida"), em `commands/pedidos.rs`, passaram a `pub`, para o novo módulo usar os mesmos nomes de pasta. É só mudança de visibilidade.
- **Numeração sem reuso.** O próximo `P-NNN` considera as abertas, as respondidas e as arquivadas em `<archive>/*/<nome da pasta de perguntas>/`, onde o nome é o último trecho de `paths.questions`. Assim uma pergunta nova nunca repete o ID de uma antiga. Alternativa descartada: olhar só as abertas, que reusaria números depois da resposta.
- **Gravação com `create_new`**, tentando o número seguinte se o arquivo já existir, como `write_request` do painel. Assim duas execuções juntas não se sobrescrevem.
- **`status: aberta` fixo no front-matter.** Só o nome da pasta é configurável (RN-06, decisão 11 da spec). Alternativa descartada: gravar o nome da pasta como status, que quebraria o painel com nomes em inglês.
- **`cli.rs` entra em "Mudar"**, fora dos componentes do plano: sem ele, o subcomando não existe na linha de comando.

**Riscos.** O arquivamento das perguntas (TASK-05) precisa usar o mesmo destino, `<archive>/<SPEC>/<último trecho de paths.questions>/`, para a numeração continuar sem reuso. Isso está combinado no detalhamento das duas tarefas.

**Arquivos em comum na onda 2.** `apps/cli/src/commands/mod.rs` é alterado também pela TASK-03 (registro do `vostok avais`). O Coordenador executa as duas em sequência. Se `cli.rs` também for alterado pela TASK-03, vale a mesma regra.

**Feito.**
- `apps/cli/src/commands/perguntas.rs` (novo): `PerguntasCommand::Abrir`, `run` e a abertura, com validação do ID, busca do pedido, recusa de texto vazio, leitura de `spec` e `tarefa` do pedido, numeração e gravação.
- `apps/cli/src/commands/mod.rs`: registro de `pub mod perguntas;` (2 inserções).
- `apps/cli/src/cli.rs`: variante `Perguntas` no enum `Command` e ramo no `run` (9 inserções, 2 remoções).
- `apps/cli/src/commands/pedidos.rs`: `OPEN` e `ANSWERED` passaram a `pub`.
- `apps/cli/tests/perguntas.rs` (novo): 7 testes `abrir_*`, cobrindo a gravação com o formato (CT-01), a numeração depois da maior pergunta, pedido em qualquer estado, pedido inexistente, IDs inválidos (`PED-1`, `X-001`), texto vazio e pastas configuradas (RF-16, RN-06).
- Verificação: testes passaram; regressão passou (um comando igual ao baseline e um com acréscimo); evidência gravada em `TASK-04/evidencias/`. Clippy e fmt sem aviso.

**Pontos de atenção.**
- `pedidos.rs` mudou só na visibilidade das constantes e não estava na lista de arquivos da tarefa; vale conferir isso no commit.
- `is_request_id` está copiada como função privada em `perguntas.rs`, igual à de `pedidos.rs`. Se a regra do ID mudar, as duas precisam mudar juntas.
- A numeração depende do nome da pasta de perguntas. Se `paths.questions` mudar, o arquivamento da TASK-05 precisa acompanhar.

**Referências da spec.** RF-01, RF-16, CT-01, RN-06; decisão 11; contrato de `vostok perguntas abrir` e do front-matter da pergunta.
