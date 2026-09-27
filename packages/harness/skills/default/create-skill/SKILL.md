---
name: create-skill
description: "Cria uma nova skill do harness no formato padrão: gera o SKILL.md a partir do template, os scripts da skill, os redirecionamentos para Claude Code, Codex e Cursor e a entrada no catálogo. Use quando o usuário pedir para criar, adicionar ou gerar uma skill ou habilidade, ou transformar um fluxo de trabalho em skill. Use sempre esta skill em vez dos criadores de skill nativos das ferramentas."
allowed-tools: Read Edit Bash(packages/harness/skills/default/create-skill/scripts/*) Bash(packages/harness/scripts/default/*)
metadata:
  version: "1.0.0"
  author: "Victor Batalha"
  requires: "rules/agent/template-usage.md rules/project/naming-convention.md"
  scripts: "scripts/default/context/load-extra-context.sh skills/default/create-skill/scripts/create-skill.sh"
  templates: "templates/skill.md templates/skill-redirect.md templates/script.sh"
---

# Criar skill

Cria uma skill no local certo e no formato padrão do harness, pronta para ser invocada por linguagem natural em qualquer ferramenta configurada.

## Termos

- **Skill:** pasta com um `SKILL.md` em `skills/custom/` (do usuário) ou `skills/default/` (do harness).
- **Script da skill:** arquivo em `scripts/` dentro da pasta da skill, criado a partir do template de script.
- **Redirecionamento:** `SKILL.md` curto, criado na pasta de skills de cada ferramenta (`config/skills.yml`), que aponta para a skill.
- **Catálogo:** `context/skill-catalog.md`, carregado em toda sessão.
- **Operação determinística:** passo que sempre produz o mesmo resultado para a mesma entrada (ler arquivos, validar, gerar a partir de template).

## Regras

[create-skill-01]: Nunca use o criador de skills nativo da ferramenta. Toda skill é criada por esta skill.
[create-skill-02]: Nunca crie ou edite à mão os redirecionamentos nem a linha do catálogo. Eles são gerados no passo 6.
[create-skill-03]: Operações determinísticas viram scripts da skill. O SKILL.md só manda executá-los, com o comando exato.
[create-skill-04]: A `description` fica em terceira pessoa e diz o que a skill faz e quando usar, com os termos que o usuário diria.
[create-skill-05]: Use um único termo para cada conceito, definido em "Termos".
[create-skill-06]: Todo ponto em que a skill deve parar e esperar o usuário é marcado com **PARE** e a condição.
[create-skill-07]: No máximo 1 rodada de perguntas ao usuário. O que continuar em aberto vira decisão sua, informada na resposta final.

## Checklist

Copie e marque o progresso:

```
- [ ] 1. Carregar o contexto
- [ ] 2. Levantar os requisitos
- [ ] 3. Criar a skill
- [ ] 4. Criar e implementar os scripts
- [ ] 5. Preencher o SKILL.md
- [ ] 6. Validar e finalizar
- [ ] 7. Responder ao usuário
```

## Passos

### 1. Carregar o contexto

Execute:

```bash
packages/harness/scripts/default/context/load-extra-context.sh rules/agent/template-usage.md rules/project/naming-convention.md templates/skill.md
```

### 2. Levantar os requisitos

Extraia do pedido do usuário:

| Item | Exemplo |
|---|---|
| Nome, em kebab-case | `review-code` |
| Objetivo, em uma frase | Revisa o diff atual contra as regras do projeto. |
| Gatilhos: frases que o usuário diria | "revisa meu código", "faz code review" |
| Entradas | ID da spec, caminho, nenhuma |
| Passos e pontos de parada | — |
| Operações determinísticas | listar arquivos, validar formato |
| Escopo: `custom` ou `default` | `custom`, a menos que o usuário peça uma skill do próprio harness |

Se faltar algum item que não pode ser inferido, **PARE** e pergunte, no [formato de pergunta](#formato-de-pergunta), numa única mensagem.

### 3. Criar a skill

Execute:

```bash
packages/harness/skills/default/create-skill/scripts/create-skill.sh <nome>
```

Para uma skill do próprio harness, use `--default <nome>`. Se o script falhar (nome inválido ou skill já existente), **PARE** e mostre o erro ao usuário.

### 4. Criar e implementar os scripts

Para cada operação determinística, execute:

```bash
packages/harness/skills/default/create-skill/scripts/create-skill.sh --script <nome> <nome-do-script>
```

Depois, implemente o script criado:

- Substitua todas as variáveis `#{...}` do template.
- Leia valores configuráveis de `config/` com `config_get`, `config_list` e `config_block` (`scripts/lib/config.sh`). Não deixe valores fixos no código.
- Reaproveite os helpers de `scripts/lib/`.
- Termine com erro e uma mensagem clara quando a entrada for inválida.
- Execute o script ao menos uma vez com uma entrada válida e uma inválida.

Pule este passo se a skill não tiver operações determinísticas.

### 5. Preencher o SKILL.md

Substitua as variáveis do template:

| Variável | Conteúdo | Opcional |
|---|---|---|
| `#{SKILL_DESCRIPTION}` | Até 1024 caracteres, seguindo [create-skill-04]. Escape aspas duplas com `\"`. | Não |
| `#{SKILL_ALLOWED_TOOLS}` | Ferramentas liberadas, separadas por espaço. Ex.: `Read Edit Bash(packages/harness/skills/custom/<nome>/scripts/*)` | Sim |
| `#{SKILL_REQUIRES}` | Regras e arquivos carregados no passo 1 da skill, relativos à raiz do harness. | Sim |
| `#{SKILL_SCRIPTS}` | Scripts executados pela skill, relativos à raiz do harness. | Sim |
| `#{SKILL_TEMPLATES}` | Templates usados pela skill, relativos à raiz do harness. | Sim |
| `#{SKILL_TITLE}` | Nome da skill em português, como ação. Ex.: `Revisar código`. | Não |
| `#{SKILL_SUMMARY}` | Uma frase com o resultado da skill. | Não |
| `#{SKILL_TERMS}` | Lista `- **Termo:** definição.` | Sim |
| `#{SKILL_RULES}` | Uma regra por linha, no formato `[<nome>-NN]: regra.` | Sim |
| `#{SKILL_CHECKLIST}` | Uma linha `- [ ] N. Passo` para cada passo. | Não |
| `#{SKILL_STEPS}` | Uma seção `### N. Passo` para cada passo, com comandos exatos e condições **PARE**. | Não |
| `#{SKILL_RESPONSE_FORMATS}` | Templates exatos das mensagens ao usuário. | Sim |

Para um campo opcional que não se aplica, remova a linha dele do front-matter. Para uma seção opcional que não se aplica, remova a seção inteira.

Se a skill carrega contexto, o passo 1 dela executa `load-extra-context.sh` com os mesmos caminhos de `requires`.

### 6. Validar e finalizar

Execute:

```bash
packages/harness/skills/default/create-skill/scripts/create-skill.sh --finalize <nome>
```

O script valida a skill e lista todos os problemas encontrados: variáveis sem preencher, `name` diferente do nome da pasta, `description` vazia, longa demais ou com tags XML, SKILL.md com mais de 500 linhas e caminhos de `metadata` que não existem. Sem problemas, ele gera os redirecionamentos e atualiza o catálogo.

Corrija os problemas e execute de novo. Se falhar pela segunda vez, **PARE** e mostre os erros ao usuário.

### 7. Responder ao usuário

Responda com:

1. O caminho do SKILL.md e dos scripts criados.
2. Os redirecionamentos gerados.
3. As decisões tomadas sem perguntar ([create-skill-07]).

## Formatos de resposta

### Formato de pergunta

```markdown
# 1. Título da pergunta

Contexto da pergunta, em até 3 frases.

| Opção | Descrição |
|---|---|
| A | Descrição em prosa da opção. |
| B | Descrição em prosa da opção. |
```

Feche a mensagem com: `Responda com o número e a letra (ex.: 1A, 2B).`
