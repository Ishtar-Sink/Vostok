### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test task --test plan --test finalize 2>&1 | grep -E '^test ' | sort`
- `cargo test --manifest-path apps/cli/Cargo.toml --test scenarios 2>&1 | grep -E '^test ' | sort`

### Verificação

```bash
# evidência: docs-tests.txt
cargo test --manifest-path apps/cli/Cargo.toml --test task -- docs_
# evidência: scenarios-tests.txt
cargo test --manifest-path apps/cli/Cargo.toml --test scenarios
cargo test --manifest-path apps/cli/Cargo.toml
cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings
```

Resultado esperado: os testes `docs_` e os cenários passam, toda a suíte passa e o clippy termina sem avisos.

### Evidências

- `docs-tests.txt` (automática, da verificação)
  - Prova: os documentos ficam intactos em todos os comandos, a aprovação recusa o template e o finish exige o enriquecimento (RF-01, RF-02, RF-03, RF-05, RF-06, RF-07, CT-01 a CT-06).
  - Trecho: as linhas `test docs_... ok`.
  - Falha: `FAILED` ou ausência dos testes `docs_`.
  - Espera: `docs_finish_keeps_structured_documents ... ok`
- `scenarios-tests.txt` (automática, da verificação)
  - Prova: os cenários de referência dão a mesma transcrição com documentos estruturados e enriquecidos nas três variantes (RF-02, CT-08).
  - Trecho: as dez linhas `test scenario_... ok`.
  - Falha: `FAILED` em algum `scenario_`.
  - Espera: `scenario_full_flow ... ok`
