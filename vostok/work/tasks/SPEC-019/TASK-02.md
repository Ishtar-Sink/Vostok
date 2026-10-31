---
id: TASK-02
spec: SPEC-019
title: "maxTurns nas definições de agente"
profile: padrao
persona: desenvolvedor
depends_on: []
wave: 1
covers: [RF-07, CT-08, CB-07]
type: tarefa
labels: [SPEC-019, padrao]
status: done
content_hash: e313020661e72b42f2731d93d72d690c26055ef0a39f7f6d9184485f0a146973
adjustments: 0
adjusted_at: null
approved_at: 2026-10-31T10:32:31Z
approved_hash: e68ccfd34fcc751a6956df6e90f8b88a981cf8f1e2da00a0e7756804d1d64203
attempts: 0
tests_result: passed
regression_result: passed
evidence_result: passed
model: sonnet
---

# TASK-02: maxTurns nas definições de agente

## Execução

### Ler

- `vostok/work/specs/refined/SPEC-019-reduzir-o-consumo-de-tokens-dos-subagentes.md` (RF-05, RF-07, contratos)
- `apps/cli/src/routing.rs` (`render_agent`, `agent_targets`, `write_agents`, `AgentFormat`, os alvos de agente do Claude Code e do Cursor)
- `apps/cli/tests/init_update.rs` (testes `ct08_*` das definições de agente)

### Mudar

- `apps/cli/src/routing.rs` (alterar): `maxTurns` por persona nas definições do Claude Code e instrução inicial que segue o briefing.
- `apps/cli/tests/init_update.rs` (alterar): testes do `maxTurns` e da instrução nova.

### Passos

1. Em `apps/cli/tests/init_update.rs`, escreva os testes e confirme que falham:
   - `max_turns_in_claude_code_agents` (CT-08, RF-07): `vostok init --yes --tools claude-code`; `.claude/agents/vostok-desenvolvedor.md` e `vostok-ui-ux-designer.md` têm a linha `maxTurns: 60` logo depois de `model: inherit`; `vostok-qa.md`, `vostok-arquiteto.md`, `vostok-engenheiro.md`, `vostok-qa-lead.md` e `vostok-planejador.md` têm `maxTurns: 30`.
   - `max_turns_only_for_claude_code` : com `--tools cursor` e com `--tools codex`, nenhuma definição gerada contém `maxTurns`.
   - `agents_follow_the_task_brief` (RF-05): as definições do Claude Code dizem para seguir a mensagem do Coordenador e citam `vostok task brief` para tarefas e `vostok context load --persona <nome>` para o resto.
2. `apps/cli/src/routing.rs`:
   - A instrução de `render_agent` passa a ser: `Persona <título> do harness Vostok, como subagente.` / `Numa tarefa, a mensagem do Coordenador manda rodar `vostok task brief <SPEC_ID> <TASK_ID>`: rode só ele e siga o que ele imprime.` / `Em qualquer outro trabalho, antes de qualquer ação, execute `vostok context load --persona <nome>` e siga a persona e o contexto carregados.` / `O que entregar e o modelo vêm da mensagem do Coordenador.` (o mesmo texto para os três formatos).
   - `maxTurns`: uma função `fn max_turns(persona: &str) -> u32` (60 para `desenvolvedor` e `ui-ux-designer`, 30 para as demais); a linha `maxTurns: N` entra só na definição do Claude Code. Se o formato Markdown também serve ao Cursor, passe a `render_agent` a informação do alvo (por exemplo o `Tool` ou um `bool`) para decidir; não mude o formato do Cursor nem o do Codex.
3. Ajuste os testes `ct08_*` que conferem o texto antigo da instrução (`vostok context load --persona {name}` continua presente).
4. Rode `cargo test --manifest-path apps/cli/Cargo.toml --test init_update` até passar, depois fmt, clippy e `cargo test --manifest-path apps/cli/Cargo.toml` inteiro.

### Baseline

- `cargo run --quiet --manifest-path apps/cli/Cargo.toml -- rules validate`
- `cargo test --manifest-path apps/cli/Cargo.toml --test init_update 2>&1 | grep -E '^test ' | grep -v '^test result' | grep -vE 'max_turns_|agents_follow_the_task_brief' | sort`

### Verificação

```bash
cargo fmt --manifest-path apps/cli/Cargo.toml --check
cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path apps/cli/Cargo.toml --test init_update max_turns_
cargo test --manifest-path apps/cli/Cargo.toml --test init_update agents_follow_the_task_brief
cargo test --manifest-path apps/cli/Cargo.toml
cargo run --quiet --manifest-path apps/cli/Cargo.toml -- rules validate
```

Resultado esperado: formatação e clippy sem avisos; os testes novos e toda a suíte passam; `rules validate` imprime `Regras e mapa válidos.`

### Evidências

- `agent-max-turns-tests.txt` (automática): `cargo test --manifest-path apps/cli/Cargo.toml --test init_update -- max_turns_ agents_follow_the_task_brief 2>&1 | grep -E '^test (max_turns_|agents_follow)' | sort`
  - Prova: `maxTurns` por persona só no Claude Code (RF-07, CT-08) e a instrução que segue o briefing (RF-05).
  - Trecho: três linhas `test ... ok`.
  - Falha: `FAILED` ou menos de três linhas.

### Concluída quando

- [x] As definições do Claude Code têm `maxTurns` 60 para `desenvolvedor` e `ui-ux-designer` e 30 para as demais, e as do Codex e do Cursor não mudam (RF-07, CT-08).

  > Como comprovar: em `agent-max-turns-tests.txt`, ver `max_turns_in_claude_code_agents` e `max_turns_only_for_claude_code` com `ok`.

- [x] A instrução das definições manda seguir o `vostok task brief` numa tarefa (RF-05).

  > Como comprovar: ver `agents_follow_the_task_brief ... ok`.

- [x] A suíte inteira, o fmt e o clippy passam.

  > Como comprovar: rodar a Verificação e ver todos os comandos sem erro.

## Descrição

### Resumo executivo

Põe um teto de turnos nos subagentes e faz cada um começar pelo resumo curto da tarefa, em vez de carregar o contexto completo do harness. Perfil padrão, risco baixo.

### Detalhes técnicos

- **Abordagem:** `render_agent` ganha `maxTurns` por persona no Claude Code e uma instrução que prioriza o `vostok task brief`.
- **Decisões:** inferência D (só Claude Code, por persona, sem `effort`); a instrução nova evita que a definição de agente carregue o contexto obrigatório antes do briefing, o que anularia o ganho do RF-05 (detalhe que a spec não previa nos componentes, mas o arquivo é o mesmo `routing.rs`).
- **Riscos:** `maxTurns` corta uma tarefa grande; 60 dá folga para `desenvolvedor`.
- **Referências:** RF-05, RF-07, CT-08, CB-07.
- **Arquivos em comum:** nenhum com a TASK-01 da mesma onda.

### Card

- **Título:** maxTurns nas definições de agente
- **Tipo:** tarefa
- **Descrição:** As definições de agente do Claude Code ganham `maxTurns` por persona e passam a começar pelo briefing da tarefa.
- **Critérios de aceite:**
  - [ ] maxTurns por persona no Claude Code
  - [ ] Definições seguem o vostok task brief
- **Rótulos:** SPEC-019, padrao
- **Estimativa:** padrao
- **Bloqueada por:** -

## Registro de execução

| # | Data | Etapa | Resultado | Detalhe |
|---|---|---|---|---|
| 1 | 2026-10-31 | início | in_progress | aprovada em 2026-10-31T10:32:31Z; modelo: sonnet |
| 2 | 2026-10-31 | baseline | coletado | 2 comando(s); códigos de saída: 0, 0; 1 evidência(s) automática(s) gravada(s) para comparação |
| 3 | 2026-10-31 | testes | passed | Verificação sem falhas; saída em validation/tests.out |
| 4 | 2026-10-31 | regressão | passed | 2 comando(s) iguais ao baseline |
| 5 | 2026-10-31 | evidências | passed | 1 evidência(s) em evidence/TASK-02/ |
| 6 | 2026-10-31 | conclusão | done | testes, regressão e evidências passed; critérios de conclusão marcados |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
