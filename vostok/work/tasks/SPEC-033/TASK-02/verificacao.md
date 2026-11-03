### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test pedidos 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort | grep -v 'folders_'`
- `cargo test --manifest-path apps/cli/Cargo.toml --test trava_plano_commit 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`
- `cargo test --manifest-path apps/cli/Cargo.toml --lib aval:: 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`
- `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel:: 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort | grep -v 'next_request_id_follows_the_highest\|pedidos_e_perguntas_vem_das_subpastas'`
- `cargo test --manifest-path apps/cli/Cargo.toml --features painel --test painel 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`

### Verificação

```bash
# evidência: pastas-pedidos-testes.txt
cargo test --manifest-path apps/cli/Cargo.toml --test pedidos
cargo test --manifest-path apps/cli/Cargo.toml --lib -- commands::pedidos aval::
cargo test --manifest-path apps/cli/Cargo.toml --test trava_plano_commit
# evidência: pastas-painel-testes.txt
cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::
cargo test --manifest-path apps/cli/Cargo.toml --features painel --test painel
cargo clippy --manifest-path apps/cli/Cargo.toml --features painel --all-targets -- -D warnings
```

### Evidências

- `pastas-pedidos-testes.txt` (automática, da verificação)
  - Prova: o pedido passa de `pendente/` a `em_andamento/` e a `feito/` por `rename`, com uma linha de histórico por transição; `listar pendente` lê só `pendente/`; os pedidos soltos do formato antigo e os de pastas renomeadas são movidos uma vez; as pastas configuradas em inglês são usadas; o commit travado acha o pedido na subpasta (RF-08, CT-09, CT-17, RNF-01, CB-07).
  - Trecho: `test folders_start_and_finish_move_the_file ... ok`, `test folders_record_the_history ... ok`, `test folders_list_pending_reads_only_pendente ... ok`, `test folders_migrate_loose_requests ... ok`, `test folders_migrate_renamed_folders ... ok`, `test folders_follow_the_config ... ok` e os testes de `trava_plano_commit` com `ok`.
  - Falha: algum desses testes com `FAILED` ou erro de compilação.
- `pastas-painel-testes.txt` (automática, da verificação)
  - Prova: o painel grava o pedido novo em `pendente/` com o próximo ID de todas as subpastas e lê pedidos e perguntas abertas das subpastas (RF-08, CT-09).
  - Trecho: `test painel::actions::tests::next_request_id_follows_the_highest_of_every_folder ... ok`, `test painel::read::tests::pedidos_e_perguntas_vem_das_subpastas ... ok` e `test approve_edit_request_and_live_update_end_to_end ... ok`.
  - Falha: algum desses testes com `FAILED` ou aviso do clippy.
