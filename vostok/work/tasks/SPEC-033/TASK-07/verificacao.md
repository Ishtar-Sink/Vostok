### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test pedidos 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort | grep -v 'prune_'`
- `cargo test --manifest-path apps/cli/Cargo.toml --features painel --test painel_server 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`
- `cargo test --manifest-path apps/cli/Cargo.toml --features painel --test painel 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`

### Verificação

```bash
# evidência: disparo-testes.txt
cargo test --manifest-path apps/cli/Cargo.toml --features painel --test disparo
# evidência: vigiar-testes.txt
cargo test --manifest-path apps/cli/Cargo.toml --test pedidos -- vigiar_
cargo clippy --manifest-path apps/cli/Cargo.toml --features painel --all-targets -- -D warnings
cargo fmt --manifest-path apps/cli/Cargo.toml -- --check
```

### Evidências

- `disparo-testes.txt` (automática, da verificação)
  - Prova: com `request_dispatch` ligado, o `vostok painel` chama o agente falso uma vez por pedido, um por vez e em ordem, com o `codex` ou o `claude`. Desligado, não chama nada. Sem o agente no PATH, avisa e deixa o pedido pendente, e um valor inválido impede o painel de subir (RF-13, CT-14, RN-04, CB-04).
  - Trecho: `test disparo_com_codex_chama_o_agente_uma_vez_por_pedido_em_ordem ... ok`, `test disparo_desligado_nao_chama_o_agente ... ok`, `test disparo_sem_o_agente_no_path_avisa_e_nao_recusa ... ok` e `test result: ok`.
  - Falha: algum teste com `FAILED`, ou o alvo `disparo` inexistente.
  - Requer: `apps/cli/tests/disparo.rs`
- `vigiar-testes.txt` (automática, da verificação)
  - Prova: o `vostok pedidos vigiar`, que agora usa o mesmo laço, continua com o mesmo comportamento (RF-13).
  - Trecho: `test vigiar_runs_codex_once_per_request_in_order ... ok` e `test vigiar_refuses_a_request_codex_left_pending ... ok`.
  - Falha: algum `vigiar_` com `FAILED`.
