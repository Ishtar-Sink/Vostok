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

1. Confira em `docs/estudos/documentos-finais.md`, seção `## Escolha do modelo`, que o estudo de conteúdo foi aprovado sem ajustes; se não, **PARE** com o template `blocker` pedindo o ajuste desta tarefa.
2. `packages/harness/templates/finalize/content.md` (alterar): troque as seções pelas do estudo, nesta ordem, com um comentário por seção com o propósito, o formato e o tamanho da tabela do estudo:
   - `# Executivo`: `## Em resumo`, `## Em números`, `## Objetivo`, `## O que foi entregue`, `## Valor e impacto`, `## Ganhos secundários`, `## Riscos e limites`, `## Próximos passos`;
   - `# Técnico`: `## Em resumo`, `## Em números`, `## Contexto`, `## Como funciona`, `## Decisões`, `## O que foi feito`, `## Operação e diagnóstico`, `## Mapa do código`, `## Limites e dívidas`, `## Referências`;
   - `# Evidências`: `## Em resumo`, `## Mapa de cobertura` (tabela `| ID | O que foi verificado | Evidência |`) e os blocos `## <TASK> / <arquivo>`, cada um com `- **Prova:**`, `- **Como chegar:**` e `- **Como ler:**`.
   `## Em números` tem até 3 itens no formato `- **<valor>** <rótulo>`, ou "Não se aplica."; é a coluna de números da faixa do resumo (TASK-05). Saem `Panorama`, `Indicadores de valor`, `Recorte`, `Entregas` (com os campos por entrega), `Antes e depois`, `Riscos e limitações`, `Visão geral`, `Arquitetura e componentes`, `Aprendizados` e `Pontos de atenção`.
3. `apps/cli/src/commands/finalize.rs` (alterar): leia as seções novas no `build` e, no `validate`, confira só as regras que o estudo marcou como verificáveis, cada uma com mensagem própria:
   - seção presente e não vazia; `## Próximos passos` e `## Em números` aceitam "Não se aplica." (CB-01);
   - `## Em números` com no máximo 3 itens, todos no formato `- **<valor>** <rótulo>`;
   - no técnico, `## Mapa do código` com uma tabela de pelo menos uma linha e `## Operação e diagnóstico` com pelo menos um passo numerado ou uma linha de tabela;
   - nas evidências, `## Mapa de cobertura` citando todos os IDs `RF-` e `RNF-` da spec, e cada bloco de evidência com "Prova", "Como chegar" e "Como ler";
   - no executivo, nenhuma linha com código entre crases (caminho ou comando, RN-01);
   - nenhum item de lista acima de 300 caracteres;
   - um aviso, não um erro, quando o executivo passa de 1.000 palavras.
   Remova a checagem dos campos por entrega (`delivery_errors`), que deixa de existir.
4. `packages/harness/templates/finalize/*.html` (alterar): coloque as seções novas no modelo da TASK-05: `## Em resumo` na faixa, `## Em números` na coluna de números da faixa e as demais como cartões, na ordem do `content.md`; no de evidências, o mapa de cobertura como o primeiro cartão.
5. `packages/harness/skills/default/finalize-spec/SKILL.md` (alterar): troque as regras e o passo 4 pelas regras de escrita do estudo: prosa por padrão e cada seção abrindo pela conclusão; lista só para passos em ordem ou itens curtos e paralelos, até 7, de no máximo duas linhas; tabela para comparar, para números e para o mapa de cobertura; números sempre com a frase que os explica; decisões com contexto, alternativa e consequência; o público e o tamanho de cada documento (executivo de 400 a 800 palavras, técnico de 1.000 a 2.000) como faixa de referência.
6. `apps/cli/tests/finalize.rs` (alterar): testes `content_validate_refuses_an_empty_new_section`, `content_executive_has_the_new_sections`, `content_technical_has_the_support_sections`, `content_validate_refuses_code_in_the_executive`, `content_validate_requires_every_requirement_in_the_coverage_map` e `content_numbers_render_in_the_summary_band`. Regrave os goldens da finalização que mudarem e revise o diff de um. Confira: `cargo test --test finalize`.
