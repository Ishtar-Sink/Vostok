### Concluída quando

- [ ] Os três documentos são gerados no modelo escolhido, com tema claro padrão e sem requisição externa (RF-09, RF-10, CT-07, CT-10).

  > Como comprovar: em `visual-tests.txt`, ver aprovados os testes do tema e das requisições, e conferir os prints `executivo-novo.png` e `evidencias-novo.png`.

- [ ] Os títulos do técnico de cada tarefa aparecem como títulos (RF-12, CT-09).

  > Como comprovar: em `visual-tests.txt`, ver aprovado `visual_task_technical_headings_render_as_headings`.

- [ ] Os goldens da finalização passam com o HTML novo.

  > Como comprovar: rodar `cargo test --test finalize` e ver o `golden_finalize` aprovado.
