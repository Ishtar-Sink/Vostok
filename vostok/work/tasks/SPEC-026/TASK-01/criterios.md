### Concluída quando

- [x] O comando grava a cor principal, a secundária, o nome e o ícone no `documents.yml` (RF-04, RF-05, RF-06, RF-07).

  > Como comprovar: rodar `vostok documents brand --primary "#0f766e" --name "Atria"` numa cópia do harness e ver o grupo `brand` preenchido e os tokens de marca mudados; o teste `brand_primary_rewrites_the_brand_tokens_of_both_themes` passa.

- [x] As cores de estado não mudam (RN-01).

  > Como comprovar: no mesmo teste, `success`, `warning`, `danger` e `info` continuam iguais.

- [x] Os pares de texto passam 4,5:1 nos dois temas e o ajuste é avisado (RF-09, CT-08).

  > Como comprovar: o teste `brand_text_pairs_pass_aa_in_both_themes` passa e `brand-saida.txt` mostra o aviso para `#facc15`.

- [x] `--reset` volta à identidade padrão (RF-10, CT-09).

  > Como comprovar: o teste `brand_reset_restores_the_default_tokens` passa.

- [x] A saída mostra o antes e depois de cada token (RF-11, CT-10).

  > Como comprovar: `brand-saida.txt` tem uma linha `tema.token: antes -> depois` por token alterado.

- [x] Entradas inválidas são recusadas sem gravar nada (CT-14, CB-01, CB-06).

  > Como comprovar: o teste `brand_refuses_invalid_input` passa.

- [x] A suíte de `init_update` e o clippy passam.

  > Como comprovar: `brand-tests.txt` sem `FAILED` e o clippy sem avisos.
