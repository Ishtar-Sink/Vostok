# Contexto obrigatório por ferramenta

Data do estudo e das consultas: 2026-11-11. O estudo vem só da documentação oficial; nada foi executado nas ferramentas. A leitura foi por extração automática de texto; o que a página não descreve está marcado "não documentado".

## Escopo e método

Para Claude Code (referência), Codex, Cursor, Gemini CLI e GitHub Copilot, o estudo responde: (a) o evento de início de sessão e se ele dispara na retomada, na limpeza e depois da compactação; (b) o evento antes de um comando de shell e como ele recusa com um motivo; (c) onde a configuração fica no projeto e o formato; (d) o limite de saída que a ferramenta injeta no contexto antes de cortar ou mandar para arquivo.

Medição em 2026-11-11: `vostok context load | wc -c` = 31.553 bytes; `wc -c vostok/context/skill-catalog.md` = 13.674 bytes; núcleo (diferença) = 17.879 bytes (o Coordenador mediu 17.845 no baseline; a diferença de 34 bytes vem de separadores e não muda a conclusão). Atenção: o limite do Claude Code é em caracteres e o texto tem acentos em UTF-8, então o número de caracteres é um pouco menor que o de bytes, mas segue bem acima do limite.

## Claude Code

- (a) Evento `SessionStart`, com `matcher` pela origem: `startup`, `resume`, `clear`, `compact` e `fork`. Dispara de novo na retomada e depois da compactação (origem `compact`). O texto vai para o contexto antes do primeiro prompt.
- (b) `PreToolUse` com `matcher` `Bash`; recusa com JSON `hookSpecificOutput.permissionDecision: "deny"` e `permissionDecisionReason`, ou com saída 2 e o motivo no stderr.
- (c) `.claude/settings.json` (versionado), `.claude/settings.local.json`, `~/.claude/settings.json`; JSON: evento, grupo de `matcher`, lista de handlers `{type: command, command}`.
- (d) Limite exato: 10.000 caracteres para cada string de `additionalContext`, `systemMessage`, `initialUserMessage` e para o stdout puro; cada campo é medido separadamente, mesmo com vários hooks no mesmo evento. Passou do limite (exceto stdout de PreCompact): o Claude Code grava a saída num arquivo no diretório da sessão e no lugar coloca o caminho do arquivo e uma prévia dos primeiros 2.000 caracteres. A documentação diz que o Claude Code não manda o modelo ler o arquivo, então o que o modelo precisa sempre ver deve caber no limite. Não há configuração nem variável de ambiente para subir o limite.
- `additionalContext` (JSON) tem o mesmo limite de 10.000 caracteres do stdout puro; a diferença é só que, com vários hooks, cada valor é medido por si e todos chegam ao modelo (então dividir em vários hooks ou vários campos é uma opção, cada um até 10.000).
- Fontes: https://code.claude.com/docs/en/hooks (consultada em 2026-11-11).

## Codex

- (a) `SessionStart`, com `matcher` pela origem `startup`, `resume`, `clear`, `compact` ou `fork`; stdout em texto vira contexto de desenvolvedor e o JSON aceita `hookSpecificOutput.additionalContext`.
- (b) `PreToolUse` com `matcher` `Bash` (comando em `tool_input.command`); recusa com `permissionDecision: "deny"` e `permissionDecisionReason`, com a forma antiga `decision: "block"` e `reason`, ou saída 2 com o motivo no stderr.
- (c) `<repo>/.codex/hooks.json` ou tabelas `[hooks]` em `<repo>/.codex/config.toml` (e as versões em `~/.codex/`). Hook de projeto só carrega com a camada `.codex/` confiável, e os hooks não gerenciados precisam ser revisados e confiados em `/hooks` antes de rodar.
- (d) Cada mensagem de saída visível ao modelo é limitada a cerca de 2.500 tokens por padrão (aprox. 10 mil caracteres); acima disso a saída vai para `<temp_dir>/hook_outputs/<session_id>/<uuid>.txt` e o modelo recebe uma prévia do começo e do fim. O handler aceita `additionalContextLimit` (inteiro positivo muda o limite; `0` passa o texto inteiro), válido só para `additionalContext`.
- Fontes: https://developers.openai.com/codex/hooks, que redireciona para https://learn.chatgpt.com/docs/hooks (consultada em 2026-11-11).

## Cursor

- (a) `sessionStart`, ao criar uma conversa (composer); é fire-and-forget (o laço do agente não espera). Pode injetar `additional_context` no contexto inicial do sistema e definir `env`. Retomada: não documentado.
- (b) `beforeShellExecution`; recusa com `"permission": "deny"` mais `user_message` e `agent_message`, ou saída 2. JSON inválido bloqueia; falha ou timeout deixam passar, salvo `failClosed: true`.
- (c) `<projeto>/.cursor/hooks.json` (e `~/.cursor/hooks.json`); formato `{"version": 1, "hooks": {"<evento>": [{"command", "timeout", "matcher"}]}}`.
- Compactação: `preCompact` só observa; não bloqueia nem muda a compactação, e não injeta contexto. Não há evento documentado para reinjetar depois dela.
- (d) Limite de saída: não documentado.
- Fonte: https://cursor.com/docs/hooks (consultada em 2026-11-11).

## Gemini CLI

- (a) `SessionStart` com `source` `startup`, `resume` ou `clear` (não há `compact`). `additionalContext` entra como primeiro turno do histórico no modo interativo e é anexado ao prompt no não interativo.
- (b) `BeforeTool` com `matcher` pelo nome da ferramenta (`run_shell_command`); recusa com `decision: "deny"` (ou `block`) e `reason`, enviado ao agente como erro da ferramenta, ou saída 2 com o motivo no stderr.
- (c) Objeto `hooks` em `settings.json`; a página de hooks não diz o caminho do arquivo (não confirmado `.gemini/settings.json` por essa fonte). Timeout padrão de 60.000 ms. O arquivo de contexto padrão é `GEMINI.md`, e `context.fileName` aceita uma lista que pode incluir `AGENTS.md`.
- Compactação: `PreCompress` (`trigger` `auto` ou `manual`) é só consultivo e assíncrono; não bloqueia nem injeta contexto.
- (d) Limite de saída: não documentado.
- Fontes: https://geminicli.com/docs/hooks/reference/ e https://geminicli.com/docs/cli/gemini-md/ (consultadas em 2026-11-11).

## GitHub Copilot

- (a) `sessionStart`, "uma sessão nova ou retomada começa"; a saída pode trazer `additionalContext` (único campo consumido). No agente de nuvem dispara uma vez por job, como sessão nova. Limpeza: não documentado.
- (b) `preToolUse`; recusa com JSON `permissionDecision: "deny"` e `permissionDecisionReason`, mostrado ao agente. Falha ou saída diferente de zero (inclusive 2) nega a chamada (fail-closed); timeout deixa passar. No agente de nuvem, `ask` vale como `deny`.
- (c) Copilot CLI: `.github/hooks/*.json` na raiz do repositório (também `~/.copilot/hooks/`, blocos `hooks` em settings, plugins e políticas em `/etc/github-copilot/policy.d/`). Agente de nuvem: só `.github/hooks/*.json` do repositório clonado. VS Code: caminho não descrito na página.
- Compactação: `preCompact` só notifica; a saída não é processada.
- (d) A saída de hook é limitada a 10 MiB por invocação, e o `additionalContext` combinado de vários hooks segue o mesmo limite; na prática, sem corte para o núcleo de 17,9 KB.
- Fonte: https://docs.github.com/en/copilot/reference/hooks-configuration (consultada em 2026-11-11).

## Tabela comparativa

| Ferramenta | Início de sessão | Compactação | Bloqueio de comando | Configuração | Limite de saída |
|---|---|---|---|---|---|
| Claude Code | `SessionStart` (startup, resume, clear, compact, fork), injeta stdout ou `additionalContext` | Reinjeta com `SessionStart` origem `compact` | `PreToolUse` `Bash`, `deny` com motivo ou saída 2 | `.claude/settings.json` (JSON) | 10.000 caracteres por campo; acima, arquivo + prévia de 2.000 caracteres |
| Codex | `SessionStart` (startup, resume, clear, compact, fork) | Reinjeta com origem `compact` | `PreToolUse` `Bash`, `deny` com motivo ou saída 2 | `.codex/hooks.json` ou `.codex/config.toml`, com confiança | ~2.500 tokens; acima, arquivo + prévia (ajustável por `additionalContextLimit`) |
| Cursor | `sessionStart` com `additional_context`, fire-and-forget | `preCompact` só observa; sem reinjeção | `beforeShellExecution`, `deny` com mensagens ou saída 2 | `.cursor/hooks.json` | Não documentado |
| Gemini CLI | `SessionStart` (startup, resume, clear) com `additionalContext` | `PreCompress` consultivo; sem reinjeção | `BeforeTool` `run_shell_command`, `deny` com `reason` ou saída 2 | `hooks` em `settings.json` (caminho não confirmado) | Não documentado |
| GitHub Copilot | `sessionStart` (nova ou retomada) com `additionalContext` | `preCompact` só notifica; sem reinjeção | `preToolUse`, `deny` com motivo (fail-closed) | `.github/hooks/*.json` | 10 MiB por invocação |

## Volume do núcleo

O núcleo mede 17.879 bytes (cerca de 17,8 mil caracteres, medição de 2026-11-11), contra um limite de 10.000 caracteres no Claude Code e de cerca de 2.500 tokens (aprox. 10 mil caracteres) no Codex. Passa dos dois: no Claude Code o modelo veria só o caminho de um arquivo e 2.000 caracteres de prévia (sem ler o arquivo sozinho); no Codex, uma prévia de começo e fim. Cursor e Gemini CLI não documentam limite, e o Copilot admite 10 MiB, então o núcleo cabe.

Opções para o RNF-01 (garantia de contexto sem corte):
1. Enxugar o núcleo para menos de 10.000 caracteres: corta quase metade do conteúdo (perda de regras, ou regras só por demanda).
2. Injetar só um índice curto (regras que nunca podem faltar, os IDs e o pedido explícito `vostok context load`): cabe com folga, mas depende de o agente rodar o comando.
3. Dividir a injeção em vários campos ou hooks de até 10.000 caracteres cada (o limite do Claude Code é por campo, e todos chegam ao modelo): mantém o núcleo inteiro, mas é frágil se a regra do limite mudar (hoje a documentação diz por campo, e não há garantia de que se mantenha).
4. Para Codex, subir `additionalContextLimit` no handler, o que mantém o núcleo inteiro sem dividir.

Recomendação: injetar no `SessionStart` um índice abaixo de 10.000 caracteres (as regras de limites mais o pedido de rodar `vostok context load`, e o `hook session-start` sempre medindo o tamanho), e dividir em campos só se o índice não couber; para o Codex, usar `additionalContextLimit`. O `hook session-start` deve falhar alto (aviso) se passar do limite da ferramenta, em vez de deixar o corte silencioso.

## Decisão por ferramenta

- Claude Code: a TASK-05 configura `.claude/settings.json` com `SessionStart` (startup, resume, clear, compact) rodando `vostok hook session-start` e `PreToolUse` `Bash` rodando `vostok hook pre-bash`; garantia de início, retomada e compactação, desde que a saída fique abaixo de 10.000 caracteres por campo.
- Codex: a TASK-05 configura `.codex/hooks.json` com `SessionStart` (inclui `compact`) para `vostok hook session-start`, com `additionalContextLimit`, e `PreToolUse` `Bash` para `vostok hook pre-bash`; lacuna: o hook de projeto exige confiança do usuário em `/hooks`, então sem ela o `AGENTS.md` pede o `vostok context load`.
- Cursor: a TASK-05 configura `.cursor/hooks.json` com `sessionStart` para `vostok hook session-start` e `beforeShellExecution` para `vostok hook pre-bash`; lacuna: início sem espera do agente, sem reinjeção depois da compactação e limite não documentado, então o `AGENTS.md` pede o `vostok context load`.
- Gemini CLI: a TASK-05 configura o `settings.json` com `SessionStart` (startup, resume, clear) e `BeforeTool` `run_shell_command`; lacuna: sem reinjeção depois da compactação, limite e caminho do arquivo não confirmados, então o `AGENTS.md` (via `context.fileName`) pede o `vostok context load`.
- GitHub Copilot: a TASK-05 configura `.github/hooks/*.json` com `sessionStart` para `vostok hook session-start` e `preToolUse` para `vostok hook pre-bash` (fail-closed); lacuna: sem reinjeção depois da compactação, então o `AGENTS.md` (instruções do repositório) pede o `vostok context load`.
