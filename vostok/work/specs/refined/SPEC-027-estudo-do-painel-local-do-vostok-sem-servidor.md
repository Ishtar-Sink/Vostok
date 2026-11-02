---
id: SPEC-027
title: "Estudo do painel local do Vostok sem servidor"
status: implemented
author: "Victor Batalha"
created_at: 2026-11-02T09:36:36Z
updated_at: 2026-11-02T12:45:55Z
tags: []
type: study
source: "draft/SPEC-027-estudo-do-painel-local-do-vostok-sem-servidor.md"
refined_at: 2026-11-02T09:41:56Z
adjustments: 1
adjusted_at: 2026-11-02T09:45:52Z
content_hash: 4fa8cdb48e14a0c66df9eea1c90bb95e595ed35f55582c3e208d0fd1e802e2d7
approved_at: 2026-11-02T09:45:56Z
approved_hash: 4fa8cdb48e14a0c66df9eea1c90bb95e595ed35f55582c3e208d0fd1e802e2d7
---

# Estudo do painel local do Vostok sem servidor - Especificação Refinada

Estudo de viabilidade de um painel local, aberto por pasta com um comando do `vostok`, que lê e grava os arquivos do harness como banco de dados, no lugar da premissa de servidor. O estudo inclui um protótipo funcional descartável, com os componentes do Atria e a marca da pasta, e, se viável, define a melhor forma de construí-lo mantendo a CLI como um binário leve.

## Objetivo

A premissa final do Vostok previa servidor HTTP, banco e MCP (`project/architecture-05`, `project/stack-04`), que nunca saíram do papel: `apps/server`, `apps/mcp` e `apps/web` estão vazias. O estudo responde se um painel local, sem servidor de armazenamento, pode fazer praticamente tudo o que hoje é feito pelo chat, e qual é a melhor forma de construí-lo sem pesar o binário de 4,6 MB.

## Escopo

### Dentro do escopo

- Viabilidade de um painel local aberto pela CLI, com os `.md` e os arquivos de `vostok/` como fonte de dados e banco.
- Protótipo funcional descartável, num branch fora da `main`, aberto numa pasta real, para medir e para o usuário usar antes de aprovar a conclusão.
- Medição do impacto no binário (instalado e compactado) e proposta do limite de tamanho.
- Comparação do servidor HTTP embutido e do empacotamento do frontend em React com os componentes do Atria.
- Inventário das ações do chat e de como cada uma seria feita pelo painel.
- Aprovação pelo painel e a relação com as regras de aprovação do harness.
- Edição simultânea dos mesmos arquivos pelo painel e pelo agente.
- Desenho do loop engineering: como um chat percebe e processa os pedidos feitos no painel.
- Vários painéis ao mesmo tempo, um por pasta.
- Identidade visual: componentes do Atria com a marca da pasta (grupo `brand` de `config/documents.yml`).
- Lista recomendada do que muda nas regras, na memória e nas pastas para remover a premissa de servidor.
- Se viável, a arquitetura recomendada e a proposta das specs de implementação.

### Fora do escopo

- Implementar o painel na `main`: o código do protótipo não é mergeado.
- Aplicar a remoção da premissa de servidor (regras, memória, `apps/server`, `apps/mcp`): o estudo só recomenda.
- Implementar o loop engineering.
- Servidor de armazenamento, banco e MCP.
- Alterar o repositório do Atria: os componentes são só copiados para o protótipo.
- Ler a pasta `secrets` do Atria ou qualquer arquivo de segredo.

## Perguntas da investigação

| ID | Pergunta | Como será respondida |
|---|---|---|
| PQ-01 | É viável um painel local que use os arquivos de `vostok/` como fonte de dados e os `.md` como banco, sem servidor de armazenamento? | Protótipo aberto em `/home/victor/projetos/ai-hub`, lendo specs, planos, tarefas e evidências e gravando ao menos uma aprovação e uma edição. |
| PQ-02 | Qual servidor HTTP embutido no `vostok` pesa menos e atende o painel (por exemplo `tiny_http`, `axum` ou `hyper`)? | Medição no protótipo do tamanho do binário de release com cada opção, comparado ao `vostok` 0.2.10. |
| PQ-03 | Como embutir no binário o frontend em React com os componentes do Atria, e com que peso? | Medição do bundle com React e com Preact com compat, com e sem compressão, embutido com `include_dir` como o harness já é; inclui o impacto no `cargo install --path apps/cli` e na release, que hoje não precisam de Node. |
| PQ-04 | Qual limite de tamanho mantém o binário leve? | Proposta a partir das medições de PQ-02 e PQ-03 e do tamanho atual (4,6 MB instalado, cerca de 1,7 MB compactado na release). |
| PQ-05 | Quais ações do chat o painel consegue fazer, e de que forma? | Inventário das skills e dos comandos `vostok`, separando as determinísticas (o painel chama a lógica da CLI) das que precisam do agente (o painel grava um pedido na pasta). |
| PQ-06 | Como uma aprovação feita no painel passa a valer como aprovação do usuário, sem enfraquecer as regras do harness? | Leitura de `rules/agent/` e dos comandos `approve` de spec, plano e tarefa, e da trava de aval prevista depois da SPEC-025. |
| PQ-07 | Como evitar perda de dados quando o painel e o agente editam o mesmo arquivo? | Teste no protótipo com edição simultânea e comparação de controle por hash ou data de modificação, ligado ao registro de edição manual (`adjust record-manual`). |
| PQ-08 | Como um chat em loop (Claude Code ou Codex) percebe e processa os pedidos do painel? | Desenho do formato do pedido na pasta e teste com um chat do Claude Code em `/loop` processando um pedido gravado pelo protótipo. |
| PQ-09 | Como abrir vários painéis ao mesmo tempo, um por pasta? | Protótipo aberto em `/home/victor/projetos/ai-hub` e em `/home/victor/projetos/ishtar-sink/atria` ao mesmo tempo, com porta livre e escuta só em `127.0.0.1`. |
| PQ-10 | Como aplicar os componentes do Atria com a marca de cada pasta? | Mapeamento dos tokens de `@atria/theme` para as cores do grupo `brand` de `config/documents.yml`, aplicado no protótipo nas duas pastas. |
| PQ-11 | O que muda nas regras, na memória e nas pastas do projeto para remover a premissa de servidor? | Leitura de `project/architecture`, `project/stack` e da memória de arquitetura, com a lista de mudanças recomendadas. |
| PQ-12 | Como o painel mostra as mudanças que o agente faz nos arquivos sem recarregar a página? | Comparação no protótipo de vigiar a pasta (eventos do sistema de arquivos ou consulta periódica) e de enviar a atualização ao navegador. |

## Premissas e dependências

- O harness instalado em `vostok/` continua sendo a única fonte de verdade; o painel não cria outro armazenamento.
- Os componentes de `packages/ui` do Atria (React 19, CSS Modules, `@atria/theme`) podem ser copiados para o protótipo, porque vão virar uma biblioteca depois.
- O Vostok passa a ter a mesma licença do Atria, AGPL-3.0 (`apps/cli/Cargo.toml` e `LICENSE`), então copiar os componentes não cria conflito de licença.
- O protótipo usa um branch próprio (`spike/painel-local`) e não entra na `main`.
- Node e pnpm estão disponíveis na máquina para gerar o bundle do protótipo.
- O Atria tem o harness instalado em `/home/victor/projetos/ishtar-sink/atria/vostok`.

## Riscos

| Risco | Impacto | Mitigação |
|---|---|---|
| Gerar o frontend passa a exigir Node no `cargo install` e na release | médio | PQ-03 compara gerar o bundle na CI e embutir o resultado já pronto. |
| O bundle em React passa do limite de binário leve | médio | PQ-03 mede Preact com compat e compressão; PQ-04 propõe o limite com os números. |
| Uma aprovação pelo painel contorna as regras de aprovação do harness | alto | PQ-06 define como o painel registra a aprovação do usuário antes de qualquer implementação. |
| Painel e agente sobrescrevem a mesma edição | médio | PQ-07 testa a edição simultânea no protótipo. |
| O loop depende de recursos do Claude Code que o Codex não tem | médio | PQ-08 desenha o pedido em arquivo, independente da ferramenta, e aponta o que muda em cada uma. |

## Critérios de aceite

- [ ] CA-01: `docs/estudos/painel-local.md` responde a cada pergunta, de PQ-01 a PQ-12, com a conclusão e a fonte ou medição que a sustenta (PQ-01, PQ-02, PQ-03, PQ-04, PQ-05, PQ-06, PQ-07, PQ-08, PQ-09, PQ-10, PQ-11, PQ-12)
- [ ] CA-02: O estudo declara se o painel local sem servidor é viável, com os motivos (PQ-01)
- [ ] CA-03: O estudo traz uma tabela com o tamanho do binário instalado e compactado do `vostok` 0.2.10 e de cada opção medida, com os comandos para reproduzir a medição (PQ-02, PQ-03)
- [ ] CA-04: O estudo propõe um limite de tamanho do binário, justificado pelas medições (PQ-04)
- [ ] CA-05: O protótipo, no branch `spike/painel-local`, abre o painel em `/home/victor/projetos/ai-hub`, mostra a visão geral, uma spec, um plano e uma tarefa com evidências, com os componentes do Atria e a marca da pasta, e o usuário o usa antes de aprovar a conclusão (PQ-01, PQ-10)
- [ ] CA-06: O protótipo grava uma aprovação e uma edição de `.md` feitas no painel, e o arquivo gravado passa no comando de validação correspondente do `vostok` (PQ-01, PQ-06)
- [ ] CA-07: Dois painéis abertos ao mesmo tempo, em `/home/victor/projetos/ai-hub` e `/home/victor/projetos/ishtar-sink/atria`, respondem em portas diferentes, só em `127.0.0.1`, cada um com a marca da própria pasta (PQ-09, PQ-10)
- [ ] CA-08: O estudo traz o inventário das ações do chat, cada uma marcada como feita pela CLI, por pedido ao agente ou fora do painel (PQ-05)
- [ ] CA-09: O estudo descreve o formato do pedido gravado pelo painel e registra o teste de um chat do Claude Code em `/loop` processando um pedido (PQ-08)
- [ ] CA-10: O estudo registra o resultado do teste de edição simultânea e o controle recomendado (PQ-07, PQ-12)
- [ ] CA-11: O estudo lista as mudanças recomendadas nas regras, na memória e nas pastas para remover a premissa de servidor (PQ-11)
- [ ] CA-12: Se viável, o estudo traz a arquitetura recomendada e a lista ordenada das specs de implementação propostas (PQ-01, PQ-05)
- [ ] CA-13: Nenhum arquivo do protótipo entra na `main`; ela recebe só o estudo e as imagens do protótipo (PQ-01)

## Referências

| Tipo | Referência | Uso |
|---|---|---|
| memória | `vostok-arquitetura` (memória do agente) | Premissa atual: servidor como único escritor do banco, MCP, aprovação pela web e pela ferramenta. |
| memória | `spec-trava-aval` (memória do agente) | Trava de aval prevista para finish, commit e aprovações, ligada a PQ-06. |
| memória | DEC-001 | Nome do produto e da CLI, sem impacto no escopo. |
| arquivo | `vostok/rules/project/architecture.md` | Regras `architecture-01`, `-05`, `-06`, `-07` que citam servidor, web e MCP (PQ-11). |
| arquivo | `vostok/rules/project/stack.md` | Regras `stack-02` (React com Vite), `-03`, `-04`, `-06`, `-07` (PQ-03, PQ-11). |
| arquivo | `apps/cli/Cargo.toml` | Dependências atuais do binário (`clap`, `include_dir`, `sha2`, `anyhow`) e versão 0.2.10. |
| arquivo | `apps/cli/src/commands/task.rs` | Fluxo de aprovação de tarefas que o painel precisaria reproduzir (PQ-05, PQ-06). |
| arquivo | `apps/cli/src/commands/documents.rs` | Leitura e gravação do grupo `brand` (PQ-10). |
| arquivo | `vostok/config/documents.yml` | Tokens de cor da identidade dos documentos (PQ-10). |
| arquivo | `packages/harness/templates/finalize/` | Templates dos documentos finais, referência da identidade. |
| arquivo | `/home/victor/projetos/ishtar-sink/atria/packages/ui` | Componentes React do Atria a copiar para o protótipo. |
| arquivo | `/home/victor/projetos/ishtar-sink/atria/package.json` | Licença AGPL-3.0 do Atria, adotada também pelo Vostok. |
| documentação | `docs/estudos/documentos-finais/README.md` | Formato de protótipos gerados dentro de um estudo, como na SPEC-025. |
| documentação | `docs/estudos/tarefa-como-pasta.md` | Formato dos estudos do projeto (escopo e método, medições). |
| arquivo | `/home/victor/.cargo/bin/vostok` | Tamanho atual do binário instalado: 4,6 MB. |

## O que mudou

| # | Assunto | Estado no Original | Estado Refinado |
|---|---|---|---|
| 1 | Tipo | Não definido | `study`, com o resultado em `docs/estudos/painel-local.md` |
| 2 | Remoção do servidor | "o servidor cai fora" | Só recomendada pelo estudo; aplicada por outra spec |
| 3 | Protótipo | Não citado | Protótipo funcional descartável no branch `spike/painel-local`, usado pelo usuário antes de aprovar |
| 4 | Identidade visual | Identidade dos documentos | Componentes do Atria com a marca da pasta (grupo `brand`) |
| 5 | Frontend | Não citado | React com os componentes do Atria; a comparação é do empacotamento |
| 6 | Binário leve | Sem limite | O estudo mede e propõe o limite |
| 7 | Perguntas | Viabilidade e melhor forma | 12 perguntas, de PQ-01 a PQ-12 |
| 8 | Licença | Não citada | Vostok em AGPL-3.0, a mesma do Atria |

## Registro de decisões

| # | Decisão | Alternativa descartada | Motivo |
|---|---|---|---|
| 1 | A remoção da premissa de servidor só é recomendada pelo estudo | Aplicar já nesta spec | resposta 1 |
| 2 | Protótipo funcional descartável, usado pelo usuário antes de aprovar, como na SPEC-025 | Só pesquisa | resposta 2 |
| 3 | Visual com o estilo do Atria, podendo copiar os componentes dele | Identidade só dos documentos | resposta 2 |
| 4 | O estudo pode recomendar fora de React com Vite | Manter a regra `stack-02` | resposta 3 |
| 5 | O estudo propõe o limite do binário | +1 MB ou +2 MB fixos | resposta 4 |
| 6 | Componentes do Atria com a marca da pasta | Atria por inteiro | resposta 2.1 |
| 7 | Frontend em React com os componentes do Atria; compara-se o empacotamento | Comparar inclusive sem React | resposta 2.2 |
| 8 | Um protótipo funcional, sem modelos estáticos | Três modelos visuais | resposta 2.3 |
| 9 | Resultado em `docs/estudos/painel-local.md` | Outro formato | inferência A |
| 10 | Painel só em `127.0.0.1`, porta livre por pasta | Porta fixa | inferência B |
| 11 | Duas formas de agir comparadas: lógica da CLI ou pedido ao agente | Uma só forma | inferência C |
| 12 | Edição simultânea e aprovação pelo painel entram no estudo | Ficar para a implementação | inferência D |
| 13 | Loop engineering só desenhado e testado, sem implementação | Implementar no estudo | inferência E |
| 14 | Marca vinda do grupo `brand` de `config/documents.yml` | Configuração própria do painel | inferência F |
| 15 | Protótipo em `spike/painel-local`, com as imagens em `docs/estudos/painel-local/` | Protótipo na `main` | inferência G |
| 16 | Telas do protótipo: visão geral, spec, plano, tarefa com evidências e aprovação | Todas as telas | inferência H |
| 17 | O Vostok passa de MIT para AGPL-3.0, como o Atria, e a pendência de licença é resolvida | Manter MIT e reescrever os componentes | ajuste 1 |

## Pendências

| # | Pendência | Opções em aberto |
|---|---|---|
| - | Nenhuma | - |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
| 1 | 2026-11-02 | agente | Licença do Vostok passa a AGPL-3.0; risco de licença removido; pendência de licença resolvida |
