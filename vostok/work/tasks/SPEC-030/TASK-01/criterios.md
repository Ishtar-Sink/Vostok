### Concluída quando

- [ ] O `avais.log` é lido nos formatos de 4 e 5 colunas, e uma linha malformada é apontada sem valer como aval (RF-02, CT-02, CB-02, CB-03).

  > Como comprovar: em `aval-leitura.txt`, ver os testes de leitura passando.

- [ ] A chave `approval_lock` vem desligada e, sem ela, a trava fica desligada (RF-01).

  > Como comprovar: ver `approval_lock: false` no `main.yml` do pacote e o teste `lock_is_off_without_the_key` ok em `aval-leitura.txt`.
