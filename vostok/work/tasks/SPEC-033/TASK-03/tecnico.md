<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** `apps/cli/src/aval.rs` passa a gravar e ler os avais em `<paths.approvals>/<SPEC-NNN>.log` e `<paths.approvals>/<folders.approvals_without_spec>` (TASK-01), com `append`, `read_for` (só o log do ID) e `read_all` (todos, para as atividades). O formato da linha não muda (4 ou 5 colunas). O comando `vostok avais <ID>` fica em `apps/cli/src/commands/avais.rs`, declarado em `commands/mod.rs` e registrado em `cli.rs`, onde ficam os subcomandos.

**Decisões (perfil julgamento: alternativas e critérios).**
- *Spec de um aval.* Escolhido: a parte antes da primeira `/` do ID, quando é `SPEC-NNN`; o resto vai para o arquivo sem spec. Alternativa descartada: procurar a spec do pedido (`PED-NNN`) no front-matter dele, que faria a trava depender de um arquivo que o painel e o agente mudam. Critério: a mitigação da spec (a spec vem do ID do aval) e a trava lendo um arquivo só, sem atribuir um aval à spec errada.
- *Quando migrar o `avais.log`.* Escolhido: no início de toda leitura e gravação (`migrate_legacy`), barato quando o arquivo não existe. Alternativas descartadas: um comando explícito (a trava leria o formato novo antes de alguém rodá-lo e recusaria avais válidos) e a migração no `vostok update` (projetos que só trocam o binário ficariam sem ela). Critério: a trava nunca vê um estado intermediário.
- *O que fazer com o `avais.log` depois.* Escolhido: apagar só depois de todas as linhas gravadas, copiadas byte a byte. Alternativa descartada: renomear para `avais.log.migrado`, que deixaria dois lugares com os mesmos avais e, com um binário antigo ainda gravando, um segundo arquivo para migrar. Uma falha no meio mantém o `avais.log`, e a repetição pode duplicar linhas já copiadas, o que não muda a trava (ela procura qualquer linha que bata). Critério: RN-05, nenhuma linha reescrita, e nenhum aval perdido.
- *Saída do `vostok avais`.* Tabela Markdown, como o `vostok pedidos listar`, do mais novo para o mais antigo, como no contrato. Para uma spec, inclui as tarefas dela (`SPEC-NNN/TASK-NN`).
- *Mensagem de recusa.* Passa a citar `vostok avais <ID>`, para o diagnóstico não abrir os arquivos (RNF-02). A regra de contexto que proíbe ler os arquivos de aval não está nesta tarefa (não está na cobertura do plano).
- `packages/harness/.gitignore` ganha `work/avais/`: o `avais.log` já era local de cada clone, e os logs por spec não podem ir para o git.

**Arquivos fora dos componentes da tarefa.** `commands/avais.rs` (criado: o comando registrado em `mod.rs`), `cli.rs` (registro do subcomando), `spec.rs` e `task.rs` (chamadores de `has_panel_approval`), `painel/actions.rs` (componente da TASK-08, onda 3: a gravação do aval), `painel/read.rs` (componente da TASK-02, mesma onda, e da TASK-06: a leitura nas atividades), `.gitignore` e os testes.

**Arquivos em comum com a onda 2.** `apps/cli/src/commands/mod.rs` e `apps/cli/src/cli.rs` também mudam na TASK-04 (`vostok perguntas`). `apps/cli/src/aval.rs`, `apps/cli/src/painel/actions.rs`, `apps/cli/src/painel/read.rs` e `apps/cli/tests/painel.rs` também mudam na TASK-02 (pedidos por estado; no `aval.rs`, só `request_state` e `request_text_hash`). O Coordenador executa a TASK-02, a TASK-03 e a TASK-04 em sequência.

**Riscos.** Os avais de uma spec arquivada saem da pasta na TASK-05; até lá, o `vostok avais` lê só a pasta de trabalho. Linhas duplicadas por uma migração interrompida aparecem duas vezes no `vostok avais`.

**Referências da spec.** RF-09, RF-16, RN-05, RNF-02, FA-04, CB-03, CT-10, CT-17, CT-19 (parte dos avais), decisões 6 e 8, risco "Migração do `avais.log` atribuir um aval à spec errada".
