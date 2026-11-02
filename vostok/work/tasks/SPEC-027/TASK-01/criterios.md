### Concluída quando

- [ ] Cada skill do catálogo aparece no inventário com a forma no painel e o motivo (PQ-05).

  > Como comprovar: em `levantamento.diff`, ver uma linha por skill na tabela `Inventário das ações` e a contagem por forma.

- [ ] A aprovação pelo painel compara as três alternativas e recomenda uma, citando as regras de `rules/agent/` (PQ-06).

  > Como comprovar: em `levantamento.diff`, ver a seção `Aprovação pelo painel` com os IDs das regras e a linha `Recomendação:`.

- [ ] As regras, a memória e as pastas ligadas ao servidor estão listadas com a mudança recomendada, sem nenhuma alteração feita (PQ-11).

  > Como comprovar: em `levantamento.diff`, ver a tabela `Remoção do servidor`; `git status` não mostra `rules/` nem `apps/` alterados.
