### Ler

- `vostok/work/specs/refined/SPEC-034-corrigir-o-fluxo-do-painel-antes-da-v0-4-0.md` (RF-06, RF-07, CT-08, CT-09, CB-04, causa raiz e decisões 9 e 10)
- `apps/cli/src/commands/spec.rs` (fn drop_approval, fn record, fn record_manual, fn record_status, fn approve perto da chamada `record_status(&refined, &specs.status("approved")?)`, fn panel_approval_error e o `use crate::commands::task::{...}` do topo)
- `apps/cli/src/commands/task.rs` (const PANEL_WITHOUT_AVAL, linha 245, e fn panel_approval_error perto da linha 3000)
- `apps/cli/tests/spec.rs` (testes approve_records_the_status_history e spec_approve_records_origin_chat)
- `apps/cli/tests/aval.rs` (const PANEL_WITHOUT_AVAL e teste panel_approval_without_aval_is_reported)
- `apps/cli/tests/common/golden.rs` (fn normalize: as fixtures ignoram as linhas `status_history:`)

### Mudar

- `apps/cli/src/commands/spec.rs` (alterar): `drop_approval` grava a volta para refined no `status_history`; `panel_approval_error` usa a mensagem com o log da spec
- `apps/cli/src/commands/task.rs` (alterar): a constante `PANEL_WITHOUT_AVAL` vira a função `panel_without_aval(spec_id)`, que cita `work/avais/<SPEC>.log`
- `apps/cli/tests/spec.rs` (alterar): testes do `status_history` no ajuste da spec aprovada e da só refinada
- `apps/cli/tests/aval.rs` (alterar): a mensagem esperada cita o log da spec e nunca `avais.log`

### Passos

1. `apps/cli/tests/spec.rs` (alterar): depois de `approve_records_the_status_history`, acrescente dois auxiliares e três testes. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test spec -- adjust_history_` compila e os testes `adjust_history_records_the_return_to_refined` e `adjust_history_manual_adjust_records_the_return_to_refined` falham antes da mudança.
   - `fn refined_spec(project: &Project) -> std::path::PathBuf`: o mesmo preparo de `approve_records_the_status_history` até o `adjust record-manual` (`spec new Exemplo`, `spec refine SPEC-001 improvement`, `fill-refined.py`, `AGENTS.md` vazio, `spec adjust record-manual SPEC-001`), devolvendo o caminho `work/specs/refined/SPEC-001-exemplo.md` do harness. Não mude os testes atuais.
   - `fn status_history(file: &std::path::Path) -> String`: o valor depois de `status_history: ` no arquivo, ou texto vazio sem a linha.
   - `adjust_history_records_the_return_to_refined` (CT-08, RF-06): `refined_spec`, `spec approve approve SPEC-001`, `spec adjust record SPEC-001 "Ajuste de teste"`. O `status:` do arquivo é `refined`; o histórico tem duas entradas separadas por `; `, a primeira termina em ` approved` e a última em ` refined`.
   - `adjust_history_manual_adjust_records_the_return_to_refined` (RF-06): `refined_spec`, `spec approve approve SPEC-001`, troque no arquivo a primeira ocorrência de `Objetivo.` por `Objetivo editado à mão.` (edição dentro de uma seção, que mantém a spec válida) e rode `spec adjust record-manual SPEC-001`. O histórico termina em ` refined` e tem duas entradas.
   - `adjust_history_unchanged_for_a_refined_spec` (CB-04): `refined_spec`, guarde `status_history(&file)` (vazio: só a aprovação grava o histórico), rode `spec adjust record SPEC-001 "Ajuste de teste"`; o histórico continua igual ao guardado e o `status:` continua `refined`.
2. `apps/cli/src/commands/spec.rs` (alterar): em `fn drop_approval`, logo depois de `frontmatter_put(file, APPROVED_BY_FIELD, "null")?;` e antes do `println!`, acrescente `record_status(file, &refined_status)?;`. O retorno antecipado de spec fora de `approved` fica como está, para a spec só refinada não ganhar linha (CB-04). Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test spec` sem falhas (o `golden_spec` ignora `status_history:`).
3. `apps/cli/tests/aval.rs` (alterar): troque a constante `PANEL_WITHOUT_AVAL` por `"approved_by: painel sem aval correspondente em work/avais/SPEC-001.log"` e, depois de `panel_approval_without_aval_is_reported`, acrescente `panel_without_aval_cites_the_spec_log` (CT-09, RF-07): o mesmo preparo das duas partes de `panel_approval_without_aval_is_reported` (tarefa aprovada com `approved_by: painel` e `task adjust validate`; spec aprovada com `approved_by: painel` e `spec validate SPEC-001`), sem gravar aval; cada recusa contém `work/avais/SPEC-001.log` e não contém `avais.log`; a da tarefa contém `TASK-01: approved_by: painel`. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test aval -- panel_` compila e os dois testes falham antes do passo 4.
4. `apps/cli/src/commands/task.rs` (alterar): substitua a constante da linha 245 e o comentário dela por:

   ```rust
   /// Problema de validação de uma aprovação do painel sem a linha correspondente no log de avais da spec.
   pub fn panel_without_aval(spec_id: &str) -> String {
       format!("approved_by: painel sem aval correspondente em work/avais/{spec_id}.log")
   }
   ```

   Em `fn panel_approval_error`, troque `Ok(Some(format!("{id}: {PANEL_WITHOUT_AVAL}")))` por `Ok(Some(format!("{id}: {}", panel_without_aval(spec))))`. Em `apps/cli/src/commands/spec.rs` (alterar), troque `PANEL_WITHOUT_AVAL` por `panel_without_aval` no `use crate::commands::task::{...}` e, em `fn panel_approval_error`, `Ok(Some(PANEL_WITHOUT_AVAL.to_string()))` por `Ok(Some(panel_without_aval(spec_id)))`. Confira: `grep -rn "PANEL_WITHOUT_AVAL" apps/cli/src` sem resultado e `cargo test --manifest-path apps/cli/Cargo.toml --test aval` sem falhas.
5. `apps/cli` (conferir): `cargo test --manifest-path apps/cli/Cargo.toml --test task` e `cargo test --manifest-path apps/cli/Cargo.toml --lib commands::task::` sem falhas, e `cargo clippy --manifest-path apps/cli/Cargo.toml --features painel --all-targets -q -- -D warnings` sem saída.
