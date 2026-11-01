### Ler

- `vostok/work/specs/refined/SPEC-025-documentos-finais-com-visual-e-conteudo-profissionais.md` (RF-02, RF-03, RNF-01 a RNF-04)
- `docs/estudos/documentos-finais.md` (direções visuais da TASK-01 e estudo de conteúdo da TASK-02)
- `/home/victor/projetos/ishtar-sink/atria/docs/decisoes-do-prototipo.md` (Identidade e componentes, Layout e navegação), `/home/victor/projetos/ishtar-sink/atria/packages/theme/src/tokens.css` e `fonts.css`, e `/home/victor/projetos/ishtar-sink/atria/prototype/atria-prototype.html` (só por busca: tem ~930 KB)
- `/home/victor/projetos/ishtar-sink/atria/packages/theme/fonts/` (Outfit e Inter, licença OFL)
- `vostok/work/.archive/SPEC-023/` (spec, tarefas e `content.md`: o conteúdo real a reescrever)

### Mudar

- `docs/estudos/documentos-finais` (criar): `modelo-a/`, `modelo-b/` e `modelo-c/`, cada uma com `executivo.html` e `tecnico.html`

### Passos

1. Reescreva o executivo e o técnico da SPEC-023 no modelo de conteúdo da TASK-02, uma vez, com fatos só das fontes arquivadas. O mesmo texto vai para os três modelos.
2. Gere o subconjunto das fontes Outfit e Inter com os caracteres do português (com `pyftsubset`, se houver, ou documente outro meio) e embuta como `data:` em `@font-face`. Sem nenhuma requisição externa.
3. `docs/estudos/documentos-finais/modelo-a/` (criar): o modelo da direção com abas, `executivo.html` e `tecnico.html`. `modelo-b/` e `modelo-c/`: as duas direções sem abas. Cada página tem tema claro padrão, alternância para o escuro, tokens de 2026-10 com comentário citando o documento da organização, e funciona em 400 px e 1280 px.
4. `docs/estudos/documentos-finais/README.md` (criar dentro da pasta): uma linha por modelo com o nome da direção e o caminho das páginas.
5. Confira: abra cada página sem rede e nos dois temas, e rode a Verificação.
