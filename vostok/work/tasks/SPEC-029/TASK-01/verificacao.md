### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test task --test spec --test plan 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`

### Verificação

```bash
# evidência: aprovacao-origem.txt
cargo test --manifest-path apps/cli/Cargo.toml --test task --test spec -- approved_by approve_records_origin golden
cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings
```

Resultado esperado: os testes novos e os golden de tarefa e spec passam; clippy sem avisos.

### Evidências

- `aprovacao-origem.txt` (automática, da verificação)
  - Prova: a CLI grava `approved_by: chat`, a queda da aprovação limpa o campo e os golden continuam iguais aos scripts (RF-07, CT-06, CT-07).
  - Trecho: `approve_records_origin_chat ... ok`, `adjust_drops_approved_by ... ok`, `spec_approve_records_origin_chat ... ok`, `golden_task ... ok`, `golden_spec ... ok`.
  - Falha: `FAILED` em qualquer um.
  - Espera: `test result: ok`
