### Ler

- `vostok/work/specs/refined/SPEC-027-estudo-do-painel-local-do-vostok-sem-servidor.md`
- `vostok/context/skill-catalog.md`
- `apps/cli/src/cli.rs`
- `vostok/rules/agent/` (todas as regras de aprovação, commit e limites)
- `apps/cli/src/commands/spec.rs` (aprovação: `approve validate` e `approve approve`)
- `apps/cli/src/commands/plan.rs` (aprovação do plano: `create-tasks`, `mark-planned`)
- `apps/cli/src/commands/task.rs` (fn run_approve, fn approval_problems, fn approve)
- `apps/cli/src/commands/commit.rs` (liberação de commits: `commit check`)
- `vostok/rules/project/architecture.md`
- `vostok/rules/project/stack.md`
- `/home/victor/.claude/projects/-home-victor-projetos-ai-hub/memory/vostok-arquitetura.md`
- `/home/victor/.claude/projects/-home-victor-projetos-ai-hub/memory/spec-trava-aval.md`

### Mudar

- `docs/estudos/painel-local/levantamento.md` (criar): inventário das ações do chat, desenho da aprovação pelo painel e mudanças para remover a premissa de servidor (PQ-05, PQ-06, PQ-11).

### Passos

1. `docs/estudos/painel-local/levantamento.md` (criar): título `# Levantamento do painel local`, a data, a versão (`vostok --version`) e a seção `## Escopo e método`, com as fontes de "Ler". Confira: `head -12 docs/estudos/painel-local/levantamento.md`.
2. Mesmo arquivo, seção `## Inventário das ações` (PQ-05): tabela `| Ação | Skill ou comando | Forma no painel | Motivo |`, uma linha por skill do catálogo `vostok/context/skill-catalog.md` (padrão e customizadas) e uma por grupo de comando de `apps/cli/src/cli.rs` que o usuário dispara sem skill. "Forma no painel" é uma de três: `CLI` (o painel chama a mesma lógica do `vostok`, sem agente: é determinística), `pedido ao agente` (precisa de leitura, escrita ou decisão de um modelo: o painel grava um pedido na pasta) ou `fora do painel` (só faz sentido no chat ou no terminal). Critério: se a skill só chama comandos `vostok` com os valores que o usuário escolhe, é `CLI`; se ela escreve texto, código ou decide, é `pedido ao agente`. Feche com uma contagem por forma. Confira: o número de linhas da tabela é pelo menos o número de skills do catálogo.
3. Mesmo arquivo, seção `## Aprovação pelo painel` (PQ-06): liste as regras de `vostok/rules/agent/` que exigem a confirmação do usuário (cite os IDs) e compare três alternativas: (a) o painel chama a mesma aprovação do `vostok` e grava a origem `painel` no estado; (b) o painel grava um pedido de aprovação e o agente aprova; (c) o painel grava o aval num arquivo que a trava de aval (memória `spec-trava-aval`) confere antes de `finish`, commit e aprovações. Critérios de escolha, nesta ordem: a aprovação só nasce de um gesto do usuário; o agente não consegue forjá-la; não depende da ferramenta (Claude Code, Codex); custo de implementação. Recomende uma, com o que muda em `rules/agent/` e nos comandos `approve`. Confira: a seção cita ao menos um ID de `rules/agent/` e termina com `Recomendação:`.
4. Mesmo arquivo, seção `## Remoção do servidor` (PQ-11): tabela `| Item | Hoje | Mudança recomendada |`, uma linha para cada regra de `vostok/rules/project/architecture.md` e `stack.md` que cita web, server, MCP, banco ou Node (`architecture-01`, `-05`, `-06`; `stack-03`, `-04`, `-06`, `-07` e qualquer outra encontrada), uma para a memória `vostok-arquitetura`, uma para cada pasta `apps/server`, `apps/mcp`, `apps/web` e `packages/contracts`, e uma para `docs/mapa-vostok.html` se ele citar o servidor. Não altere nenhum desses itens: só recomende. Confira: `grep -c '^| ' docs/estudos/painel-local/levantamento.md`.
