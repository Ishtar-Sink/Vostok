<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Subcomando `detect` no módulo da TASK-01, com uma varredura própria por `std::fs` e reconhecimento por linha dos quatro formatos (CSS, SCSS, Less e Tailwind), sem crate de glob nem parser de CSS ([project/architecture-03]).

**Decisões.**
- Varredura por linha com padrões simples; descartado um parser de CSS ou de JavaScript, caro para o ganho, já que a escolha final é do usuário (RN-06).
- Relevância pelo nome (`primary`, `brand`, `accent`); descartado ranquear pela frequência, que favorece cinzas e cores de borda.
- Pastas e segredos ignorados por lista fixa (RN-05), seguindo a regra de segurança do projeto de não ler segredos.

**Riscos.** Cores de terceiros em CSS versionado podem aparecer; a ordenação e o arquivo e linha deixam isso visível.

**Referências.** RF-02, RF-03, RN-05, CT-01, CT-02, CB-07; decisão 2 da spec.

**Arquivos em comum.** Altera `documents.rs` e `tests/documents.rs`, criados pela TASK-01, e por isso roda depois dela, sozinha na onda 2.

## O que foi feito

`apps/cli/src/commands/documents.rs` ganhou o subcomando `detect`. `collect_style_files` percorre a raiz do projeto (pasta pai de `Harness::root`) com `std::fs`, em ordem de nome, pulando `node_modules`, `target`, `dist`, `build`, `vostok` e toda pasta oculta, e os arquivos `.env*`, `.pem`, `.key`, `.p12` e `.crt`; lê só `.css`, `.scss`, `.less` e `tailwind.config.{js,cjs,mjs,ts}` de até 1 MB. `color_declaration` reconhece por linha `--nome`, `$nome` e `@nome` nas folhas de estilo e chaves simples no Tailwind, com a primeira cor `#rgb` ou `#rrggbb` do valor, normalizada pelo `parse_hex` e `to_hex` da TASK-01. As candidatas são deduplicadas por nome e cor, ordenadas com as de nome `primary`, `brand` ou `accent` primeiro e depois por caminho e linha, e cortadas em 10, com a linha `... e mais N cores`.

Os testes `detect_*` (4) em `apps/cli/tests/documents.rs` cobrem a ordem por relevância, as pastas e segredos ignorados, o limite de 10 e o caso sem cores.

Ajuste: a evidência `detect-saida.txt` previa rodar neste repositório, mas os protótipos de `docs/estudos/` têm CSS dentro de HTML, que a spec não cobre, e a saída seria "Nenhuma cor encontrada". Ela passou a rodar num repositório de exemplo temporário, com um CSS, um tema do Tailwind e uma cor em `node_modules` que não aparece. Ponto de atenção: cores em CSS embutido em HTML, em JSON de design tokens ou em `rgb()` e `hsl()` não são detectadas.
