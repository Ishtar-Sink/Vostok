### Ler

- `vostok/work/specs/refined/SPEC-030-trava-de-aval-e-loop-de-pedidos-do-painel.md` (RF-15, CT-16, CB-06, RNF-04, FA-02)
- `apps/cli/src/commands/pedidos.rs` (da TASK-04)
- `apps/cli/tests/pedidos.rs` (da TASK-04)
- `packages/harness/skills/default/process-requests/SKILL.md` (da TASK-08)
- `docs/estudos/painel-local/testes.md` (seção "Codex (sem teste)")
- `apps/cli/src/cli.rs`

### Mudar

- `apps/cli/src/commands/pedidos.rs` (alterar): subcomando `vigiar`.
- `apps/cli/src/cli.rs` (alterar): nada além do que o `PedidosCommand` já registra; ajuste a ajuda do comando `pedidos` para citar o `vigiar`.
- `apps/cli/tests/pedidos.rs` (alterar): teste do `vigiar` com um `codex` simulado.

### Passos

1. `apps/cli/tests/pedidos.rs` (alterar): teste `vigiar_runs_codex_once_per_request_in_order` (CT-16, RNF-04): crie dois pedidos pendentes e, numa pasta temporária no início do `PATH`, um script `codex` que grava `inicio <id>` e `fim <id>` num log, dorme 0,3 s e roda `vostok pedidos iniciar` e `concluir` do pedido citado. Rode `vostok pedidos vigiar --agente codex --intervalo 1` em segundo plano, espere os dois pedidos ficarem `feito` (até 20 s) e encerre o processo; confira que o log tem `inicio PED-001`, `fim PED-001`, `inicio PED-002`, `fim PED-002`, nessa ordem. Teste `vigiar_without_codex_stops_with_instructions` (CB-06): com `PATH` sem `codex`, sai com código 1 e a instrução de instalar. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test pedidos -- vigiar` falha.
2. `apps/cli/src/commands/pedidos.rs` (alterar): `Vigiar { #[arg(long)] agente: String, #[arg(long, default_value_t = 15)] intervalo: u64 }`. Só `codex` é aceito em `--agente`. Confira `codex` no `PATH` antes do laço (CB-06). No laço: o menor pedido `pendente`; sem pedido, dorme `intervalo` segundos; com pedido, roda `codex exec "<instrução>"` na raiz do projeto e espera terminar, com a instrução `Use a skill process-requests para processar o pedido <ID> de vostok/work/pedidos/ e só ele.` Depois da execução, se o pedido continuar `pendente`, marque-o `recusado` com `o codex terminou sem processar o pedido; veja a saída do vostok pedidos vigiar`, para o laço não repetir o mesmo pedido. Imprima uma linha por pedido (`PED-NNN: feito|recusado|em_andamento`).
3. `apps/cli/src/cli.rs` (alterar): confira que a ajuda do `pedidos` lista `vigiar`.
4. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test pedidos` passa e `cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings` sem avisos.
