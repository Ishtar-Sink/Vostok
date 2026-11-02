### Concluída quando

- [x] O spike está na worktree `/home/victor/projetos/ai-hub-painel`, no branch `spike/painel-local`, sem nada na `main` além do documento (CA-13).

  > Como comprovar: `git -C /home/victor/projetos/ai-hub-painel branch --show-current` imprime `spike/painel-local`; `git status` do repositório principal só mostra `docs/estudos/painel-local/`.

- [x] O tamanho do binário de cada servidor HTTP e de cada empacotamento está medido contra a base (PQ-02, PQ-03).

  > Como comprovar: em `medicoes.txt`, ver as 5 variantes com os bytes do binário, do tar.gz e do bundle.

- [x] O limite do binário está proposto com a justificativa e a variante recomendada (PQ-04).

  > Como comprovar: em `medicoes.diff`, ver a seção `Limite proposto` com o número e o motivo.
