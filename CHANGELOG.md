# Changelog

Mudanças de cada versão publicada do vostok. A versão mais recente fica no topo.

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
