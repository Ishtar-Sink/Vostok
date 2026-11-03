### Ler

- `vostok/work/specs/refined/SPEC-033-operar-o-projeto-de-ponta-a-ponta-pelo-painel-v0-4-0.md` (RF-12, CT-13)
- `docs/estudos/painel-ponta-a-ponta.md` (seções "Status das entidades" e "Fontes de atividade")
- `vostok/work/tasks/SPEC-033/TASK-02/tecnico.md` (formato do histórico do pedido e das pastas por estado)
- `vostok/work/tasks/SPEC-033/TASK-05/tecnico.md` (destino `<archive>/<SPEC>/pedidos/`, sem subpasta de estado)
- `apps/cli/src/commands/spec.rs` (fn approve, fn frontmatter_put, fn content_hash, fn front_matter_body)
- `apps/cli/src/commands/plan.rs` (fn mark_planned e a conferência do hash da spec em `sha256_hex(frontmatter_body(...))`)
- `apps/cli/src/commands/task.rs` (fn finish, trecho que grava `status.implemented`)
- `apps/cli/src/commands/finalize.rs` (fn copy_to_archive, trecho que grava `status.archived` no `spec.md` arquivado)
- `apps/cli/src/commands/pedidos.rs` (fn transition, com o histórico da TASK-02)
- `apps/cli/src/painel/read.rs` (fn activities, fn request_rows, fn adjustment_rows, fn archived, struct ActivityRow, fn activity_project e os testes `atividades_*`)
- `apps/cli/tests/common/golden.rs` (fn normalize)
- `apps/cli/tests/spec.rs` (teste `spec_approve_records_origin_chat`)
- `apps/cli/tests/plan.rs` (fn plan_with_evidence, teste `evidence_name_with_a_path_is_renamed_by_mark_planned`)
- `apps/cli/tests/task.rs` (fn running_task, fn task_before_baseline, teste `accepted_difference_lets_the_task_finish`)
- `apps/cli/tests/finalize.rs` (fn built_folder_project, teste `finalize_folder_archive_moves_the_task_folders_with_their_evidence`)

### Mudar

- `apps/cli/src/commands/spec.rs` (alterar): `pub fn record_status`, que acrescenta a mudança de status ao `status_history` do front-matter, e a chamada na aprovação (RF-12)
- `apps/cli/src/commands/plan.rs` (alterar): chamada de `spec::record_status` ao marcar `planned`, porque é o comando que grava esse status
- `apps/cli/src/commands/task.rs` (alterar): chamada de `spec::record_status` ao marcar `implemented`, porque é o comando que grava esse status
- `apps/cli/src/commands/finalize.rs` (alterar): chamada de `spec::record_status` ao gravar `archived` no `spec.md` arquivado, porque é o comando que grava esse status
- `apps/cli/src/painel/read.rs` (alterar): o histórico de status das specs e as transições dos pedidos, abertos e arquivados, na `GET /api/atividades` (RF-12, CT-13)
- `apps/cli/tests/common/golden.rs` (alterar): a normalização ignora a linha `status_history:`, que os scripts bash nunca gravaram
- `apps/cli/tests/spec.rs` (alterar): teste da linha `approved`
- `apps/cli/tests/plan.rs` (alterar): teste da linha `planned`
- `apps/cli/tests/task.rs` (alterar): teste da linha `implemented`
- `apps/cli/tests/finalize.rs` (alterar): teste da linha `archived`

### Passos

1. Testes dos comandos (alterar), cada um conferindo que o front-matter da spec ganhou `status_history:` com o status no fim do valor:
   - `apps/cli/tests/spec.rs`: `approve_records_the_status_history`, que copia o preparo de `spec_approve_records_origin_chat` e confere ` approved` em `status_history:`;
   - `apps/cli/tests/plan.rs`: `mark_planned_records_the_status_history`, que reaproveita `plan_with_evidence` com a evidência `- \`saida.txt\` (automática): \`echo ok\`` e `SUBITEMS`, roda `plan approve mark-planned SPEC-001` e confere ` planned` no `status_history:` da spec refinada, que continua com o mesmo `content_hash`;
   - `apps/cli/tests/task.rs`: `finish_records_implemented_in_the_status_history`, que parte de `running_task(&[])`, apaga `vostok/work/tasks/SPEC-001/TASK-02.md`, grava `vostok/work/specs/refined/SPEC-001-exemplo.md` com `---\nid: SPEC-001\nstatus: planned\nupdated_at: x\n---\n`, roda `tests`, `regression`, `evidence` e `finish` e confere `status: implemented` e ` implemented` em `status_history:`;
   - `apps/cli/tests/finalize.rs`: `archive_records_archived_in_the_status_history`, que usa `built_folder_project()`, roda `finalize archive SPEC-001` e confere ` archived` no `status_history:` de `vostok/work/.archive/SPEC-001/spec.md`.

   Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test spec -- status_history` falha antes do passo 3.
2. `apps/cli/src/painel/read.rs` (alterar, testes): acrescente os testes abaixo. Confira: falham antes do passo 5.
   - `atividades_trazem_o_historico_de_status_das_specs` (CT-13): spec aberta com `status_history: 2026-11-01T10:00:00Z approved; 2026-11-02T10:00:00Z planned` e spec arquivada com `... archived`. `GET /api/atividades?tipo=spec` traz uma linha por entrada, com `step` `status` e `result` igual ao status, da mais nova para a mais antiga.
   - `atividades_trazem_as_transicoes_dos_pedidos` (CT-13): pedido em `feito/` com o histórico no formato gravado pela TASK-02 (criação, `em_andamento` e `feito`). `GET /api/atividades?tipo=pedido` traz uma linha por transição, com `result` igual ao estado novo.
   - `atividades_leem_os_pedidos_arquivados`: pedido em `.archive/SPEC-000/pedidos/PED-009.md` aparece com `spec` `SPEC-000`.
3. `apps/cli/src/commands/spec.rs` (alterar): acrescente `pub const STATUS_HISTORY_FIELD: &str = "status_history";` e `pub fn record_status(file: &Path, status: &str) -> Result<()>`. A função lê o valor atual com `frontmatter_get`, ausente vira vazio, acrescenta `<utc_now()> <status>` separado por `; ` e grava com o `frontmatter_put` do próprio módulo. Em `fn approve`, chame `record_status(&refined, &specs.status("approved")?)` logo depois de gravar `APPROVED_BY_FIELD`. A linha fica no front-matter, fora do corpo, então o `content_hash` e o `approved_hash` não mudam. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test spec -- status_history`.
4. Chamadas nos outros comandos, sem gravar a linha de outro jeito:
   - `apps/cli/src/commands/plan.rs` (alterar): em `fn mark_planned`, depois de `write(&spec, ...)` com o status `planned`, chame `crate::commands::spec::record_status(&spec, &planned)?`;
   - `apps/cli/src/commands/task.rs` (alterar): em `fn finish`, depois de `frontmatter_put(&spec_file, "updated_at", ...)`, chame `crate::commands::spec::record_status(&spec_file, &implemented)?`;
   - `apps/cli/src/commands/finalize.rs` (alterar): em `fn copy_to_archive`, depois do `frontmatter_put(&spec, "updated_at", ...)`, chame `crate::commands::spec::record_status(&spec, &archived)` com `.map_err(|e| format!("{e}\n"))`.

   Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test plan -- status_history`, e o mesmo com `--test task` e `--test finalize`.
5. `apps/cli/src/painel/read.rs` (alterar):
   - acrescente `fn status_rows(&self) -> Vec<ActivityRow>`, que lê `status_history` das specs abertas (`self.specs()`) e arquivadas (`self.archived()`). Cada entrada `<data> <status>` vira uma linha com `kind` `spec`, `step` `status`, `result` igual ao status, `target` e `spec` iguais ao ID e `detail` vazio. Entrada sem os dois campos é ignorada. A função entra em `fn activities` junto das outras fontes;
   - troque `fn request_rows` para ler os pedidos das subpastas de estado (como a TASK-02 deixou) e de `<archive>/*/<último trecho de paths.requests>/`. Gere uma linha por entrada do histórico do pedido, com `step` `criado` na primeira e `estado` nas demais, e `result` igual ao estado. Sem histórico, mantenha as duas linhas de hoje (criação e estado atual).

   Confira: `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::read`.
6. `apps/cli/tests/common/golden.rs` (alterar): em `fn normalize`, no mesmo `filter` de `approved_by`, descarte também a linha que começa com `status_history:`, com o comentário "Os scripts bash nunca gravaram o histórico de status, que só a CLI grava.". Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test spec -- golden_spec`, e o mesmo com `--test plan -- golden_plan`, `--test task -- golden_task` e `--test finalize -- golden_finalize`.
7. Rode `cargo clippy --manifest-path apps/cli/Cargo.toml --features painel --all-targets -- -D warnings` e `cargo fmt --manifest-path apps/cli/Cargo.toml -- --check`. Confira: sem aviso nem diferença.
