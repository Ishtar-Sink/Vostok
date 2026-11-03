### Concluída quando

- [ ] O quadro mostra as seis colunas lado a lado numa tela de 1440 px, sem rolagem lateral (RF-14, CT-15).

  > Como comprovar: no print `quadro-1440.png`, as seis colunas aparecem inteiras, de "Para aprovar" a "Concluídas".

- [ ] Os títulos e as descrições com aspas aparecem sem as barras de escape, no quadro e na página da tarefa (RF-14, CT-15).

  > Como comprovar: no print `quadro-1440.png`, o cartão `Separar a coluna "Em validação" no quadro` sem barras; no print `tarefa-sem-escape.png`, o subtítulo com `"Em validação"` sem barras; em `titulo-sem-escape.txt`, o teste `quoted_task_title_comes_without_escapes` passa.

- [ ] A coluna "Em validação" tem a bolinha de cor de status, como Em execução, Bloqueadas e Concluídas (RF-14, CT-15).

  > Como comprovar: no print `quadro-1440.png`, a bolinha antes de "Em validação"; em `quadro-testes.txt`, o teste `mostra a cor de status nas colunas do fluxo` passa.
