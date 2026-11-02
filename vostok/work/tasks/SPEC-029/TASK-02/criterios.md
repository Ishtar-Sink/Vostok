### Concluída quando

- [x] `vostok painel` acha o harness, escuta só em `127.0.0.1` numa porta livre e para com mensagem fora de um projeto (RF-01, CT-02).

  > Como comprovar: em `servidor-testes.txt`, ver os testes de integração passando; em `sem-pasta.txt`, ver a mensagem com `vostok/config/main.yml` e código 1.

- [x] Toda ação sem o token de sessão é recusada com 403, e a página sai com o token (RF-08, CT-05).

  > Como comprovar: em `servidor-testes.txt`, ver os testes do token e do 403 passando.

- [x] O `vostok` compila sem Node e sem a feature, e o `build.rs` só gera o bundle quando pode (RF-12, CT-14).

  > Como comprovar: em `servidor-testes.txt`, ver o `cargo build` sem a feature passando.
