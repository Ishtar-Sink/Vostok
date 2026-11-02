---
name: customize-documents
description: "Customiza a identidade visual dos documentos finais do repositório: detecta as cores do projeto com vostok documents detect, propõe a cor principal, a secundária, o nome e o ícone, pergunta quando não encontra nada e, só com a confirmação do usuário, grava com vostok documents brand. Use quando o usuário pedir para customizar, personalizar ou aplicar a identidade, a marca, as cores ou o logo nos documentos finais, ou para voltar ao visual padrão."
allowed-tools: Read Glob Bash(vostok *)
metadata:
  version: "1.0.0"
  author: "Victor Batalha"
  scripts: "vostok context load, vostok documents detect, vostok documents brand"
  templates: "templates/responses/question.md templates/responses/blocker.md templates/responses/completion.md"
---

# Customizar os documentos

Aplica a identidade visual do repositório aos documentos finais (executivo, técnico e evidências): a cor principal e a secundária, de onde o `vostok` deriva as cores dos temas claro e escuro, o nome do produto e o ícone no topo.

## Termos

- **Identidade:** o grupo `brand` de `config/documents.yml`, com `name`, `icon`, `primary` e `secondary`. Vazio, os documentos usam a identidade padrão do harness.
- **Candidata:** uma cor que o `vostok documents detect` encontrou no repositório, com arquivo, linha, nome e valor.
- **Cor principal:** a cor da marca, usada em títulos, links e destaques; obrigatória.
- **Cor secundária:** a segunda cor do gradiente da marca; opcional. Sem ela, o `vostok` a deriva da principal.
- **Ícone:** um arquivo `.svg`, `.png` ou `.webp` de até 256 KB, com caminho relativo à raiz do projeto, embutido no topo dos documentos.

## Regras

[customize-documents-01]: Nunca grave a identidade sem a confirmação do usuário. A proposta é sempre mostrada antes do `vostok documents brand`.
[customize-documents-02]: Nunca leia arquivos de segredo (`.env*`, chaves, certificados) para achar cores ou ícones.
[customize-documents-03]: As cores de estado (sucesso, aviso, erro e informação) não são customizáveis; se o usuário pedir, explique que elas ficam fixas para manter o significado.
[customize-documents-04]: Não edite `config/documents.yml` à mão. Só o `vostok documents brand` grava a identidade e as cores derivadas.

## Checklist

Copie e marque o progresso:

```
- [ ] 1. Carregar o contexto
- [ ] 2. Identificar o pedido
- [ ] 3. Detectar as cores
- [ ] 4. Propor ou perguntar
- [ ] 5. Gravar a identidade
- [ ] 6. Responder ao usuário
```

## Passos

### 1. Carregar o contexto

Execute:

```bash
vostok context load --skill customize-documents
```

### 2. Identificar o pedido

**Voltar ao padrão:** confirme com o template `question` que a identidade do repositório será apagada e os documentos voltarão ao visual padrão do harness. **PARE** e aguarde. Confirmado, execute:

```bash
vostok documents brand --reset
```

e vá para o passo 6. Recusado, encerre sem gravar nada.

**Customizar:** siga para o passo 3. Se o usuário já informou as cores, o nome ou o ícone na mensagem, use-os na proposta do passo 4 no lugar das candidatas.

### 3. Detectar as cores

Execute:

```bash
vostok documents detect
```

O comando lista até 10 cores candidatas, primeiro as de nome com `primary`, `brand` ou `accent`, ou imprime `Nenhuma cor encontrada no repositório.`.

Procure também um ícone com Glob, nesta ordem, ignorando `node_modules`, `target`, `dist`, `build` e `vostok`: `**/favicon.svg`, `**/logo.svg`, `**/favicon.png`, `**/logo.png`, `**/favicon.webp`, `**/logo.webp`. Use o primeiro encontrado.

### 4. Propor ou perguntar

**Com candidatas:** monte uma única pergunta com o template `question`, propondo:

- **Cor principal:** a primeira candidata, com o arquivo e a linha de onde veio.
- **Cor secundária:** a segunda candidata de marca, se houver; senão, "derivada da principal".
- **Nome:** o nome da pasta do repositório.
- **Ícone:** o arquivo encontrado no passo 3, ou "nenhum (mantém o quadrado de gradiente)".

Mostre as demais candidatas numa tabela para o usuário trocar alguma, e ofereça as opções "Aplicar a proposta", "Trocar algum item" e "Cancelar".

**Sem candidatas:** pergunte com o template `question` a cor principal (obrigatória, em `#rgb` ou `#rrggbb`), a secundária, o nome (proponha o nome da pasta) e o ícone (proponha o encontrado no passo 3), os três opcionais.

**PARE** e aguarde a resposta. Cancelado ou recusado: encerre sem gravar nada ([customize-documents-01]). Item trocado: use o valor do usuário.

### 5. Gravar a identidade

Execute, só com as opções confirmadas:

```bash
vostok documents brand --primary <cor> [--secondary <cor>] [--name <nome>] [--icon <caminho>]
```

O comando valida as cores e o ícone, deriva as cores dos dois temas, ajusta as que não passam no contraste de 4,5:1 e imprime cada cor alterada (`<tema>.<token>: <antes> -> <depois>`) e os avisos de ajuste (`aviso: ...`). Se ele recusar, **PARE** e responda com o template `blocker`, com a mensagem do comando e o que o usuário pode trocar.

### 6. Responder ao usuário

Responda com o template `completion`:

- **O que foi feito:** a identidade gravada (cores, nome e ícone) ou a volta ao padrão.
- **Verificação:** o comando executado e um resumo das cores alteradas por tema, sem colar a saída inteira.
- **Pendências:** cada aviso de contraste, com a cor informada e a ajustada, e a observação de que os documentos já gerados não mudam: a identidade vale para as próximas finalizações.
- **Próximos passos:** oferecer a geração de uma prévia com uma spec arquivada, se houver uma em `vostok/work/.archive/`.

## Formatos de resposta

Respostas ao usuário usam os templates de `templates/responses/`. Formatos próprios desta skill:

### Tabela de candidatas

```markdown
| Cor | Nome | Onde |
|---|---|---|
| #0f766e | --color-primary | src/theme.css:3 |
```
