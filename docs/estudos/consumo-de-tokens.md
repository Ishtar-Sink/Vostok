# Estudo de consumo de tokens

Data do estudo: 2026-10-31. Versões: `vostok` 0.2.3; Claude Code (sessão do app desktop). Subsídio da SPEC-019.

## Escopo e método

O estudo responde três perguntas:

1. Onde o fluxo do harness gasta tokens quando o Coordenador dispara subagentes.
2. Quais técnicas a documentação oficial e a prática atual recomendam.
3. Quais alavancas o harness tem para gastar menos, e o custo de cada uma.

As medições usam o tamanho em caracteres da saída dos comandos e dos arquivos deste repositório, convertido em tokens com a regra aproximada de 4 caracteres por token. Os totais por subagente vêm das notificações de término das sessões de 2026-10-30. A notificação não diz se o total conta as leituras do cache, então vale só para comparar uma execução com outra.

Fontes, em ordem de peso:

- Documentação oficial do Claude Code: as páginas `costs` e `sub-agents`.
- O texto de engenharia da Anthropic sobre o sistema de pesquisa multiagente, lido por resumos de terceiros.
- Guias de terceiros, só como indício.

Não houve medição do modo "não oficial" anterior, que o usuário relata ter gastado menos. A seção "Como medir" propõe essa comparação.

## Como o custo cresce

- Cada chamada ao modelo reenvia o contexto inteiro da conversa. Com o prompt caching, a parte repetida é lida do cache a uma fração do preço, mas continua contando no uso do plano.
  - O custo de um agente é aproximadamente o tamanho do contexto multiplicado pelo número de turnos (cada chamada de ferramenta é um turno), somado à saída e ao raciocínio.
- Cada subagente abre um contexto novo. Ele grava no cache tudo o que carrega no início e paga de novo a leitura a cada turno. Subagentes não reduzem o trabalho: só tiram o volume do contexto do chat principal.
  - A documentação oficial diz isso explicitamente: as requisições do subagente contam no mesmo limite de uso.
- Na escala, a Anthropic relata que um agente usa cerca de 4 vezes os tokens de um chat, e um sistema multiagente cerca de 15 vezes. Para *agent teams*, a documentação do Claude Code fala em cerca de 7 vezes uma sessão comum.
- O contrário também vale. Num chat principal longo, cada turno relê o histórico todo.
  - Exemplo: num chat com 150 mil tokens de contexto, uma tarefa com 13 chamadas de ferramenta relê cerca de 2 milhões de tokens do cache.
  - A mesma tarefa num subagente com 50 mil tokens de contexto relê cerca de 650 mil.
  - Ou seja, "tudo no Coordenador" só é mais barato enquanto o contexto do chat é pequeno.

## Onde o harness gasta hoje

### Carga inicial de cada subagente de tarefa

Hoje o pedido ao subagente da `execute-task` manda rodar os comandos abaixo e seguir o roteiro da skill:

| Item | Tamanho aproximado |
|---|---|
| `vostok context load --persona desenvolvedor` (contexto obrigatório, com 17 arquivos de regras, mais a persona) | 7,9 mil tokens |
| `vostok context load --skill execute-task` | 3,2 mil tokens |
| Leitura de `skills/default/execute-task/SKILL.md` para seguir o roteiro | 3,3 mil tokens |
| `vostok task load` (tarefa detalhada, ex.: SPEC-018 TASK-01) | 3,2 mil tokens |
| Corpo da definição `vostok-<persona>`, ambiente, *git status*, ferramentas | não medido |
| **Subtotal do harness, antes de ler o código** | **cerca de 17,6 mil tokens** |

Boa parte disso não serve ao subagente:
- O contexto obrigatório traz regras de todas as categorias.
- A skill inteira traz os passos do Coordenador (lista de tarefas executáveis, escolha de modelo, grupos de sequência, resposta ao usuário), e o subagente só precisa do roteiro dele: carregar a tarefa, conferir o baseline, testes e código.

### Leitura do código

A lista "Ler" das tarefas cita arquivos inteiros, e os subagentes leem os arquivos inteiros:

| Arquivo | Tamanho aproximado |
|---|---|
| `apps/cli/src/commands/task.rs` (3.687 linhas) | 34,7 mil tokens |
| `apps/cli/tests/task.rs` (1.898 linhas) | 17,2 mil tokens |

Uma tarefa da CLI que lê os dois começa a implementar com mais de 70 mil tokens de contexto, e cada turno seguinte relê isso. Esse é o maior item do custo, maior que o próprio harness.

### Totais observados em 2026-10-30

| Subagente | Chamadas de ferramenta | Total informado |
|---|---|---|
| SPEC-017 TASK-02 (skill e persona) | 4 | 54,7 mil |
| SPEC-016 TASK-02 (skills) | 8 | 65,0 mil |
| SPEC-016 TASK-01 (CLI) | 15 | 74,0 mil |
| SPEC-017 TASK-01 (CLI) | 8 | 72,1 mil |
| SPEC-018 TASK-01 (CLI) | 13 | 92,2 mil |
| Finalização de spec (cada uma de 3) | 8 a 11 | 67,5 a 71,5 mil |

Mesmo uma tarefa só de texto, com 4 chamadas, chegou a 55 mil tokens. O piso de cada subagente é alto, independente do tamanho da tarefa.

### Custos do Coordenador

- Disparar 3 ou 4 subagentes em paralelo multiplica o piso de carga inicial.
- Ao consolidar, o Coordenador relê diffs completos para conferir o trabalho. É trabalho repetido no contexto mais caro, porque o chat principal é o mais longo.
- Ajustes que o subagente devolve como pergunta, como as fixtures golden da SPEC-016, viram mais turnos no chat principal.

## Técnicas recomendadas

Da documentação oficial do Claude Code:

- **Modelo por tarefa:** Sonnet para a maior parte do trabalho, Haiku (`model: haiku`) para subagentes simples e Opus só para raciocínio difícil. `CLAUDE_CODE_SUBAGENT_MODEL` define um padrão para todos os subagentes.
- **Limitar o subagente:**
  - `maxTurns` limita os turnos;
  - `tools` restringe as ferramentas;
  - `effort` (`low` a `max`) reduz o raciocínio nas tarefas simples, e o raciocínio é cobrado como saída;
  - `omitClaudeMd` dispensa o CLAUDE.md;
  - `skills` injeta no início só as skills listadas.
- **Prompt de disparo focado:** tudo o que vai no prompt entra no contexto do subagente desde o início. Times pequenos, porque o consumo é proporcional ao número de agentes ativos.
- **Delegar só o volumoso:** saída de testes, logs e buscas ficam no subagente, e só o resumo volta. Relatórios longos de muitos subagentes em paralelo enchem o chat principal do mesmo jeito.
- **Pré-processar antes de o modelo ler:** hooks ou comandos que devolvem só o necessário. A documentação dá o exemplo de filtrar a saída de testes para só as falhas, o que reduz de dezenas de milhares de tokens para centenas.
- **Contexto base enxuto:** as instruções de fluxo ficam em skills carregadas sob demanda, e o CLAUDE.md fica curto.
- **Pedidos específicos:** citar a função e o arquivo em vez de pedir para explorar.
- **Medir:** `/usage` mostra a parte do uso atribuída a skills e subagentes, e `/context` mostra o que ocupa o contexto.

Da prática de engenharia de contexto, segundo a Anthropic e resumos de terceiros:

- A meta é o menor conjunto de tokens de alto sinal que leva ao resultado.
- O subagente devolve um resumo condensado, de cerca de 1 a 2 mil tokens.
- Contexto "na hora certa": o agente recebe ponteiros (caminho, função, faixa de linhas) e lê só o trecho, em vez de receber o arquivo inteiro.

Cuidado: o fork, o subagente que herda a conversa inteira, leva o contexto completo do chat principal. Nunca deve ser usado para tarefas de onda.

## Alavancas para o harness

| # | Alavanca | Ganho esperado | Custo ou risco |
|---|---|---|---|
| A1 | **Execução no Coordenador por padrão:** onda de uma tarefa, ou tarefas `mecanica`, rodam na própria sessão (o FA-01 já existe). Subagentes só para ondas com duas ou mais tarefas independentes. | Elimina o piso de cerca de 55 mil tokens por subagente nas ondas pequenas. | Num chat principal já longo, cada turno relê o histórico. Pode precisar de um limiar de contexto. |
| A2 | **Paralelismo configurável e menor:** uma chave em `config/`, com padrão de 2 ou 3 no lugar dos 6 de hoje. | Limita o pico de consumo. | A onda leva mais tempo. |
| A3 | **Briefing único do subagente:** um comando (ex.: `vostok task brief`) que imprime só a persona, o roteiro do subagente e a seção "Execução" da tarefa, sem a skill inteira nem o contexto obrigatório todo. | De cerca de 17,6 mil para 5 mil tokens de carga inicial. | É preciso escolher quais regras do contexto obrigatório o subagente precisa. |
| A4 | **"Ler" com trechos:** a `approve-plan` cita em "Ler" a função ou a faixa de linhas, e o roteiro manda ler por busca e trecho, nunca o arquivo grande inteiro. | O maior item: de cerca de 52 mil para poucos mil tokens nas tarefas da CLI. | O subagente pode deixar de ver o contexto ao redor. Os passos precisam citar bem os pontos. |
| A5 | **Definições de agente com `maxTurns` e `effort`** pelo perfil (`mecanica`: `effort` baixo e Haiku; `padrao`: médio). | Corta exploração e raciocínio sem fim. | Uma tarefa cortada no limite volta parcial. |
| A6 | **Retorno curto e consolidação barata:** o subagente devolve um bloco fixo de até 5 linhas, e o Coordenador confere com `git diff --stat` e com os comandos da `finish-task`, sem reler os diffs inteiros. | Menos turnos e menos leitura no contexto mais caro. | A revisão de qualidade passa a depender da finalização. |
| A7 | **Saídas filtradas nos comandos do harness:** `vostok task tests` e as verificações devolvem resumo e falhas, e a saída completa fica em arquivo, como já acontece em `validation/tests.out`. | Turnos mais leves no subagente e no QA. | Pouco. |
| A8 | **Finalização de spec no Coordenador** quando ele já tem as fontes no contexto, ou num subagente com modelo menor. | Evita cerca de 70 mil tokens por spec. | O chat principal cresce. |

## Como medir

1. Escolher uma tarefa de tamanho médio da CLI e executá-la em três modos, em sessões novas (`/clear` antes de cada uma):
   - o modo "não oficial" antigo;
   - o fluxo atual com subagente;
   - o fluxo com as alavancas escolhidas.
2. Em cada modo, anotar o `/usage`: a sessão, a atribuição a subagentes e o cache. Anotar também o total da notificação do subagente.
3. A spec deve definir a meta, por exemplo "uma onda de uma tarefa gasta no máximo X% do fluxo atual", e o critério de aceite usa essa medição.

## Medição antes e depois

Medição de 2026-10-31 (SPEC-019 TASK-04), com a mesma tarefa média da CLI (linha nova no `vostok task status` e um teste) em sessões novas do Claude Code, Coordenador no Opus 5.5 e o pedido `executa a TASK-01 da SPEC-007 com subagente` nos dois modos:

| Modo | Subagente (Sonnet): leitura / escrita no cache | Uso de limites do subagente | Custo da sessão |
|---|---|---|---|
| 0.2.3, contexto completo | 345,3k / 63,3k | 33% | US$ 0,77 |
| Fluxo novo, com `vostok task brief` e `vostok task check` | 226,1k / 30,7k | 22% | US$ 0,66 |

- O custo total caiu 14%. No subagente, a escrita no cache caiu 51% e a leitura 35%, com 5 requisições em vez de 7; o Coordenador ficou praticamente igual.
- A execução da tarefa no Coordenador, sem subagente, não foi medida, porque o pedido citou subagente nos dois modos.
- A resolução do uso do plano (`session-0`, 33% nos dois) não mostrou diferença numa tarefa desse tamanho.

## Lacunas

- O consumo do modo "não oficial" anterior não foi medido.
- A execução de uma tarefa sozinha no Coordenador, sem subagente, ainda não foi medida.
- O detalhamento de tokens do `/usage` lista só o modelo principal; o consumo dos subagentes aparece apenas na divisão por modelo e em "O que está consumindo seus limites de uso".
- Não se sabe se o total da notificação do subagente inclui as leituras do cache.
- O custo fixo das definições de ferramentas e do prompt de sistema de um subagente não foi medido.

## Fontes

- [Manage costs effectively (Claude Code)](https://code.claude.com/docs/en/costs)
- [Subagents (Claude Code)](https://code.claude.com/docs/en/sub-agents)
- [How we built our multi-agent research system (Anthropic)](https://www.anthropic.com/engineering/multi-agent-research-system), lido por [ByteByteGo](https://blog.bytebytego.com/p/how-anthropic-built-a-multi-agent) e [The Decoder](https://the-decoder.com/anthropic-shares-blueprint-for-claude-research-agent-using-multiple-ai-agents-in-parallel/)
- Engenharia de contexto, segundo resumos de terceiros: [agentic-ai](https://agentic-ai.readthedocs.io/en/latest/ContextEngineering/anthropic/), [Inkeep](https://inkeep.com/blog/fighting-context-rot)
- Guias de terceiros, só como indício: [Composio](https://composio.dev/blog/ways-to-cut-token-consumption-in-claude-code), [KDnuggets](https://kdnuggets.com/7-practical-ways-to-reduce-claude-code-token-usage)
