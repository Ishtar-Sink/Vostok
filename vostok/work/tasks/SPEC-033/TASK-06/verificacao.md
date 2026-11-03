### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test spec 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort | grep -v 'status_history'`
- `cargo test --manifest-path apps/cli/Cargo.toml --test plan 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort | grep -v 'status_history'`
- `cargo test --manifest-path apps/cli/Cargo.toml --test task 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort | grep -v 'status_history'`
- `cargo test --manifest-path apps/cli/Cargo.toml --test finalize 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort | grep -v 'status_history\|archive_leva_pedidos'`
- `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::read 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort | grep -v 'atividades_trazem\|atividades_leem'`

### Verificação

```bash
# evidência: historico-status-testes.txt
cargo test --manifest-path apps/cli/Cargo.toml --test spec -- status_history golden_spec
cargo test --manifest-path apps/cli/Cargo.toml --test plan -- status_history golden_plan
cargo test --manifest-path apps/cli/Cargo.toml --test task -- status_history golden_task
cargo test --manifest-path apps/cli/Cargo.toml --test finalize -- status_history golden_finalize
# evidência: atividades-historico-testes.txt
cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::read::tests::atividades_
cargo clippy --manifest-path apps/cli/Cargo.toml --features painel --all-targets -- -D warnings
cargo fmt --manifest-path apps/cli/Cargo.toml -- --check
```

### Evidências

- `historico-status-testes.txt` (automática, da verificação)
  - Prova: aprovar, marcar como planejada, concluir a última tarefa e arquivar acrescentam cada um a sua linha ao histórico de status da spec, sem mudar o hash do conteúdo nem as fixtures de equivalência (RF-12, CT-13).
  - Trecho: `test approve_records_the_status_history ... ok`, `test mark_planned_records_the_status_history ... ok`, `test finish_records_implemented_in_the_status_history ... ok`, `test archive_records_archived_in_the_status_history ... ok` e os quatro `golden_*` com `ok`.
  - Falha: algum desses testes com `FAILED`, ou um `golden_*` com `conteúdo diferente`.
- `atividades-historico-testes.txt` (automática, da verificação)
  - Prova: a `GET /api/atividades` traz o histórico de status das specs abertas e arquivadas e uma linha por transição de pedido, inclusive dos arquivados (RF-12, CT-13).
  - Trecho: `test painel::read::tests::atividades_trazem_o_historico_de_status_das_specs ... ok`, `test painel::read::tests::atividades_trazem_as_transicoes_dos_pedidos ... ok` e `test painel::read::tests::atividades_leem_os_pedidos_arquivados ... ok`.
  - Falha: algum desses testes com `FAILED`, ou `atividades_reunem_todas_as_fontes` com `FAILED`.
