### Baseline

- `cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings >/dev/null 2>&1 && echo clippy-ok`
- `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::read 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`

### Verificação

```bash
# evidência: perguntas-testes.txt
cargo test --manifest-path apps/cli/Cargo.toml --test perguntas
cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path apps/cli/Cargo.toml -- --check
```

### Evidências

- `perguntas-testes.txt` (automática, da verificação)
  - Prova: o `vostok perguntas abrir` grava a pergunta aberta com o pedido, a spec e a tarefa, numera sem repetir, respeita as pastas configuradas e recusa pedido inexistente, ID inválido e texto vazio (RF-01, CT-01, RF-16).
  - Trecho: `test abrir_grava_a_pergunta_aberta_com_o_pedido ... ok`, `test abrir_usa_as_pastas_configuradas ... ok` e `test result: ok`.
  - Falha: algum teste com `FAILED`, o alvo `perguntas` inexistente ou erro de compilação.
  - Requer: `apps/cli/tests/perguntas.rs`, `apps/cli/src/commands/perguntas.rs`
