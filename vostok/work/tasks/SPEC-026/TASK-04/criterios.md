### Concluída quando

- [ ] A skill detecta, propõe e só grava depois da confirmação (RF-01, RN-06, CT-12).

  > Como comprovar: ler os passos 3 a 6 em `customize-documents.diff` e ver no print da conversa a pergunta antes do `vostok documents brand`.

- [ ] Sem cores no repositório, a skill pergunta a cor principal e a secundária opcional (RF-03, CT-13).

  > Como comprovar: o passo 4 cobre o caso sem candidatas, e o print mostra a pergunta.

- [ ] A skill está no catálogo e no mapa de contexto, e as suítes de instalação e de contexto passam.

  > Como comprovar: `skill-tests.txt` sem `FAILED`.
