### Concluída quando

- [x] Dois painéis rodam ao mesmo tempo em portas diferentes, só em `127.0.0.1`, cada um com a sua marca, e o fluxo completo funciona (RF-01, CT-01, RNF-02).

  > Como comprovar: em `integracao-testes.txt`, ver os testes de `--test painel` passando; em `painel-final.png`, ver o painel com dados reais.

- [x] A release gera o bundle e confere o tamanho, e o binário cabe em 6,0 MB e 2,3 MB (RF-13, CT-15, RNF-01).

  > Como comprovar: em `tamanho.txt`, ver os dois tamanhos dentro do limite; no `release.yml`, ver os passos de Node, do bundle e do tamanho.

- [x] A suíte da CLI passa com e sem a feature (RNF-04).

  > Como comprovar: em `integracao-testes.txt`, ver as duas execuções do `cargo test` com `test result: ok`.
