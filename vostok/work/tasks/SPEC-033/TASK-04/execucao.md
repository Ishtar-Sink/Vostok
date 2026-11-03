### Ler

- `vostok/work/specs/refined/SPEC-033-operar-o-projeto-de-ponta-a-ponta-pelo-painel-v0-4-0.md` (RF-01, RF-16, CT-01, RN-06 e o contrato de `<questions>/<open>/P-001.md`)
- `docs/estudos/painel-ponta-a-ponta.md` (seções "Perguntas do agente" e "Arquivos de comunicação")
- `vostok/work/tasks/SPEC-033/TASK-01/tecnico.md` (como ler `paths.questions`, `paths.requests` e a seção `folders`)
- `apps/cli/src/harness.rs` (fn path e o leitor de `folders` criado pela TASK-01)
- `apps/cli/src/commands/pedidos.rs` (enum PedidosCommand, fn run, fn is_request_id, fn read_stdin: modelo de comando com texto pela entrada padrão)
- `apps/cli/src/commands/mod.rs`
- `apps/cli/src/cli.rs`
- `apps/cli/src/painel/read.rs` (fn questions, que lê o front-matter `id`, `spec`, `task`, `status`, `created_at`, e o teste `overview_lists_open_and_archived_specs_activity_and_questions`)
- `apps/cli/src/painel/actions.rs` (fn write_request e fn next_request_id: criação sem sobrescrever com `create_new`)
- `vostok/templates/responses/question.md` (formato do texto que chega pela entrada padrão)
- `apps/cli/tests/pedidos.rs` (fixture `write_request` e `stdout_of`)
- `apps/cli/tests/common/mod.rs`

### Mudar

- `apps/cli/src/commands/perguntas.rs` (criar): comando `vostok perguntas abrir <PEDIDO>`, que grava `<questions>/<open>/P-NNN.md` (RF-01, CT-01)
- `apps/cli/src/commands/mod.rs` (alterar): registro do módulo `perguntas`
- `apps/cli/src/cli.rs` (alterar): subcomando `Perguntas` no enum `Command` e no `run`, sem o qual o comando não existe na linha de comando
- `apps/cli/tests/perguntas.rs` (criar): testes de integração do `vostok perguntas abrir`

### Passos

1. `apps/cli/tests/perguntas.rs` (criar): com `mod common;` e `common::{failure_stderr, Project}`, escreva o helper `write_request(project, state_folder, id, spec, task)`, que grava `vostok/work/pedidos/<state_folder>/<id>.md` com o front-matter de `tests/pedidos.rs` (`id`, `acao: refinar-spec`, `spec`, `tarefa`, `estado`, `origem: painel`, `criado_em`, `atualizado_em`), e os testes:
   - `abrir_grava_a_pergunta_aberta_com_o_pedido` (CT-01): pedido `PED-001` em `em_andamento/` com `spec: SPEC-001` e `tarefa: TASK-02`; roda `vostok perguntas abrir PED-001` com a entrada `# 1. Qual caminho?\n\n| Opção | Descrição |\n|---|---|\n| A | Seguir (recomendada) |\n`; confere que `vostok/work/perguntas/aberta/P-001.md` existe, tem as linhas `id: P-001`, `pedido: PED-001`, `spec: SPEC-001`, `task: TASK-02`, `status: aberta` e uma `created_at: ` no front-matter, e o corpo `## Pergunta\n\n# 1. Qual caminho?`; e que a saída cita `P-001`.
   - `abrir_numera_depois_da_maior_pergunta`: com `perguntas/aberta/P-001.md` e `perguntas/respondida/P-004.md` já gravados, a nova é `aberta/P-005.md`.
   - `abrir_acha_o_pedido_em_qualquer_estado`: pedido em `feito/`; a pergunta é gravada com a spec dele.
   - `abrir_recusa_pedido_inexistente`, `abrir_recusa_id_invalido` (`PED-1` e `X-001`) e `abrir_recusa_texto_vazio` (entrada só com espaços): falham, a mensagem cita o pedido ou o motivo, e `vostok/work/perguntas/` continua sem arquivo.
   - `abrir_usa_as_pastas_configuradas` (RF-16): com `vostok::support::config_set` em `vostok/config/main.yml`, troque `paths.questions` para `./work/questions` e a subpasta das abertas (a chave de `folders` da TASK-01) para `open`; a pergunta vai para `vostok/work/questions/open/P-001.md`, com `status: aberta` (RN-06).
   Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test perguntas` não compila ou falha antes do passo 4.
2. `apps/cli/src/commands/perguntas.rs` (criar): escreva o módulo com:
   - `PerguntasCommand` (clap `Subcommand`) com a variante `Abrir { pedido: String }`, documentada como "Grava uma pergunta aberta do agente para o pedido; o texto vem da entrada padrão.";
   - `pub fn run(command: PerguntasCommand) -> Result<()>`, que localiza o harness com `Harness::from_current_dir()` e chama `open`;
   - `fn open(harness, request_id)`: recusa um ID fora de `PED-NNN` (mesma regra de `is_request_id` de `pedidos.rs`, copiada como função privada) com "ID de pedido inválido '<id>': use o formato PED-NNN"; procura `<id>.md` em cada subpasta de estado de `harness.path("requests")` (os quatro nomes de `folders` da TASK-01, na ordem pendente, em andamento, feito, recusado) e recusa com "Pedido <id> não encontrado em <pasta>" quando não acha; lê o texto da entrada padrão, sem espaços nas pontas, e recusa o vazio com "o texto da pergunta veio vazio: envie as perguntas e as opções pela entrada padrão"; lê `spec` e `tarefa` do pedido com `frontmatter_get` (ausente vira vazio);
   - `fn next_question_id(dirs)`: o maior número `P-NNN` entre a subpasta das abertas, a das respondidas e `<archive>/*/<nome da pasta de perguntas>/` (o nome é o último trecho de `paths.questions`, como a TASK-05 arquiva), mais um, com três dígitos;
   - a gravação com `OpenOptions::new().write(true).create_new(true)` em `<questions>/<open>/`, criando a pasta, e tentando o número seguinte se o arquivo já existir (como `write_request` do painel), com o conteúdo `---\nid: P-NNN\npedido: <PED>\nspec: <spec>\ntask: <tarefa>\nstatus: aberta\ncreated_at: <utc_datetime()>\n---\n\n## Pergunta\n\n<texto>\n`;
   - a saída `Pergunta P-NNN aberta para o pedido <PED>: <caminho>`.
   O valor `aberta` é fixo no front-matter; só o nome da pasta vem da configuração (RN-06, decisão 11 da spec). Confira: `cargo build --manifest-path apps/cli/Cargo.toml`.
3. `apps/cli/src/commands/mod.rs` (alterar): acrescente `pub mod perguntas;` em ordem alfabética, entre `pedidos` e `plan`. Confira: compila.
4. `apps/cli/src/cli.rs` (alterar): importe `perguntas`, acrescente ao enum `Command`, depois de `Pedidos`, a variante `/// Perguntas do agente ao usuário, respondidas no painel.` `#[command(subcommand)] Perguntas(perguntas::PerguntasCommand)` e, no `run`, `Command::Perguntas(command) => perguntas::run(command)`. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test perguntas` passa.
5. Rode `cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings` e `cargo fmt --manifest-path apps/cli/Cargo.toml -- --check`. Confira: sem aviso nem diferença.
