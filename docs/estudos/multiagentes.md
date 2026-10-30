# Estudo de multiagentes

Data do estudo: 2026-10-29. Versões: Claude Code não instalada; Codex não instalada; Cursor (`cursor-agent`) não instalada; Gemini CLI não instalada; GitHub Copilot CLI (`gh copilot`) não instalada. O estudo vem só da documentação oficial; nada foi executado na máquina.

## Escopo e método

O estudo cobre cinco ferramentas: Claude Code, Codex e Cursor em profundidade (as três que o `vostok` já suporta em `apps/cli/src/routing.rs`), e Gemini CLI e GitHub Copilot de forma leve. Para cada uma responde cinco perguntas: como disparar subagentes, se o agente lista modelos e escolhe um na chamada, quais apelidos estáveis de modelo existem, como definir personas ou agentes e quais são os limites de paralelismo.

Só vale documentação oficial (domínios `code.claude.com`, `developers.openai.com` e `learn.chatgpt.com`, `cursor.com`, `geminicli.com`, `docs.github.com`). Blogs, fóruns e vídeos não contam. O que a documentação consultada não descreve está marcado "não documentado", nunca deduzido. As páginas foram lidas por extração automática de texto; onde a leitura foi truncada, isso está dito. As páginas mudam rápido (vários nomes de modelo abaixo são de versões recentes), então os exemplos de nomes completos valem como amostra de formato, não como lista.

## Claude Code

### Subagentes

- A ferramenta de delegação se chama `Agent` (antes `Task`, renomeada na v2.1.63; referências `Task(...)` seguem como apelido).
- O Claude delega sozinho quando a tarefa casa com a `description` de um subagente, ou por pedido explícito (nomear no prompt, @-menção, `claude --agent <name>`, chave `agent` em settings).
- Contexto isolado: o subagente começa limpo, sem o histórico da conversa. Recebe o corpo do arquivo como prompt de sistema, detalhes básicos do ambiente e a mensagem de tarefa que o Claude escreve. Não recebe o prompt de sistema do Claude Code.
- Devolve só o resumo ou relatório final. Exceção: o fork (`/subtask`, ou o próprio Claude em modo fork, padrão desde a v2.1.302) herda a conversa inteira.
- Exemplo mínimo (copiado da documentação):

```markdown
---
name: code-reviewer
description: Reviews code for quality and best practices
tools: Read, Glob, Grep
model: sonnet
---

You are a code reviewer. When invoked, analyze the code and provide
specific, actionable feedback on quality, security, and best practices.
```

### Listagem e escolha de modelo

- O agente listar os modelos da sessão: não documentado (a página de modelos descreve só o seletor `/model`, para o usuário).
- Escolha na chamada: sim. A chamada da ferramenta `Agent` aceita o parâmetro `model`, só como apelido (`sonnet`, `opus`, `haiku`, `fable`); não aceita nome completo nem `inherit`.
- Escolha na definição: campo `model` do front-matter (apelido, nome completo ou `inherit`).
- Ordem de resolução: parâmetro `model` da chamada, `model` do front-matter, variável `CLAUDE_CODE_SUBAGENT_MODEL`, modelo da conversa principal. `CLAUDE_CODE_SUBAGENT_MODEL_FORCE=1` força um modelo por cima de tudo.
- Modelo indisponível: valor inválido não é conferido na partida e falha na primeira requisição ("There's an issue with the selected model"). Cadeias de reserva (`--fallback-model`, `fallbackModel` em settings, até três modelos) valem também para subagentes em caso de sobrecarga ou indisponibilidade. Quando `availableModels` bloqueia o modelo, o subagente roda num modelo de reserva.

### Apelidos estáveis de modelo

- Lista completa documentada: `default` (limpa a escolha, não é modelo), `best`, `fable`, `sonnet`, `opus`, `haiku`, `sonnet[1m]`, `opus[1m]`, `opusplan`, mais `inherit` nas definições de agente.
- Onde cada um vale: `/model`, `--model`, `ANTHROPIC_MODEL`, chave `model` de settings e `model` de skill/comando aceitam apelidos e nomes completos. No front-matter de subagente: `sonnet`, `opus`, `haiku`, `fable`, nome completo ou `inherit`. Na chamada do `Agent`: só `sonnet`, `opus`, `haiku`, `fable`.
- Os apelidos acompanham a versão mais recente do provedor (hoje `opus`, `sonnet` e `haiku` resolvem para as versões 5.5 na API da Anthropic; em outros provedores podem resolver para versões mais antigas).
- Formato dos nomes completos (exemplos da documentação): `claude-opus-5-5`, `claude-sonnet-4-5-20250929`, `claude-fable-5-1`, `claude-haiku-5-5`, `claude-opus-4-6[1m]`. Letras minúsculas, números e hífens, com sufixo opcional entre colchetes. No Bedrock o valor é um ARN e no Foundry um nome de implantação (fora do formato; o `vostok` não os trata).

### Personas e agentes

- Arquivo Markdown com front-matter YAML; o corpo é o prompt de sistema.
- Pastas: `.claude/agents/` (projeto), `~/.claude/agents/` (usuário), `agents/` de plugin, flag `--agents` e configuração gerenciada. Busca recursiva; a identidade vem só de `name`. Prioridade: gerenciada, `--agents`, projeto, usuário, plugin.
- Obrigatórios: `name` (até 256 caracteres, sem `:`) e `description`. Opcionais: `tools`, `disallowedTools`, `model`, `permissionMode`, `maxTurns`, `autoCompactWindow`, `skills`, `mcpServers`, `hooks`, `memory`, `background`, `omitClaudeMd`, `effort`, `isolation` (`worktree`), `color`, `initialPrompt`, `experimental`.
- Leitura de pastas de outras ferramentas: não documentado.

### Paralelismo e limites

- Vários subagentes ao mesmo tempo; limite padrão de 20 simultâneos (erro `Concurrent subagent limit reached`), ajustável por `CLAUDE_CODE_MAX_CONCURRENT_SUBAGENTS`. Sem limite no total por sessão.
- Subagente pode disparar subagente, até três camadas abaixo da conversa principal (`CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH=1` desliga). Um fork não dispara forks.
- Segundo plano: sim (`background: true` ou ao disparar), com conjunto menor de ferramentas.
- Isolamento de arquivos: `isolation: worktree` no front-matter, ou `isolation: "worktree"` na chamada; o worktree sai da branch padrão e é removido se nada mudar.
- Os subagentes contam para os mesmos limites de uso da conta. A leitura da página de subagentes foi completa até o fim.

### Fontes

- https://code.claude.com/docs/en/sub-agents (acessado em 2026-10-29)
- https://code.claude.com/docs/en/model-config (acessado em 2026-10-29; leitura truncada após 100000 caracteres)

## Codex

### Subagentes

- No Codex local, o subagente é disparado quando o usuário pede, ou quando um `AGENTS.md` ou skill aplicável pede. A documentação orienta dizer como dividir o trabalho, se espera todos e que resumo devolver. A ferramenta ou API interna do disparo: não documentado.
- Cada subagente roda na própria thread, herda sandbox e aprovações do pai (o arquivo de agente não pode trocar `sandbox_mode`, `approval_policy`, `model_provider` nem `mcp_servers`).
- O que recebe além da tarefa: não documentado. Devolve resumos; a thread principal consolida e só responde quando todos terminam.
- Exemplo mínimo (copiado da documentação):

```toml
name = "docs_researcher"
description = "Documentation specialist that uses the docs MCP server to verify APIs and framework behavior."
model = "gpt-6-luna"
model_reasoning_effort = "high"
developer_instructions = """
Use the docs MCP server to confirm APIs, options, and version-specific behavior.
Return concise answers with links or exact references when available.
Do not make code changes.
"""
```

### Listagem e escolha de modelo

- O agente listar modelos: não documentado.
- Escolha na chamada: sim, o prompt de disparo pode pedir modelo e esforço; valor explícito do disparo vence. Os valores aceitos são os nomes de modelo; formato da chamada estruturada: não documentado.
- Na definição: campos `model` e `model_reasoning_effort` do arquivo. Padrões globais: `agents.default_subagent_model` e `agents.default_subagent_reasoning_effort` em `[agents]`.
- Ordem: valor explícito do disparo, padrão de `[agents]`, valor do pai. Comportamento com modelo indisponível: não documentado.

### Apelidos estáveis de modelo

- Não documentado. A página só mostra nomes versionados (`gpt-6.1-sol`, `gpt-6-luna`). Formato: minúsculas, números, pontos e hífens. Esforço aceita `ultra`, `max`, `xhigh`, `high`, `medium`, `low`.

### Personas e agentes

- Arquivos TOML, um agente por arquivo, em `.codex/agents/` (projeto) e `~/.codex/agents/` (usuário). `.codex/config.toml` guarda a configuração de projeto.
- Obrigatórios: `name` (fonte de verdade; nome do arquivo é só convenção), `description`, `developer_instructions`. Opcionais: `model`, `model_reasoning_effort`, `model_reasoning_summary`, `model_verbosity`, chaves `features.*` e `skills.*`.
- Leitura de pastas de outras ferramentas: não documentado.

### Paralelismo e limites

- `agents.max_concurrent_threads_per_session` limita as threads abertas (alias antigo `agents.max_threads`); sem valor, o Codex escolhe o padrão (número não documentado). `agents.enabled` padrão `true`. `max_depth`: não documentado.
- Segundo plano: painel de agentes na IDE; nos demais clientes, não documentado. Worktrees: não documentado.

### Fontes

- https://developers.openai.com/codex/subagents (redireciona, 308, para a página abaixo; acessado em 2026-10-29)
- https://learn.chatgpt.com/docs/agent-configuration/subagents (acessado em 2026-10-29)

## Cursor

### Subagentes

- O Agent lança subagentes sozinho (lê a `description`), ou por pedido: `/nome` ou citando o nome. Vários rodam ao mesmo tempo quando o Agent envia várias chamadas da ferramenta `Task` numa mensagem.
- Contexto limpo; o pai precisa colocar no prompt tudo que importa. Devolve só a mensagem final; cada execução devolve um ID de agente que permite retomar com o contexto.
- Exemplo (início copiado da documentação):

```markdown
---
name: security-auditor
description: Security specialist. Use when implementing auth, payments, or handling sensitive data.
model: inherit
readonly: true
---
```

### Listagem e escolha de modelo

- O agente listar modelos: não documentado. A CLI aceita `--model` (exemplo `agent -p "..." --model "gpt-5"`); comando para listar modelos: não documentado nas páginas consultadas.
- Escolha na chamada: não documentado. Na definição: campo `model` (`inherit`, ou ID de modelo, com parâmetros entre colchetes, ex.: `composer-2.5[fast=false]`, `claude-opus-5[effort=high]`, `claude-opus-5[context=300k]`).
- Modelo indisponível (bloqueado por administrador, fora do plano ou sem Max Mode): o Cursor usa um modelo compatível.

### Apelidos estáveis de modelo

- Só `inherit` (valor padrão do campo). `Auto` é um modo de roteamento (Cost, Balance, Intelligence) descrito na página de modelos, não um valor documentado do campo `model`. Nenhum apelido que não envelheça com a versão: não documentado. Formato dos IDs: minúsculas, números, pontos e hífens, com sufixos (`-fast`, `-high`) e colchetes opcionais.

### Personas e agentes

- Markdown com front-matter YAML. Pastas de projeto: `.cursor/agents/`, `.claude/agents/`, `.codex/agents/`; de usuário: `~/.cursor/agents/`, `~/.claude/agents/`, `~/.codex/agents/`. Ou seja, o Cursor lê as pastas do Claude Code e do Codex. Projeto vence usuário; no mesmo nível, `.cursor/` vence as demais.
- Campos: `name` (padrão: nome do arquivo; minúsculas e hífens), `description`, `model` (padrão `inherit`), `readonly` (padrão `false`), `is_background` (padrão `false`). Nenhum é marcado obrigatório na documentação.
- Nota: um arquivo do Codex é TOML; como o Cursor trata `.codex/agents/` com esse formato: não documentado.

### Paralelismo e limites

- Vários ao mesmo tempo; limite numérico: não documentado. Checkouts compartilhados podem conflitar; dá para pedir isolamento por worktree do Git ou VM na nuvem, com branch própria.
- Aninhamento (desde o Cursor 2.5): só o agente principal e seus subagentes diretos disparam subagentes; um subagente de segundo nível não dispara outro.
- Segundo plano: `is_background: true`.

### Fontes

- https://cursor.com/docs/subagents (acessado em 2026-10-29)
- https://cursor.com/docs/models (acessado em 2026-10-29)
- https://cursor.com/docs/cli/overview (acessado em 2026-10-29)

## Gemini CLI

### Subagentes

Subagentes expostos ao agente principal como ferramentas de mesmo nome; o agente delega sozinho pela `description`, ou o usuário força com `@nome`. Cada um tem prompt de sistema próprio (corpo do arquivo) e pode ter ferramentas e servidores MCP próprios. Um subagente não chama outro (proteção contra recursão).

### Listagem e escolha de modelo

Listagem pelo agente: não documentado. Escolha na chamada: não documentado. Na definição: campo `model` (padrão `inherit`, ou ID como `gemini-3-flash-preview`). A documentação de modelos diz que `/model` e `--model` não alteram os modelos dos subagentes. Modelo indisponível: não documentado.

### Apelidos estáveis de modelo

Só `inherit` na definição. `auto`, `pro` e `flash` não são documentados como valores. IDs de exemplo: `gemini-3-pro-preview`, `gemini-3-flash-preview`, `gemini-2.5-pro`, `gemini-2.5-flash`. Aviso das páginas: o Gemini CLI foi substituído pelo Antigravity CLI em 18/06/2026 para usuários do nível gratuito e Google One; é um risco para o suporte futuro.

### Personas e agentes

Markdown com front-matter YAML em `.gemini/agents/*.md` (projeto) e `~/.gemini/agents/*.md` (usuário). Obrigatórios: `name` (minúsculas, números, hífens e sublinhados) e `description`. Opcionais: `kind` (`local` ou `remote`), `tools`, `mcpServers`, `model`, `temperature`, `max_turns` (padrão 30), `timeout_mins` (padrão 10). Prioridade entre os escopos e leitura de pastas de outras ferramentas: não documentado.

### Paralelismo e limites

Limite de paralelismo: não documentado. Sem aninhamento. Worktrees e segundo plano: não documentado.

### Fontes

- https://geminicli.com/docs/core/subagents/ (acessado em 2026-10-29)
- https://geminicli.com/docs/cli/model/ (acessado em 2026-10-29)

## GitHub Copilot

### Subagentes

A documentação separa selecionar um agente personalizado para a sessão de delegar uma subtarefa a ele, que roda em janela de contexto separada. A ferramenta `agent` (alias de ferramenta) permite invocar outro agente. No Copilot CLI, as instruções do repositório entram na sessão do agente, mas por padrão não no subagente personalizado (há opção para mudar).

### Listagem e escolha de modelo

Listagem pelo agente: não documentado. Escolha na chamada: não documentado. Na definição: propriedade `model` (se ausente, herda o padrão); em quais ambientes vale: não documentado.

### Apelidos estáveis de modelo

Não documentado.

### Personas e agentes

Markdown com front-matter YAML, corpo de até 30.000 caracteres. Obrigatório: `description`. Opcionais: `name`, `target` (`vscode` ou `github-copilot`), `tools` (aliases `execute`, `read`, `edit`, `search`, `agent`, `web`, `todo`), `model`, `disable-model-invocation`, `user-invocable`, `mcp-servers`, `metadata`. Níveis: repositório, organização e empresa; o Copilot CLI também aceita perfis pessoais. Pasta e sufixo do arquivo (como `.github/agents/*.agent.md`): não documentado nas páginas consultadas; só se sabe que o nome sem `.md` ou `.agent.md` serve para deduplicar.

### Paralelismo e limites

"Tarefas independentes também podem rodar em paralelo", sem número nem configuração: limites não documentados.

### Fontes

- https://docs.github.com/en/copilot/reference/custom-agents-configuration (acessado em 2026-10-29)
- https://docs.github.com/en/copilot/concepts/agents/coding-agent/about-custom-agents (acessado em 2026-10-29)

## Tabela comparativa

| Ferramenta | Subagentes | Agente lista modelos | Modelo na chamada do subagente | Apelidos estáveis | Definição de agente (pasta e formato) | Paralelismo | Compatível com o harness |
|---|---|---|---|---|---|---|---|
| Claude Code | sim (ferramenta `Agent`) | não documentado | sim (parâmetro `model`: `sonnet`, `opus`, `haiku`, `fable`) | sim (`haiku`, `sonnet`, `opus`, `fable`, `inherit`) | `.claude/agents/<nome>.md`, Markdown com front-matter | sim, 20 simultâneos por padrão; aninha até 3 camadas | sim |
| Codex | sim (a pedido ou por `AGENTS.md`) | não documentado | sim, pelo prompt de disparo (formato estruturado não documentado) | não documentado | `.codex/agents/<nome>.toml`, TOML | sim, `agents.max_concurrent_threads_per_session` (padrão não documentado) | sim |
| Cursor | sim (ferramenta `Task`) | não documentado | não documentado | parcial (só `inherit`) | `.cursor/agents/<nome>.md` e lê `.claude/agents` e `.codex/agents`, Markdown | sim, limite não documentado; um nível de aninhamento | sim |
| Gemini CLI | sim (ferramenta com o nome do agente) | não documentado | não documentado | parcial (só `inherit`) | `.gemini/agents/<nome>.md`, Markdown | não documentado; sem aninhamento | parcial (definição documentada; ferramenta ainda não suportada pelo `vostok`) |
| GitHub Copilot | parcial (delegação a agente personalizado) | não documentado | não documentado | não documentado | Markdown com front-matter; pasta não documentada | parcial (citado, sem limites) | não |

## Recomendação para o harness

### Ferramentas compatíveis

| Ferramenta | Valor de harness.tool | Gera definições de subagente | Motivo |
|---|---|---|---|
| Claude Code | `claude-code` | sim | Documenta `.claude/agents/<persona>.md` com `model`; é a ferramenta com apelidos e parâmetro de modelo na chamada. |
| Codex | `codex` | sim | Documenta `.codex/agents/<persona>.toml` com `name`, `description` e `developer_instructions` obrigatórios. |
| Cursor | `cursor` | sim, mas só se a pasta do Claude Code não for gerada | Documenta `.cursor/agents/<persona>.md` e já lê `.claude/agents/` e `.codex/agents/`; gerar nas duas pastas duplicaria as personas. |
| Gemini CLI | `gemini-cli` | sim | Documenta `.gemini/agents/<persona>.md`. O `vostok` ainda não suporta a ferramenta em `TOOLS`; só entra quando ela for adicionada. |
| GitHub Copilot | `copilot` | não | Pasta e sufixo dos arquivos de agente não estão na documentação consultada; "gera" exige definição documentada em arquivo do projeto. |

Regra de duplicação: quando `claude-code` está entre as ferramentas do projeto, o `vostok` não gera `.cursor/agents/`, porque o Cursor lê `.claude/agents/` (o mesmo raciocínio de `.cursor/skills` em `config/skills.yml`). Um arquivo `.codex/agents/*.toml` é TOML e não deve ser contado como definição lida pelo Cursor (formato não documentado).

### Definições de subagente por ferramenta

Campos preenchidos a partir de `vostok/personas/<persona>.md`: nome (identificador da persona), descrição (quando usar), modelo (omitido no padrão `inherit`, pois o Coordenador escolhe o modelo na chamada ou pelo perfil da tarefa) e instruções que mandam ler a persona e rodar `vostok context load --persona <persona>`.

| Ferramenta | Arquivo | Formato | Campos |
|---|---|---|---|
| `claude-code` | `.claude/agents/<persona>.md` | Markdown com front-matter YAML | `name`, `description`, corpo com as instruções (e `model: inherit`) |
| `codex` | `.codex/agents/<persona>.toml` | TOML | `name`, `description`, `developer_instructions` |
| `cursor` | `.cursor/agents/<persona>.md` (só sem `claude-code`) | Markdown com front-matter YAML | `name`, `description`, `model: inherit`, corpo |
| `gemini-cli` | `.gemini/agents/<persona>.md` | Markdown com front-matter YAML | `name`, `description`, `model: inherit`, corpo |

Exemplo completo para a persona `arquiteto` no Claude Code (`.claude/agents/arquiteto.md`):

```markdown
---
name: arquiteto
description: Arquiteto do harness Vostok. Use para executar tarefas que pedem desenho técnico e decisões de estrutura.
model: inherit
---

Você é o Arquiteto. Leia `vostok/personas/arquiteto.md` e rode
`vostok context load --persona arquiteto` antes de começar. Siga as regras do
projeto carregadas e devolva um resumo curto do que fez.
```

Exemplo para o Codex (`.codex/agents/arquiteto.toml`):

```toml
name = "arquiteto"
description = "Arquiteto do harness Vostok. Use para executar tarefas que pedem desenho técnico e decisões de estrutura."
developer_instructions = """
Você é o Arquiteto. Leia vostok/personas/arquiteto.md e rode
`vostok context load --persona arquiteto` antes de começar. Siga as regras do
projeto carregadas e devolva um resumo curto do que fez.
"""
```

### Apelidos de reserva por perfil

Só apelidos estáveis documentados. Só o Claude Code tem; Codex, Cursor, Gemini CLI e Copilot ficam sem entrada (o `vostok` mostra "modelo da sessão").

```yaml
profiles:
  mecanica:
    description: passos diretos e repetitivos; prefere modelo rápido
    fallback:
      claude-code: haiku
  padrao:
    description: implementação com decisões locais; prefere modelo intermediário
    fallback:
      claude-code: sonnet
  julgamento:
    description: desenho, ambiguidade e decisões amplas; prefere o modelo mais capaz
    fallback:
      claude-code: opus
```

Expressão regular do formato de apelido (RF-20), em minúsculas, com sufixo opcional entre colchetes:

```text
^[a-z][a-z0-9._-]*(\[[a-z0-9=,._-]+\])?$
```

Ela aceita todos os valores encontrados no estudo: `haiku`, `sonnet`, `opus`, `fable`, `inherit`, `sonnet[1m]`, `opus[1m]`, `claude-opus-5-5`, `claude-sonnet-4-5-20250929`, `claude-fable-5-1`, `claude-opus-4-6[1m]`, `gpt-6-luna`, `gpt-6.1-sol`, `gpt-5.2-high`, `composer-2.5[fast=false]`, `claude-opus-5[effort=high]`, `gemini-3-flash-preview`, `gemini-2.5-pro`. Recusa, por exemplo, espaço, maiúsculas e `:`. O `vostok` confere só o formato, não a disponibilidade.

### Escolha do modelo pelo Coordenador

| Ferramenta | Lista modelos | Onde passa o modelo | Sem poder escolher | Subagentes simultâneos |
|---|---|---|---|---|
| `claude-code` | não documentado | Parâmetro `model` do `Agent` (só apelido) ou `model` da definição | Usar o apelido de reserva do perfil (`haiku`, `sonnet`, `opus`) | até 20 (padrão); o harness usa no máximo 6 por onda |
| `codex` | não documentado | Pedido de modelo no prompt de disparo, ou `model` do arquivo `.toml` | Modelo da sessão (sem apelido) | `agents.max_concurrent_threads_per_session` (padrão não documentado); o harness usa no máximo 6 por onda |
| `cursor` | não documentado | Campo `model` da definição (chamada: não documentado) | Modelo da sessão (`inherit`) | limite não documentado; o harness usa no máximo 6 por onda e pede isolamento por worktree |
| `gemini-cli` | não documentado | Campo `model` da definição | Modelo da sessão (`inherit`) | não documentado; uma por vez |
| `copilot` | não documentado | Propriedade `model` da definição | Modelo da sessão | não documentado; uma por vez |

Como o Coordenador decide: como nenhuma ferramenta documenta que o agente liste os modelos, o Coordenador usa o que a ferramenta oferecer na própria sessão (por exemplo, o seletor ou os apelidos que ele conhece) e escolhe pela descrição do perfil. Quando não consegue listar ou escolher, usa o apelido de reserva do perfil e, sem ele, o modelo da sessão; a execução segue com um agente e esse modelo, como hoje (FA-01). O modelo usado vai para o registro de execução da tarefa. **Limite do harness:** a execução em onda dispara no máximo 6 subagentes ao mesmo tempo, em qualquer ferramenta, mesmo quando a ferramenta aceita mais (o Claude Code aceita 20 por padrão, o único limite documentado). Numa onda com mais de 6 tarefas, as excedentes esperam uma vaga. Gemini CLI e Copilot, sem limite documentado e fora do foco, seguem uma tarefa por vez.

### Riscos e reavaliação

- Os nomes completos de modelo e os padrões mudam a cada versão; por isso a reserva usa apelidos e o `vostok` só confere formato. Sinal de problema: apelido recusado na primeira requisição ("There's an issue with the selected model" no Claude Code).
- O apelido (`opus`, `sonnet`, `haiku`) resolve para versões diferentes por provedor; o mesmo perfil pode render modelos de capacidade diferente em Bedrock ou Foundry.
- Codex, Cursor e Gemini CLI mudam de domínio e de nome com frequência (a documentação do Codex já redireciona para `learn.chatgpt.com`; o Gemini CLI foi substituído pelo Antigravity CLI para parte dos usuários). Sinal: o endereço de "Fontes" deixa de abrir ou a pasta de agentes muda.
- Pontos "não documentado" podem passar a ser documentados (lista de modelos, parâmetro de modelo no Cursor e no Codex, pasta de agentes do Copilot); cada um muda a tabela acima.
- Refazer o estudo a cada release maior de uma das ferramentas, quando um apelido for recusado, ou ao adicionar `gemini-cli` ou `copilot` a `TOOLS`.
