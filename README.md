# Vostok

Harness para agentes de IA: regras, skills, scripts e memória que levam qualquer ferramenta (Claude Code, Codex, Cursor) pelo mesmo caminho de spec, aprovação, plano e tarefas.

O nome vem da Vostok, a nave do primeiro voo tripulado. Em russo, *vostok* significa "oriente": o harness orienta cada agente. A CLI se chamará `vostok`.

O Vostok faz parte do ecossistema [Ishtar Sink](https://github.com/Ishtar-Sink).

## Situação

| Fase | O que entrega | Status |
|---|---|---|
| 1. Harness local | Regras, skills e scripts funcionando neste repositório. | Em andamento |
| 2. Serviços e painel | Servidor HTTP com MongoDB, servidor MCP e painel web. | Planejada |
| 3. Harness nos dois sentidos | Harness local sincronizado com os serviços. | Planejada |

O [mapa do projeto](docs/mapa-vostok.html) mostra, em diagramas interativos, o que já existe e como as partes se conectam.

## Como funciona

- **Harness passivo.** O agente trabalha na ferramenta do desenvolvedor e segue o harness. O harness não depende de nenhuma ferramenta.
- **Determinístico sempre que possível.** O agente escreve o conteúdo; scripts criam arquivos, validam e registram.
- **Configurável.** Caminhos, status, tipos, seções e sequências de IDs ficam em `packages/harness/config/`.
- **Respostas com template.** Toda resposta do agente segue um dos templates de `templates/responses/`.
- **Memória.** Decisões (DEC), issues (ISS) e conhecimento (KNW) em Markdown, com scripts para criar, buscar e atualizar.
- **Serviços por pull** (fase 2). O agente busca trabalho no servidor MCP; o servidor HTTP é o único que grava no MongoDB. Specs podem ser aprovadas na ferramenta de IA, com envio pelo MCP, ou no painel web.

### Carregamento de contexto

`CLAUDE.md` aponta para `AGENTS.md`, que executa `load-context.sh`. As fontes carregadas em toda sessão estão em `config/context.yml`. As regras do harness prevalecem sobre a memória e as instruções próprias da ferramenta.

### Ciclo da spec

```
pedido → draft → refined → approved → plano → tarefas → execução
                    ↑  ↓
                  ajustes
```

| Etapa | Skill | O que faz |
|---|---|---|
| Rascunho | `create-new-spec.sh` | Cria a spec em `work/specs/draft/`. |
| Refinamento | `refine-spec` | Investiga, pergunta e monta a spec refinada com seções por tipo (feature, improvement, fix, study, analysis). |
| Ajuste | `adjust-spec` | Aplica um ou vários ajustes por mensagem e mantém o histórico. Ajustar uma spec aprovada derruba a aprovação. |
| Aprovação | `approve-spec` | Confere os impedimentos, grava a versão aprovada e remove o rascunho. |
| Plano e tarefas | — | Planejado: quebra da spec aprovada em tarefas. |

Há também a skill `create-skill`, que cria skills no formato padrão e gera os redirecionamentos para cada ferramenta (`.claude/skills/`, `.agents/skills/`).

## Estrutura

```
apps/
  mcp/          servidor MCP (fase 2)
  server/       servidor HTTP (fase 2)
  web/          painel web (fase 2)
packages/
  contracts/    contratos compartilhados (fase 2)
  harness/
    config/     tudo o que é configurável
    context/    catálogo de skills, carregado em toda sessão
    memory/     decisões, issues e conhecimento
    rules/      regras do agente e do projeto
    scripts/    scripts chamados pelo agente e bibliotecas
    skills/     skills padrão (default/) e do usuário (custom/)
    templates/  respostas, specs, skills e memória
    tests/      suíte dos scripts e cenários com agente
    work/       specs e tarefas em andamento
docs/           mapa do projeto
```

## Requisitos

- bash 3.2 ou mais recente, com awk e sed do GNU ou do BSD (Linux e macOS).
- `sha256sum` ou `shasum`.
- git, para detectar as seções alteradas em edições manuais.
- python3, só para a suíte de testes.

## Testes

A suíte roda sobre uma cópia descartável do repositório:

```bash
packages/harness/tests/run-tests.sh
```

Os cenários com agente estão descritos em [agent-scenarios.md](packages/harness/tests/agent-scenarios.md).

## Idioma

Documentação em português do Brasil; código em inglês.
