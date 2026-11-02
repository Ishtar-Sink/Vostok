---
id: SPEC-026
title: "Documentos finais com a identidade visual do repositório"
status: planned
author: "Victor Batalha"
created_at: 2026-11-01T22:49:30Z
updated_at: 2026-11-02T00:28:31Z
tags: []
type: feature
source: "draft/SPEC-026-documentos-finais-com-a-identidade-visual-do-repositorio.md"
refined_at: 2026-11-02T00:10:52Z
adjustments: 0
adjusted_at: null
content_hash: e719e3222d4a543aa463da5492e15638ad792bfde8a7cf186e056c5f2bfb7d9a
approved_at: 2026-11-02T00:17:58Z
approved_hash: e719e3222d4a543aa463da5492e15638ad792bfde8a7cf186e056c5f2bfb7d9a
---

# Documentos finais com a identidade visual do repositório - Especificação Refinada

Uma skill nova, `customize-documents`, detecta ou pergunta a cor principal, a secundária, o nome e o ícone do produto, e o `vostok` deriva deles os tokens dos dois temas, para que os documentos finais saiam com a identidade visual do repositório.

## Objetivo

Hoje todo repositório que usa o harness gera os documentos finais com as cores, o nome e a marca do Ishtar Sink. O resultado esperado é que cada repositório informe, uma vez, a cor principal (e opcionalmente a secundária), o nome e o ícone do produto, e que os documentos executivo, técnico e de evidências passem a sair com essa identidade, mantendo o contraste de leitura nos dois temas.

## Escopo

### Dentro do escopo

- Skill `customize-documents`, que detecta as cores no repositório, propõe, confirma com o usuário ou pergunta quando não encontra nada.
- Comando `vostok documents detect`, que lista as cores candidatas do repositório.
- Comando `vostok documents brand`, que grava a cor principal, a secundária, o nome e o ícone e deriva os tokens de cor dos dois temas.
- Nome do produto no topo dos documentos e ícone da marca embutido no lugar do quadrado de gradiente.
- Ajuste automático de contraste das cores derivadas, com aviso.
- Volta à identidade padrão com `vostok documents brand --reset`.
- Coluna de IDs do mapa de cobertura sem quebra de linha.

### Fora do escopo

- Fontes: Inter e Outfit continuam embutidas; quem quiser muda as pilhas de fonte à mão no `documents.yml`.
- Cores de estado (sucesso, aviso, erro, informação) e cores de diff.
- Regenerar documentos já publicados em `docs/specs/`.
- Remover as chaves antigas (`color_*`, `font.body`) do `documents.yml` instalado.
- Customizar a estrutura ou as seções dos documentos.

## Requisitos funcionais

| ID | Requisito |
|---|---|
| RF-01 | A skill `customize-documents` deve rodar `vostok documents detect`, propor a cor principal e a secundária encontradas e pedir a confirmação ou a troca ao usuário antes de gravar. |
| RF-02 | O comando `vostok documents detect` deve listar as cores candidatas do repositório (propriedades CSS, variáveis SCSS e Less, tema do Tailwind) com arquivo, linha, nome e valor, primeiro as de nome com `primary`, `brand` ou `accent`, até 10 itens. |
| RF-03 | Sem cores candidatas, a skill deve perguntar ao usuário a cor principal e a secundária opcional. |
| RF-04 | O comando `vostok documents brand --primary <cor>` deve gravar `brand.primary` e derivar os tokens de marca e os neutros tingidos dos grupos `light` e `dark` do `documents.yml`. |
| RF-05 | O comando `vostok documents brand` deve aceitar `--secondary <cor>` opcional e, sem ela, derivar a secundária da principal. |
| RF-06 | O comando `vostok documents brand --name <nome>` deve gravar `brand.name`, e o build deve mostrar esse nome no topo dos três documentos no lugar de "Vostok". |
| RF-07 | O comando `vostok documents brand --icon <caminho>` deve gravar `brand.icon`, e o build deve embutir o ícone em base64 no lugar do quadrado de gradiente do topo. |
| RF-08 | O build deve gerar os documentos com o nome "Vostok", o quadrado de gradiente e os tokens atuais quando `brand` não estiver configurado. |
| RF-09 | O comando `vostok documents brand` deve ajustar a claridade de uma cor derivada que não passe no contraste AA e avisar a cor informada e a ajustada. |
| RF-10 | O comando `vostok documents brand --reset` deve voltar os tokens de marca e os neutros aos valores padrão do harness e limpar o grupo `brand`. |
| RF-11 | O comando `vostok documents brand` deve imprimir os tokens gravados em cada tema, com o valor anterior e o novo. |
| RF-12 | O template de evidências deve mostrar os IDs do mapa de cobertura (ex.: `RNF-01`) sem quebra de linha. |

## Requisitos não funcionais

| ID | Requisito | Como verificar |
|---|---|---|
| RNF-01 | Nenhuma cor fixa nos templates de `templates/finalize/`; todas vêm do `documents.yml`. | O teste `visual_colors_come_only_from_documents_yml` continua passando. |
| RNF-02 | Documentos sem requisição externa, com o ícone embutido. | O teste `visual_documents_have_no_external_requests` passa com um ícone configurado. |
| RNF-03 | Texto de leitura e cor de marca usada em texto com contraste mínimo de 4,5:1 nos dois temas. | Teste que calcula a razão de contraste dos pares de texto depois de `brand` com cores claras, escuras e saturadas. |
| RNF-04 | Projetos sem `brand` configurado geram os mesmos documentos de hoje. | Os casos golden passam sem mudança nas fixtures. |
| RNF-05 | Ícone de até 256 KB, em SVG, PNG ou WebP. | Teste com ícone acima do limite e com extensão não suportada. |

## Regras de negócio

| ID | Regra |
|---|---|
| RN-01 | As cores de estado e de diff nunca mudam com a identidade do repositório. |
| RN-02 | A identidade fica no `vostok/config/documents.yml` do projeto e continua valendo depois do `vostok update`, que só acrescenta chaves. |
| RN-03 | Só o `vostok documents brand` reescreve os tokens derivados; rodá-lo de novo sobrescreve edições manuais desses tokens. |
| RN-04 | O rodapé dos documentos continua citando o Vostok como gerador. |
| RN-05 | O `vostok documents detect` ignora `.git`, `node_modules`, `target`, `vostok/` e arquivos de segredo (`.env*`, chaves e certificados). |
| RN-06 | A skill nunca grava a identidade sem a confirmação do usuário. |

## Fluxos

### Fluxo principal

1. O usuário pede para customizar os documentos (`customize-documents`).
2. A skill roda `vostok documents detect` e mostra as cores candidatas.
3. A skill propõe a principal e a secundária, o nome (padrão: nome da pasta do repositório) e um ícone encontrado (favicon ou logo), e pergunta.
4. O usuário confirma ou troca.
5. A skill roda `vostok documents brand --primary ... [--secondary ...] --name ... [--icon ...]`.
6. O comando grava o grupo `brand`, deriva os tokens, ajusta o contraste e imprime o antes e depois.
7. A skill oferece gerar uma prévia com uma spec arquivada e responde com o resumo.

### Fluxos alternativos

- FA-01 (passo 2): nenhuma cor candidata; a skill pergunta a cor principal e a secundária opcional (RF-03).
- FA-02 (passo 4): o usuário recusa tudo; a skill para sem gravar (RN-06).
- FA-03 (passo 1): o usuário pede para voltar ao padrão; a skill roda `vostok documents brand --reset` depois de confirmar (RF-10).

## Casos de borda

| ID | Situação | Comportamento esperado | Requisitos |
|---|---|---|---|
| CB-01 | Cor em formato inválido (`#12`, `azul`) | O comando recusa, explica os formatos aceitos (`#rgb`, `#rrggbb`) e não grava nada | RF-04, RF-05 |
| CB-02 | Caminho do ícone inexistente | O comando recusa com o caminho; no build, um `brand.icon` que deixou de existir é erro com o caminho | RF-07 |
| CB-03 | Ícone acima de 256 KB ou de extensão não suportada | O comando recusa e informa o limite e as extensões | RF-07 |
| CB-04 | Cor principal muito clara (amarelo) ou muito escura | A cor em texto é ajustada até passar 4,5:1, e o aviso mostra a informada e a ajustada | RF-09 |
| CB-05 | Secundária igual à principal | Aceita; os gradientes ficam de uma cor só | RF-05 |
| CB-06 | `documents.yml` sem a chave `bg` (harness não atualizado) | O comando recusa e pede `vostok update` | RF-04 |
| CB-07 | Mais de 10 cores candidatas | Lista as 10 mais relevantes e informa quantas ficaram de fora | RF-02 |
| CB-08 | Nome com caracteres HTML (`<`, `&`) | O build escapa o nome | RF-06 |

## Contratos e interfaces

Comandos:

```text
vostok documents detect
vostok documents brand --primary <cor> [--secondary <cor>] [--name <nome>] [--icon <caminho>]
vostok documents brand --reset
```

Saída do `detect`, uma cor por linha:

```text
<arquivo>:<linha>  <nome>  <cor>
```

Grupo novo em `config/documents.yml` (o `vostok update` o acrescenta vazio aos projetos):

```yaml
# Identidade do repositório, gravada pelo `vostok documents brand`. Vazio usa a do harness.
brand:
  name: ""
  icon: ""        # caminho relativo à raiz do projeto: .svg, .png ou .webp, até 256 KB
  primary: ""
  secondary: ""
```

Tokens reescritos pelo `brand` em `light` e `dark`: `primary`, `primary_hover`, `primary_soft`, `primary_on`, `accent`, `accent_soft`, `brand_from`, `brand_to`, `calm_from`, `calm_to`, `bg`, `bg_elevated`, `surface_muted`, `surface_hover`, `border`, `border_strong`, `text`, `text_muted`, `text_faint`, `code_bg`, `shadow_sm`, `shadow_md`.

Variáveis novas dos templates: `#{BRAND_NAME}` e `#{BRAND_MARK}` (o `<img>` do ícone ou o `<i>` do gradiente).

## Componentes afetados

| Caminho | Ação | Motivo |
|---|---|---|
| `apps/cli/src/commands/documents.rs` | criar | Comandos `detect` e `brand`, derivação de cores e contraste |
| `apps/cli/src/commands/mod.rs` | alterar | Registrar o módulo `documents` |
| `apps/cli/src/cli.rs` | alterar | Registrar o subcomando `documents` |
| `apps/cli/src/commands/finalize.rs` | alterar | `BRAND_NAME` e `BRAND_MARK` no build, ícone embutido |
| `packages/harness/config/documents.yml` | alterar | Grupo `brand` |
| `packages/harness/templates/finalize/executive.html` | alterar | Nome e marca do topo por variável |
| `packages/harness/templates/finalize/technical.html` | alterar | Nome e marca do topo por variável |
| `packages/harness/templates/finalize/evidence.html` | alterar | Nome e marca do topo por variável e coluna de IDs sem quebra |
| `packages/harness/skills/default/customize-documents/SKILL.md` | criar | Skill de customização |
| `packages/harness/context/skill-catalog.md` | alterar | Entrada da skill nova |
| `packages/harness/config/context.yml` | alterar | Contexto da skill nova (`config/documents.yml`) |
| `apps/cli/tests/documents.rs` | criar | Testes de `detect`, `brand` e contraste |
| `apps/cli/tests/finalize.rs` | alterar | Testes de nome, ícone e coluna de IDs no build |

## Premissas e dependências

- A SPEC-025 está concluída: tokens de 2026-10 em `documents.yml`, templates no modelo C com o índice do B e cores só por variável.
- O `vostok update` acrescenta o grupo `brand` vazio sem tocar no resto (`merge_keys` em `apps/cli/src/commands/update.rs`).
- Os redirecionamentos da skill nova para Claude Code, Codex e Cursor são gerados pelo `vostok update` a partir das skills padrão.

## Riscos

| Risco | Impacto | Mitigação |
|---|---|---|
| Cor derivada destoar da marca depois do ajuste de contraste | médio | O aviso mostra a cor informada e a ajustada; o usuário pode trocar ou ajustar o token à mão |
| Detecção trazer cores irrelevantes (cinzas, cores de terceiros) | médio | Ordenar por nome e mostrar arquivo e linha; a escolha final é sempre do usuário |
| Neutros tingidos ficarem fortes com cores muito saturadas | baixo | Manter a claridade e reduzir a saturação dos neutros em relação aos valores atuais |
| Edições manuais de tokens perdidas ao rodar `brand` de novo | baixo | O comando lista o antes e o depois de cada token (RF-11) |

## Casos de teste

| ID | Cenário | Passos | Resultado esperado | Requisitos |
|---|---|---|---|---|
| CT-01 | Detectar cores | Repositório com `--color-primary` num CSS e `colors.brand` num `tailwind.config.js`; rodar `detect` | Lista os dois com arquivo, linha, nome e valor, `primary` e `brand` primeiro | RF-02 |
| CT-02 | Detecção ignora pastas e segredos | Cores em `node_modules/`, `.env` e `vostok/`; rodar `detect` | Nenhuma delas listada | RF-02 |
| CT-03 | Gravar a cor principal | `brand --primary "#0f766e"` | `brand.primary` gravado e os tokens de marca dos dois temas mudam; estados iguais | RF-04 |
| CT-04 | Secundária derivada e informada | `brand --primary` sem e com `--secondary` | Sem: `accent` derivado da principal; com: `accent` a partir da secundária | RF-05 |
| CT-05 | Nome no topo | `brand --name "Atria"` e build | Os três documentos mostram "Atria" no topo e "Vostok" no rodapé | RF-06 |
| CT-06 | Ícone embutido | `brand --icon logo.svg` e build | `<img>` com `data:image/svg+xml;base64` no topo dos três documentos | RF-07 |
| CT-07 | Sem identidade | Build com o grupo `brand` vazio | Documentos iguais aos de hoje; goldens sem mudança | RF-08 |
| CT-08 | Ajuste de contraste | `brand --primary "#facc15"` | Aviso com a cor informada e a ajustada; pares de texto ≥ 4,5:1 nos dois temas | RF-09 |
| CT-09 | Voltar ao padrão | `brand --reset` depois de um `brand` | Tokens iguais aos padrão e grupo `brand` vazio | RF-10 |
| CT-10 | Resumo do antes e depois | `brand --primary` | Saída lista cada token por tema com valor anterior e novo | RF-11 |
| CT-11 | Coluna de IDs | Build de evidências com `RNF-01` na cobertura e print em 1440 px | `RNF-01` numa linha só | RF-12 |
| CT-12 | Skill propõe e confirma | Seguir a skill num repositório com cores | Mostra as candidatas e só grava depois da confirmação | RF-01 |
| CT-13 | Skill sem cores | Seguir a skill num repositório sem cores | Pergunta a principal e a secundária opcional | RF-03 |
| CT-14 | Entradas inválidas | `brand --primary azul`, ícone inexistente, ícone de 300 KB | Recusas com a mensagem e nada gravado | RF-04, RF-07 |

## Critérios de aceite

- [ ] CA-01: A skill detecta, propõe e só grava a identidade depois da confirmação; sem cores, pergunta (RF-01, RF-03).
- [ ] CA-02: `vostok documents detect` lista até 10 cores candidatas, sem pastas ignoradas nem segredos (RF-02).
- [ ] CA-03: `vostok documents brand` grava a principal, a secundária, o nome e o ícone e deriva os tokens dos dois temas, imprimindo o antes e o depois (RF-04, RF-05, RF-06, RF-07, RF-11).
- [ ] CA-04: Os três documentos saem com a cor, o nome e o ícone configurados, e com a identidade atual quando nada foi configurado (RF-06, RF-07, RF-08).
- [ ] CA-05: Os pares de texto passam 4,5:1 nos dois temas com cores claras, escuras e saturadas, e o ajuste é avisado (RF-09).
- [ ] CA-06: `--reset` volta à identidade padrão (RF-10).
- [ ] CA-07: Os IDs do mapa de cobertura ficam numa linha só (RF-12).

## Referências

| Tipo | Referência | Uso |
|---|---|---|
| arquivo | `vostok/work/specs/draft/SPEC-026-documentos-finais-com-a-identidade-visual-do-repositorio.md` | Rascunho |
| arquivo | `packages/harness/config/documents.yml` | Tokens atuais e formato do YAML |
| arquivo | `apps/cli/src/commands/finalize.rs` | `css_tokens`, `legacy_css_tokens` e build |
| arquivo | `apps/cli/src/commands/update.rs` | `merge_keys`, que só acrescenta chaves |
| arquivo | `packages/harness/templates/finalize/executive.html` | Marca "Vostok" fixa no topo e no rodapé |
| arquivo | `packages/harness/config/skills.yml` | Catálogo e pastas das skills |
| arquivo | `packages/harness/config/context.yml` | Contexto por skill |
| documentação | `docs/estudos/documentos-finais.md` | Regras de contraste e tokens do estudo da SPEC-025 |
| documentação | `docs/specs/SPEC-025/tecnico.html` | Defeito da coluna de IDs no mapa de cobertura |
| memória | DEC-001 | O Vostok continua como gerador citado no rodapé |

## O que mudou

| # | Assunto | Estado no Original | Estado Refinado |
|---|---|---|---|
| 1 | Entrada da skill | Cor principal ou paleta | Cor principal e secundária opcional; o resto é derivado |
| 2 | Origem das cores | Informada pelo usuário | Detectada no repositório e confirmada; sem nada encontrado, perguntada |
| 3 | Marca | Não citada | Nome e ícone configuráveis no topo; rodapé cita o Vostok |
| 4 | Fontes | Não citada | Fora do escopo |
| 5 | Contraste | Não citado | Ajuste automático com aviso |
| 6 | Mapa de cobertura | Não citado | Coluna de IDs sem quebra |

## Registro de decisões

| # | Decisão | Alternativa descartada | Motivo |
|---|---|---|---|
| 1 | Principal e secundária opcional, com derivação | Só a principal; paleta completa | resposta 1A |
| 2 | Detectar e propor; sem nada, perguntar | Só perguntar | resposta 2 (texto do usuário) |
| 3 | Nome configurável no topo | Nome e logo; manter Vostok | resposta 3A |
| 4 | Fontes fora do escopo, ícone da marca dentro | Fontes do sistema; fonte própria | resposta 4 (texto do usuário) |
| 5 | Identidade no `documents.yml` do projeto | Arquivo separado | inferência A |
| 6 | Skill `customize-documents` e comando `vostok documents` | Só a skill editando o YAML | inferência B |
| 7 | Ajuste de contraste automático com aviso | Recusar a cor | inferência C |
| 8 | Estados e diffs fixos | Derivar tudo | inferência D |
| 9 | Só documentos novos usam a identidade | Regenerar os publicados | inferência E |
| 10 | Corrigir a coluna de IDs nesta spec | Spec própria | inferência F |
| 11 | Chaves antigas ficam | Limpar no update | inferência G |
| 12 | Ícone SVG, PNG ou WebP de até 256 KB, embutido; favicon ou logo propostos pela skill | Só SVG; link externo | inferência H, da resposta 4 |
| 13 | Neutros tingidos com o matiz da principal, mantendo a claridade atual | Neutros fixos lavanda | inferência I |
| 14 | Secundária derivada da principal quando ausente | Gradiente de uma cor só | inferência J |

## Pendências

| # | Pendência | Opções em aberto |
|---|---|---|
| - | Nenhuma | - |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
