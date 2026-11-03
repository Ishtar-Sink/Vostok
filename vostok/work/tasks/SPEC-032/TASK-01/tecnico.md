<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**O que foi feito.** Estudo novo em `docs/estudos/painel-ponta-a-ponta.md`, no formato de `docs/estudos/painel-local.md` (SPEC-027): pergunta, resposta curta, método e uma seção por tema, com tabelas de medição (RNF-04).
- **Técnicas:** compara o pedido em loop, o disparo sob demanda (reaproveitando o `vostok pedidos vigiar`) e o chat no painel. Recomenda o pedido em loop, com o disparo como opção desligada por padrão.
- **Processos:** uma linha por skill do catálogo, sem as do harness e as de configuração inicial. Cada linha diz o que o painel já faz (`REQUEST_ACTIONS` ou `LOCKED_ACTIONS` em `apps/cli/src/painel/actions.rs`) e o que falta.
- **Status das entidades:** onde cada entidade grava o status hoje e em qual fluxo existente posicionar o que falta.
- **Perguntas do agente:** propõe o `vostok perguntas abrir`. A `process-requests` passa a abrir a pergunta em vez de recusar o pedido.
- **Arquivos de comunicação:** medição com 10 a 10.000 pedidos, gerados numa pasta descartável. Recomenda:
  - pastas por estado, com a transição por `rename`;
  - arquivamento de pedidos, perguntas e avais (um log por spec, só de acréscimo) junto com a spec no `vostok finalize archive`;
  - `vostok pedidos prune`, que só lista o que apagaria e apaga com `--confirmar`, também no painel como chamada direta com confirmação na tela, e que nunca apaga avais;
  - `vostok avais <ID>`, para o agente não abrir os logs.
- **Fontes de atividade:** tempo de leitura de cada fonte. Todas juntas ficam abaixo de 50 ms com o volume real, longe dos 500 ms da RNF-01.

**Decisões.**
- O chat dentro do painel fica registrado como alternativa, fora do escopo (decisão 4 da spec).
- A preocupação com o `avais.log` é o consumo de contexto do agente, não o tempo. Isso levou ao log por spec e à consulta pela CLI.
- Pedidos sem spec ficam na pasta de trabalho até o `prune` (concluídos com mais de 30 dias).

**Impacto na SPEC-032.** Nenhuma tarefa precisa de ajuste: a TASK-05 inclui as cinco fontes como planejado.

**Verificação.** `vostok task tests` confere as seções do estudo. A regressão mostra só o acréscimo do arquivo novo em `docs/estudos`. A evidência é o diff do estudo (`estudo.diff`).

**Referências da spec.** RF-01, RNF-04, CT-01; decisões 1, 2, 4 e 10; DEC-003.
