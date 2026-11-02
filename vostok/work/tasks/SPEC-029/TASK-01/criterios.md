### Concluída quando

- [x] As aprovações pela CLI gravam `approved_by: chat`, e um ajuste que derruba a aprovação limpa o campo (RF-07, CT-06, CT-07).

  > Como comprovar: em `aprovacao-origem.txt`, ver os três testes novos passando.

- [x] Os golden de tarefa e spec continuam passando, e as funções para o painel existem sem argumento de origem na linha de comando (RF-07, RN-01).

  > Como comprovar: em `aprovacao-origem.txt`, ver `golden_task` e `golden_spec` ok; `vostok task approve --help` não cita origem.
