### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test base 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`
- `cargo run -q --manifest-path apps/cli/Cargo.toml -- --help`

### Verificação

```bash
# evidência: servidor-testes.txt
cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel
cargo test --manifest-path apps/cli/Cargo.toml --features painel --test painel_server
cargo build --manifest-path apps/cli/Cargo.toml
cargo clippy --manifest-path apps/cli/Cargo.toml --features painel --all-targets -- -D warnings
```

Resultado esperado: testes do módulo e de integração passando, build sem a feature passando e clippy sem avisos.

### Evidências

- `servidor-testes.txt` (automática, da verificação)
  - Prova: o servidor escuta só em `127.0.0.1`, recusa ação sem token, acha o harness e explica a falta do bundle (RF-01, RF-08, RF-12, CT-02, CT-05, CT-14).
  - Trecho: os testes de `painel::` e de `painel_server` com `ok` e `test result: ok`.
  - Falha: `FAILED` ou erro de compilação.
  - Espera: `test result: ok`
- `sem-pasta.txt` (automática): `cargo run -q --manifest-path apps/cli/Cargo.toml --features painel -- painel --pasta / --sem-navegador; echo "codigo: $?"`
  - Prova: fora de um projeto com harness, o comando para e diz o que procurou (RF-01, CT-02).
  - Trecho: a mensagem citando `vostok/config/main.yml` e `codigo: 1`.
  - Falha: o painel subir ou código 0.
  - Requer: `apps/cli/src/commands/painel.rs`
