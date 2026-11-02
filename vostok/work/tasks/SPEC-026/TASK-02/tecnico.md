<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Subcomando `detect` no módulo da TASK-01, com uma varredura própria por `std::fs` e reconhecimento por linha dos quatro formatos (CSS, SCSS, Less e Tailwind), sem crate de glob nem parser de CSS ([project/architecture-03]).

**Decisões.**
- Varredura por linha com padrões simples; descartado um parser de CSS ou de JavaScript, caro para o ganho, já que a escolha final é do usuário (RN-06).
- Relevância pelo nome (`primary`, `brand`, `accent`); descartado ranquear pela frequência, que favorece cinzas e cores de borda.
- Pastas e segredos ignorados por lista fixa (RN-05), seguindo a regra de segurança do projeto de não ler segredos.

**Riscos.** Cores de terceiros em CSS versionado podem aparecer; a ordenação e o arquivo e linha deixam isso visível.

**Referências.** RF-02, RF-03, RN-05, CT-01, CT-02, CB-07; decisão 2 da spec.

**Arquivos em comum.** Altera `documents.rs` e `tests/documents.rs`, criados pela TASK-01, e por isso roda depois dela, sozinha na onda 2.
