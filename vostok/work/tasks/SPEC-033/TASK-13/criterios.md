### Concluída quando

- [x] A CLI compilada responde `vostok 0.4.0` (RF-15, CT-16).

  > Como comprovar: abrir `versao.txt` e ver a linha `vostok 0.4.0`.

- [x] O `Cargo.toml`, o `Cargo.lock` e o teste da versão estão em 0.4.0, sem outras mudanças (RF-15).

  > Como comprovar: em `versao-testes.txt`, `ct01_version_is_0_1_0` passa; o diff do `Cargo.lock` tem só a linha da versão do `vostok`.
