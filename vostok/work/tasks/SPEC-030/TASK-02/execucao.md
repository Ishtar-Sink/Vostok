### Ler

- `vostok/work/specs/refined/SPEC-030-trava-de-aval-e-loop-de-pedidos-do-painel.md` (RF-10, RF-12, CT-11, CT-13, "Contratos e interfaces")
- `apps/cli/src/painel/actions.rs` (const REQUEST_ACTIONS, fn approve, fn record_aval, fn request, fn request_markdown e os testes de aval e de pedidos)
- `apps/cli/src/commands/task.rs` (fn approve_from_panel, para ver de onde vem o hash aprovado)
- `apps/cli/src/support.rs` (fn frontmatter_get, fn sha256_hex)

### Mudar

- `apps/cli/src/painel/actions.rs` (alterar): aval de 5 colunas, aval gravado junto dos pedidos travados e a ação `fazer-commit`.

### Passos

1. `apps/cli/src/painel/actions.rs` (alterar): escreva primeiro os testes no `mod tests`:
   - `approval_aval_has_the_action_column`: depois de aprovar uma tarefa, a última linha do `avais.log` termina em `\tpainel\taprovar` (CT-13);
   - `finish_request_records_the_aval`: `POST /api/pedidos?acao=finalizar-tarefa&spec=SPEC-001&tarefa=TASK-01` numa tarefa aprovada grava o `PED-001.md` e a linha `SPEC-001/TASK-01`, o `approved_hash` da tarefa, `painel`, `finalizar-tarefa` (CT-11);
   - `plan_and_spec_requests_record_the_aval`: `aprovar-plano` grava o `content_hash` do plano com o ID da spec; `finalizar-spec` grava o `content_hash` da spec refinada;
   - `commit_request_records_the_aval`: `acao=fazer-commit` com o corpo `Commit da onda 1.` grava o pedido e a linha `PED-001`, `sha256` do texto, `painel`, `fazer-commit`;
   - `unlocked_requests_record_no_aval`: `ajustar-tarefa` não cria o `avais.log`;
   - `request_actions_are_eleven`: `REQUEST_ACTIONS.len() == 11`.
   Confira: `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::actions` falha nesses testes.
2. Em `record_aval`, receba a ação e grave a quinta coluna: `data\tID\thash\tpainel\tação`; a aprovação passa `aprovar` (RF-12).
3. Acrescente `"fazer-commit"` a `REQUEST_ACTIONS` e atualize o teste que contava 10 ações.
4. Em `request`, depois de gravar o pedido, para as ações `finalizar-tarefa`, `aprovar-plano`, `finalizar-spec` e `fazer-commit`, grave o aval com o hash do alvo (RF-10):
   - `finalizar-tarefa`: `approved_hash` do `estado.yml` da tarefa em pasta ou do front-matter da tarefa em arquivo único; sem `approved_hash`, responda 422 `a tarefa não está aprovada` e apague o pedido criado;
   - `aprovar-plano`: `content_hash` de `<plans>/<SPEC>.md`;
   - `finalizar-spec`: `content_hash` da spec refinada (o arquivo de `refined/` com o ID);
   - `fazer-commit`: `sha256_hex` do texto do pedido sem espaços nas pontas, com o ID do pedido (`PED-NNN`).
   Uma falha ao gravar o aval responde 500 e apaga o pedido, para não sobrar pedido travado sem aval.
5. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel` e `cargo test --manifest-path apps/cli/Cargo.toml --features painel --test painel_server` passam; `cargo clippy --manifest-path apps/cli/Cargo.toml --features painel --all-targets -- -D warnings` sem avisos.
