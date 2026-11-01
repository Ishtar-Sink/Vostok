<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

Seis HTML autossuficientes em docs/estudos/documentos-finais/modelo-{a,b,c}/, um par por direção da TASK-01, com o conteúdo da SPEC-023 reescrito no modelo da TASK-02. As fontes vão em subconjunto como data: (sem CDN, regra da organização); os tokens são os de 2026-10 num bloco CSS que cita a origem (inferência C da spec). As evidências são prints do Chrome headless do cache do Playwright, conferidos pelo usuário na finalização: um print por página e um do executivo em 400 px por modelo. Depende da TASK-02.

## O que foi feito

As seis páginas e o `README.md` foram criados em `docs/estudos/documentos-finais/`. O conteúdo foi escrito uma vez, como seções com título, grupo e HTML, e um gerador em Node fora do repositório monta os três modelos a partir dele, com os tokens de 2026-10 copiados de `atria/packages/theme/src/tokens.css` (com comentário citando `identidade-ecossistema.md`), `[hidden]` forçado para as abas do modelo A e um token próprio para o fundo de código, porque `surface-muted` sumia sobre o cartão no tema escuro. O tema segue a escolha guardada no navegador ou a preferência do sistema, com o claro como padrão.

Desvio do plano: as fontes foram embutidas inteiras (Inter woff2 e Outfit ttf, cerca de 620 KB por página), porque não há `fonttools` na máquina e o download pelo `uvx` foi negado; o subconjunto fica para a TASK-05. A variante escolhida pelo usuário, modelo C com o índice lateral do B (`modelo-c-indice`), foi gerada só em `F:\downloads` e não está em Mudar; a TASK-04 registra a escolha e a TASK-05 a implementa. Os nove prints foram conferidos pelo usuário.
