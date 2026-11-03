### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test aval 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort | grep -v 'by_spec_'`
- `cargo test --manifest-path apps/cli/Cargo.toml --test trava_plano_commit 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`
- `cargo test --manifest-path apps/cli/Cargo.toml --lib aval:: 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`
- `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel:: 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`
- `cargo test --manifest-path apps/cli/Cargo.toml --features painel --test painel 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`

### Verificação

```bash
# evidência: avais-testes.txt
cargo test --manifest-path apps/cli/Cargo.toml --test aval
cargo test --manifest-path apps/cli/Cargo.toml --lib aval::
cargo test --manifest-path apps/cli/Cargo.toml --test trava_plano_commit
# evidência: avais-painel-testes.txt
cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::
cargo test --manifest-path apps/cli/Cargo.toml --features painel --test painel
cargo clippy --manifest-path apps/cli/Cargo.toml --features painel --all-targets -- -D warnings
```

### Evidências

- `avais-testes.txt` (automática, da verificação)
  - Prova: o aval vai para o log da spec, o sem spec e as linhas malformadas para o log à parte, o `avais.log` antigo é dividido uma vez sem mudar as linhas, as pastas configuradas são usadas e o `vostok avais` mostra os avais de uma spec ou tarefa, do mais novo ao mais antigo (RF-09, CT-10, CT-17, CB-03, RNF-02).
  - Trecho: `test by_spec_task_approval_goes_to_the_spec_log ... ok`, `test by_spec_avais_command_lists_newest_first ... ok`, `test by_spec_avais_command_without_avais_says_so ... ok`, `test by_spec_legacy_log_is_split_once ... ok` e `test by_spec_folders_follow_the_config ... ok`.
  - Falha: algum desses testes, ou um dos testes atuais da trava, com `FAILED`.
- `avais-painel-testes.txt` (automática, da verificação)
  - Prova: as aprovações e os pedidos travados do painel gravam o aval no log da spec, só por acréscimo, e as atividades leem todos os logs (RF-09, RN-05).
  - Trecho: `test painel::actions::tests::task_approval_records_the_panel_origin_and_the_aval ... ok`, `test painel::actions::tests::avais_log_only_receives_appends ... ok`, `test painel::read::tests::avais_malformados_sao_ignorados ... ok` e `test approve_edit_request_and_live_update_end_to_end ... ok`.
  - Falha: algum com `FAILED` ou aviso do clippy.
