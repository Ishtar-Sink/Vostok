### Ler

- `vostok/work/specs/refined/SPEC-029-painel-local-do-vostok.md` (RF-01, RF-13, RNF-01, RNF-02, RNF-04, CT-01, CT-15)
- `.github/workflows/release.yml`
- `apps/cli/tests/painel_server.rs` (da TASK-02, exemplo de teste com o servidor)
- `apps/painel/scripts/print.sh` (da TASK-03)
- `docs/estudos/painel-local/medicoes.md` (como o tar.gz é gerado e medido)

### Mudar

- `apps/cli/tests/painel.rs` (criar): testes de integração do painel completo em duas pastas.
- `.github/workflows/release.yml` (alterar): Node e pnpm, bundle antes do `cargo build`, feature `painel` e conferência do tamanho.

### Passos

1. `apps/cli/tests/painel.rs` (criar): testes com `--features painel` e o bundle gerado.
   - **Duas pastas:** sobe `vostok painel --sem-navegador --porta 0` em duas cópias mínimas do harness ao mesmo tempo, com marcas diferentes. Confere que as portas são diferentes, que as duas escutam só em `127.0.0.1` e que cada `/api/marca` traz a sua cor (CT-01, RNF-02).
   - **Fluxo completo numa cópia:** lê o token da página, aprova uma tarefa, edita os critérios, grava um pedido e recebe o `mudou` pelo WebSocket. Confere que o `vostok task adjust validate` passa.
   Confira: `cargo test --manifest-path apps/cli/Cargo.toml --features painel --test painel` passa.
2. `.github/workflows/release.yml` (alterar): antes do `cargo build --release --locked`, acrescente:
   - os passos de pnpm e de Node (`actions/setup-node` com a versão 24 e cache do pnpm);
   - `pnpm --dir apps/painel install --frozen-lockfile` e `pnpm --dir apps/painel build`.
   O `cargo build` passa a usar `--features painel`. Depois do empacotamento, um passo confere o binário em até 6 000 000 bytes e o tar.gz em até 2 300 000 bytes, e falha com os dois números quando passa (RF-13, RNF-01).
3. Gere o binário de release localmente (`pnpm --dir apps/painel build` e `cargo build --release --manifest-path apps/cli/Cargo.toml --features painel`) e meça o binário e o tar.gz como a release faz.
4. Refaça os prints das telas com o binário novo (`PAINEL_BIN=apps/cli/target/release/vostok`) e confira que mostram os mesmos dados do spike.
5. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --features painel` inteiro passa; `cargo test --manifest-path apps/cli/Cargo.toml` sem a feature também passa; `cargo clippy --manifest-path apps/cli/Cargo.toml --features painel --all-targets -- -D warnings` sem avisos.
