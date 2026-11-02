### Concluída quando

- [ ] O `detect` lista as cores do repositório com arquivo, linha, nome e valor, primeiro as de marca (RF-02, CT-01).

  > Como comprovar: o teste `detect_lists_css_and_tailwind_colors_by_relevance` passa e `detect-saida.txt` mostra as linhas no formato.

- [ ] Pastas geradas e arquivos de segredo não são lidos (RN-05, CT-02).

  > Como comprovar: o teste `detect_ignores_generated_folders_and_secrets` passa.

- [ ] No máximo 10 cores, com a contagem das que ficaram de fora (CB-07).

  > Como comprovar: o teste `detect_lists_at_most_ten_candidates` passa.

- [ ] Sem cores, a saída diz isso e termina em 0 (RF-03).

  > Como comprovar: o teste `detect_without_candidates_says_so` passa.
