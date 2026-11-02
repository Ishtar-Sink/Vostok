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

## O que foi feito

`apps/cli/src/commands/documents.rs` (novo) traz o subcomando `vostok documents brand`, registrado em `cli.rs` e `commands/mod.rs`. As cores são convertidas entre hex, RGB e HSL por funções próprias, sem crate nova; `contrast` calcula a razão WCAG 2.x e `readable` anda a claridade de 1 em 1 ponto até 4,5:1 contra os fundos do tema, devolvendo se houve ajuste (cada ajuste vira uma linha `aviso:`). A derivação parte dos valores padrão do `documents.yml` embutido, não do instalado, para que `--reset` e uma segunda execução não dependam do que já foi alterado. A gravação reescreve só as linhas dos tokens derivados e do grupo `brand`, preservando comentários e a ordem do arquivo, e nada é gravado se alguma validação falhar.

`packages/harness/config/documents.yml` ganhou o grupo `brand` vazio no fim. Os testes `brand_*` em `apps/cli/tests/documents.rs` (7) cobrem a derivação nos dois temas, a secundária derivada e informada, o contraste com amarelo, quase preto, magenta e verde-azulado, o reset, o antes e depois e as recusas (cor inválida, ícone ausente, grande ou de extensão não suportada, `documents.yml` sem `bg`).

Ponto de atenção: o passo pedia `calm_*` com "saturação 80%" no claro e "50%" no escuro, e isso foi implementado como valor absoluto, não como fração da saturação da principal. A secundária derivada de um amarelo sai vermelha (matiz -60°), o que funciona no gradiente mas pode não agradar a todas as marcas; quem quiser outra informa `--secondary`.

Verificação: os 7 testes `brand_` passam, a suíte `init_update` e o clippy também; na regressão, a suíte da finalização só ganhou testes e a ajuda do CLI ganhou `documents`, como esperado.
