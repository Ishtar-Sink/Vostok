### Ler

- `vostok/work/specs/refined/SPEC-024-executivo-e-tecnico-preservados-e-escritos-desde-o-planejamento.md` (Defeito, RF-01 a RF-07, CB-01 a CB-08, CT-01 a CT-06, CT-08)
- `apps/cli/src/commands/task.rs` (struct TaskDoc: fn text, fn write_text, fn store_hashes; fn split_task_text; fn task_errors; fn start; fn finish; struct TaskLayout: fn hashed_files)
- `apps/cli/tests/task.rs` (fn folder_task, fn folder_started, fn folder_validated, testes folder_format_*, describe_*, migrate_*)
- `apps/cli/tests/scenarios.rs` (fn folder_files, fn write_task, impl Scenario: fn task, fn run)

### Mudar

- `apps/cli/src/commands/task.rs` (alterar): divisão e gravação da pasta sem tocar no executivo nem no técnico, validação dos documentos, hash no `start` e exigência no `finish`
- `apps/cli/tests/task.rs` (alterar): testes `docs_*` e enriquecimento dos documentos nos testes de pasta que concluem a tarefa
- `apps/cli/tests/scenarios.rs` (alterar): documentos com títulos nas variantes de pasta, enriquecidos depois do `start`
- `apps/cli/tests/plan.rs` (alterar): `fill_folder_task` escreve o executivo e o técnico, que a aprovação da tarefa passa a exigir

### Passos

1. `apps/cli/tests/task.rs` (alterar): escreva primeiro o teste que reproduz a issue #9, `docs_finish_keeps_structured_documents`: tarefa em pasta com `executivo.md` = `## O que mudou\n\nTexto do executivo.\n` e `tecnico.md` = `## O que foi feito\n\nTexto do técnico.\n`, escritos depois do `start`; rode tests, regression, evidence e finish; os dois arquivos ficam idênticos e o `registro.md` não contém `O que mudou`. Confira que ele falha hoje: `cargo test --test task -- docs_finish_keeps`.
2. `apps/cli/src/commands/task.rs` (alterar): em `fn split_task_text`, depois de um `### Resumo executivo` ou `### Detalhes técnicos`, só um `## <record_section>` ou `## <título do histórico>` muda de parte; toda outra linha (`# `, `## `, `### `, cercas de código) fica na parte atual. Em `TaskDoc::write_text`, pule `executivo.md` e `tecnico.md`: a gravação nunca os regrava (RF-01). Confira: `cargo test --test task -- docs_finish_keeps`.
3. `apps/cli/tests/task.rs` (alterar): `docs_commands_keep_structured_documents`: executivo com `# `, `## ` e `### `, técnico com um bloco de código que contém `## `; rode start, tests, um `resume` de bloqueio, `adjust record-manual` e `adjust record`; os dois arquivos ficam idênticos byte a byte (RF-02, CB-01, CB-02, CB-08). Confira: `cargo test --test task -- docs_commands_keep`.
4. `apps/cli/src/commands/task.rs` (alterar): em `fn task_errors`, na tarefa em pasta, acrescente um erro por documento com variável de template: `o documento <arquivo> de <TASK> ainda está no template; escreva-o com o que a tarefa vai fazer` (RF-03). Teste `docs_approve_refuses_a_document_in_the_template`. Confira: `cargo test --test task -- docs_approve`.
5. `apps/cli/src/commands/task.rs` (alterar): em `fn start`, depois do `record_step` de início, grave no `estado.yml` `executive_hash` e `technical_hash` com o `sha256_hex` de cada documento, só na tarefa em pasta (RF-05). Em `fn finish`, na tarefa em pasta: com o hash gravado, recuse o documento cujo hash atual for igual, com `o documento <arquivo> de <TASK> não foi enriquecido desde o início; enriqueça-o a partir de vostok task describe` (RF-06, CB-05); sem o hash, mantenha só a checagem de template (RF-07, CB-04). O arquivo único não muda (CB-06). Testes `docs_start_stores_the_document_hashes`, `docs_finish_refuses_documents_not_enriched`, `docs_finish_without_start_hashes_checks_only_the_template`. Confira: `cargo test --test task -- docs_`.
6. `apps/cli/tests/task.rs` (alterar): nos testes que concluem uma tarefa em pasta iniciada (`folder_format_keeps_the_evidence_inside_the_task`, `describe_finish_*`, `migrate_keeps_an_approval_valid_for_a_task_not_started` se concluir), enriqueça os documentos depois do `start`. Confira: `cargo test --test task`.
7. `apps/cli/tests/scenarios.rs` (alterar): em `fn folder_files`, escreva `executivo.md` como `## Plano\n\nResumo.\n` e `tecnico.md` como `## Plano\n\n```text\n## não é título\n```\n\nDetalhes.\n`; em `Scenario::task`, depois de um `start` numa tarefa em pasta, acrescente aos dois documentos `\n## Feito\n\nEnriquecido.\n` (fora da transcrição). Não altere nenhuma fixture de `apps/cli/tests/fixtures/scenarios`; diferença na transcrição é regressão: **PARE** com o template `blocker`. Confira: `cargo test --test scenarios`.
8. Rode a suíte inteira e o clippy (Verificação).
