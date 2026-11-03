### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --features painel --test painel 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort | grep -v 'plan_outdated_'`
- `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::read 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`
- `cargo test --manifest-path apps/cli/Cargo.toml --test plan 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`
- `cargo test --manifest-path apps/cli/Cargo.toml --lib commands::plan 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`

### Verificação

```bash
# evidência: plano-desatualizado-api.txt
cargo test --manifest-path apps/cli/Cargo.toml --features painel --test painel -- plan_outdated_
cargo test --manifest-path apps/cli/Cargo.toml --features painel --test painel
cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::read
cargo test --manifest-path apps/cli/Cargo.toml --test plan
cargo clippy --manifest-path apps/cli/Cargo.toml --features painel --all-targets -- -D warnings
```

### Evidências

- `plano-desatualizado-api.txt` (automática, da verificação)
  - Espera: test plan_outdated_false_without_a_plan ... ok
  - Prova: o painel em execução responde, na visão geral e na spec, se o plano está desatualizado pela regra da CLI: verdadeiro com o `spec_hash` do plano diferente do `approved_hash` da spec ou com a spec sem aprovação, falso com os hashes iguais e falso sem plano (RF-01, CT-01, CB-02).
  - Trecho: `test plan_outdated_true_when_the_spec_was_approved_again ... ok`, `test plan_outdated_false_when_the_hashes_match ... ok`, `test plan_outdated_true_when_the_spec_lost_the_approval ... ok`, `test plan_outdated_false_without_a_plan ... ok` e `test result: ok. 4 passed`.
  - Falha: algum desses testes com `FAILED`, a mensagem do `assert!` com o JSON sem `"plan_outdated"`, ou `running 0 tests`.
