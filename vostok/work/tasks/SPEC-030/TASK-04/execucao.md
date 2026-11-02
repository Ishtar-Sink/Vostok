### Ler

- `vostok/work/specs/refined/SPEC-030-trava-de-aval-e-loop-de-pedidos-do-painel.md` (RF-13, CT-14, CB-05, RN-03, "Contratos e interfaces")
- `apps/cli/src/painel/actions.rs` (fn request_markdown: formato do `PED-NNN.md`)
- `apps/cli/src/cli.rs` (enum Command e o despacho)
- `apps/cli/src/commands/mod.rs`
- `apps/cli/src/commands/memory.rs` (exemplo de comando com subcomandos e leitura da entrada padrão)
- `apps/cli/src/support.rs` (fn frontmatter_get, fn frontmatter_set, fn utc_datetime)
- `apps/cli/tests/common/mod.rs`

### Mudar

- `apps/cli/src/commands/pedidos.rs` (criar): subcomandos `listar`, `iniciar`, `concluir` e `recusar`.
- `apps/cli/src/commands/mod.rs` (alterar): `pub mod pedidos;`.
- `apps/cli/src/cli.rs` (alterar): comando `pedidos` no enum e no despacho.
- `apps/cli/tests/pedidos.rs` (criar): testes dos subcomandos.

### Passos

1. `apps/cli/tests/pedidos.rs` (criar): com `common::Project`, grave `vostok/work/pedidos/PED-001.md` no formato do painel (`estado: pendente`, `## Pedido` e `## Resultado` vazio) e teste:
   - `listar_shows_pending_requests` (ID, ação e estado; `--estado feito` filtra);
   - `iniciar_moves_pending_to_em_andamento` (estado e `atualizado_em` mudam);
   - `iniciar_refuses_a_request_not_pending` (CB-05: erro e arquivo igual);
   - `concluir_writes_the_result` (resultado pela entrada padrão em `## Resultado`, estado `feito`);
   - `recusar_writes_the_reason` (estado `recusado`);
   - `concluir_requires_em_andamento`.
   Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test pedidos` falha.
2. `apps/cli/src/commands/pedidos.rs` (criar): `#[derive(Subcommand)] pub enum PedidosCommand { Listar { #[arg(long)] estado: Option<String> }, Iniciar { id: String }, Concluir { id: String }, Recusar { id: String } }` e `pub fn run(command) -> Result<()>`. Os pedidos ficam em `<work>/pedidos/` (o `work` vem de `Harness::from_current_dir()`). Regras: `iniciar` só de `pendente`; `concluir` e `recusar` só de `em_andamento`; o texto vem da entrada padrão e substitui o conteúdo de `## Resultado`; `atualizado_em` recebe `utc_datetime()`; ID fora do padrão `PED-NNN` é recusado. `listar` imprime uma tabela Markdown `| ID | Ação | Spec | Tarefa | Estado |` em ordem de ID.
3. `apps/cli/src/commands/mod.rs` (alterar): `pub mod pedidos;`.
4. `apps/cli/src/cli.rs` (alterar): `/// Pedidos gravados pelo painel` `Pedidos { #[command(subcommand)] command: pedidos::PedidosCommand }` e o despacho para `pedidos::run`. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test pedidos` passa e `vostok pedidos --help` lista os quatro subcomandos.
5. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test base` passa (o `--help` geral ganha a linha) e `cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings` sem avisos.
