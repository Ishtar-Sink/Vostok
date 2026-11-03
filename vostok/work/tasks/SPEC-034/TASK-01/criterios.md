### Concluída quando

- [x] A visão geral e a spec do painel dizem que o plano está desatualizado quando a spec foi aprovada de novo depois dele ou perdeu a aprovação (RF-01, CT-01).

  > Como comprovar: em `plano-desatualizado-api.txt`, os testes `plan_outdated_true_when_the_spec_was_approved_again` e `plan_outdated_true_when_the_spec_lost_the_approval` aparecem com `ok`: as duas respostas do painel trazem `"plan_outdated":true`.

- [x] Com o plano feito para a versão aprovada da spec, o painel diz que ele está em dia (RF-01, CT-01).

  > Como comprovar: em `plano-desatualizado-api.txt`, o teste `plan_outdated_false_when_the_hashes_match` aparece com `ok`: as respostas trazem `"plan_outdated":false`.

- [x] Uma spec aprovada sem plano não aparece com plano desatualizado (CB-02).

  > Como comprovar: em `plano-desatualizado-api.txt`, o teste `plan_outdated_false_without_a_plan` aparece com `ok`: as respostas trazem `"plan":false,"plan_outdated":false`.

- [x] A regra é a mesma da CLI, sem cópia, e o restante do painel e dos planos continua funcionando (RF-01).

  > Como comprovar: a função nova do plano chama a regra já usada pelo `vostok plan validate`; os testes atuais do painel e do plano passam sem regressão contra o baseline, e o clippy termina sem avisos.
