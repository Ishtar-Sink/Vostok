### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test plan --test finalize --test commit_skill 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`

### Verificação

```bash
# evidência: trava-plano-commit.txt
cargo test --manifest-path apps/cli/Cargo.toml --test trava_plano_commit
cargo test --manifest-path apps/cli/Cargo.toml --test plan --test finalize --test commit_skill
cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings
```

Resultado esperado: os testes da trava e os de plano, finalização e commit passam; clippy sem avisos.

### Evidências

- `trava-plano-commit.txt` (automática, da verificação)
  - Prova: com a trava, aprovar o plano, arquivar a spec, fazer e liberar commit exigem o aval do painel; sem a trava, os comandos seguem iguais (RF-04, RF-05, RF-06, RF-07, CT-05, CT-06, CT-07, CT-08, CB-04).
  - Trecho: `mark_planned_needs_the_plan_aval ... ok`, `archive_needs_the_spec_aval ... ok`, `commit_needs_a_request_with_aval ... ok`, `commit_approve_needs_the_request ... ok`.
  - Falha: `FAILED`.
  - Espera: `test result: ok`
