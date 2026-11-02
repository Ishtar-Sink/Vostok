<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Um módulo novo, `apps/cli/src/commands/documents.rs`, com o subcomando `brand`. A conversão de cores, o contraste WCAG 2.x e a derivação são funções puras do módulo, sem crate nova ([project/architecture-03]). A gravação reescreve só as linhas dos tokens derivados e do grupo `brand`, preservando comentários e ordem, porque o `documents.yml` instalado é editado pelo usuário e pelo `vostok update` (RN-02, RN-03).

**Decisões.**
- Derivação em HSL, ajustando a claridade de 1 em 1 até 4,5:1; descartado OKLCH, que dá cores mais uniformes mas exige mais código sem necessidade presente (RF-09, inferência C).
- Neutros com o matiz da principal e a claridade do padrão, saturação limitada a 40% (inferência I); descartado manter os neutros lavanda, que destoam de marcas de outra cor.
- Secundária ausente = principal com matiz -60° (inferência J); descartado gradiente de uma cor, que perde o desenho da faixa.
- Defaults lidos do `documents.yml` embutido, para que `--reset` e a derivação não dependam do arquivo instalado já alterado.

**Riscos.** Cores muito saturadas podem gerar neutros fortes; o limite de saturação mitiga. Editar o YAML por linha exige cuidado com as linhas de comentário; os testes comparam o arquivo inteiro nos casos de recusa.

**Referências.** RF-04, RF-05, RF-09 a RF-11, RN-01 a RN-03, CT-03, CT-04, CT-08 a CT-10, CT-14, CB-01, CB-04 a CB-06; decisões 5 a 8, 13 e 14 da spec.

**Arquivos em comum com a onda 1.** Nenhum: a TASK-03 não toca `documents.yml`, `cli.rs`, `mod.rs` nem `documents.rs`.
