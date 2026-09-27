---
name: refine-spec
description: Refina uma especificação (spec) em rascunho. Lê a spec, identifica ambiguidades, conflitos e lacunas, resolve com o usuário em rodadas de perguntas e gera a versão refinada em `refined/`. Use quando o usuário pedir para refinar, detalhar, esclarecer ou revisar uma spec, citar um ID como SPEC-001, ou perguntar quais specs em draft estão prontas para refinamento.
allowed-tools: Read Edit Bash(packages/harness/scripts/default/*)
metadata:
  version: "1.0.0"
  author: "Victor Batalha"
  requires: "rules/agent/template-usage.md rules/project/naming-convention.md"
  scripts: "scripts/default/context/load-extra-context.sh scripts/default/specs/list-specs.sh scripts/default/specs/create-refined-spec.sh"
  templates: "templates/refined-spec.md"
---

# Refinar spec

Transforma uma spec com status `draft` em uma spec refinada: sem ambiguidades conhecidas e com cada decisão registrada.

## Termos

- **Rascunho:** a spec original, em `draft/`. Não é editada por esta skill.
- **Refinada:** a spec gerada em `refined/` pelo script `create-refined-spec.sh`.
- **Ambiguidade:** trecho com mais de uma interpretação válida.
- **Conflito:** trechos da spec que se contradizem.
- **Lacuna:** informação necessária que não está na spec.
- **Inferência:** decisão tomada pelo agente sem perguntar ao usuário.
- **Rodada:** uma mensagem com todas as perguntas abertas, seguida da resposta do usuário.

## Regras

[refine-spec-01]: Nunca edite o rascunho. Só o script muda o status dele, no passo 8.
[refine-spec-02]: Não pergunte durante a análise. Junte todas as perguntas e envie-as numa única mensagem por rodada.
[refine-spec-03]: No máximo 2 rodadas. O que continuar em aberto vai para a seção "Pendências" da refinada.
[refine-spec-04]: Não crie requisitos. Tudo o que não está no rascunho e não veio de uma resposta do usuário é inferência: aparece na tabela de inferências e no "Registro de decisões".
[refine-spec-05]: Tabelas trazem só texto relevante para a decisão, sem links, imagens ou formatação que dificulte a leitura.
[refine-spec-06]: Todo critério de aceite da refinada precisa ser verificável.

## Checklist

Copie e marque o progresso:

```
- [ ] 1. Carregar o contexto
- [ ] 2. Identificar a spec
- [ ] 3. Ler e analisar a spec
- [ ] 4. Rodada de perguntas (máx. 2)
- [ ] 5. Criar a refinada
- [ ] 6. Preencher a refinada
- [ ] 7. Validar e finalizar
- [ ] 8. Responder ao usuário
```

## Passos

### 1. Carregar o contexto

Execute:

```bash
packages/harness/scripts/default/context/load-extra-context.sh rules/agent/template-usage.md rules/project/naming-convention.md
```

### 2. Identificar a spec

**O usuário indicou uma spec** (ID, título ou arquivo):

1. Execute `packages/harness/scripts/default/specs/list-specs.sh` e encontre o ID correspondente.
2. Não encontrou: **PARE** e informe que a spec não existe.
3. Status diferente de `draft`: **PARE** e informe o status atual. Só rascunhos são refinados.

**O usuário não indicou nenhuma spec:**

1. Execute `packages/harness/scripts/default/specs/list-specs.sh draft`.
2. Nenhuma spec: **PARE** e informe que não há specs em draft.
3. Uma spec: use-a.
4. Mais de uma: **PARE** e pergunte qual refinar, com o template `question`, com uma opção por spec (ID e título).

### 3. Ler e analisar a spec

Leia o rascunho inteiro. Classifique cada problema encontrado como ambiguidade, conflito ou lacuna e decida como tratá-lo:

| Situação | Ação |
|---|---|
| Afeta escopo, arquitetura, critérios de aceite, custo ou prazo | Pergunta |
| Há uma interpretação claramente dominante e de baixo impacto | Inferência |
| Conflito entre trechos da spec | Pergunta, sempre |

### 4. Rodada de perguntas

Sem perguntas: vá para o passo 5 e mostre as inferências na resposta final.

Com perguntas: envie numa única mensagem todas as perguntas, com o template `question`, seguidas da [tabela de inferências](#tabela-de-inferências). **PARE** e aguarde a resposta.

Ao receber as respostas:

- Se surgirem ambiguidades novas e esta for a 1ª rodada, faça a 2ª rodada.
- O que continuar sem resposta depois da 2ª rodada vira pendência ([refine-spec-03]).
- Uma inferência corrigida pelo usuário vira decisão.

### 5. Criar a refinada

Execute:

```bash
packages/harness/scripts/default/specs/create-refined-spec.sh <SPEC_ID>
```

O script imprime o caminho do arquivo criado. Se ele falhar, **PARE** e responda com o template `blocker`.

### 6. Preencher a refinada

Edite o arquivo criado:

- **`#{AGENT_DETAILED_DESCRIPTION}`:** substitua pela spec refinada, com as mesmas seções do rascunho (Contexto, Recursos, Detalhes Técnicos, Roteiro de Testes, Critérios de Aceite), já com as respostas e inferências aplicadas.
- **O que mudou:** uma linha por assunto alterado em relação ao rascunho.
- **Registro de decisões:** uma linha por resposta do usuário e uma por inferência. No motivo, indique a origem: `resposta 1B` ou `inferência A`.
- **Pendências:** uma linha por ambiguidade não resolvida. Se não houver nenhuma, escreva `Nenhuma` numa única linha.

Não deixe nenhuma variável `#{...}` no arquivo.

### 7. Validar e finalizar

Execute:

```bash
packages/harness/scripts/default/specs/create-refined-spec.sh --finalize <SPEC_ID>
```

O script falha se ainda houver variáveis `#{...}` e lista as linhas onde elas estão. Corrija essas linhas e execute de novo. Se falhar pela segunda vez, **PARE** e responda com o template `blocker`.

### 8. Responder ao usuário

Responda com o template `completion`:

- **O que foi feito:** o caminho da refinada e um resumo de 1 a 3 frases do que mudou.
- **Pendências:** o número de pendências registradas na refinada.
- Acrescente a [tabela de inferências](#tabela-de-inferências) com o cabeçalho "Inferências" depois de "O que foi feito", se ela ainda não foi mostrada numa rodada.

## Formatos de resposta

Perguntas usam o template `question`. Nas rodadas do passo 4, o contexto de cada pergunta cita o trecho da spec, e o fechamento ganha a frase: `Para corrigir uma inferência, cite a letra dela.`

### Tabela de inferências

Identifique as inferências por letras, a partir de A.

```markdown
| # | Inferência |
|---|---|
| A | O que foi inferido e com base em qual trecho da spec. |
```
