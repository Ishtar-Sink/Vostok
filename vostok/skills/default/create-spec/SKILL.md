---
name: create-spec
description: "Cria uma especificação (spec) em rascunho a partir da conversa: cria o arquivo com vostok spec new e preenche as seções do rascunho só com o que o usuário já disse, sem refinar. Use quando o usuário pedir para criar, abrir, registrar ou escrever uma spec ou um rascunho de spec sobre um assunto, ou /create-spec. Não use para refinar uma spec (skill refine-spec) nem para ajustar uma spec refinada (skill adjust-spec)."
allowed-tools: Read Edit Bash(vostok *)
metadata:
  version: "1.0.0"
  author: "Victor Batalha"
  scripts: "vostok context load, vostok spec list, vostok spec new"
  templates: "templates/spec/new.md templates/responses/question.md templates/responses/completion.md templates/responses/blocker.md"
---

# Criar spec

Registra um assunto da conversa como uma spec em rascunho, pronta para o refinamento.

## Termos

- **Rascunho:** o arquivo criado em `vostok/work/specs/draft/` pelo `vostok spec new`, com status `draft`.
- **Assunto:** o que o usuário quer construir, corrigir, estudar ou analisar.
- **Conversa:** o pedido, as mensagens anteriores e os arquivos que o usuário citou.
- **Texto do template:** as linhas em itálico `_Insira ..._` de cada seção do rascunho.

## Regras

[create-spec-01]: Só crie e preencha o rascunho. Nunca refine, valide nem aprove a spec nesta skill: o refinamento é da skill refine-spec.
[create-spec-02]: O conteúdo vem só da conversa. Organize e resuma, mas não invente requisitos, recursos, detalhes técnicos nem critérios.
[create-spec-03]: Uma spec por pedido. Assuntos independentes na mesma conversa: pergunte com o template question qual registrar ou se cria uma spec por assunto.
[create-spec-04]: Não edite o front-matter: o ID, o status draft e as datas são os do vostok spec new.
[create-spec-05]: Seção sem informação na conversa fica com o texto do template, para o refinamento tratá-la como lacuna.
[create-spec-06]: Não faça commit nem stage do rascunho ([agent/boundaries-01], [agent/boundaries-02]).

## Checklist

Copie e marque o progresso:

```
- [ ] 1. Carregar o contexto
- [ ] 2. Reunir o assunto
- [ ] 3. Conferir as specs existentes
- [ ] 4. Escolher o título
- [ ] 5. Criar o rascunho
- [ ] 6. Preencher o rascunho
- [ ] 7. Conferir o rascunho
- [ ] 8. Responder ao usuário
```

## Passos

### 1. Carregar o contexto

Execute:

```bash
vostok context load --skill create-spec
```

### 2. Reunir o assunto

Separe da conversa o que vai para cada seção: resumo (parágrafo logo abaixo do título), Contexto (o que será construído e de que tipo, dor ou problema, limitações, regras de negócio), Recursos, Detalhes Técnicos (caminhos, comandos, stack citados), Roteiro de Testes e Critérios de Aceite.

Sem assunto na conversa (por exemplo, só `/create-spec`): **PARE** e pergunte o assunto com o template `question`, com as opções "descrever o assunto numa mensagem (recomendada)" e "citar um arquivo ou issue com o assunto".

Vários assuntos independentes: **PARE** e pergunte com o template `question` ([create-spec-03]).

### 3. Conferir as specs existentes

Execute:

```bash
vostok spec list
```

Se já houver spec com o mesmo assunto, **PARE** e pergunte com o template `question`, com as opções "usar a spec existente, pela refine-spec ou pela adjust-spec (recomendada)" e "criar uma spec nova mesmo assim".

### 4. Escolher o título

Curto (até 60 caracteres), no idioma do projeto, descrevendo o resultado esperado (ex.: "Exportar relatório em PDF"), sem ID, sem aspas e sem ponto final.

### 5. Criar o rascunho

Execute:

```bash
vostok spec new "<Título>"
```

A saída `SPEC-NNN criada em <caminho>` traz o ID e o arquivo. Se o comando recusar, **PARE** e responda com o template `blocker`, com a mensagem dele.

### 6. Preencher o rascunho

Abra o arquivo e, em cada seção com informação reunida no passo 2, troque só as linhas em itálico `_Insira ..._` pelo conteúdo:

- resumo: um parágrafo;
- Contexto: prosa curta;
- Recursos: lista;
- Detalhes Técnicos: lista, citando caminhos e comandos em código;
- Roteiro de Testes: lista numerada;
- Critérios de Aceite: troque o item vazio `- [ ]` por um item `- [ ] <critério>` por critério.

Seções sem informação ficam como estão ([create-spec-05]); o front-matter não muda ([create-spec-04]).

### 7. Conferir o rascunho

Execute:

```bash
vostok spec list draft
```

Confira que o ID aparece com o status `draft`.

### 8. Responder ao usuário

Use o template `completion`.

- **O que foi feito:** o ID, o caminho do rascunho, as seções preenchidas e as que ficaram com o texto do template.
- **Pendências:** "para refinar, peça para refinar a SPEC-NNN (skill refine-spec)".

## Formatos de resposta

Respostas ao usuário usam os templates de `templates/responses/`. Formatos próprios desta skill:

Esta skill não tem formatos próprios.
