### Ler

- `vostok/work/specs/refined/SPEC-025-documentos-finais-com-visual-e-conteudo-profissionais.md` (Objetivo, Estado atual, RF-01, RNF-01 a RNF-04, Registro de decisões)
- `/home/victor/projetos/ishtar-sink/org/docs/identidade-ecossistema.md` (seções "Sistema visual", "Tokens", "Tipografia"; atualize antes com `git -C /home/victor/projetos/ishtar-sink/org pull --ff-only`)
- `/home/victor/projetos/ishtar-sink/atria/docs/decisoes-do-prototipo.md` (Identidade e componentes, Layout e navegação), `/home/victor/projetos/ishtar-sink/atria/packages/theme/src/tokens.css` e `fonts.css`, e `/home/victor/projetos/ishtar-sink/atria/prototype/atria-prototype.html` (só por busca: tem ~930 KB)
- `packages/harness/templates/finalize/executive.html`, `technical.html` e `evidence.html`
- `packages/harness/config/documents.yml` (tokens atuais)
- `docs/specs/SPEC-023/executivo.html` e `docs/specs/SPEC-023/tecnico.html` (documentos atuais gerados)
- `docs/estudos/tarefa-como-pasta.md` (formato de um estudo do projeto)

### Mudar

- `docs/estudos/documentos-finais.md` (criar): estudo visual dos documentos finais, primeira parte do estudo

### Passos

1. Leia o documento de identidade, as decisões do Atria e os documentos atuais. Anote o que os documentos atuais seguem e o que contrariam da identidade (tema padrão, paleta, tipografia, raios, sombras).
2. `docs/estudos/documentos-finais.md` (criar): escreva, em prosa, as seções `## Escopo e método`, `## Identidade do Ishtar Sink aplicada a documentos`, `## Documentos atuais` (problemas visuais com exemplos dos HTML atuais) e `## Três direções visuais`. Cada direção tem um nome, o público em que pensa, a estrutura de navegação, a tipografia, o uso de cor e de superfícies, e os prós e contras. Uma das três mantém abas; as outras duas não. As três precisam ser distintas entre si.
3. No mesmo arquivo, escreva `## Requisitos comuns`: tema claro padrão com alternância para o escuro, tokens de 2026-10 citando a origem, Outfit e Inter embutidas em subconjunto (sem CDN), contraste AA, funcionamento em 400 px e 1280 px, e o tamanho estimado das fontes embutidas.
4. Deixe a seção `## Estudo de conteúdo` com uma linha dizendo que é escrita na TASK-02, e `## Escolha do modelo` dizendo que é registrada na TASK-04. Confira: `grep -n '^## ' docs/estudos/documentos-finais.md`.
