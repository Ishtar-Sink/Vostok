<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Um módulo novo, `apps/cli/src/commands/perguntas.rs`, com o subcomando `vostok perguntas abrir <PEDIDO>`. O texto chega pela entrada padrão, no formato do template `question`. O comando grava `<questions>/<open>/P-NNN.md` com o front-matter do contrato da spec: `id`, `pedido`, `spec`, `task`, `status: aberta` e `created_at`. O corpo é `## Pergunta` seguido do texto. É o mesmo front-matter que `fn questions` de `painel/read.rs` já lê. As pastas vêm de `paths.questions`, `paths.requests` e da seção `folders`, lidas pelo que a TASK-01 criou em `harness.rs` (RF-16).

**Decisões.**
- **Pedido procurado nas quatro subpastas de estado, sem pedir um estado específico.** A `process-requests` abre a pergunta com o pedido `em_andamento`, mas um pedido em outro estado não atrapalha a pergunta. Alternativa descartada: exigir `em_andamento`, que acoplaria o comando ao fluxo da skill sem proteger nada. A busca é feita pelo próprio módulo, porque a TASK-04 não depende da TASK-02. Ela só usa os nomes de pasta da configuração.
- **Numeração sem reuso.** O próximo `P-NNN` considera as abertas, as respondidas e as já arquivadas em `<archive>/*/perguntas/`, para uma pergunta nova nunca repetir o ID de uma antiga. Alternativa descartada: olhar só as abertas, que reusaria números depois da resposta.
- **Gravação com `create_new`**, tentando o número seguinte se o arquivo já existir, como `write_request` do painel. Assim duas execuções juntas não se sobrescrevem.
- **`status: aberta` fixo no front-matter.** Só o nome da pasta é configurável (RN-06, decisão 11 da spec). Alternativa descartada: gravar o nome da pasta como status, que quebraria o painel com nomes em inglês.
- **`cli.rs` entra em "Mudar"**, fora dos componentes do plano: sem ele, o subcomando não existe na linha de comando.

**Riscos.** O arquivamento das perguntas (TASK-05) precisa usar o mesmo destino, `<archive>/<SPEC>/<último trecho de paths.questions>/`, para a numeração continuar sem reuso. Isso está combinado no detalhamento das duas tarefas.

**Arquivos em comum na onda 2.** `apps/cli/src/commands/mod.rs` é alterado também pela TASK-03 (registro do `vostok avais`). O Coordenador executa as duas em sequência. Se `cli.rs` também for alterado pela TASK-03, vale a mesma regra.

**Referências da spec.** RF-01, RF-16, CT-01, RN-06; decisão 11; contrato de `vostok perguntas abrir` e do front-matter da pergunta.
