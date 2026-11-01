### Ler

- `vostok/work/specs/refined/SPEC-025-documentos-finais-com-visual-e-conteudo-profissionais.md` (RF-09, RF-10, RF-12, RNF-01 a RNF-04, CB-02 a CB-05)
- `docs/estudos/documentos-finais.md` (seções `## Requisitos comuns` e `## Escolha do modelo`)
- `docs/estudos/documentos-finais` (as páginas do modelo escolhido)
- `/mnt/f/downloads/documentos-finais/modelo-c-indice/` (a variante aprovada na TASK-04, ainda fora do repositório)
- `/home/victor/projetos/ishtar-sink/atria/packages/theme/fonts/` (Inter e Outfit, licença OFL)
- `apps/cli/src/commands/finalize.rs` (fn build, fn markdown_html, fn done_html, fn build_evidence)
- `apps/cli/tests/finalize.rs` (fn golden_finalize e testes finalize_folder_*)
- `apps/cli/tests/common/golden.rs` (como os casos são regravados)

### Mudar

- `packages/harness/templates/finalize/executive.html` (alterar): modelo escolhido
- `packages/harness/templates/finalize/technical.html` (alterar): modelo escolhido
- `packages/harness/templates/finalize/evidence.html` (alterar): modelo escolhido
- `packages/harness/templates/finalize/evidence-item.html` (alterar): item de evidência no modelo
- `packages/harness/config/documents.yml` (alterar): tokens de 2026-10, tema claro padrão e fontes
- `packages/harness/templates/finalize/fonts/Inter.woff2` (criar): subconjunto da Inter
- `packages/harness/templates/finalize/fonts/Outfit.woff2` (criar): subconjunto da Outfit
- `packages/harness/templates/finalize/fonts/OFL-Inter.txt` (criar): licença
- `packages/harness/templates/finalize/fonts/OFL-Outfit.txt` (criar): licença
- `packages/harness/templates/finalize/fonts/README.md` (criar): origem, comando do subconjunto e tamanhos
- `docs/estudos/documentos-finais/modelo-c-indice/executivo.html` (criar): página de referência do modelo escolhido
- `docs/estudos/documentos-finais/modelo-c-indice/tecnico.html` (criar): página de referência do modelo escolhido
- `docs/estudos/documentos-finais/README.md` (alterar): linha do modelo escolhido
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

1. Confira em `docs/estudos/documentos-finais.md`, seção `## Escolha do modelo`, que a escolha é o modelo C com o índice do B; se for outra, **PARE** com o template `blocker` pedindo o ajuste desta tarefa.
2. `docs/estudos/documentos-finais/modelo-c-indice/` (criar): copie `executivo.html` e `tecnico.html` de `/mnt/f/downloads/documentos-finais/modelo-c-indice/`. Em `docs/estudos/documentos-finais/README.md` (alterar), acrescente a linha "**Modelo escolhido (TASK-04), C com o índice do B:**" com os links das duas páginas. Essas páginas são a referência visual dos passos 5 e 6.
3. `packages/harness/templates/finalize/fonts/` (criar): gere os subconjuntos, com a autorização do usuário dada no ajuste, rodando para cada fonte de `/home/victor/projetos/ishtar-sink/atria/packages/theme/fonts/` (`InterVariable.woff2` para `Inter.woff2` e `Outfit-Variable.ttf` para `Outfit.woff2`): `uvx --from 'fonttools[woff]' pyftsubset <origem> --unicodes='U+0000-00FF,U+0131,U+0152-0153,U+02C6,U+02DA,U+02DC,U+2000-206F,U+20AC,U+2122,U+2190-2193,U+2212' --layout-features='*' --flavor=woff2 --output-file=<destino>`. Copie `OFL-Inter.txt` e `OFL-Outfit.txt` da mesma pasta. Escreva o `README.md` com a origem, o comando e o tamanho de cada arquivo. Confira: `ls -l packages/harness/templates/finalize/fonts`, cada `.woff2` abaixo de 120 KB. Se o `uvx` for recusado, **PARE** com o template `blocker`.
4. `packages/harness/config/documents.yml` (alterar): troque os blocos `dark:` e `light:` pelos tokens de 2026-10 de `atria/packages/theme/src/tokens.css`, com os nomes da organização em snake_case (`bg`, `bg_elevated`, `surface`, `surface_muted`, `surface_hover`, `border`, `border_strong`, `primary`, `primary_hover`, `primary_soft`, `primary_on`, `accent`, `accent_soft`, `text`, `text_muted`, `text_faint`, `success`, `success_soft`, `warning`, `warning_soft`, `danger`, `danger_soft`, `info`, `info_soft`, `shadow_sm`, `shadow_md`), mais `code_bg` (`surface_muted` no claro e `surface_hover` no escuro), `calm_from` e `calm_to` (`#f3ebff` e `#e0f7fb` no claro, `#1d1438` e `#0f2a36` no escuro) e `brand_from` e `brand_to` (`#7c3aed` e `#06b6d4`). Atualize o comentário: o claro é o padrão em `:root`, o escuro entra pela preferência do sistema e por `data-theme="dark"`, e a origem é `Ishtar-Sink/.github`, `docs/identidade-ecossistema.md`. O diff de evidências usa `success` e `danger`.
5. `apps/cli/src/commands/finalize.rs` (alterar): gere o CSS de tokens com o bloco `light` em `:root`, o `dark` em `@media (prefers-color-scheme: dark) { :root:not([data-theme="light"]) }` e de novo em `:root[data-theme="dark"]`, e os gradientes montados de `calm_*` e `brand_*`; nenhuma cor literal fora do que vem de `documents.yml`. Leia as fontes de `templates/finalize/fonts/` e embuta-as em `@font-face` com `data:font/woff2;base64,`, codificando com uma função própria pequena (`fn base64`), sem crate nova. Em `fn markdown_html`, converta `#`, `##` e `###` em títulos (rebaixados um nível dentro do técnico de cada tarefa) e mantenha código e tabelas (RF-12, CB-02).
6. `packages/harness/templates/finalize/*.html` (alterar): aplique o modelo da página de referência aos três documentos, com o CSS de layout copiado dela e as cores só por variável:
   - cabeçalho com a marca Vostok e o botão de tema (claro por padrão, escolha guardada no navegador com `try/catch`), rótulo "<tipo> · <SPEC>" em Outfit caixa alta, título em Outfit 800 e a linha de metadados;
   - faixa "Em resumo" com o gradiente `calm`, mostrando a primeira seção do documento; quando o documento tiver a seção `## Em números` (criada no `content.md` pela TASK-06), cada item `- **<valor>** <rótulo>` vira um bloco da coluna de números à direita, como na página de referência, e a seção não aparece entre os cartões; sem ela, a faixa ocupa a largura toda;
   - abaixo, grade com o índice de 220 px preso à esquerda, com as seções numeradas e a seção em leitura marcada (`IntersectionObserver`), e os cartões: rótulo "NN · <seção>", título e corpo com texto em até 68 caracteres; tabelas e código rolam dentro do cartão;
   - abaixo de 860 px, o índice vira um bloco de duas colunas logo depois da faixa, e os cartões ficam em uma coluna;
   - rodapé com o link para o outro documento; no de evidências, um cartão por evidência (`evidence-item.html`), no mesmo modelo. Sem abas.
7. `apps/cli/tests/finalize.rs` (alterar): testes `visual_documents_default_to_the_light_theme`, `visual_documents_have_no_external_requests`, `visual_task_technical_headings_render_as_headings` e `visual_colors_come_only_from_documents_yml` (gera os documentos e confere que toda cor hexadecimal do HTML, fora das fontes em base64, é um valor de `documents.yml`). Confira: `cargo test --test finalize -- visual_`.
8. Regrave os casos golden da finalização com a variável de regravação de `tests/common/golden.rs`, revise o diff de um caso e confira que só o HTML mudou. Confira: `cargo test --test finalize`.
