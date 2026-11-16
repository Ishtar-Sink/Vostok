# Comunicação do agente para pessoas

Estudo feito antes do refinamento da SPEC-055, em 2026-11-16. Ele junta duas frentes: o diagnóstico de textos reais escritos pelo agente neste repositório e uma pesquisa de guias e estudos sobre escrita legível. No fim estão as propostas de mudança.

## Conclusão

Os textos do Vostok já são bons onde a finalização da spec manda escrever para um leitor definido: o executivo final não tem siglas, abre pelo resultado e mostra o antes e o depois. O problema está no resto. As specs, as tarefas e as respostas no chat não têm regra de estilo, e por isso acumulam IDs no meio das frases (cerca de 40% das linhas de uma spec), jargão interno do harness em textos para quem decide e instruções de reprodução vagas.

A proposta central é estender a todo texto lido por pessoas o que a finalização já faz bem, numa regra de escrita curta, e mudar os poucos templates que empurram para o formato atual. Nada disso cria campo obrigatório: o teste de qualidade passa a ser três perguntas, "acho o que mudou?", "entendo sem abrir o código?" e "consigo reproduzir?".

Duas ampliações, no fim do estudo, tratam dos textos que mais pesam. O executivo final deve virar um documento de decisão: situação, resultado, tempo, qualidade e esforço no topo, em 10 segundos, com números que o harness já tem e não mostra. E as evidências devem dizer o resultado no título, onde olhar no log, no print ou no vídeo, e o que significa cada falha, sem IDs no texto.

## Como o estudo foi feito

O diagnóstico leu specs refinadas e arquivadas (SPEC-051 a SPEC-053), as tarefas em pasta da SPEC-051 e da SPEC-052, os documentos finais em `docs/specs/` e os templates e regras que moldam a escrita. As medidas foram tiradas por um script fora do repositório: tamanho médio de frase, parte das linhas com IDs e crases por 100 palavras. São aproximadas.

A pesquisa reuniu 23 fontes: a Lei 15.263/2025 (Política Nacional de Linguagem Simples), a ISO 24495-1, os guias de estilo do GOV.UK, do Google e da Microsoft, o Diátaxis, pesquisas do Nielsen Norman Group sobre leitura na web, a pirâmide de Minto, os registros de decisão de arquitetura de Michael Nygard, os design docs do Google, o guia de relato de bugs da Mozilla, o Keep a Changelog e estudos sobre a comunicação de modelos de linguagem. Quase todas foram conferidas na fonte; as que vieram só de fonte secundária estão marcadas na seção de referências.

## O que os textos mostram hoje

| Texto | Frase média | Linhas com ID | Crases a cada 100 palavras |
|---|---|---|---|
| Spec refinada (SPEC-051 a 053) | 13 a 17 palavras | 38% a 45% | 3 a 4 |
| Critérios de uma tarefa | 21 palavras | 41% | 4 |
| Resumo executivo de uma tarefa | 21 a 24 palavras | 0% a 20% | 0 |
| Detalhes técnicos de uma tarefa | 15 palavras | 24% a 26% | 7 a 17 |
| Executivo final da spec | 11 a 18 palavras | 0% | 0 |
| Mapa de cobertura das evidências | 12 palavras | 58% | 3 |

O tamanho das frases está dentro do que os guias recomendam (até cerca de 25 palavras). O que pesa é o resto: siglas e trechos de código no meio da prosa.

### Os oito problemas de maior impacto

**1. Não existe regra de estilo para specs e tarefas.** A regra de idioma do projeto só define o idioma. As regras de escrita da finalização (público definido, abrir pela conclusão, nenhum número sem explicação) não valem para a spec, a tarefa nem o chat. É a origem dos demais problemas.

**2. Blocos de IDs no fim das frases.** Critérios terminam em "(RF-13, CT-03, CB-05, CB-06)", e o técnico final fecha temas com "Atende RF-13, RF-14, RNF-03, RN-01, RN-04 e RN-05". Quem lê sem a spec aberta não sabe o que cada sigla é. A origem está no passo 4 da aprovação do plano, que pede critérios "ligados aos IDs cobertos", e no template do conteúdo final.

**3. "Como reproduzir" vago ou ausente.** A evidência final diz "`cargo test` dos testes de recusa em `apps/cli/tests/settings.rs`", sem o comando que roda. O critério de conclusão manda olhar um arquivo, mas não diz como gerá-lo de novo. A exceção boa é a seção "Operação e diagnóstico" do técnico final, com passos numerados e uma tabela de sintoma, causa e o que fazer.

**4. Jargão do harness em textos para quem decide.** O executivo de uma tarefa diz "Perfil de julgamento: risco alto", o card mostra "Estimativa: julgamento" e o técnico fala em "decisão do Coordenador" e "onda". São termos que só o harness entende. A origem é o próprio passo 4 da aprovação do plano, que pede o perfil de execução no resumo executivo, em conflito com a regra que proíbe termos técnicos ali.

**5. Decisões com motivo que aponta para a conversa.** O registro de decisões diz "resposta 1, rodada 1" ou "inferência A". Quem lê a spec depois não tem a conversa nem a letra da inferência.

**6. "Estado atual" como inventário de funções.** Um item da SPEC-051 junta cinco cláusulas e vários nomes de código para descrever uma busca, e não diz o que o usuário vê hoje. A regra que pede "informação concreta do projeto (caminhos, comandos, campos, IDs)" é lida como pedido de listar código.

**7. O mesmo critério em três ou quatro lugares.** O mesmo texto aparece no card, nos critérios de conclusão e, parafraseado, na prova da evidência. Cada cópia é uma chance de divergir e um parágrafo a mais para ler.

**8. Respostas no chat organizadas por arquivo.** O template de conclusão pede "caminho: o que mudou" e "comando: resultado". O leitor quer o efeito ("agora a página salva só o que mudou"), e o arquivo é detalhe. A regra que limita a prosa a três frases e manda o resto para listas favorece fragmentos sem ligação.

### O que funciona e deve ser mantido

- O executivo final: 0% de IDs, a tabela "Situação, Antes, Depois" e frases que dizem o ganho em termos do usuário.
- A seção "Operação e diagnóstico", com passos numerados e a tabela de sintomas: é o melhor "como reproduzir" do repositório.
- As decisões do técnico final, cada uma com contexto, alternativa descartada e consequência.
- O título curto e simples de cada evidência.
- O objetivo das specs com um caso concreto ("por exemplo, de 9 tarefas").

## O que a literatura recomenda

Os guias e estudos convergem em poucos princípios. Os que mais se aplicam ao Vostok:

**Resultado na primeira frase.** A lei de linguagem simples manda pôr primeiro a informação mais importante; o Nielsen Norman Group, a Microsoft e a pirâmide de Minto dizem o mesmo. Quem para de ler cedo já leva o essencial.

**Um leitor definido por texto.** O executivo, o técnico, a spec e o chat respondem a perguntas diferentes. O Diátaxis separa quatro tipos de texto (tutorial, guia de ação, referência e explicação) e mostra que misturá-los piora todos. A seção de reprodução é um guia de ação e não deve virar explicação.

**Encontrar, compreender, usar.** A Lei 15.263 define linguagem simples pelo que ela permite: encontrar, compreender e usar a informação. A ISO 24495-1 usa os mesmos critérios. Isso vira um teste de revisão sem campos obrigatórios.

**Reprodução com passos exatos.** A Mozilla considera os passos para reproduzir a parte mais importante de um relato: ações em ordem, comando pronto para copiar e o resultado esperado separado do obtido. O Google acrescenta a condição antes da instrução ("se o teste falhar, rode X").

**Siglas nomeadas.** A lei manda escrever o nome antes da sigla, e o GOV.UK manda explicar o termo técnico na primeira vez, mesmo para especialistas. Para o Vostok: "SPEC-048 (correções da 0.6.0)" na primeira menção, o nome nas seguintes.

**Formatação a serviço do conteúdo.** Um estudo de 2024 (Zhang e outros) mostra que modelos de linguagem foram treinados a preferir listas, negrito e emojis porque avaliadores premiam esse formato, não porque ele ajuda o leitor. O Nielsen Norman Group recomenda destacar no máximo 30% do texto. Nygard pede decisões em frases completas, não em marcadores. A regra prática: lista para sequência e itens paralelos, tabela para comparar, prosa para raciocínio.

**Concisão proporcional e dúvida localizada.** Estudos sobre o excesso de texto em modelos de linguagem (YapBench, 2026; Saito e outros, 2023) apontam repetir o pedido, ressalvas genéricas e explicações não pedidas como os vícios mais comuns. Um experimento com 404 pessoas (Kim e outros, 2024) mostrou que a dúvida dita em primeira pessoa, no ponto exato ("não verifiquei no macOS"), ajuda o leitor; a ressalva genérica, não.

**Decisões contrastivas.** Os registros de decisão de arquitetura e os design docs do Google trazem contexto, decisão, alternativa descartada e consequência. As explicações que as pessoas aceitam são contrastivas ("por que isto e não aquilo") e seletivas (poucas causas).

### Tensões

**Rastreabilidade ou leitura fluida.** Os IDs garantem a auditoria entre spec, tarefa e evidência, mas uma sequência de siglas quebra a leitura. A saída é manter os IDs nas colunas e nos metadados e, na prosa, preferir o nome.

**Completude ou concisão.** O relato de bug pede detalhe; os guias de estilo pedem o mínimo para decidir. A saída é ter camadas: o essencial no topo e o completo um nível abaixo, com o critério "o leitor consegue decidir ou reproduzir?".

**Estrutura ou prosa.** Listas ajudam a escanear, mas o excesso é vício de treino. A saída é estrutura para passos e comparações, prosa para raciocínio e decisões.

## Propostas

As propostas estão em ordem de impacto. As cinco primeiras são a base da SPEC-055; as demais podem entrar nela ou numa spec seguinte.

1. **Criar uma regra de escrita para pessoas.** Uma categoria nova de regras do harness (por exemplo, `rules/agent/writing.md`), carregada por toda skill que escreve texto lido por pessoas. Ela traz o que a finalização já pede, generalizado: um leitor por texto, resultado na primeira frase, frases de até cerca de 25 palavras, voz ativa, sigla nomeada na primeira menção, nenhum termo interno sem explicação, estrutura só para passos e comparações, e as três perguntas de revisão. A regra substitui o limite fixo de "até três frases" das respostas por "o mínimo para o leitor decidir".

2. **Tirar os IDs da prosa e deixá-los em colunas.** Nos critérios de conclusão, nas provas das evidências e no técnico final, o ID vai para uma coluna ou para o fim da linha num formato discreto, e o texto diz o que foi comprovado em palavras. O mapa de cobertura continua com IDs, mas cada linha ganha o nome do requisito. A rastreabilidade feita pelos comandos não muda.

3. **Tornar a reprodução executável.** A evidência final e o "Como comprovar" dos critérios passam a trazer o comando exato, pronto para copiar, e o resultado esperado, no modelo da seção "Operação e diagnóstico". O comando pode vir da própria verificação da tarefa, sem paráfrase.

4. **Tirar o jargão do harness dos textos para quem decide.** O resumo executivo da tarefa deixa de citar o perfil de execução; o card troca "Estimativa: julgamento" por uma estimativa legível ou sai do texto. Termos como onda, Coordenador, baseline e lista fechada ganham um glossário curto, e cada texto explica o termo na primeira vez ou usa a palavra comum.

5. **Reorganizar a resposta de conclusão pelo efeito.** O template de conclusão passa a abrir pelo que mudou para o usuário, depois o que foi verificado e o que não foi, e só então os arquivos, como detalhe. O "como conferir você mesmo" entra quando houver algo a conferir.

6. **Escrever o motivo das decisões sem apontar para a conversa.** Na spec, a coluna de motivo diz a razão ("o usuário preferiu finalizar em lote para não repetir o pedido 9 vezes"), e a origem (resposta ou inferência) fica numa coluna própria.

7. **Descrever o estado atual pelo que o usuário vê.** O template do estado atual pede primeiro o comportamento visível e só depois os componentes, citados por caminho.

8. **Dizer cada coisa uma vez.** O critério é escrito nos critérios de conclusão; o card e a prova da evidência apontam para ele em vez de repeti-lo.

## Ampliação 1: o documento executivo

O executivo final é o texto mais bem escrito do harness, mas não serve bem a um executivo. Ele conta a entrega como narrativa e mostra números de processo, quando quem decide quer saber em 10 segundos se deu certo, o que mudou para o negócio e se precisa decidir algo.

### O que o executivo mostra hoje

O topo tem o título, as datas, um parágrafo "Em resumo" e até três números. Abaixo vêm sete blocos de prosa e um bloco "Como foi executado", o único montado pela CLI com dados. Lendo os executivos da SPEC-050, 051 e 052:

- **O resumo do topo é narrativa, não conclusão.** O da SPEC-052 tem cerca de 80 palavras e não diz se houve problema nem se há decisão pendente.
- **Os números medem processo, não resultado.** "12 configurações liberadas", "25 evidências conferidas", "5 tarefas em três ondas". O único número de resultado dos três documentos (na SPEC-050, "2.106 caracteres contra um limite de 10.000") ficou fora do topo.
- **Os indicadores da CLI são sempre iguais.** Uma spec finalizada mostra sempre "9 / 9 tarefas concluídas" e "evidências geradas X / X", porque o código compara o total com ele mesmo. Nada pode sair amarelo ou vermelho.
- **A linha do tempo não informa.** As specs duram horas, mas a linha mostra só o dia: as 9 linhas da SPEC-051 dizem "2026-11-13".
- **Faltam peças de um relatório de status:** situação geral, tempo gasto, esforço, decisão pedida, riscos com nível e próximos passos com dono.
- **O conteúdo mais executivo está escondido.** A tabela "Antes e depois" fica no terceiro bloco.

### Dados que o harness já tem e não mostra

Todos conferidos nos arquivos da SPEC-052:

| Dado | Valor na SPEC-052 |
|---|---|
| Ciclo, com hora | criada 22:29, aprovada 22:37, implementada 00:26, arquivada 00:29; cerca de 1 h 49 min da aprovação à implementação |
| Tarefas e ondas | 5 tarefas em 3 ondas |
| Retrabalho | 0 tentativas extras, 0 bloqueios, 1 ajuste durante a execução |
| Qualidade | testes e regressão aprovados nas 5 tarefas; 5 de 5 critérios de aceite atendidos |
| Evidências | 25 (12 prints, 11 textos, 2 vídeos), 25 conferidas, 0 recusadas |
| Volume | 5 commits de entrega, 40 arquivos, 5.077 linhas acrescentadas e 13 removidas |

Hoje o `vostok finalize sources` não calcula nenhum desses números; o agente teria de contar à mão.

### O que a literatura diz sobre textos para executivos

- **Conclusão e ação primeiro.** O BLUF (normas de escrita militar dos EUA) e a pirâmide de Minto mandam abrir pela conclusão e pela ação pedida, em uma ou duas frases. O resto sustenta o topo e pode ser cortado em qualquer ponto (pirâmide invertida, Nielsen Norman Group).
- **Documento de decisão.** O leitor deve poder dizer sim, dizer não ou saber que pergunta fazer.
- **Situação com critério escrito.** Relatórios de status usam a cor por dimensão (escopo, prazo, qualidade, risco), com critérios fixos, e evitam o "verde por fora, vermelho por dentro".
- **Sempre comparar.** Stephen Few e Edward Tufte insistem em "comparado com o quê?": contra a meta, contra o antes ou contra o período anterior. O título de cada gráfico diz a conclusão, não o assunto (Cole Nussbaumer Knaflic).
- **Gráficos mínimos.** O bullet graph de Few mostra o valor, a referência e as faixas numa barra só; a sparkline de Tufte cabe numa linha de texto.
- **Velocidade e estabilidade.** As métricas DORA de entrega de software medem velocidade (tempo até entregar) e estabilidade (falhas e retrabalho). No Vostok, os equivalentes possíveis são o tempo da aprovação à conclusão e o retrabalho (ajustes, tentativas e recusas).

### Proposta para o executivo

O alvo é que o topo responda tudo em 10 segundos e que o texto inteiro caia de 400 a 800 para cerca de 300 a 450 palavras.

**No topo, sem rolar:**

1. Um **selo de situação calculado pela CLI**, com critério escrito: verde com todos os critérios atendidos e sem recusa, regressão nem bloqueio; âmbar se houve bloqueio, recusa ou critério parcial; vermelho com critério não atendido. Sempre com texto, não só com cor.
2. **Uma frase de até 30 palavras** com o que mudou para quem usa.
3. **A decisão pedida**, ou "Nenhuma decisão pendente".
4. **Quatro cartões fixos:** resultado (a métrica de valor, com antes e depois, escrita pelo agente), tempo (aprovação à conclusão), qualidade (critérios atendidos, evidências conferidas, recusas) e esforço (tarefas, ondas e retrabalho). Os três últimos são calculados pela CLI.
5. **A barra do ciclo**, com os marcos nas horas reais, no lugar da linha do tempo de datas iguais.

**No corpo, em ordem de importância:** o antes e depois, o que foi entregue em uma frase por entrega, os riscos com nível, os próximos passos com dono, os ganhos secundários e, no fim, a execução por onda.

**Gráficos simples, sem dependência externa** (HTML e SVG gerados pela CLI, nas cores do tema): barra das evidências por estado, bullet graph para a métrica de resultado, barra do ciclo, ondas em blocos e, com o histórico das specs arquivadas, uma sparkline do tempo de entrega.

Esboço do topo com os dados reais da SPEC-052:

```
SPEC-052 · Configurações das evidências no painel
[● CONCLUÍDA · VERDE]  5/5 critérios de aceite · 0 regressões · 0 bloqueios

Quem usa o painel passa a ajustar as 12 configurações das evidências em
poucos cliques, com valor inválido recusado antes de gravar.

Decisão pedida: escolher o próximo grupo de configurações a voltar ao painel.

RESULTADO              TEMPO                 QUALIDADE              ESFORÇO
12 configurações       1 h 49 min            25/25 evidências       5 tarefas, 3 ondas
editáveis no painel    da aprovação à        conferidas,            0 tentativas extras,
antes: só à mão        implementação         0 recusadas            1 ajuste na execução

Ciclo  criada 22:29 · aprovada 22:37 · planejada 22:49 ══ execução ══ implementada 00:26 · arquivada 00:29
```

Duas ressalvas. O harness não registra prazo planejado, então o selo não pode dizer "no prazo" sem uma estimativa no plano. E números como "de minutos para alguns cliques" não são medidos; pela regra de não inventar números, ficam fora.

## Ampliação 2: os textos das evidências

As evidências são o ponto em que você mais lê o texto do agente, e é onde ele está mais próximo da linguagem de teste. O formato tem boas bases: o título curto, a falha concreta nos prints e, no painel, a conferência de cada trecho em verde ou vermelho com a linha destacada no log.

### Problemas na definição da tarefa

- **O título descreve o teste, não o resultado.** "Testes da rota e do botão Configurações" não diz o que passou a funcionar.
- **A prova é longa e termina em IDs.** A de uma evidência da SPEC-052 tem cerca de 90 palavras e fecha com "(RF-13, CT-03, CT-05, CB-03 a CB-06, RN-01)".
- **O trecho é nome de função de teste:** "`test validacao_recusa_numeros_fora_da_faixa ... ok`".
- **O trecho é incompleto sem aviso.** Ele cita 5 testes, o log tem 10 e o resumo diz "17 filtered out", o que pode assustar quem confere.
- **A falha dos logs é genérica:** "`FAILED`, `panicked` ou `running 0 tests`" em todas, sem dizer o que quebrou.
- **Prints não dizem onde olhar, e vídeos não dizem quando.** O texto lista elementos de uma imagem de 2.000 px de altura e descreve o vídeo "em ordem", sem tempo.
- **Não há comando curto para reproduzir.** O único é a linha inteira do teste, com compilação, servidor e script juntos.

A origem é o passo 4 da aprovação do plano: a prova pede "os critérios ou IDs", não tem limite de tamanho e o exemplo oficial ensina o trecho e a falha no formato de log de teste. Nenhum campo pede a região da tela, o momento do vídeo ou por que importa.

### Problemas no painel

- **A ordem favorece quem executa, não quem confere.** A janela mostra o que prova, como foi feita e o comando de reprodução, às vezes enorme, antes do que olhar e do sinal de falha.
- **Blocos vazios em prints e vídeos.** "Trecho esperado" e "Sinais de falha" aparecem com "Não se aplica", duas vezes por evidência visual.
- **Uma cor só no log.** A linha destacada usa a mesma cor para o trecho esperado e para a falha; o verde e o vermelho só aparecem na lista ao lado.
- **IDs no topo.** "O que prova" mostra a prova crua, e as siglas são a primeira coisa lida.

### Problemas no documento final

- O cabeçalho de cada evidência é o nome do arquivo, não o título.
- "Como chegar" não é um comando executável: "`cargo test` dos testes de recusa em `apps/cli/tests/settings.rs`", quando o comando real está no próprio log.
- "Como ler" fica mais pobre que o trecho da tarefa ("os testes de fidelidade com `ok`").
- A falha e o aval do usuário não aparecem, e parte da cobertura se perde na reescrita.

A origem é o template do conteúdo final, que pede "o comando ou os passos" e "o que observar", e leva o agente a reescrever do zero em vez de herdar os campos da tarefa.

### Proposta para as evidências

Um formato em que cada campo tem um papel só, e os IDs saem do texto:

```
- `arquivo` (tipo): `comando`
  - Título: o resultado, em frase afirmativa (até 60 caracteres)
  - Mostra: uma frase em linguagem de usuário, sem IDs (até 200 caracteres)
  - Cobre: RF-01, CT-01                      (o painel mostra como etiquetas discretas)
  - Onde olhar: no log, a linha e a posição; no print, a região; no vídeo, o tempo e o que acontece
  - Trecho: só os textos que o painel procura no log
  - Falha: "se aparecer X, então Y quebrou"
  - Reproduzir: um comando curto, pronto para copiar, quando o da evidência for longo
```

No painel: abrir por "Mostra", "Onde olhar" e "Falha", recolher "Como foi feita", "Reproduzir" e "Cobre", esconder os blocos "Não se aplica", pintar no log o trecho de verde e a falha de vermelho, e tornar clicáveis os tempos do vídeo. No documento final: usar o título como cabeçalho, herdar da tarefa "Mostra", "Onde olhar" e "Falha", e usar o comando real do log.

Três exemplos reais da SPEC-052, reescritos:

| Campo | Antes | Depois |
|---|---|---|
| **Log (TASK-01)** | | |
| Título | A CLI recusa valores inválidos chave por chave | Valor inválido é recusado e nada é gravado |
| Mostra | Cerca de 90 palavras, terminando em "(RF-13, CT-03, CT-05, CB-03 a CB-06, RN-01)" | A CLI recusa número fora da faixa, chave desconhecida, caminho inválido e extensão repetida, mostra o erro em cada chave e não grava nada. |
| Onde olhar | (não havia) | No fim do log, `10 passed`; os 17 "filtered out" são testes de outros grupos, fora deste comando. |
| Falha | `FAILED`, `panicked` ou `running 0 tests` | Se aparecer `FAILED`, um valor inválido foi aceito; se aparecer `running 0 tests`, o filtro não achou os testes. |
| **Print (TASK-04)** | | |
| Título | Página Configurações lida do projeto, tema claro | Página Configurações mostra os valores do projeto |
| Onde olhar | Lista de elementos, sem posição | De cima para baixo: as quatro seções em ordem; na primeira, "120 s", "60 caracteres" e "10 MB"; o interruptor desligado; no rodapé, "Nenhuma mudança" com Salvar apagado. |
| Falha | "seção faltando ou fora da ordem, mensagem de erro…" | Se houver erro no topo, a leitura falhou; se os campos de pasta aparecerem com o interruptor desligado, a persistência está errada. |
| **Vídeo (TASK-05)** | | |
| Título | Mudar e salvar as configurações no painel real | Salvar grava a mudança e Descartar desfaz |
| Onde olhar | "em ordem: o painel inicial e o clique em Configurações…" | Por volta de 0:01, o clique em Configurações; 0:03, Diff desmarcada e a tag ndjson; 0:05, "Configurações salvas"; 0:07 a 0:09, Manual volta a marcada depois de Descartar. |
| Falha | "mensagem de erro do servidor (403, 409 ou 400)…" | Se aparecer 403, faltou o token; se aparecer 409, o arquivo mudou durante o teste; se Manual continuar desmarcada, o Descartar não funciona. |

Os tempos do vídeo foram estimados pelas pausas do roteiro de gravação e precisam ser confirmados no vídeo.

## O que fica para o refinamento

- Se a regra de escrita vale também para os textos feitos só para o agente (`execucao.md` e `verificacao.md`), que hoje são densos de propósito.
- Se a SPEC-055 inclui a revisão dos textos já escritos ou só muda o que vem a seguir.
- Como medir o resultado: por exemplo, a parte das linhas com ID e o tamanho das frases, medidos antes e depois numa spec de amostra.
- Se o glossário fica no harness distribuído ou só neste projeto.

## Referências

Conferidas na fonte, salvo indicação.

- Lei 15.263/2025, Política Nacional de Linguagem Simples. https://www.planalto.gov.br/ccivil_03/_ato2023-2026/2025/lei/l15263.htm
- Federal Plain Language Guidelines, Digital.gov (só a página-índice). https://digital.gov/guides/plain-language
- ISO 24495-1:2023, Plain language (conferida por fonte secundária). https://www.iso.org/standard/78907.html
- GOV.UK, Writing to GOV.UK standards, clear language. https://guidance.publishing.service.gov.uk/writing-to-gov-uk-standards/writing-guidelines/clear-language/
- Sara Vincent, Sentence length: why 25 words is our limit, 2014. https://insidegovuk.blog.gov.uk/2014/08/04/sentence-length-why-25-words-is-our-limit/
- Google developer documentation style guide, highlights. https://developers.google.com/style/highlights
- Microsoft Writing Style Guide, top 10 tips. https://learn.microsoft.com/en-us/style-guide/top-10-tips-style-voice
- Daniele Procida, Diátaxis. https://diataxis.fr/
- Amy Schade, Inverted Pyramid: Writing for Comprehension, NN/g, 2018. https://www.nngroup.com/articles/inverted-pyramid/
- Kara Pernice, F-Shaped Pattern of Reading on the Web, NN/g, 2017. https://www.nngroup.com/articles/f-shaped-pattern-reading-web-content/
- Jakob Nielsen, How Users Read on the Web, NN/g, 1997. https://www.nngroup.com/articles/how-users-read-on-the-web/
- Huei-Hsin Wang e Megan Chan, Formatting Long-Form Content, NN/g, 2023. https://www.nngroup.com/articles/formatting-long-form-content/
- BLUF e pirâmide de Minto (conferidos por fonte secundária). https://en.wikipedia.org/wiki/BLUF_(communication)
- Michael Nygard, Documenting Architecture Decisions, 2011. https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions
- Malte Ubl, Design Docs at Google, 2020. https://www.industrialempathy.com/posts/design-docs-at-google/
- Mozilla, Bug Writing Guidelines. https://bugzilla.mozilla.org/page.cgi?id=bug-writing.html
- Keep a Changelog 1.1.0. https://keepachangelog.com/en/1.1.0/
- Anthropic, Building Effective Agents, 2024. https://www.anthropic.com/engineering/building-effective-agents
- Saleema Amershi e outros, Guidelines for Human-AI Interaction, CHI 2019 (conferido por fonte secundária). https://www.microsoft.com/en-us/research/publication/guidelines-for-human-ai-interaction/
- Sunnie Kim e outros, "I'm Not Sure, But...", FAccT 2024 (conferido por resumo). https://arxiv.org/abs/2405.00623
- Xuanchang Zhang e outros, From Lists to Emojis: How Format Bias Affects Model Alignment, 2024. https://arxiv.org/abs/2409.11704
- Keita Saito e outros, Verbosity Bias in Preference Labeling by LLMs, 2023. https://arxiv.org/abs/2310.10076
- Vadim Borisov e outros, Do Chatbot LLMs Talk Too Much?, 2026. https://arxiv.org/abs/2601.00624
- Tim Miller, Explanation in Artificial Intelligence: Insights from the Social Sciences (só o resumo). https://arxiv.org/abs/1706.07269
- Barbara Minto, The Pyramid Principle, 1987 (conferido por fonte secundária). https://www.mckinsey.com/alumni/news-and-insights/global-news/alumni-news/barbara-minto-mece-i-invented-it-so-i-get-to-say-how-to-pronounce-it
- Colin Bryar e Bill Carr, Working Backwards, 2021 (conferido por fonte secundária). https://commoncog.com/working-backwards/
- Semáforo RAG em relatórios de status (guias de prática, conferidos por fonte secundária). https://www.clearpointstrategy.com/establish-rag-statuses-for-kpis/
- Stephen Few, Information Dashboard Design, 2006, e Bullet Graph Design Specification, 2013 (conferido por fonte secundária). https://www.perceptualedge.com/articles/misc/Bullet_Graph_Design_Spec.pdf
- Edward Tufte, The Visual Display of Quantitative Information, 1983, e Beautiful Evidence, 2006 (conhecimento geral, não conferido online).
- Cole Nussbaumer Knaflic, Storytelling with Data, 2015 (conferido por resumo).
- DORA, software delivery performance metrics (só a introdução). https://dora.dev/guides/dora-metrics-four-keys
