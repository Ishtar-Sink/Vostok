### Ler

- `vostok/work/specs/refined/SPEC-025-documentos-finais-com-visual-e-conteudo-profissionais.md` (RF-06 a RF-08, RF-11, RN-01, RN-02, RNF-05, CB-01, CB-03, CB-05)
- `docs/estudos/documentos-finais.md` (seções `## Estudo de conteúdo` e `## Escolha do modelo`)
- `packages/harness/templates/finalize/content.md`
- `packages/harness/skills/default/finalize-spec/SKILL.md`
- `apps/cli/src/commands/finalize.rs` (fn validate, fn create, fn build e o que lê as seções do content.md)
- `apps/cli/tests/finalize.rs` (testes de validate e build)

### Mudar

- `packages/harness/templates/finalize/content.md` (alterar): seções do estudo de conteúdo
- `apps/cli/src/commands/finalize.rs` (alterar): leitura e validação das seções novas
- `apps/cli/tests/finalize.rs` (alterar): testes `content_*`
- `packages/harness/skills/default/finalize-spec/SKILL.md` (alterar): regras de escrita do estudo
- `packages/harness/templates/finalize/executive.html` (alterar): seções novas do executivo
- `packages/harness/templates/finalize/technical.html` (alterar): seções novas do técnico
- `packages/harness/templates/finalize/evidence.html` (alterar): conteúdo novo das evidências

### Passos

1. Antes de executar, confira se a TASK-04 aprovou o estudo de conteúdo e se esta tarefa já foi ajustada com as seções finais; se não, **PARE** com o template `blocker` pedindo o ajuste (skill `adjust-task`).
2. `packages/harness/templates/finalize/content.md` (alterar): troque as seções pelas do estudo, com um comentário por seção dizendo o propósito, o formato e o tamanho esperado.
3. `apps/cli/src/commands/finalize.rs` (alterar): leia as seções novas no `build` e confira no `validate` as regras que o estudo marcou como verificáveis (seção presente, tamanho, termos proibidos no executivo, RN-01). Spec sem número comprovável leva "Não se aplica." (CB-01).
4. `packages/harness/templates/finalize/*.html` (alterar): coloque as seções novas no modelo da TASK-05.
5. `packages/harness/skills/default/finalize-spec/SKILL.md` (alterar): troque as regras e o passo 4 pelas regras de escrita do estudo (formato, tamanho, público de cada documento).
6. `apps/cli/tests/finalize.rs` (alterar): testes `content_validate_refuses_an_empty_new_section`, `content_executive_has_the_new_sections` e `content_technical_has_the_support_sections`. Regrave os goldens da finalização que mudarem e revise o diff de um. Confira: `cargo test --test finalize`.
