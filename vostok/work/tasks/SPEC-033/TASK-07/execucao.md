### Ler

- `vostok/work/specs/refined/SPEC-033-operar-o-projeto-de-ponta-a-ponta-pelo-painel-v0-4-0.md` (RF-13, RN-02, RN-04, CB-04, CT-14, decisão 1, riscos e premissas)
- `docs/estudos/painel-ponta-a-ponta.md` (seção "Técnicas para operar sem o chat", disparo sob demanda)
- `vostok/work/tasks/SPEC-033/TASK-01/tecnico.md` (leitura de `request_dispatch`)
- `vostok/work/tasks/SPEC-033/TASK-02/tecnico.md` (pastas por estado e funções de pedido em `pedidos.rs`)
- `apps/cli/src/commands/pedidos.rs` (fn watch, fn in_path, fn next_pending, fn process_with_codex, const LEFT_PENDING)
- `apps/cli/src/commands/painel.rs` (fn run com a feature `painel`)
- `apps/cli/src/painel/server.rs` (fn serve, que bloqueia a thread principal)
- `apps/cli/src/harness.rs`
- `apps/cli/tests/pedidos.rs` (fn fake_codex, fn path_with, fn state_of e os testes `vigiar_*`)
- `apps/cli/tests/painel.rs` (struct Running, fn start)
- `apps/cli/tests/common/mod.rs`

### Mudar

- `apps/cli/src/commands/pedidos.rs` (alterar): o disparo por `request_dispatch`, com o processamento de um pedido generalizado para `codex` e `claude` e o laço reaproveitado pelo `vigiar` (RF-13, RN-04)
- `apps/cli/src/commands/painel.rs` (alterar): o `vostok painel` inicia o disparo antes de servir, porque é o comando que a RF-13 cita e o `serve` bloqueia a thread principal
- `apps/cli/tests/disparo.rs` (criar): testes de ponta a ponta do disparo com agentes falsos no PATH (CT-14, CB-04)

### Passos

1. `apps/cli/tests/disparo.rs` (criar): com `#![cfg(feature = "painel")]`, `mod common;` e `common::Project`, escreva os helpers e os testes abaixo.

   Helpers:
   - `set_dispatch(project, value)`: troca a linha `request_dispatch:` de `vostok/config/main.yml` por `request_dispatch: <value>`, linha a linha;
   - `write_pending(project, id)`: grava `vostok/work/pedidos/pendente/<id>.md` com o front-matter de `tests/pedidos.rs` e `estado: pendente`;
   - `fake_agent(project, name, log)`: grava em `<raiz>/bin/<name>` um `sh` que anota `inicio <PED>` e `<name> $*` no log, dorme 0,3 s, roda `vostok pedidos iniciar` e `vostok pedidos concluir` no pedido citado e anota `fim <PED>`, como `fake_codex` de `tests/pedidos.rs`;
   - `path_with(bin)`, igual ao de `tests/pedidos.rs`;
   - `start(project, path)`: sobe `vostok painel --sem-navegador --porta 0 --pasta <raiz>` com `env("PATH", path)`, `stdout` e `stderr` em pipe, lê a primeira linha do stdout e devolve um `Running` que mata o processo no `Drop`;
   - `state_of(project, id)`: o estado lido do arquivo em qualquer subpasta de `vostok/work/pedidos/`.

   Testes:
   - `disparo_com_codex_chama_o_agente_uma_vez_por_pedido_em_ordem` (CT-14, RN-04): `request_dispatch: codex` e PED-001 e PED-002 pendentes. Espera até 20 s os dois ficarem `feito` e confere que o log tem exatamente `inicio PED-001`, `fim PED-001`, `inicio PED-002` e `fim PED-002` nessa ordem, sem duas execuções sobrepostas, e uma linha `codex exec --full-auto` por pedido.
   - `disparo_com_claude_usa_o_modo_nao_interativo` (CT-14): `request_dispatch: claude`, com um `claude` falso. O pedido fica `feito` e o log tem `claude -p` com `PED-001`.
   - `disparo_desligado_nao_chama_o_agente` (CT-14): `request_dispatch: ""`, com o `codex` falso no PATH. Depois de 4 s, o log não existe e PED-001 continua `pendente`.
   - `disparo_sem_o_agente_no_path_avisa_e_nao_recusa` (CB-04): `request_dispatch: codex` e PATH sem `codex`. O painel sobe, o stderr cita `codex` e `PATH`, e depois de 4 s PED-001 continua `pendente`.
   - `disparo_com_valor_invalido_recusa_o_painel`: `request_dispatch: gemini`. O `vostok painel` termina com erro que cita `codex` e `claude`.

   Confira: `cargo test --manifest-path apps/cli/Cargo.toml --features painel --test disparo` falha antes do passo 3.
2. `apps/cli/src/commands/pedidos.rs` (alterar): generalize o processamento de um pedido.
   - Troque `process_with_codex` por `fn process_with(agent: &str, project: &Path, requests: &Path, id: &str) -> Result<()>`. Ela monta o comando por `fn agent_command(agent, instruction) -> Command`:
     - `codex`: `codex exec --full-auto <instrução>`, como hoje;
     - `claude`: `claude -p <instrução> --permission-mode acceptEdits --allowedTools Bash`.
   - A instrução cita o ID e não a pasta: `Use a skill process-requests para processar o pedido <ID> e só ele.`.
   - `LEFT_PENDING` vira `fn left_pending(agent)`, com o texto `o <agente> terminou sem processar o pedido; veja a saída do vostok pedidos vigiar ou do vostok painel`. O pedido continua recusado quando o agente sai e o deixa pendente.
   - Acrescente `const AGENTS: &[&str] = &["codex", "claude"];` e `fn run_loop(agent, project, requests, interval: Duration) -> Result<()>`, com o laço de hoje: o próximo pendente, ou dormir. A `watch` passa a chamá-lo.
   - O `vigiar` continua aceitando só `--agente codex`, com as mesmas mensagens, para não mudar o contrato dele.

   Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test pedidos -- vigiar_` passa como antes.
3. `apps/cli/src/commands/pedidos.rs` (alterar): acrescente `const DISPATCH_INTERVAL: Duration = Duration::from_secs(2);` e `pub fn start_dispatch(harness: &Harness) -> Result<Option<std::thread::JoinHandle<()>>>`. Ela lê `request_dispatch` pelo leitor da TASK-01, e uma chave ausente conta como vazia.
   - Vazio: devolve `Ok(None)` sem imprimir nada.
   - Valor fora de `AGENTS`: falha com `request_dispatch '<valor>' inválido em config/main.yml: use "", codex ou claude`.
   - Agente fora do PATH (`in_path`): imprime no stderr `Disparo sob demanda desligado: o <agente> não está no PATH; os pedidos ficam pendentes até um agente em loop (vostok pedidos vigiar ou /process-requests).` e devolve `Ok(None)`, sem tocar em nenhum pedido (CB-04).
   - Agente encontrado: imprime no stderr, para a primeira linha do stdout continuar sendo a URL do painel, `Disparo sob demanda com o <agente> a cada 2 s.` e abre uma única thread com `run_loop(agent, project, requests, DISPATCH_INTERVAL)`. Um erro do laço, como o agente sumir do PATH, é impresso no stderr com `Disparo sob demanda parado: <erro>` e encerra a thread, com o pedido ainda pendente. A única thread executa um agente por vez, porque `process_with` espera o processo terminar (RN-04).

   Confira: compila.
4. `apps/cli/src/commands/painel.rs` (alterar): no `run` com a feature `painel`, depois de `Painel::new(&harness)?` e antes de `server::serve`, chame `let _dispatch = crate::commands::pedidos::start_dispatch(&harness)?;`. Um valor inválido impede o painel de subir. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --features painel --test disparo` passa.
5. Rode `cargo clippy --manifest-path apps/cli/Cargo.toml --features painel --all-targets -- -D warnings` e `cargo fmt --manifest-path apps/cli/Cargo.toml -- --check`. Confira: sem aviso nem diferença.
