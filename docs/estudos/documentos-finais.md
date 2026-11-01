# Estudo dos documentos finais

Data do estudo: 2026-11-01. Versão: `vostok` 0.2.8. Subsídio da SPEC-025 (documentos finais com visual e conteúdo profissionais).

## Escopo e método

O estudo responde, em duas partes, como devem ficar os três documentos que a finalização de uma spec gera: o executivo, o técnico e o de evidências. A primeira parte trata do visual e termina em três direções, que viram protótipos para o usuário escolher. A segunda trata do conteúdo: o que cada documento conta e em que formato.

Para o visual, foram lidos o documento de identidade da organização (`Ishtar-Sink/.github`, `docs/identidade-ecossistema.md`, na versão de 2026-10), o Atria como a aplicação mais recente dessa identidade (`packages/theme`, `prototype/atria-prototype.html` e `docs/decisoes-do-prototipo.md`) e os documentos atuais, gerados a partir dos modelos em `packages/harness/templates/finalize/` com as cores de `packages/harness/config/documents.yml`. Os documentos da SPEC-023, publicados em `docs/specs/SPEC-023/`, foram abertos no Chrome sem interface, em 1280 px, para ver o que o leitor vê.

## Identidade do Ishtar Sink aplicada a documentos

A identidade do ecossistema é uma só para todos os produtos, e o Vostok faz parte dele. Ela já define quase tudo o que um documento precisa: a paleta, a tipografia, os raios e as sombras. O que muda num documento, em relação a uma aplicação, é o uso. Um documento é lido do começo ao fim, muitas vezes impresso ou repassado, e quase não tem interação. Por isso, das regras da organização, as que mais pesam aqui são a tipografia e o contraste.

O tema claro é o padrão do ecossistema desde a revisão feita no Atria, e o escuro é a alternativa. O fundo claro é lavanda (`#F3F0FA`) e os cartões são brancos, com uma sombra tingida de roxo que cria profundidade sem borda grossa. A cor de marca em texto é o roxo `#6D28D9`; o roxo vivo `#A855F7` e o ciano `#06B6D4` ficam para preenchimento e gradiente, porque reprovam contraste de texto no claro. O gradiente de marca vai do roxo ao ciano, e há um gradiente suave (`calm`, de `#F3EBFF` a `#E0F7FB`) que o Atria usa na faixa de boas-vindas do painel. Os estados (sucesso, aviso, erro e informação) têm sempre um fundo suave e o texto na cor cheia, para que o estado não dependa só do matiz.

A tipografia é um par. A Outfit, em peso 700 a 800 e com espaçamento negativo, serve aos títulos grandes e aos números de destaque; em peso 600, caixa alta e espaçamento positivo, serve aos rótulos pequenos acima dos títulos. A Inter serve ao texto corrido, com altura de linha de 1,6 e largura de até 68 caracteres. Para um documento, que é quase só texto corrido, a Inter é a fonte principal. A regra da organização é servir as fontes pelo próprio produto, nunca por um CDN; num HTML de arquivo único, isso significa embuti-las, como o protótipo do Atria faz.

Do Atria vêm também decisões de forma que cabem num documento: conteúdo com largura máxima e centralizado, cabeçalho que fica preso no topo ao rolar e compacta, raio de 12 px em botões e campos e pílula só em selos, mensagens de estado calmas, em itálico e sem fundo colorido, e cartões com a sombra da camada 1. O raio dos cartões é 18 px.

## Documentos atuais

Os documentos atuais não seguem a identidade em nenhum dos pontos principais. Abrem no tema escuro, que deixou de ser o padrão do ecossistema; a paleta vem de `documents.yml`, derivada do mapa do Vostok, e não dos tokens da organização; e as fontes não estão embutidas, então o navegador cai na fonte do sistema (no Linux, a DejaVu), o que tira toda a personalidade tipográfica. O cabeçalho é uma faixa escura com gradiente, nos dois temas.

O problema mais sério, porém, é a estrutura. O técnico da SPEC-023 tem doze abas (Visão geral, Arquitetura, O que foi feito, Requisitos, Arquivos, Diffs, Commits, Decisões, Aprendizados, Atenção e Referências, além da primeira), e a primeira mostra um único parágrafo numa página quase vazia. O leitor precisa clicar onze vezes para ler o documento inteiro, não consegue rolar de uma seção para outra e não tem como imprimir ou ler de ponta a ponta. Abas fragmentam um texto que foi escrito para ser lido em sequência.

O executivo tem cinco abas e mostra os indicadores como cartões em que o rótulo vira um título enorme ("Comportamento preservado", "Defeitos evitados") e o número que importa fica pequeno, dentro da frase. A tabela de entregas reduz cada entrega a uma linha de valor, sem contar o que mudou. O recorte aparece num bloco destacado, com o mesmo peso visual de uma informação importante. Somando a forma ao conteúdo raso, o documento não cumpre o papel de explicar a entrega para quem não é técnico.

O de evidências segue o mesmo modelo escuro, com uma peça por evidência. Ele é o único dos três que não é publicado em `docs/specs/`.

## Três direções visuais

As três direções usam a mesma identidade e os mesmos requisitos comuns (abaixo). O que muda entre elas é como o leitor percorre o documento: por abas, por leitura contínua com um índice ao lado, ou por blocos que se destacam um a um.

**Direção A, Painel com abas.** É a evolução do modelo atual, para quem prefere ver uma parte de cada vez. O cabeçalho é uma faixa clara com o gradiente suave do Atria, o título em Outfit pesada e um resumo de duas ou três linhas. Abaixo, uma barra de abas presa no topo, no estilo dos seletores do Atria (pílula `primarySoft` no item ativo), com no máximo quatro abas por documento, agrupando o que hoje está espalhado em doze. Cada aba é um cartão branco com conteúdo substancial, nunca um parágrafo solto. Uma opção "ver tudo" mostra todas as abas em sequência, para leitura e impressão. A vantagem é a sensação de aplicação e a navegação rápida entre partes; a desvantagem é que ainda esconde conteúdo e depende de clique.

**Direção B, Relatório editorial.** Um documento de leitura contínua, como um relatório bem diagramado. O cabeçalho é sóbrio: rótulo em caixa alta, título grande em Outfit, uma linha de metadados (spec, datas, tarefas) e um resumo em destaque. O texto corre numa coluna de até 68 caracteres, com títulos de seção numerados e espaço generoso entre eles. No desktop, um índice fica preso à esquerda e marca a seção em leitura; no celular, ele vira um menu no topo. Números de destaque entram no texto ou numa faixa discreta, não em cartões gigantes. É a direção mais profissional e a melhor para ler de ponta a ponta e imprimir; o risco é parecer austera se o conteúdo for curto.

**Direção C, Dossiê em blocos.** Um meio-termo moderno, próximo do painel do Atria, mas linear. Abaixo do cabeçalho, uma faixa "em resumo" reúne os fatos principais (o que foi entregue, o ganho principal, o estado) em poucos blocos com números em Outfit. Depois, cada seção é um cartão branco com rótulo, título e conteúdo, empilhados numa coluna larga, e algumas seções curtas lado a lado em duas colunas no desktop. Uma linha de atalhos em pílula, presa no topo, leva a cada seção. Tudo fica visível rolando, sem clique, e os cartões dão ritmo à leitura. O risco é o excesso de cartões transformar o documento num painel e quebrar a leitura em pedaços.

## Requisitos comuns

Os três modelos abrem no tema claro e oferecem um botão para o escuro, guardando a escolha só no navegador de quem lê; sem escolha, seguem a preferência do sistema. As cores são os tokens de 2026-10, declarados num bloco CSS com um comentário que aponta o documento da organização como origem, sem valores inventados. Texto de leitura passa em contraste AA (4,5:1) nos dois temas, usando só os pares já medidos pela organização, e `textFaint` fica restrito a decoração.

As fontes Outfit e Inter, de licença OFL, vão embutidas no próprio HTML em `@font-face` com `data:`, sem nenhuma requisição externa. As fontes completas do Atria somam 463 KB (Inter variável em woff2, 352 KB, e Outfit variável em ttf, 111 KB), o que daria cerca de 620 KB em base64 por documento. Com subconjunto dos caracteres do português e conversão da Outfit para woff2, a estimativa é ficar bem abaixo disso, na casa das dezenas de KB por fonte; a medição real ficou pendente porque a ferramenta de subconjunto (`fonttools`) não está instalada na máquina e o download dela não foi autorizado nesta etapa. Fica para a TASK-03 confirmar o tamanho e, se não houver como gerar o subconjunto, embutir as fontes inteiras.

Os documentos funcionam de 400 px a 1280 px ou mais, sem rolagem horizontal da página; tabelas e blocos de código largos rolam dentro do próprio contêiner. A largura máxima do conteúdo segue o Atria, e o texto corrido fica em até 68 caracteres. Os três documentos de uma mesma spec usam o mesmo modelo, para que pareçam uma família.

## Estudo de conteúdo

Escrito na TASK-02.

## Escolha do modelo

Registrada na TASK-04.
