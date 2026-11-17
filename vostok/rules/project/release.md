# Lançamento de versão

Define o que acompanha cada tag de versão publicada do vostok.


[project/release-01]: Antes de criar a tag `vX.Y.Z`, acrescente ao topo do `CHANGELOG.md` da raiz a seção `## [X.Y.Z] - AAAA-MM-DD`, com o que mudou desde a tag anterior em `### Recursos`, `### Melhorias` e `### Correções` (só as que tiverem itens), cada item em linguagem de usuário citando a spec e a issue quando houver, e inclua o arquivo no commit da versão.
[project/release-02]: Monte a seção a partir de `git log <tag-anterior>..HEAD` e das specs finalizadas no intervalo; commits de harness, documentação e testes que não mudam o comportamento ficam de fora.
[project/release-03]: Depois que o workflow publicar a release `vX.Y.Z` no GitHub, troque as notas pela seção `## [X.Y.Z]` do `CHANGELOG.md`, sem o título, com `gh release edit vX.Y.Z --notes-file <arquivo>`, e confira com `gh release view vX.Y.Z`; vale como parte do envio da tag já autorizado pelo usuário.
