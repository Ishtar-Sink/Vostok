---
id: SPEC-025
title: "Documentos finais com visual e conteúdo profissionais"
status: implemented
author: "Victor Batalha"
created_at: 2026-11-01T22:16:17Z
updated_at: 2026-11-01T23:44:22Z
tags: []
type: improvement
source: "draft/SPEC-025-documentos-finais-com-visual-e-conteudo-profissionais.md"
refined_at: 2026-11-01T22:22:06Z
adjustments: 0
adjusted_at: null
content_hash: 92166dbda0fa8d13f83f16979477a8800c16101403b1f3aa86267f2232e3045c
approved_at: 2026-11-01T22:25:10Z
approved_hash: 92166dbda0fa8d13f83f16979477a8800c16101403b1f3aa86267f2232e3045c
---

# Documentos finais com visual e conteúdo profissionais - Especificação Refinada

Refaz os documentos finais de uma spec (executivo, técnico e evidências): primeiro o visual, com um estudo e três protótipos em HTML na identidade do Ishtar Sink (base Atria) para o usuário escolher o modelo, e depois o conteúdo, com um estudo de como montar e escrever cada documento para leitura humana, e a implementação do modelo e do conteúdo escolhidos na finalização.

## Objetivo

Os documentos finais hoje trazem conteúdo pífio, e mesmo o executivo não cumpre o seu papel. O resultado esperado são documentos com visual profissional e moderno, na identidade do ecossistema, e conteúdo equilibrado: o executivo explica a entrega para quem não é técnico, e o técnico explica o que foi feito, por que e como, servindo de KT para a sustentação.

## Estado atual

A finalização (`vostok finalize build`, em `apps/cli/src/commands/finalize.rs`) gera `executivo.html`, `tecnico.html` e `evidencias.html` a partir de `packages/harness/templates/finalize/executive.html`, `technical.html` e `evidence.html`, com o conteúdo de `content.md` (template em `packages/harness/templates/finalize/content.md`, escrito pela skill `finalize-spec`). As cores vêm de `packages/harness/config/documents.yml`, uma paleta roxa derivada de `docs/mapa-vostok.html`, com tema escuro como padrão e cabeçalho em faixa com gradiente; o executivo tem abas. O conteúdo segue seções fixas (Panorama, Indicadores, Entregas etc.) com muitos campos curtos em lista, e o técnico mostra os títulos `##` do técnico de cada tarefa como texto literal (`markdown_html` não converte títulos). O executivo e o técnico publicados ficam em `docs/specs/<SPEC_ID>/`.

## Escopo

### Dentro do escopo

- Estudo do visual na identidade do Ishtar Sink, com o Atria como referência, antes dos protótipos.
- Três protótipos em HTML, cada um com o executivo e o técnico, um com abas e dois sem, montados com uma spec real reescrita no modelo de conteúdo proposto.
- Escolha do modelo visual pelo usuário, registrada no estudo.
- Estudo de conteúdo: o que cada documento traz e o formato de melhor leitura (prosa, listas, tabelas).
- Implementação do modelo escolhido nos três documentos (executivo, técnico e evidências) e do conteúdo novo no `content.md`, na validação e na skill `finalize-spec`.
- Correção dos títulos literais no técnico de cada tarefa.

### Fora do escopo

- Refazer os documentos já publicados em `docs/specs/` (SPEC-001 a SPEC-024): o modelo novo vale a partir da próxima finalização (resposta 2A).
- Mudar o pacote de tema dos produtos ou o documento de identidade da organização.
- Mudar os documentos da tarefa (`executivo.md` e `tecnico.md` da pasta da tarefa), além de como a finalização os lê.
- Publicar o documento de evidências em `docs/specs/`.

## Requisitos funcionais

| ID | Requisito |
|---|---|
| RF-01 | O sistema deve ter um estudo do visual dos documentos, com base na identidade do Ishtar Sink e no Atria, antes dos protótipos. |
| RF-02 | O sistema deve ter três protótipos de modelo visual em HTML, um com abas e dois sem, cada um com o executivo e o técnico. |
| RF-03 | Os protótipos devem usar a mesma spec real, com o texto reescrito no modelo de conteúdo proposto. |
| RF-04 | O estudo deve registrar o modelo visual escolhido pelo usuário e o motivo. |
| RF-05 | O sistema deve ter um estudo de conteúdo com as seções de cada documento e a regra de quando usar prosa, lista e tabela. |
| RF-06 | O executivo deve explicar, para quem não é técnico, o objetivo, o que foi entregue, o valor e o impacto, os ganhos secundários e os próximos passos quando se aplicam. |
| RF-07 | O técnico deve explicar o que foi feito, por que e como. |
| RF-08 | O técnico deve servir de KT, com como operar e diagnosticar, o mapa do código e os limites e dívidas, nas seções que o estudo de conteúdo definir. |
| RF-09 | O documento de evidências deve seguir o modelo visual escolhido e o conteúdo definido pelo estudo. |
| RF-10 | O `vostok finalize build` deve gerar os três documentos no modelo escolhido. |
| RF-11 | O template `content.md`, o `vostok finalize validate` e a skill `finalize-spec` devem seguir as seções e as regras de escrita do estudo de conteúdo. |
| RF-12 | O técnico deve mostrar os títulos do técnico de cada tarefa como títulos, não como texto literal. |

## Requisitos não funcionais

| ID | Requisito | Como verificar |
|---|---|---|
| RNF-01 | Os documentos usam a paleta e a tipografia do documento de identidade da organização (tokens de 2026-10, Outfit e Inter), com tema claro como padrão e escuro como alternativa. | Comparar os tokens do CSS com `identidade-ecossistema.md` |
| RNF-02 | Os documentos são autossuficientes: as fontes vão embutidas, sem CDN nem requisição externa. | Abrir o HTML sem rede e ver as fontes e o layout iguais |
| RNF-03 | Texto de leitura passa em contraste AA (4,5:1) nos dois temas. | Conferir os pares usados contra a tabela de contraste medido da organização |
| RNF-04 | Os documentos funcionam do celular (cerca de 400 px) ao desktop, sem rolagem horizontal da página. | Abrir em 400 px e em 1280 px |
| RNF-05 | O conteúdo é equilibrado: o estudo de conteúdo define o tamanho esperado de cada seção. | Conferir um documento gerado contra os limites do estudo |

## Regras de negócio

| ID | Regra |
|---|---|
| RN-01 | O executivo não usa caminhos de arquivo, comandos, nomes de funções nem IDs de requisito. |
| RN-02 | Nenhum número entra nos documentos sem vir das fontes da finalização (contado ou medido). |
| RN-03 | A implementação só começa depois de o usuário escolher o modelo visual e aprovar o estudo de conteúdo. |
| RN-04 | A paleta não é redeclarada por conta própria: os tokens citam o documento da organização como origem. |

## Fluxos

### Fluxo principal

1. O estudo visual analisa a identidade do Ishtar Sink, o Atria e os documentos atuais e propõe três direções.
2. O estudo de conteúdo propõe as seções de cada documento e as regras de escrita.
3. Os três protótipos são montados com a mesma spec real reescrita no modelo de conteúdo proposto.
4. O usuário escolhe o modelo visual e aprova o estudo de conteúdo; a escolha fica registrada no estudo.
5. A finalização passa a gerar os três documentos no modelo escolhido, com o `content.md`, a validação e a skill no conteúdo novo.

### Fluxos alternativos

- FA-01 (passo 4): o usuário não aprova nenhum protótipo; os protótipos são ajustados com o retorno dele e o passo 4 se repete.
- FA-02 (passo 4): o usuário combina elementos de mais de um protótipo; o modelo escolhido registra a combinação.

## Casos de borda

| ID | Situação | Comportamento esperado | Requisitos |
|---|---|---|---|
| CB-01 | Spec sem números comprováveis nas fontes | O executivo diz que não se aplica, sem inventar indicador | RF-06 |
| CB-02 | Técnico de tarefa com títulos, código e tabelas | Renderizado como título, código e tabela | RF-12 |
| CB-03 | Spec com uma única tarefa e sem evidência | Os três documentos são gerados sem seção vazia quebrada | RF-09, RF-10 |
| CB-04 | Documento aberto sem rede | Fontes e layout iguais | RF-10 |
| CB-05 | Spec com tarefas no formato de arquivo único | Os documentos leem os resumos do formato antigo | RF-10 |

## Contratos e interfaces

O estudo e os protótipos ficam em:

```text
docs/estudos/documentos-finais.md
docs/estudos/documentos-finais/<modelo>/executivo.html
docs/estudos/documentos-finais/<modelo>/tecnico.html
```

As seções do `content.md`, a validação do `vostok finalize validate` e os tokens de `config/documents.yml` são definidos pelo estudo e entram nas tarefas de implementação.

## Componentes afetados

| Caminho | Ação | Motivo |
|---|---|---|
| `docs/estudos/documentos-finais.md` | criar | Estudos visual e de conteúdo, com a escolha do modelo |
| `docs/estudos/documentos-finais` | criar | Protótipos dos três modelos |
| `packages/harness/templates/finalize/executive.html` | alterar | Modelo escolhido no executivo |
| `packages/harness/templates/finalize/technical.html` | alterar | Modelo escolhido no técnico |
| `packages/harness/templates/finalize/evidence.html` | alterar | Modelo escolhido nas evidências |
| `packages/harness/templates/finalize/evidence-item.html` | alterar | Item de evidência no modelo novo |
| `packages/harness/templates/finalize/content.md` | alterar | Seções do conteúdo novo |
| `packages/harness/config/documents.yml` | alterar | Tokens da organização e tema claro como padrão |
| `apps/cli/src/commands/finalize.rs` | alterar | Geração no modelo novo, validação e títulos do técnico das tarefas |
| `apps/cli/tests/finalize.rs` | alterar | Testes do modelo e da validação novos |
| `packages/harness/skills/default/finalize-spec/SKILL.md` | alterar | Regras de escrita do conteúdo novo |

## Premissas e dependências

- O documento de identidade da organização (`Ishtar-Sink/.github`, `docs/identidade-ecossistema.md`) é a fonte da paleta e da tipografia.
- O Atria (`packages/theme`, `prototype/atria-prototype.html`, `docs/decisoes-do-prototipo.md`) é a referência de aplicação da identidade.
- As fontes Outfit e Inter têm licença OFL, que permite embuti-las.
- A 0.2.8 está publicada, com o executivo e o técnico de cada tarefa lidos pela finalização.

## Riscos

| Risco | Impacto | Mitigação |
|---|---|---|
| Fontes embutidas deixarem o HTML pesado | médio | Subconjunto das fontes com os caracteres do português |
| Estudo de conteúdo virar regra que o agente não consegue seguir | alto | Regras verificáveis pelo `finalize validate` sempre que possível e exemplos no template |
| Goldens da finalização mudarem em massa (28 casos de `apps/cli/tests/fixtures/golden/finalize` rodam o `build`) | médio | A tarefa de implementação os inclui em Mudar e regrava só esses casos, revisando o diff de um deles |
| Os três modelos ficarem parecidos demais | médio | O estudo visual define três direções distintas antes de montar |

## Casos de teste

| ID | Cenário | Passos | Resultado esperado | Requisitos |
|---|---|---|---|---|
| CT-01 | Estudo visual | Ler `docs/estudos/documentos-finais.md` | Análise da identidade, do Atria e dos documentos atuais, com três direções | RF-01 |
| CT-02 | Protótipos | Abrir os seis HTML | Três modelos, um com abas, cada um com executivo e técnico da mesma spec real reescrita | RF-02, RF-03 |
| CT-03 | Escolha | Ler o estudo depois da decisão | Modelo escolhido e motivo registrados | RF-04 |
| CT-04 | Estudo de conteúdo | Ler o estudo | Seções de cada documento, regra de prosa, lista e tabela e o KT do técnico | RF-05, RF-08 |
| CT-05 | Executivo gerado | Finalizar uma spec de exemplo | O executivo traz objetivo, entregas, valor e impacto, ganhos secundários e próximos passos, sem caminhos nem comandos | RF-06, RF-10 |
| CT-06 | Técnico gerado | Finalizar uma spec de exemplo | O técnico traz o quê, por quê, como, operação e diagnóstico, mapa do código e limites | RF-07, RF-08, RF-10 |
| CT-07 | Evidências geradas | Finalizar uma spec de exemplo | O documento de evidências no modelo e no conteúdo novos | RF-09, RF-10 |
| CT-08 | Validação | `vostok finalize validate` com uma seção nova vazia | Recusa citando a seção | RF-11 |
| CT-09 | Títulos do técnico da tarefa | Técnico de tarefa com `## ` e código | Títulos e código renderizados no `tecnico.html` | RF-12 |
| CT-10 | Identidade, contraste e celular | Abrir os documentos nos dois temas, sem rede, em 400 px | Tokens da organização, AA, fontes embutidas e sem rolagem horizontal | RF-10 |

## Critérios de aceite

- [ ] CA-01: O estudo visual existe e propõe três direções a partir da identidade do Ishtar Sink e do Atria (RF-01).
- [ ] CA-02: Os três protótipos, um com abas, mostram o executivo e o técnico da mesma spec real reescrita (RF-02, RF-03).
- [ ] CA-03: O modelo escolhido e o motivo estão registrados no estudo (RF-04).
- [ ] CA-04: O estudo de conteúdo define as seções de cada documento, a regra de prosa, lista e tabela e o KT do técnico (RF-05, RF-08).
- [ ] CA-05: Uma spec finalizada gera o executivo, o técnico e as evidências no modelo e no conteúdo novos (RF-06, RF-07, RF-09, RF-10).
- [ ] CA-06: O template, a validação e a skill de finalização seguem o estudo de conteúdo (RF-11).
- [ ] CA-07: Os títulos do técnico de cada tarefa aparecem como títulos (RF-12).

## Referências

| Tipo | Referência | Uso |
|---|---|---|
| documentação | `Ishtar-Sink/.github`, `docs/identidade-ecossistema.md` | Paleta de 2026-10, tipografia, tema claro padrão e fontes sem CDN |
| arquivo | Atria, `packages/theme/src/tokens.css` e `fonts.css` | Aplicação dos tokens e das fontes OFL |
| arquivo | Atria, `prototype/atria-prototype.html` | HTML único com as fontes embutidas |
| arquivo | Atria, `docs/decisoes-do-prototipo.md` | Decisões de layout (largura, cabeçalho, estados) |
| arquivo | `packages/harness/templates/finalize/` | Modelos atuais dos três documentos |
| arquivo | `packages/harness/config/documents.yml` | Tokens atuais, tema escuro padrão |
| arquivo | `apps/cli/src/commands/finalize.rs` (`markdown_html`, `build`) | Geração e o defeito dos títulos |
| arquivo | `docs/estudos/tarefa-como-pasta.md` | Precedente de estudo em `docs/estudos/` |
| documentação | `docs/specs/SPEC-024/` | Exemplo de documentos atuais e dos títulos literais |

## O que mudou

| # | Assunto | Estado no Original | Estado Refinado |
|---|---|---|---|
| 1 | Tipo | Rascunho sem tipo | improvement, numa spec só, com estudos, protótipos e implementação (resposta 1B) |
| 2 | Evidências | Não citadas | Entram no visual e no conteúdo (resposta 2B) |
| 3 | Conteúdo dos protótipos | Não definido | Spec real reescrita no modelo proposto (resposta 3A) |
| 4 | Protótipos | Três documentos | Três modelos com executivo e técnico, seis páginas (resposta 4A) |
| 5 | Documentos publicados | Não citados | Só os próximos usam o modelo novo (resposta 2A da 2ª rodada) |
| 6 | KT | O quê, por quê e como | Mais operação e diagnóstico, mapa do código e limites, com as seções finais propostas pelo estudo (resposta 2 da 2ª rodada) |

## Registro de decisões

| # | Decisão | Alternativa descartada | Motivo |
|---|---|---|---|
| 1 | Uma spec só, de tipo improvement | Estudo agora e implementação noutra spec | resposta 1B |
| 2 | Evidências no visual e no conteúdo | Só no visual | resposta 2B |
| 3 | Protótipos com uma spec real reescrita | Conteúdo atual ou fictício | resposta 3A |
| 4 | Cada protótipo com executivo e técnico | Só o executivo | resposta 4A |
| 5 | Só os próximos documentos no modelo novo | Regerar os publicados | resposta 2A da 2ª rodada |
| 6 | KT com operação, mapa do código e limites, seções finais pelo estudo | Só o quê, por quê e como | resposta 2 da 2ª rodada |
| 7 | Tema claro padrão e tokens de 2026-10 | Manter o escuro padrão | inferência A |
| 8 | Fontes embutidas com subconjunto | CDN ou fontes do sistema | inferência B |
| 9 | Tokens num bloco CSS que cita a organização | Consumir o pacote de tema | inferência C |
| 10 | Estudo e protótipos em `docs/estudos/` | Outra pasta | inferência D |
| 11 | Estudo de leitura com referências públicas e a preferência por menos bullets | Só a preferência do usuário | inferência E |
| 12 | Protótipos responsivos e AA | Só desktop | inferência F |
| 13 | Títulos literais corrigidos na implementação | Nos protótipos | inferência G |
| 14 | A implementação espera a escolha do modelo e a aprovação do estudo | Implementar em paralelo | consequência da resposta 1B |

## Pendências

| # | Pendência | Opções em aberto |
|---|---|---|
| - | Nenhuma | - |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
