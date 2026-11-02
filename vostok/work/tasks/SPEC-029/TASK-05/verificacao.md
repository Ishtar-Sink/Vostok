### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`

### Verificação

```bash
# evidência: ao-vivo-testes.txt
cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::live
cargo clippy --manifest-path apps/cli/Cargo.toml --features painel --all-targets -- -D warnings
```

Resultado esperado: os testes do ao vivo passam, inclusive os dois sockets recebendo `mudou` em menos de 1 s.

### Evidências

- `ao-vivo-testes.txt` (automática, da verificação)
  - Prova: a mudança em `vostok/work/` chega às abas abertas em menos de 1 s (RF-05, CT-13, RNF-03, CB-06).
  - Trecho: `two_sockets_receive_change ... ok` e o teste do handshake ok.
  - Falha: `FAILED` ou tempo esgotado.
  - Espera: `test result: ok`
