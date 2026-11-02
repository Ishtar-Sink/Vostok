### Ler

- `vostok/work/specs/refined/SPEC-029-painel-local-do-vostok.md` (RF-05, RNF-03, CB-06)
- `apps/cli/src/painel/mod.rs` e `apps/cli/src/painel/live.rs` (esqueleto da TASK-02)
- `/home/victor/projetos/ai-hub-painel/apps/cli/src/commands/painel.rs` (fn version, fn newest, fn live e fn send, linhas 284, 698 e 1002 a 1040)
- `docs/estudos/painel-local/prototipo.md` (seção "Atualização ao vivo")

### Mudar

- `apps/cli/src/painel/live.rs` (alterar): `version` (maior data de modificação de `vostok/work/`) e `handle` (handshake do WebSocket com `sha1`, `conectado` e `mudou` quando a versão muda, conferida a cada 500 ms).

### Passos

1. `apps/cli/src/painel/live.rs` (alterar): escreva primeiro os testes unitários. `version` muda depois de gravar um arquivo numa subpasta de `work/`. `accept_key` devolve o valor da RFC 6455 para a chave do exemplo (`dGhlIHNhbXBsZSBub25jZQ==` → `s3pPLMBiTxaQ9kYGzzhZRbK+xOo=`). `frame` monta um quadro de texto com o tamanho certo para menos de 126 bytes. Confira: falham.
2. `apps/cli/src/painel/live.rs` (alterar): implemente `version`, portando `newest` e `version` do spike. Implemente `handle`: o handshake com `Sec-WebSocket-Accept` (`sha1` e base64), o envio de `conectado` e um laço que confere `version` a cada 500 ms e envia `mudou` quando ela muda. O laço termina quando a escrita falha, porque a aba fechou. Cada aba tem a sua thread (CB-06).
3. `apps/cli/src/painel/live.rs` (alterar): o teste `two_sockets_receive_change` sobe o servidor da TASK-02 numa porta livre com `--features painel` e abre dois WebSockets crus (`TcpStream`). Depois grava um arquivo em `work/` e confere que os dois recebem `mudou` em menos de 1 s (RNF-03, CT-13).
4. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::live` passa e `cargo clippy --manifest-path apps/cli/Cargo.toml --features painel --all-targets -- -D warnings` sem avisos.
