### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::actions 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort | grep -v 'request_actions_are_eleven\|request_actions_are_sixteen\|spec_requests_\|commit_release_and_undo\|print_review_\|prune_route_\|answer_\|second_answer_'`
- `cargo test --manifest-path apps/cli/Cargo.toml --test trava_plano_commit 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort | grep -v 'commit_approve_accepts_the_release_request\|commit_revert_needs_the_undo_request'`
- `cargo test --manifest-path apps/cli/Cargo.toml --test task -- evidence_print 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`

### Verificação

```bash
# evidência: acoes-painel-testes.txt
cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::actions
# evidência: commits-por-pedido-testes.txt
cargo test --manifest-path apps/cli/Cargo.toml --test trava_plano_commit
cargo test --manifest-path apps/cli/Cargo.toml --test task -- evidence_print
cargo clippy --manifest-path apps/cli/Cargo.toml --features painel --all-targets -- -D warnings
```

Resultado esperado: as quatro execuções terminam com código 0, sem `FAILED` nem avisos do clippy.

### Evidências

- `acoes-painel-testes.txt` (automática, da verificação)
  - Prova: o painel grava os pedidos novos com os campos certos, confere prints com a origem `painel` (aceito e rejeitado), lista e limpa com confirmação, e responde à pergunta movendo-a e criando o pedido de resposta, com 409 na segunda resposta (RF-03, RF-06, RF-07, CT-03, CT-07, CT-08, CB-06).
  - Trecho: `spec_requests_are_written_with_their_fields`, `commit_release_and_undo_requests_record_the_aval`, `print_review_accepted_records_the_panel_origin`, `print_review_rejected_blocks_the_task`, `prune_route_lists_without_deleting_and_deletes_with_confirmation`, `answer_moves_the_question_and_creates_the_reply_request`, `second_answer_is_409_and_keeps_the_first` e `request_actions_are_sixteen` com `ok`.
  - Falha: algum desses testes com `FAILED`, ou `test result: FAILED`.
  - Espera: `second_answer_is_409_and_keeps_the_first ... ok`
- `commits-por-pedido-testes.txt` (automática, da verificação)
  - Prova: com a trava ligada, liberar e desfazer commits só funcionam com o pedido do painel e o aval (RF-07, RN-02, CT-08).
  - Trecho: `test commit_approve_accepts_the_release_request ... ok` e `test commit_revert_needs_the_undo_request ... ok`.
  - Falha: algum desses testes com `FAILED`.
  - Espera: `test commit_revert_needs_the_undo_request ... ok`
