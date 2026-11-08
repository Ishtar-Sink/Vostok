# Levantamento do painel local

- Data: 2026-11-02
- Versão: `vostok 0.2.10`
- Spec: SPEC-027, tarefa TASK-01 (PQ-05, PQ-06, PQ-11)

## Escopo e método

Este levantamento responde a três perguntas do estudo do painel local: quais ações do chat o painel consegue fazer e de que forma (PQ-05), como uma aprovação feita no painel passa a valer como aprovação do usuário (PQ-06) e o que muda nas regras, na memória e nas pastas para remover a premissa de servidor (PQ-11). Ele só recomenda: nenhuma regra, memória ou pasta foi alterada.

Fontes lidas:

- `vostok/work/specs/refined/SPEC-027-estudo-do-painel-local-do-vostok-sem-servidor.md`: perguntas, escopo e decisões da spec (decisões 1, 11 e 12).
- `vostok/context/skill-catalog.md`: as 22 skills, 21 padrão e 1 customizada.
- `apps/cli/src/cli.rs`: os 14 grupos de comando do `vostok`.
- `vostok/rules/agent/` (`boundaries`, `memory`, `anti-loop`, `rule-suggestions`, `responses`, `template-usage`) e as regras das skills de aprovação (`approve-spec`, `approve-plan`, `approve-task`, `approve-commit`, `finish-task`, `finalize-spec`).
- `apps/cli/src/commands/spec.rs` (`spec approve validate|approve`, fn `approve`), `plan.rs` (`plan approve create-tasks|validate|mark-planned`), `task.rs` (fn `run_approve`, fn `approval_problems`, fn `approve`) e `commit.rs` (`commit check|show|approve|revert|commit`).
- `vostok/rules/project/architecture.md` e `vostok/rules/project/stack.md`.
- Memórias do agente `vostok-arquitetura` e `spec-trava-aval`.
- Busca por citações do servidor em `docs/mapa-vostok.html`, `README.md` e nas regras versionadas (`git grep`).

Critério de classificação do inventário: se a skill só chama comandos `vostok` com os valores que o usuário escolhe, a forma é `CLI` (o painel chama a mesma lógica, sem agente); se ela escreve texto, código ou decide, a forma é `pedido ao agente` (o painel grava um pedido na pasta); se só faz sentido no chat ou no terminal, é `fora do painel`.

## Inventário das ações

Uma linha por skill do catálogo (as 22, na ordem do catálogo) e, em seguida, uma por grupo de comando que o usuário dispara sem skill. Comandos usados só por dentro de uma skill (por exemplo `finalize build`, `skill finalize`, `bug link`) entram na linha da skill.

| Ação | Skill ou comando | Forma no painel | Motivo |
|---|---|---|---|
| Refinar spec em rascunho | `refine-spec` | pedido ao agente | Investiga o projeto e a memória, faz rodadas de perguntas e escreve a spec refinada. |
| Criar skill | `create-skill` | pedido ao agente | Escreve o `SKILL.md` e propõe subcomandos; `skill create` e `skill finalize` são só o envelope. |
| Ajustar spec refinada | `adjust-spec` | pedido ao agente | Interpreta o ajuste em prosa e reescreve a spec; a edição feita à mão no editor do painel, seguida de `spec adjust record-manual`, é `CLI`. |
| Aprovar spec | `approve-spec` | CLI | Só chama `spec approve validate` e `spec approve approve` com o ID que o usuário escolhe. |
| Planejar tarefas | `plan-tasks` | pedido ao agente | Divide a spec em tarefas e decide dependências; `plan create` e `plan score` só fecham o plano. |
| Ajustar plano | `adjust-plan` | pedido ao agente | Aplica ajustes em prosa (dividir, juntar, reordenar) e reescreve o plano. |
| Aprovar plano | `approve-plan` | pedido ao agente | Depois de `plan approve create-tasks`, detalha cada tarefa (Ler, Mudar, Passos, Verificação, descrição): escrita por modelo. O gesto de aprovar nasce no painel (ver Aprovação pelo painel). |
| Liberar ou desfazer commits | `approve-commit` | CLI | `commit approve <código>` e `commit revert <código>` com o código mostrado ao usuário por `commit show`. |
| Aprovar tarefas | `approve-task` | CLI | Só chama `task approve validate` e `task approve approve` com os IDs que o usuário marca. |
| Ajustar tarefa | `adjust-task` | pedido ao agente | Reescreve passos, verificação ou descrição a partir do pedido. |
| Executar tarefa ou onda | `execute-task` | pedido ao agente | Implementa testes e código. |
| Finalizar tarefa | `finish-task` | pedido ao agente | `task tests`, `regression`, `evidence` e `finish` são determinísticos, mas conferir os critérios e corrigir falhas no código exige um modelo. |
| Finalizar spec | `finalize-spec` | pedido ao agente | Sintetiza os documentos executivo, técnico e de evidências a partir das fontes. |
| Inicializar o harness | `init-harness` | fora do painel | O painel abre numa pasta que já tem `vostok/`; o levantamento das regras é uma conversa com o usuário no chat. |
| Ajustar regras | `adjust-rules` | pedido ao agente | Redige a regra e decide categoria e mapa; `rules write`, `rules load` e `rules remove` são o envelope. |
| Relatar bug do harness | `report-harness-bug` | pedido ao agente | Monta o relato, troca dados do projeto por exemplos neutros e mascara segredos. |
| Validar correção de bug | `validate-harness-bug` | pedido ao agente | Reinstala a versão, segue os passos de retomada e julga se o defeito sumiu. |
| Sugerir melhoria do harness | `suggest-harness-improvement` | pedido ao agente | Escreve a sugestão e a sanitiza. |
| Sugerir recurso do harness | `suggest-harness-feature` | pedido ao agente | Escreve a sugestão e a sanitiza. |
| Criar spec em rascunho | `create-spec` | CLI | No chat, a skill extrai o texto da conversa; no painel, o usuário escreve título e texto num formulário, e basta `spec new` mais a gravação do rascunho. |
| Customizar documentos | `customize-documents` | CLI | `documents detect` lista as cores candidatas, o usuário escolhe e `documents brand` grava. |
| Triar issues do harness | `triage-harness-issue` | pedido ao agente | Lê a issue, decide o tipo e abre o rascunho seguindo para o refinamento. |
| Fazer commit liberado | `commit commit` (parte de `approve-commit`) | pedido ao agente | A mensagem em Conventional Commits e a escolha dos arquivos são escritas pelo agente. |
| Retomar tarefa bloqueada | `task resume` (parte de `finish-task`) | CLI | A decisão é do usuário (`--accept-regression`, nota); o painel grava a decisão pela mesma lógica. |
| Ver visão geral e status | `spec list`, `task status` | CLI | Leitura determinística das specs, tarefas e ondas. |
| Ver tarefa e conferir critérios | `task load`, `task check-done` | CLI | Leitura e conferência sem escrita. |
| Rodar a verificação sem registrar | `task check` | CLI | Executa o bloco `bash` da Verificação; o painel mostra a saída. |
| Converter tarefas para pasta | `task migrate` | CLI | Conversão mecânica de formato. |
| Verificar commits | `commit check`, `commit show` | CLI | Mostra os commits sem liberação e o código. |
| Consultar memória | `memory search` | CLI | Busca por tipo e termos; `memory add` e `memory update` ficam com o agente (memory-06, memory-09). |
| Validar regras e mapa | `rules validate`, `rules detect` | CLI | Validação e detecção sem escrita. |
| Atualizar o harness instalado | `update` | CLI | Copia o conteúdo embutido no binário para `vostok/`, sem decisão. |
| Instalar o harness | `init` | fora do painel | Interativo no terminal e anterior à existência do painel na pasta. |
| Carregar contexto | `context load` | fora do painel | Serve só ao agente, no começo do trabalho. |
| Comandos internos das skills | `task start`, `baseline`, `brief`, `describe`, `persist`, `plan validate`, `finalize *`, `bug check`, `suggest check` | fora do painel | Etapas que o agente executa dentro de uma skill; o painel só mostra o resultado. |

Contagem por forma (35 linhas):

- `CLI`: 14 (5 skills: `approve-spec`, `approve-commit`, `approve-task`, `create-spec`, `customize-documents`; 9 comandos).
- `pedido ao agente`: 17 (16 skills e o commit liberado).
- `fora do painel`: 4 (1 skill, `init-harness`, e 3 comandos).

Conclusão para PQ-05: das 22 skills, 5 viram ações diretas do painel, 16 viram pedidos ao agente e 1 fica fora. Todo o lado de leitura (visão geral, spec, plano, tarefa, evidências, memória, commits) é `CLI`. As escritas de texto continuam com o agente, o que confirma a necessidade do formato de pedido em arquivo (PQ-08).

## Aprovação pelo painel

Regras que exigem a confirmação do usuário hoje:

- `[agent/boundaries-01]`: commit só com autorização expressa, pela skill `approve-commit`.
- `[agent/boundaries-02]`: stage só a pedido do usuário.
- `[agent/boundaries-04]`: só a skill `approve-commit`, a pedido do usuário, libera ou desfaz commits.
- `[agent/memory-06]`: decisão só é registrada depois da confirmação do usuário.
- `[agent/rule-suggestions-04]`: nenhuma regra é gravada sem autorização.
- Regras de skill: `[approve-spec-01]`, `[approve-plan-01]`, `[approve-task-01]` (só a pedido do usuário), `[approve-task-03]` (`--all` só com pedido explícito), `[approve-commit-01]` (mensagem posterior à que mostrou os commits), `[finish-task-08]`, `[finalize-spec-01]`.

Hoje todas são texto: a CLI não distingue quem a chama. `spec approve approve`, `task approve approve` (fn `approve`) e `plan approve mark-planned` gravam `status`, `approved_at` e `approved_hash` sem nenhum campo de origem; `commit approve` exige o código de `commit show`, que o agente também lê. A memória `spec-trava-aval` registra esse buraco e prevê uma trava antes de `finish`, commit e aprovações.

Alternativas:

| Alternativa | Gesto do usuário | Agente consegue forjar? | Depende da ferramenta? | Custo |
|---|---|---|---|---|
| (a) O painel chama a mesma aprovação do `vostok` e grava a origem `painel` no estado | Sim: clique no painel | Pelo caminho da CLI, não, se a origem não for um argumento (o painel chama a função da biblioteca com `painel`; a CLI grava sempre `chat`). Pelo arquivo, sim: o agente pode editar o front-matter | Não | Baixo: as funções já existem; falta o campo de origem |
| (b) O painel grava um pedido de aprovação e o agente aprova | Sim, mas o agente fica entre o gesto e a aprovação | Sim: o agente pode escrever o pedido ou aprovar sem ele, exatamente como hoje | Sim: depende do loop do chat (PQ-08) | Médio: formato de pedido e processamento |
| (c) O painel grava o aval num arquivo que a trava de aval confere antes de `finish`, commit e aprovações | Sim: clique no painel | Mais difícil: o aval é ligado ao hash do conteúdo aprovado e a um registro só de acréscimo; ainda forjável por quem escreve na pasta | Não | Médio: arquivo de aval e conferência em cada comando travado |

Leitura dos critérios, na ordem:

1. Gesto do usuário: (a) e (c) nascem do clique; (b) passa pelo agente, então perde já no primeiro critério.
2. Forja: nenhuma alternativa local impede um agente com shell, rodando com o mesmo usuário do sistema, de escrever na pasta. O que muda é o custo e a visibilidade da forja. (a) sozinha deixa a origem num campo editável; (c) liga o aval ao hash do conteúdo, então um aval copiado não vale para outra versão, e a trava faz a CLI recusar `finish`, `commit commit` e `approve` sem aval válido.
3. Ferramenta: (a) e (c) são do `vostok`, valem no Claude Code e no Codex; a ideia de regras `ask` no `.claude/settings.json` (memória `spec-trava-aval`) continua útil, mas é por ferramenta e fica como camada extra.
4. Custo: (a) é a mais barata; (c) reaproveita o mesmo ponto de entrada e acrescenta só o arquivo de aval.

Recomendação: (a) com o aval de (c). O painel chama, no mesmo processo, as funções de aprovação que a CLI já usa (spec, tarefa, liberação de commit e retomada), grava `approved_by: painel` (a CLI grava `chat`) e acrescenta o aval a um registro só de acréscimo em `vostok/work/` com o ID, o hash aprovado, a data e a origem. As ações que são pedido ao agente (aprovar plano, finalizar tarefa, finalizar spec, fazer commit) passam a exigir esse aval: o agente processa o pedido e a CLI recusa sem aval correspondente. A (b) fica descartada porque coloca o agente entre o gesto e a aprovação, que é o problema que a trava quer fechar. O endpoint do painel escuta só em `127.0.0.1` e exige um token de sessão entregue ao navegador na abertura, para que um `curl` do agente não aprove sem passar pelo usuário.

O que muda, a aplicar por spec própria (a trava de aval, prevista em `spec-trava-aval`):

- `rules/agent/boundaries.md`: nova regra dizendo que uma aprovação, liberação ou retomada feita no painel vale como pedido do usuário; que o agente nunca chama o endpoint do painel nem grava campos de aval ou de origem; e que `boundaries-01` e `-04` passam a aceitar o aval do painel como autorização para o commit e a liberação.
- Regras de skill `[approve-spec-01]`, `[approve-plan-01]`, `[approve-task-01]`, `[finish-task-08]`, `[finalize-spec-01]`, `[approve-commit-01]`: "a pedido do usuário" passa a incluir "ou com o aval gravado pelo painel".
- Comandos `spec approve approve`, `task approve approve` e `plan approve mark-planned`: gravam `approved_by` (`chat` pela CLI) e o aval; a origem nunca é argumento de linha de comando.
- `task finish`, `commit commit`, `commit approve`, `finalize archive` e `plan approve mark-planned`: conferem o aval do ID e do hash antes de agir, quando a trava estiver ligada.
- `validate` de spec, plano e tarefa: acusam `approved_by: painel` sem aval correspondente no registro.

## Remoção do servidor

| Item | Hoje | Mudança recomendada |
|---|---|---|
| `[project/architecture-01]` | Monorepo com `apps/` (`cli`, `web`, `server`, `mcp`) e `packages/` (`harness`, `contracts`) | Reescrever: `apps/cli` e o frontend do painel (pasta definida pela arquitetura do estudo); `packages/harness`. Tirar `server`, `mcp` e `contracts`. |
| `[project/architecture-02]` | DDD leve por domínio, inclusive "evento" | Manter; trocar "evento" por "aval" ou retirar, já que a coleção de eventos era do servidor. |
| `[project/architecture-04]` | Frontend organizado por feature, com chamadas de API | Manter: vale para o painel; "chamadas de API" passa a ser a API local do `vostok`. |
| `[project/architecture-05]` | O servidor HTTP é o único que grava no banco; web e MCP só usam a API dele | Substituir: os arquivos de `vostok/` são a fonte de verdade; o painel grava só pela lógica da CLI (mesmas funções e validações), sem outro armazenamento. |
| `[project/architecture-06]` | Contratos entre web, server e MCP em `packages/contracts` | Remover; os tipos do painel ficam junto do frontend, e o formato dos arquivos continua definido pelos templates e pela CLI. |
| `[project/architecture-07]` | A CLI empacota `packages/harness` no binário | Ampliar: a CLI também empacota o bundle do painel e abre o painel por pasta, em `127.0.0.1`, porta livre. |
| `[project/stack-02]` | Web em React com Vite, sem Next | Manter ou trocar conforme PQ-03 (React ou Preact com compat); o estudo pode recomendar fora de Vite (decisão 4 da spec). |
| `[project/stack-03]` | Backend em Node.js | Remover: o servidor do painel é o próprio `vostok`, em Rust. |
| `[project/stack-04]` | Banco MongoDB | Remover: sem banco; os `.md` e `.yml` de `vostok/` são o armazenamento. |
| `[project/stack-06]` | Web, server e MCP em TypeScript | Reescrever: frontend do painel em TypeScript. |
| `[project/stack-07]` | Servidor MCP em Node.js, junto do backend | Remover. |
| Memória `vostok-arquitetura` | Painel web + servidor HTTP self-hosted + MCP; servidor único escritor do MongoDB; aprovação pela web e pela ferramenta via MCP; modelo pull com eventos e cursor | Substituir pela nova premissa: painel local aberto pelo `vostok`, arquivos como banco, pedidos ao agente em arquivo (PQ-08), aval do painel (PQ-06). Atualizar a linha do índice `MEMORY.md`. Não editar à mão: registrar como decisão nova com `memory-06` se virar decisão do harness. |
| Pasta `apps/server` | Vazia, não rastreada pelo git | Remover. |
| Pasta `apps/mcp` | Vazia, não rastreada pelo git | Remover. |
| Pasta `apps/web` | Vazia, não rastreada pelo git | Reaproveitar como fonte do frontend do painel, se a arquitetura recomendada mantiver o frontend fora de `apps/cli`; senão, remover. |
| Pasta `packages/contracts` | Vazia, não rastreada pelo git | Remover. |
| `docs/mapa-vostok.html` | Fase 2 "Serviços e painel" com servidor HTTP, MongoDB e MCP (linhas 366, 379, 389 a 403) | Reescrever a fase 2 como "Painel local" e tirar os nós `mcp`, `server` e `mongo` e as setas deles, no bloco DATA do arquivo e do artefato (memória `mapa-vostok-artefato`). |
| `README.md` | Fase 2 com servidor HTTP, MongoDB e MCP (linhas 14 e 26) e as pastas `mcp/` e `server/` na árvore (linhas 113 e 114) | Atualizar a fase 2 e a árvore de pastas para o painel local. |
| `[agent/memory-01]` | Cita `mongodb banco dados` só como exemplo de termos de busca | Sem mudança obrigatória; trocar o exemplo é opcional. |

Conclusão para PQ-11: a premissa de servidor está em 7 regras de projeto (`architecture-01`, `-05`, `-06`; `stack-03`, `-04`, `-06`, `-07`, com `architecture-01` e `stack-06` a reescrever e as outras a remover ou substituir), em 4 regras a revisar (`architecture-02`, `-04`, `-07`; `stack-02`), na memória `vostok-arquitetura`, em 4 pastas vazias e em 2 documentos (`docs/mapa-vostok.html`, `README.md`). Nenhum código depende dela, porque as pastas estão vazias; a remoção é só de texto e pastas, aplicada por outra spec (decisão 1 da spec).
