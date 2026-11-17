# Changelog

Mudanças de cada versão publicada do vostok. A versão mais recente fica no topo.

## [0.6.9] - 2026-11-17

### Recursos

- O comando novo `vostok plan extend <SPEC_ID>` cria o plano de extensão de uma spec `planned`: só com os IDs que nenhuma tarefa cobre e os mudados cobertos só por tarefas concluídas, com as tarefas novas numeradas depois da última e a primeira onda depois da maior existente. O plano passa pelo `score`, pelo `validate` e pela aprovação como o inicial, e a aprovação cria só as tarefas novas, sem alterar as existentes. (SPEC-058, issue #15)
- A reaprovação de uma spec ajustada grava em `extend_ids` os IDs mudados cobertos só por tarefas concluídas e indica `vostok plan extend` quando há IDs que pedem tarefa nova. (SPEC-058)

### Melhorias

- O `vostok plan create` numa spec `planned` indica o `vostok plan extend`. (SPEC-058)
- As skills `approve-spec`, `plan-tasks`, `adjust-plan` e `approve-plan` tratam o plano de extensão. (SPEC-058)

## [0.6.8] - 2026-11-17

### Correções

- O selo "N a conferir" do quadro e o `vostok task reviews` deixam de contar a evidência que ainda não foi gerada: ela aparece como "não gerada", numa coluna própria do `reviews` e numa seção própria da tela da tarefa, em cinza e sem os botões de conferência. (SPEC-062)
- O `vostok task start` deixa a tarefa em `in_progress`, sem depender do `vostok task begin`; a opção `--fila` mantém a tarefa na fila para a onda com subagentes, que segue com o `begin`. Uma sessão que pule o `begin` não deixa mais a tarefa parada na fila do painel. (SPEC-062)

### Melhorias

- A skill `execute-task` usa `vostok task start --fila` só nas tarefas entregues a subagentes. (SPEC-062)

## [0.6.7] - 2026-11-17

### Correções

- Uma evidência recusada e depois trocada ou removida num ajuste não trava mais a conclusão da tarefa: a recusa sai no ajuste e em toda geração de evidências, e o `vostok task finish` não a conta. Uma tarefa já travada destrava com `vostok task evidence <SPEC> <TASK>` seguido do `finish`, depois de instalar esta versão e rodar `vostok update`. (SPEC-061, issue #19)
- A mensagem do `finish` sobre uma recusa indica o `--antecipada` só com a tarefa em execução, e `vostok task evidence <SPEC> <TASK>` nos demais status. (SPEC-061, issue #19)
- A conferência de uma evidência que saiu da tarefa fica no `conferencias.md` com o ID `-` e o estado `removida`, e o ID atual aponta só para a evidência nova. O painel a mostra como "a conferir". (SPEC-061, issue #19)

### Recursos

- O comando novo `vostok task mark-criteria <SPEC_ID> <TASK_ID> <N...>` marca os critérios de conclusão pelo número sem contar como edição manual; a skill `finish-task` passa a usá-lo. (SPEC-061)

### Melhorias

- A tarefa iniciada fica na fila (`queued`) depois do baseline e só entra em execução com `vostok task begin`; a skill de execução roda o `begin` na sessão e em cada subagente. (SPEC-054)
- A geração antecipada de evidências deixa a tarefa em `awaiting_review` até a última conferência, pelo chat ou pelo painel, que a leva a `executed`; uma recusa a devolve a `in_progress`. (SPEC-054)
- O quadro do painel ganhou as raias "Na fila" e "Aguardando conferência", e cada tarefa mostra o selo "N a conferir" com link para a conferência. (SPEC-054)

## [0.6.6] - 2026-11-17

### Recursos

- Depois de conferir as evidências de várias tarefas no painel, basta dizer "conferi" no chat: a skill nova `review-evidence` monta o lote, pergunta uma única vez pelo que ficou sem decisão, finaliza em sequência as tarefas aceitas, corrige as recusadas e responde com uma tabela do lote. (SPEC-053)
- O comando novo `vostok task reviews [SPEC_ID] [TASK_ID...]` lista numa tabela por spec o que cada tarefa tem a conferir, o que foi aceito e o que foi recusado, com a origem e o motivo, inclusive nas tarefas em arquivo único. Sem spec, usa a spec em execução e, com mais de uma, sai com código 2 e lista as candidatas. (SPEC-053)

### Melhorias

- As skills de execução e de finalização indicam a `review-evidence` quando o usuário volta do painel com várias tarefas conferidas. (SPEC-053)

## [0.6.5] - 2026-11-17

### Melhorias

- A aprovação do plano e o ajuste de tarefa recusam a evidência cuja Falha repete um texto do Trecho, que sempre acusaria numa entrega certa, e avisam a Falha com uma condição sobre o valor, como "maior que" ou "sem". (SPEC-059)
- O painel mostra essas Falhas em cinza, como "não dá para buscar" com o motivo, em vez de vermelho, inclusive nas tarefas antigas; a CLI e o painel seguem os mesmos casos. (SPEC-059)
- O agente tem uma regra nova, `rules/agent/evidence.md`, sobre como escrever a Falha, carregada em todo projeto; a skill de aprovação do plano traz um exemplo certo e um errado. (SPEC-059)
- O técnico de cada tarefa segue uma forma única, escolhida pelo usuário: uma frase, o panorama em prosa e depois tabelas com o que muda e as decisões. A aprovação do plano e a finalização da tarefa escrevem nessa forma. (SPEC-059)

## [0.6.4] - 2026-11-17

### Melhorias

- Um ajuste numa tarefa em execução mantém o resultado dos testes e da regressão quando a Verificação, o Baseline e os arquivos da tarefa não mudaram; se o código mudar depois dos testes, a finalização pede os testes de novo. (SPEC-057, issue #17)
- O ajuste diz, por evidência, se a conferência cai ou fica: cai quando o comando ou o tipo mudou, fica se a saída não mudar. A conferência mantida depois de um Trecho ou uma Falha trocados ganha a marca "conferida sob o trecho anterior", também no painel. (SPEC-057, issue #16)
- A CLI procura os textos do Trecho e da Falha com a mesma regra do painel, garantida por uma tabela de casos comum. O `vostok task evidence --antecipada` avisa o trecho que não apareceu e a falha que apareceu, e o `vostok task adjust validate` e o `vostok plan approve validate` avisam a falha que já aparece no baseline, sem recusar. (SPEC-057, issue #16)
- As skills de ajuste e de aprovação do plano descrevem o que um ajuste mantém e a regra da busca no Trecho. (SPEC-057, issue #16)

## [0.6.3] - 2026-11-16

### Melhorias

- O agente segue uma regra de escrita para pessoas e um glossário dos termos do harness: specs, tarefas, evidências e respostas abrem pelo resultado, com frases curtas e sem siglas no texto corrido. A resposta de conclusão começa pelo efeito para quem pediu. (SPEC-055)
- A spec refinada traz a razão de cada decisão, com a origem numa coluna própria, e o estado atual começa pelo que o usuário vê hoje. Os critérios das tarefas dizem em palavras o que fica comprovado, e o resumo de cada tarefa abre pelo resultado. (SPEC-055)
- As evidências aceitam os campos opcionais Mostra, Cobre, Onde olhar e Reproduzir; as tarefas no formato atual continuam válidas, e o `vostok update` acrescenta os campos aos projetos instalados. (SPEC-055)
- A janela de conferência do painel abre pelo que a evidência mostra, onde olhar e o que seria falha, sem blocos "Não se aplica"; o log mostra o trecho esperado em verde e a falha em vermelho, e os tempos citados num vídeo levam ao momento com um clique. (SPEC-055)
- A finalização calcula as métricas da spec (ciclo com hora, tarefas, retrabalho, critérios, evidências e volume) e a situação em verde, âmbar ou vermelho. O executivo abre por essa situação, uma frase de resultado, a decisão pedida, quatro cartões e as barras do ciclo e das evidências. (SPEC-055)
- No documento de evidências, cada peça abre pelo título dado na tarefa, com o que mostra, onde olhar, o que seria falha e o comando real que a gerou. (SPEC-055)

## [0.6.2] - 2026-11-13

### Recursos

- O painel ganhou a página Configurações, aberta pelo botão com engrenagem logo abaixo da troca de tema. Ela traz de volta, de forma controlada, só o grupo das evidências: quais tipos pedem conferência, os limites de gravação, a cópia das evidências para o projeto, as extensões aceitas nos documentos e os trechos da regressão. (SPEC-052)
- Cada configuração tem um controle que evita erro de digitação: caixas de seleção, números com unidade, interruptor e etiquetas com sugestões; texto livre só na pasta e no nome do guia. As mudanças ficam pendentes até Salvar, podem ser descartadas, e sair da página com elas pede confirmação. (SPEC-052)
- A gravação passa pela validação da CLI: valor inválido volta com o erro ao lado do campo e nada é gravado, só as linhas das configurações alteradas mudam nos arquivos, com os comentários mantidos, e uma edição feita por fora depois da leitura não é sobrescrita. A gravação só é aceita da página aberta, com a chave da sessão. (SPEC-052)

## [0.6.1] - 2026-11-13

### Melhorias

- A aba Evidências do painel separa as evidências em A conferir, Recusadas e Conferidas, com um resumo das contagens que também filtra, e alterna entre lista e cards, lembrando a escolha. (SPEC-051)
- Clicar numa evidência abre uma janela de conferência com o que ela prova, como foi feita e como reproduzir ao lado do arquivo; cada texto do trecho e da falha mostra o resultado em verde ou vermelho e destaca a linha no log. (SPEC-051)
- A conferência, a recusa e a retirada podem ser feitas no próprio painel, sem o agente. A gravação só é aceita da página aberta, com uma chave da sessão, e o resto do painel continua só de leitura. (SPEC-051)
- As conferências ficam num arquivo próprio de cada tarefa, com a origem (chat ou painel), e o histórico da tarefa guarda só uma linha curta por decisão. As tarefas antigas mantêm o estado, sem migração. (SPEC-051)
- As skills de execução e de finalização respeitam o que já foi decidido no painel e perguntam só pelo que falta. As evidências aceitam o subitem opcional `Como:`, de uma frase. (SPEC-051)

### Correções

- A busca do trecho e da falha esperados não acha mais falsos positivos, como `0 passed` dentro de `10 passed`, e a contagem e o destaque usam a mesma regra. (SPEC-051)

## [0.6.0] - 2026-11-11

### Recursos

- O `vostok init` e o `vostok update` gravam no Claude Code (`.claude/settings.json`) e no Codex (`.codex/hooks.json`) um hook de início de sessão. Ele entrega ao agente as regras de limites e de segurança e a ordem de rodar `vostok context load`, no início, na retomada, na limpeza e depois de cada compactação da conversa. Os hooks e as chaves do usuário são mantidos, sem duplicar. No Cursor, no Gemini CLI e no Copilot, o harness avisa que a garantia automática ainda não existe e que o agente depende do `AGENTS.md`. (SPEC-050, issue #13)
- Um hook antes de cada comando de shell recusa o `vostok context load` com a saída redirecionada, num pipe ou capturada numa variável, e explica como carregar o contexto. (SPEC-050, issue #13)
- `vostok task evidence --undo <EV-NN|arquivo> --motivo "..."` retira a conferência ou a recusa de uma evidência dada por engano, sem mexer nas outras. O `--reviewed` aceita `--motivo` com as palavras do usuário. (SPEC-050, issue #14)
- A aba Evidências do painel mostra a data de cada conferência e, quando houver, o aval do usuário. (SPEC-050, issue #14)
- O quadro de tarefas ganhou o seletor de visualização, com a lista e a visualização em blocos: Agora, A seguir e Concluídas. (SPEC-050)
- As evidências do painel ganharam visualizador de imagem com zoom, player de vídeo e botão de copiar. As evidências de interação podem ser vídeos gravados com o Playwright. (SPEC-049)

### Melhorias

- As evidências são numeradas como `EV-NN`, e a conferência e a recusa aceitam o ID. Os documentos finais citam o ID de cada evidência. (SPEC-049)
- Na retomada ou num ajuste, a conferência de uma evidência com o mesmo conteúdo é mantida, e recusar uma evidência não bloqueia mais a tarefa: ela volta para execução. (SPEC-049, issue #12)
- A aba Evidências ganhou filtro por estado e destaque no log do trecho e da falha esperados; clicar no trecho o copia. (SPEC-049)
- O menu Tarefas tem um quadro único com filtro de etapas, ordem do fluxo invertível e faixa de contagem. A página da tarefa ganhou a navegação pelas tarefas da onda e "Próxima a conferir". (SPEC-049)
- As skills de execução e de finalização perguntam a conferência das evidências por tarefa, citando os IDs, e confirmam com o usuário um aval que não cita a tarefa antes de registrar. (SPEC-049, SPEC-050, issue #14)

## [0.5.4] - 2026-11-11

### Correções

- O diff das evidências compara com o commit gravado no baseline, e não mais com o HEAD. Assim, uma mudança commitada no meio da tarefa continua aparecendo. Se o commit do baseline não existir mais, o diff usa o HEAD e mostra um aviso. (SPEC-048, issue #10)
- A identidade escura dos documentos finais deriva a superfície e as sombras da cor da marca, mantendo o contraste. (SPEC-048, issue #11)
- O `vostok update` acrescenta às listas de configuração já existentes os itens obrigatórios novos, como `Título` e `Espera` nos campos de evidência. (SPEC-048)
- A recusa de validação cita o status `executed` entre os aceitos. (SPEC-048)

## [0.5.3] - 2026-11-10

### Melhorias

- Novo status `executed`, exibido como "Pronta para finalizar". A tarefa passa a ele quando as evidências antecipadas são geradas no fim da execução, e o quadro do painel ganha uma coluna para essas tarefas. Recusar uma evidência devolve a tarefa para "em execução". (SPEC-047)

## [0.5.2] - 2026-11-09

### Correções

- Conferir uma evidência com `--reviewed` só registra a conferência: as evidências não são mais geradas de novo e os prints não são mais apagados. (SPEC-046)

## [0.5.1] - 2026-11-09

### Melhorias

- O usuário confere e recusa pelo chat qualquer tipo de evidência, com motivo. A conferência vale até um ajuste ou retomada, e cada evidência ganha um título. (SPEC-043)
- Planos e tarefas travados têm uma saída pelo chat. O agente orienta o fechamento de esperas, as dependências de fechamento e as mudanças na spec. (SPEC-044)
- No painel:
  - specs e tarefas são lidas em abas;
  - o quadro agrupa, ordena e filtra, e mostra as esperas nos cards;
  - páginas longas ganham um botão de voltar ao topo;
  - cada evidência mostra se já foi conferida;
  - o painel abre na porta configurada em `panel_port`.

  (SPEC-045)
- Os prints do painel são feitos com Playwright. (SPEC-045)

## [0.5.0] - 2026-11-08

### Recursos

- Painel local somente leitura. O trabalho volta a ser feito só pelo chat, e o painel acompanha specs, tarefas, ondas e documentos. (SPEC-042)
- O histórico de status de cada spec é registrado, e o vostok avisa quando um plano está desatualizado. (SPEC-042)

### Melhorias

- O `vostok update` migra os projetos instalados na 0.4.0. (SPEC-042)
- O projeto passa a ter a licença AGPL-3.0-only.
