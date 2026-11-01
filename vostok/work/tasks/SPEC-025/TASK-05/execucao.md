### Ler

- `vostok/work/specs/refined/SPEC-025-documentos-finais-com-visual-e-conteudo-profissionais.md` (RF-09, RF-10, RF-12, RNF-01 a RNF-04, CB-02 a CB-05)
- `docs/estudos/documentos-finais.md` (seções `## Requisitos comuns` e `## Escolha do modelo`)
- `docs/estudos/documentos-finais` (as páginas do modelo escolhido)
- `apps/cli/src/commands/finalize.rs` (fn build, fn markdown_html, fn done_html, fn build_evidence)
- `apps/cli/tests/finalize.rs` (fn golden_finalize e testes finalize_folder_*)
- `apps/cli/tests/common/golden.rs` (como os casos são regravados)

### Mudar

- `packages/harness/templates/finalize/executive.html` (alterar): modelo escolhido
- `packages/harness/templates/finalize/technical.html` (alterar): modelo escolhido
- `packages/harness/templates/finalize/evidence.html` (alterar): modelo escolhido
- `packages/harness/templates/finalize/evidence-item.html` (alterar): item de evidência no modelo
- `packages/harness/config/documents.yml` (alterar): tokens de 2026-10, tema claro padrão e fontes
- `apps/cli/src/commands/finalize.rs` (alterar): tema claro padrão, fontes embutidas e títulos no técnico das tarefas
- `apps/cli/tests/finalize.rs` (alterar): testes `visual_*`
- `apps/cli/tests/fixtures/golden/finalize/0357.case` (alterar): HTML no modelo novo
- `apps/cli/tests/fixtures/golden/finalize/0358.case` (alterar): HTML no modelo novo
- `apps/cli/tests/fixtures/golden/finalize/0366.case` (alterar): HTML no modelo novo
- `apps/cli/tests/fixtures/golden/finalize/0367.case` (alterar): HTML no modelo novo
- `apps/cli/tests/fixtures/golden/finalize/0401.case` (alterar): HTML no modelo novo
- `apps/cli/tests/fixtures/golden/finalize/0402.case` (alterar): HTML no modelo novo
- `apps/cli/tests/fixtures/golden/finalize/0403.case` (alterar): HTML no modelo novo
- `apps/cli/tests/fixtures/golden/finalize/0404.case` (alterar): HTML no modelo novo
- `apps/cli/tests/fixtures/golden/finalize/0405.case` (alterar): HTML no modelo novo
- `apps/cli/tests/fixtures/golden/finalize/0406.case` (alterar): HTML no modelo novo
- `apps/cli/tests/fixtures/golden/finalize/0407.case` (alterar): HTML no modelo novo
- `apps/cli/tests/fixtures/golden/finalize/0408.case` (alterar): HTML no modelo novo
- `apps/cli/tests/fixtures/golden/finalize/0409.case` (alterar): HTML no modelo novo
- `apps/cli/tests/fixtures/golden/finalize/0410.case` (alterar): HTML no modelo novo
- `apps/cli/tests/fixtures/golden/finalize/0411.case` (alterar): HTML no modelo novo
- `apps/cli/tests/fixtures/golden/finalize/0412.case` (alterar): HTML no modelo novo
- `apps/cli/tests/fixtures/golden/finalize/0413.case` (alterar): HTML no modelo novo
- `apps/cli/tests/fixtures/golden/finalize/0414.case` (alterar): HTML no modelo novo
- `apps/cli/tests/fixtures/golden/finalize/0415.case` (alterar): HTML no modelo novo
- `apps/cli/tests/fixtures/golden/finalize/0416.case` (alterar): HTML no modelo novo
- `apps/cli/tests/fixtures/golden/finalize/0417.case` (alterar): HTML no modelo novo
- `apps/cli/tests/fixtures/golden/finalize/0418.case` (alterar): HTML no modelo novo
- `apps/cli/tests/fixtures/golden/finalize/0419.case` (alterar): HTML no modelo novo
- `apps/cli/tests/fixtures/golden/finalize/0420.case` (alterar): HTML no modelo novo
- `apps/cli/tests/fixtures/golden/finalize/0421.case` (alterar): HTML no modelo novo
- `apps/cli/tests/fixtures/golden/finalize/0422.case` (alterar): HTML no modelo novo
- `apps/cli/tests/fixtures/golden/finalize/0423.case` (alterar): HTML no modelo novo
- `apps/cli/tests/fixtures/golden/finalize/0427.case` (alterar): HTML no modelo novo

### Passos

1. Antes de executar, confira se a TASK-04 registrou o modelo e se esta tarefa já foi ajustada com os detalhes dele; se não, **PARE** com o template `blocker` pedindo o ajuste (skill `adjust-task`).
2. `packages/harness/config/documents.yml` (alterar): troque os tokens pelos de 2026-10 do documento da organização, com o claro como padrão e o escuro em `data-theme="dark"`, e um comentário citando a origem.
3. `packages/harness/templates/finalize/*.html` (alterar): aplique o modelo escolhido aos três documentos, com o mesmo cabeçalho, tipografia e alternância de tema. As fontes em subconjunto entram como `data:` no CSS gerado pelo `finalize.rs`.
4. `apps/cli/src/commands/finalize.rs` (alterar): em `fn markdown_html`, converta `#`, `##` e `###` em títulos (rebaixados um nível dentro do técnico de cada tarefa) e mantenha código e tabelas (RF-12, CB-02). Teste `visual_task_technical_headings_render_as_headings`.
5. `apps/cli/tests/finalize.rs` (alterar): testes `visual_documents_default_to_the_light_theme`, `visual_documents_have_no_external_requests` e `visual_task_technical_headings_render_as_headings`. Confira: `cargo test --test finalize -- visual_`.
6. Regrave os casos golden da finalização com a variável de regravação de `tests/common/golden.rs`, revise o diff de um caso e confira que só o HTML mudou. Confira: `cargo test --test finalize`.
