<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** `apps/cli/src/aval.rs` passou a gravar e ler os avais em `<paths.approvals>/<SPEC-NNN>.log` e `<paths.approvals>/<folders.approvals_without_spec>` (TASK-01), com `append`, `read_for` (só o log do ID) e `read_all` (todos os logs, usado nas atividades do painel). O formato da linha não mudou (4 ou 5 colunas). O comando `vostok avais <ID>` está em `apps/cli/src/commands/avais.rs`, declarado em `commands/mod.rs` e registrado em `cli.rs`, onde ficam os subcomandos.

**Decisões (perfil julgamento: alternativas e critérios).**
- *Spec de um aval.* Escolhido: a parte antes da primeira `/` do ID, quando é `SPEC-NNN`; o resto vai para o arquivo sem spec. Alternativa descartada: procurar a spec do pedido (`PED-NNN`) no front-matter dele, que faria a trava depender de um arquivo que o painel e o agente mudam. Critério: a spec vem do ID do aval e a trava lê um arquivo só, sem atribuir um aval à spec errada.
- *Quando migrar o `avais.log`.* Escolhido: no início de toda leitura e gravação (`migrate_legacy`), barato quando o arquivo não existe. Alternativas descartadas: um comando explícito (a trava leria o formato novo antes de alguém rodá-lo e recusaria avais válidos) e a migração no `vostok update` (projetos que só trocam o binário ficariam sem ela). Critério: a trava nunca vê um estado intermediário.
- *O que fazer com o `avais.log` depois.* Escolhido: apagar só depois de todas as linhas gravadas, copiadas byte a byte. Alternativa descartada: renomear para `avais.log.migrado`, que deixaria dois lugares com os mesmos avais e, com um binário antigo ainda gravando, um segundo arquivo para migrar. Uma falha no meio mantém o `avais.log`, e a repetição pode duplicar linhas já copiadas, o que não muda a trava (ela procura qualquer linha que bata). Critério: RN-05, nenhuma linha reescrita e nenhum aval perdido.
- *Saída do `vostok avais`.* Tabela Markdown, como a do `vostok pedidos listar`, do mais novo para o mais antigo. Para uma spec, inclui as tarefas dela (`SPEC-NNN/TASK-NN`); para tarefa ou pedido, só o ID exato.
- *Mensagem de recusa.* Passou a citar `vostok avais <ID>`, para o diagnóstico não abrir os arquivos (RNF-02). A regra de contexto que proíbe ler os arquivos de aval não está nesta tarefa (não está na cobertura do plano).
- `packages/harness/.gitignore` ganhou `work/avais/`: o `avais.log` já era local de cada clone, e os logs por spec não podem ir para o git.

**Arquivos fora dos componentes da tarefa.** `commands/avais.rs` (criado; o comando registrado em `mod.rs`), `cli.rs` (registro do subcomando), `spec.rs` e `task.rs` (passam o `Harness` a `has_panel_approval`), `painel/actions.rs` (componente da TASK-08, onda 3: `record_aval` grava pelo `aval::append`), `painel/read.rs` (componente da TASK-02, mesma onda, e da TASK-06: `approval_rows` lê pelo `aval::read_all`), `.gitignore` e os testes.

**Arquivos em comum com a onda 2.** `apps/cli/src/commands/mod.rs` e `apps/cli/src/cli.rs` também são tocados pela TASK-04 (`vostok perguntas`). `apps/cli/src/aval.rs`, `apps/cli/src/painel/actions.rs`, `apps/cli/src/painel/read.rs` e `apps/cli/tests/painel.rs` também são tocados pela TASK-02 (pedidos por estado; no `aval.rs`, só `request_state` e `request_text_hash`). O Coordenador executou a TASK-02, a TASK-03 e a TASK-04 em sequência.

**Riscos.** Os avais de uma spec arquivada saem da pasta na TASK-05; até lá, o `vostok avais` lê só a pasta de trabalho. Linhas duplicadas por uma migração interrompida aparecem duas vezes no `vostok avais`.

**Referências da spec.** RF-09, RF-16, RN-05, RNF-02, FA-04, CB-03, CT-10, CT-17, CT-19 (parte dos avais), decisões 6 e 8, risco "Migração do `avais.log` atribuir um aval à spec errada".

**Feito.**
- Testes: 5 testes `by_spec_*` em `apps/cli/tests/aval.rs` (11 testes no arquivo). Unitários novos: `spec_of_reads_only_spec_ids` e `legacy_log_is_kept_when_a_write_fails`. `missing_log_reads_empty` passou a ler uma pasta sem logs. As fixtures `vostok/work/avais.log` dos testes do painel foram mantidas e passam pela migração; `apps/cli/tests/painel.rs` lê `work/avais/SPEC-001.log`. Verificação, regressão e evidências passaram (registro 5, 8 e 9 da tarefa).
- Regressão aceita: o comando [4] do baseline (testes `painel::`) mudou por causa da TASK-02, da mesma onda: o teste `next_request_id_follows_the_highest_of_every_folder` foi renomeado e `pedidos_e_perguntas_vem_das_subpastas` é novo. A diferença não vem desta tarefa. O usuário aceitou como esperada (`vostok task resume --accept-regression 4`); depois disso a regressão passou (3 comandos iguais ao baseline, 1 aceito pelo usuário e 1 com acréscimo) e as evidências também.

**Pontos de atenção.**
- A migração roda no início de toda leitura e gravação, inclusive do `vostok avais` e da trava. Um `avais.log` que sobrou por falha é tentado de novo na próxima chamada, e linhas já copiadas podem duplicar.
- Linha malformada na migração vai ao log sem spec (`folders.approvals_without_spec`). Na leitura, é reportada pelo nome do arquivo (`SPEC-001.log: linha 1 malformada`).
- Até a TASK-05, o `vostok avais` não enxerga avais arquivados.
- O log sem spec e a pasta dos avais seguem a configuração (`paths.approvals`, `folders.approvals_without_spec`).
