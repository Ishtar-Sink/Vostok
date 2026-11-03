### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test finalize 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort | grep -v 'archive_leva_pedidos\|status_history'`
- `cargo test --manifest-path apps/cli/Cargo.toml --test pedidos 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort | grep -v 'prune_'`

### Verificação

```bash
# evidência: arquivar-testes.txt
cargo test --manifest-path apps/cli/Cargo.toml --test finalize -- archive_ golden_finalize
# evidência: prune-testes.txt
cargo test --manifest-path apps/cli/Cargo.toml --test pedidos -- prune_
cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path apps/cli/Cargo.toml -- --check
```

### Evidências

- `arquivar-testes.txt` (automática, da verificação)
  - Prova: o `vostok finalize archive` leva para o arquivo da spec os pedidos concluídos, as perguntas e o log de avais dela, com os bytes do log intactos. Os pedidos em andamento e os de outras specs ficam, e a saída sem pedidos não muda (RF-10, CT-11, RN-05).
  - Trecho: `test archive_leva_pedidos_perguntas_e_avais_da_spec ... ok`, `test finalize_folder_archive_moves_the_task_folders_with_their_evidence ... ok` e `test golden_finalize ... ok`.
  - Falha: algum desses testes com `FAILED`.
- `prune-testes.txt` (automática, da verificação)
  - Prova: o `vostok pedidos prune` só lista sem `--confirmar`, apaga com ele, nunca apaga avais, segue o limite de 30 dias para os sem spec e diz quando não há o que limpar (RF-11, CT-12, RNF-03, CB-02, CB-05).
  - Trecho: `test prune_sem_confirmar_so_lista ... ok`, `test prune_com_confirmar_apaga_e_mantem_os_avais ... ok` e `test prune_sem_nada_diz_que_nao_ha_o_que_limpar ... ok`.
  - Falha: algum `prune_` com `FAILED` ou `running 0 tests`.
