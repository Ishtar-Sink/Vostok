# Changelog

Mudanças de cada versão publicada do vostok. A versão mais recente fica no topo.

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
