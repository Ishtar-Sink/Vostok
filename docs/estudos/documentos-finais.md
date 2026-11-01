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

### Referências

As referências públicas sobre leitura em tela concordam num ponto: o leitor não lê de ponta a ponta antes de decidir se o texto vale a pena. Nos testes do Nielsen Norman Group, 79% dos usuários varreram uma página nova e só 16% leram palavra por palavra; a varredura em forma de F é o padrão mais comum, e ela piora quando o texto não oferece títulos e palavras de apoio ([NN/g, padrões de varredura](https://www.nngroup.com/articles/text-scanning-patterns-eyetracking/)). A resposta clássica é a pirâmide invertida, que põe a conclusão primeiro e o detalhe depois, porque o leitor só continua rolando quando espera encontrar o que precisa mais abaixo ([NN/g, pirâmide invertida](https://www.nngroup.com/articles/inverted-pyramid/)). A comunicação militar americana chegou à mesma regra com o nome BLUF, *bottom line up front*: a mensagem abre com a decisão ou o resultado, e o resto sustenta ([BLUF](https://en.wikipedia.org/wiki/BLUF_(communication))).

Sobre listas, as fontes se dividem de forma útil. O NN/g recomenda listas como apoio à varredura, mas para itens curtos e paralelos; quando cada item fica longo, a lista deixa de ajudar e vira um parágrafo quebrado ([NN/g, listas](https://www.nngroup.com/articles/presenting-bulleted-lists/); [NN/g, conteúdo longo](https://www.nngroup.com/articles/formatting-long-form-content/)). Guias de estilo de produto fixam um teto prático de cerca de cinco itens e mandam usar tabela acima de dez ([Anaplan](https://designsystem.anaplan.com/toolkit/anaplan-writing-style/formatting/lists/)). Do outro lado, Edward Tufte, em *The Cognitive Style of PowerPoint*, mostra que a lista hierárquica fragmenta o raciocínio, esconde a relação de causa entre os itens e dá aparência de rigor sem o conteúdo ([resumo do ensaio](https://www.davidmays.org/BookNotes05/TufCogn.html)). A Amazon levou isso à prática ao trocar as apresentações por memorandos narrativos de seis páginas, "com frases de verdade, verbos e substantivos, não só tópicos" ([Anecdote, estrutura do memorando](https://www.anecdote.com/2018/05/amazons-six-page-narrative-structure/)). O GOV.UK, que escreve para todo tipo de leitor, apoia as duas coisas: linguagem simples e frases curtas, com estrutura pensada a partir de como as pessoas leem ([Writing for GOV.UK](https://www.gov.uk/guidance/content-design/writing-for-gov-uk)).

Para o documento técnico, três modelos servem de base. O registro de decisão de arquitetura (ADR) de Michael Nygard organiza cada decisão em contexto, decisão e consequências, o que obriga a dizer por que, e não só o quê ([modelo de Nygard](https://docs.rke2.io/adrs/record-architecture-decisions)). O Diátaxis separa a documentação em tutorial, guia prático, referência e explicação, e mostra que misturar os tipos é a causa mais comum de documento confuso ([Canonical sobre o Diátaxis](https://ubuntu.com/blog/diataxis-a-new-foundation-for-canonical-documentation)). E os guias de runbook de SRE pedem, para quem dá sustentação, o sintoma, como diagnosticar, como mitigar e como desfazer, com os comandos de leitura antes dos que alteram algo ([guia de runbooks](https://drdroid.io/guides/runbooks-guide-for-sre-on-call-teams)).

### Diagnóstico do conteúdo atual

O conteúdo da SPEC-023 (`vostok/work/.archive/SPEC-023/content.md`) tem cerca de 2.500 palavras, mas 86 linhas de lista contra pouco mais de uma dezena de parágrafos. Os três problemas são de forma, não de esforço.

O primeiro é que o modelo pede lista onde o leitor precisa de explicação. No técnico, "O que foi feito" é uma lista de dez itens com uma frase cada ("Camada `TaskDoc`: cerca de 80 pontos de leitura e gravação passaram por ela"), que diz o quê mas não por quê nem como as peças se ligam. "Arquitetura e componentes" lista funções sem contar o caminho de uma operação. É exatamente a fragmentação que Tufte descreve: cada item é verdadeiro, mas o leitor precisa montar sozinho o raciocínio.

O segundo é que o executivo responde às perguntas erradas. Ele tem "Indicadores de valor", "Recorte" e "Entregas" com campos fixos (impacto, o que mudou, valor, como perceber), mas não tem uma seção de objetivo, isto é, qual era o problema antes. Sem o problema, o valor não tem com que se comparar, e o leitor não técnico não entende por que a entrega importa. O "Panorama" tenta resolver tudo em um parágrafo denso e cheio de nomes de arquivo.

O terceiro é que faltam as partes de sustentação. O técnico não diz como operar o que foi entregue, como diagnosticar uma falha, onde estão os testes que protegem o comportamento nem quais limites ficaram conhecidos. "Pontos de atenção" cumpre parte disso, mas em lista e misturado com riscos de projeto. O de evidências repete "Como chegar" e "Como ler" em uma linha cada, sem dizer o que a evidência prova em relação a um requisito.

### Regra de formato

A regra que concilia as referências com a preferência do usuário ("não sou fã dos bullets, mas pode ser que em excesso não me incomode tanto") é: **prosa por padrão, lista e tabela só quando a forma do conteúdo pede**.

Prosa é o padrão para tudo o que tem causa, consequência ou contraste: objetivo, valor, decisões, como funciona, riscos. Cada seção abre com uma frase que já responde à pergunta da seção, no espírito do BLUF, e o resto do parágrafo sustenta. Parágrafos têm de duas a cinco frases e tratam de uma ideia só; um texto de várias ideias vira vários parágrafos, não uma lista.

Lista entra em três casos: passos em ordem (numerada), itens curtos e paralelos que o leitor vai consultar um a um (como os arquivos de um mapa de código) e nada além de sete itens; acima disso, ou quando os itens têm dois ou mais atributos, vira tabela. Nenhum item de lista passa de duas linhas; se precisa de mais, é parágrafo. Lista aninhada não entra.

Tabela entra para comparar (antes e depois, opções de uma decisão), para números e para o mapa entre requisito e evidência. Negrito marca só o número ou o termo que o leitor que varre a página deve achar, no máximo um ou dois por parágrafo.

Títulos são frases curtas e informativas da própria seção, sempre os mesmos entre specs, para o leitor aprender onde está cada coisa. A linguagem do executivo não leva caminho de arquivo, comando nem nome de função; o técnico leva, sempre em código.

### Executivo

O executivo é para quem decide ou acompanha sem ler código. Ele precisa ser lido em três a cinco minutos e responder, nesta ordem, o que se fez, por que e o que muda. O tamanho total esperado fica entre 400 e 800 palavras.

| Seção | Propósito | Formato | Tamanho |
|---|---|---|---|
| Em resumo | A conclusão do documento inteiro: o que foi entregue e o ganho principal | Prosa, um parágrafo | 2 a 4 frases |
| Objetivo | O problema ou a necessidade antes da entrega, e quem era afetado | Prosa | 1 ou 2 parágrafos |
| O que foi entregue | Cada entrega contada pelo que passou a ser possível, não pelo que foi programado | Prosa, um parágrafo por entrega, com o nome da entrega em negrito no início | 1 parágrafo de 2 a 4 frases por entrega |
| Valor e impacto | O que melhora para quem usa, com números quando houver | Prosa, e tabela antes e depois quando houver comparação medida | 1 ou 2 parágrafos, tabela de até 6 linhas |
| Ganhos secundários | Efeitos úteis que não eram o objetivo, como defeitos achados, base para próximas entregas | Prosa; lista curta só se forem itens independentes | 1 parágrafo ou até 4 itens |
| Riscos e limites | O que ficou de fora e o que ainda pode dar errado, em linguagem de negócio | Prosa | 1 parágrafo |
| Próximos passos | O que vem a seguir, se houver; omitida quando não houver | Lista numerada curta ou um parágrafo | até 4 itens |

A seção de "Indicadores" deixa de existir sozinha: números entram em "Valor e impacto", dentro do texto ou da tabela antes e depois, para o número nunca aparecer sem a frase que diz o que ele significa. "Recorte" vira parte de "Riscos e limites".

### Técnico

O técnico é para quem vai manter, revisar ou dar sustentação ao que foi entregue, inclusive sem ter acompanhado a spec; ele funciona como a transferência de conhecimento da entrega. Segue a separação do Diátaxis: as seções de explicação (o que, por que, como) vêm primeiro, e as de referência (operação, mapa do código) depois, sem misturar instrução com justificativa. O tamanho esperado fica entre 1.000 e 2.000 palavras, conforme o tamanho da spec.

| Seção | Propósito | Formato | Tamanho |
|---|---|---|---|
| Em resumo | O que foi construído e como as partes se conectam, para quem só vai ler isso | Prosa | 1 parágrafo de 3 a 5 frases |
| Contexto | Como era antes e qual limitação técnica motivou a mudança | Prosa | 1 ou 2 parágrafos |
| Como funciona | O desenho da solução seguindo o caminho de uma operação real, do comando ao arquivo gravado | Prosa, com bloco de código ou diagrama quando ajudar | 2 a 5 parágrafos |
| Decisões | Cada decisão no formato do ADR: o contexto, o que foi decidido, a alternativa descartada e a consequência | Um parágrafo por decisão, com a decisão em negrito no início; tabela só para comparar opções | 1 parágrafo de 3 a 5 frases por decisão |
| O que foi feito | As mudanças por tarefa ou por tema, ligando cada uma ao requisito que atende | Prosa, um parágrafo por tema, com os IDs (RF, RN) citados | 1 parágrafo por tema |
| Operação e diagnóstico | Como usar o que foi entregue, os sintomas de falha conhecidos, como diagnosticar e como desfazer | Passos numerados para operar; tabela sintoma, causa provável e o que fazer para diagnóstico | até 8 passos, tabela de até 8 linhas |
| Mapa do código | Onde fica cada parte e quais testes a protegem | Tabela caminho, papel e testes | uma linha por componente |
| Limites e dívidas | O que não foi feito de propósito, o que ficou frágil e o que pede atenção numa próxima mudança | Prosa; lista só se forem itens independentes | 1 a 3 parágrafos ou até 6 itens |
| Referências | A spec, os commits e os documentos relacionados | Lista de links | sem limite |

"Aprendizados" e "Pontos de atenção" deixam de ser seções próprias: o que era aprendizado de desenho vai para "Decisões", e o que era atenção vai para "Limites e dívidas" ou "Operação e diagnóstico", conforme seja dívida ou sintoma.

### Evidências

O documento de evidências responde a uma pergunta só: o que prova que cada requisito foi cumprido. Hoje ele é organizado por arquivo de evidência; passa a abrir com um mapa e só depois mostrar as peças.

| Seção | Propósito | Formato | Tamanho |
|---|---|---|---|
| Em resumo | Quantas evidências, de quantas tarefas, e se todas passaram | Prosa | 1 ou 2 frases |
| Mapa de cobertura | Cada requisito e critério de aceite com a evidência que o prova | Tabela ID, o que foi verificado e evidência | uma linha por ID |
| Uma seção por evidência | A evidência em si, com o que ela prova, como reproduzir e o que observar | Uma frase "Prova", o comando em bloco de código e uma frase "Como ler" | 3 partes curtas por evidência |

A frase "Prova" é a novidade: diz em linguagem direta qual requisito a evidência sustenta, para o leitor não precisar inferir pelo nome do arquivo.

### O que o `finalize validate` confere

O `finalize validate` já confere que todas as seções do modelo existem, que nenhuma está vazia, que não sobrou variável do modelo e que cada evidência tem o seu bloco com "Como chegar" e "Como ler". Com o novo conteúdo, ele pode conferir, sem julgar o texto, o que é mecânico:

1. as novas seções de cada documento presentes e não vazias, com "Próximos passos" aceitando "Não se aplica.";
2. no técnico, o mapa do código como tabela com pelo menos uma linha e "Operação e diagnóstico" com pelo menos um passo ou uma linha de tabela;
3. no de evidências, o mapa de cobertura citando todos os IDs de requisito da spec, e cada bloco de evidência com "Prova";
4. no executivo, nenhum caminho de arquivo nem comando em código, que é sinal de linguagem técnica no documento errado;
5. nenhum item de lista acima de um tamanho (por exemplo, 300 caracteres), sinal de parágrafo disfarçado de lista.

O resto fica como orientação na skill `finalize-spec`, porque depende de julgamento: abrir cada seção pela conclusão, preferir prosa, um parágrafo por ideia, números sempre com a frase que os explica, decisões com alternativa e consequência, e os tamanhos das tabelas acima, que são faixas de referência e não limites rígidos. A contagem de palavras por documento não deve virar erro, porque uma spec pequena tem documentos menores; no máximo, um aviso quando o executivo passa de 1.000 palavras.

## Escolha do modelo

Registrada na TASK-04.
