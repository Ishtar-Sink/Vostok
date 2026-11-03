<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** O `vostok task check` hoje só grava `validation/check.out`, sem resultado. Ele passa a gravar `check_result` e `checked_at` no front-matter, pelo mesmo `TaskDoc::set` que o `vostok task tests` usa para `tests_result`, e uma linha `check` no registro, o que também alimenta a atividade.

**Decisões.**
- Limpar em `reset_results` cobre o ajuste em execução e a retomada sem aceite de uma vez; o aceite de regressão limpa à parte, porque não passa por `reset_results`.
- O teste antigo que garantia "não toca a tarefa" é trocado, porque o comportamento muda de propósito (RF-03); o baseline o exclui.

**Riscos.** Gravar no front-matter não pode contar como edição manual; o `tests_result` já é gravado do mesmo jeito sem esse problema.

**Referências da spec.** RF-03, RN-02, CT-03, CT-04, CT-05, CB-01; contrato do `estado.yml`.

**Arquivos em comum com a onda 2.** Nenhum.
