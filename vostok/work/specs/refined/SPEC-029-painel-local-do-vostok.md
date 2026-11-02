---
id: SPEC-029
title: "Painel local do Vostok"
status: implemented
author: "Victor Batalha"
created_at: 2026-11-02T12:57:17Z
updated_at: 2026-11-02T15:13:04Z
tags: []
type: feature
source: "draft/SPEC-029-painel-local-do-vostok.md"
refined_at: 2026-11-02T13:03:28Z
adjustments: 0
adjusted_at: null
content_hash: 1573cd6ca1e4a4f7e9cc9a459af303f50a5557f0024393667b9308353d5926db
approved_at: 2026-11-02T13:07:17Z
approved_hash: 1573cd6ca1e4a4f7e9cc9a459af303f50a5557f0024393667b9308353d5926db
---

# Painel local do Vostok - Especificação Refinada

Implementa o painel local do Vostok. O comando `vostok painel` abre, para cada pasta, um painel no navegador que lê specs, planos, tarefas, evidências, documentos e configurações dos arquivos de `vostok/`. Pelo painel, o usuário aprova specs e tarefas, edita os `.md`, grava as configurações e grava pedidos ao agente. A implementação segue a arquitetura recomendada pela SPEC-027 e o protótipo aprovado pelo usuário.

## Objetivo

Hoje o acompanhamento do trabalho e as aprovações acontecem só pelo chat. O estudo da SPEC-027 (`docs/estudos/painel-local.md`) concluiu que um painel local sem servidor é viável e recomendou a arquitetura, validada por um protótipo no branch `spike/painel-local`. Esta spec leva o painel para a `main` com testes, no lugar do protótipo descartável. Ela junta as duas specs propostas pelo estudo, o painel de leitura e as ações e a edição, para executar com paralelismo junto com a SPEC-028.

## Escopo

### Dentro do escopo

- Comando `vostok painel`, com o servidor `tiny_http` atrás da feature `painel`.
- Frontend `apps/painel` em Preact com compat, com os componentes do Atria e a marca da pasta.
- Telas do protótipo, com o mesmo conteúdo:
  - painel, pendências, specs, spec, quadro da spec e plano;
  - tarefas, tarefa, documentos e configurações.
- Atualização ao vivo por WebSocket.
- Aprovação de spec e tarefa pelo painel, com `approved_by`, o aval em `vostok/work/avais.log` e o token de sessão.
- Edição dos `.md` de tarefa, plano e spec, com controle por hash e o registro do ajuste manual.
- Gravação das configurações de `vostok/config/`.
- Gravação dos pedidos ao agente e leitura das perguntas do agente.
- `build.rs` que gera o bundle quando há Node, um passo de Node na release e a conferência do limite do binário.

### Fora do escopo

- A conferência do aval pela CLI antes de `finish`, commit e aprovações, que fica para a SPEC-030.
- O loop que processa os pedidos e o processo do Codex, também na SPEC-030.
- As mudanças nas regras e na memória, que ficam para a SPEC-028.
- Aprovar o plano, finalizar a tarefa, finalizar a spec e fazer commit direto pelo painel: essas ações viram pedidos ao agente.
- O `notify`: a atualização ao vivo usa consulta periódica.
- A massa de dados fictícia do protótipo e o `medir.sh`.

## Requisitos funcionais

| ID | Requisito |
|---|---|
| RF-01 | O comando `vostok painel` deve achar `vostok/config/main.yml`, subindo as pastas a partir da pasta atual ou de `--pasta`. Deve escutar só em `127.0.0.1`, numa porta livre ou na de `--porta`, imprimir a URL e abrir o navegador, salvo com `--sem-navegador`. |
| RF-02 | O painel deve servir o bundle embutido e a API de leitura: `visao`, `versao`, `marca`, `documentos`, `configuracoes`, `specs/<ID>`, `planos/<ID>`, `tarefas/<SPEC>/<TASK>`, `arquivo` e `fonte`. A API lê os arquivos de `vostok/` a cada requisição. |
| RF-03 | O frontend deve ter as telas do protótipo: painel, pendências, specs, spec, quadro da spec, plano, tarefas, tarefa, documentos e configurações. Também deve ter o menu lateral, o popover das referências, os estados de carregamento, vazio e erro, e os temas claro e escuro. |
| RF-04 | O painel deve aplicar a marca da pasta: os grupos `brand`, `light` e `dark` de `vostok/config/documents.yml`, sobre os tokens do Atria. |
| RF-05 | O painel deve avisar as abas abertas pelo WebSocket `/api/ao-vivo` quando `vostok/work/` muda, e as telas devem atualizar sem recarregar a página. |
| RF-06 | O painel deve aprovar spec e tarefas pelas mesmas conferências de `vostok spec approve approve` e `vostok task approve approve`. Ele grava `approved_by: painel` e acrescenta o aval a `vostok/work/avais.log`. |
| RF-07 | A CLI deve gravar `approved_by: chat` nas aprovações, e o `drop_approval` deve limpar o `approved_by` quando um ajuste derruba a aprovação. |
| RF-08 | Toda ação (POST e PUT) deve exigir o token de sessão no cabeçalho `X-Painel-Token`. Sem ele, a resposta é 403. |
| RF-09 | O painel deve editar os `.md` de `vostok/work/` com controle otimista por SHA-256, respondendo 409 com o conteúdo atual num conflito. Depois de gravar, registra o ajuste manual e devolve os problemas da validação. |
| RF-10 | O painel deve gravar as configurações de `vostok/config/` editadas na tela de configurações, com o mesmo controle por hash. |
| RF-11 | O painel deve gravar os pedidos ao agente em `vostok/work/pedidos/PED-NNN.md`, com as 10 ações aceitas, e mostrar as perguntas de `vostok/work/perguntas/P-NNN.md`. |
| RF-12 | O `build.rs` deve gerar o bundle de `apps/painel` quando `dist/` não existe e há Node. Sem Node, compila sem o painel, e `vostok painel` explica como obtê-lo. |
| RF-13 | A release deve gerar o bundle antes do `cargo build` e conferir o tamanho do binário e do tar.gz. |

## Requisitos não funcionais

| ID | Requisito | Como verificar |
|---|---|---|
| RNF-01 | Binário de release com o painel em até 6,0 MB, e tar.gz em até 2,3 MB. | Conferência na release e `ls -l` local do binário de `--release`. |
| RNF-02 | O painel não é acessível fora da máquina. | `ss -ltnp` mostra só `127.0.0.1`. |
| RNF-03 | O aviso ao vivo chega em menos de 1 s. | O teste mede o atraso de uma mudança em `vostok/work/` até o `mudou`. |
| RNF-04 | `cargo test` passa no Linux e no macOS, com e sem a feature. | CI e `cargo test --features painel`. |
| RNF-05 | Acessibilidade: foco visível, navegação por teclado e contraste dos tokens. | Testes do frontend e revisão dos prints. |

## Regras de negócio

| ID | Regra |
|---|---|
| RN-01 | A origem da aprovação é um tipo (`Chat` ou `Panel`), nunca um argumento de linha de comando. |
| RN-02 | O painel grava só pela lógica da CLI: as mesmas funções e validações, sem outro armazenamento. |
| RN-03 | `registro.md`, `README.md` e `estado.yml` das tarefas nunca são gravados pela edição do painel (403). |
| RN-04 | Aprovar spec e tarefa é direto no painel. Aprovar o plano, executar, finalizar, ajustar e responder perguntas são pedidos ao agente. |
| RN-05 | O `avais.log` só recebe acréscimos. |
| RN-06 | Gravar por cima num conflito só acontece com um segundo clique explícito do usuário. |

## Fluxos

### Fluxo principal

1. O usuário roda `vostok painel` numa pasta com o harness.
2. O `vostok` gera o token de sessão, sobe o servidor em `127.0.0.1` e abre o navegador.
3. O navegador carrega o bundle com o token e lê a API.
4. O usuário aprova uma tarefa: o painel confere o token, aprova pela lógica da CLI, grava `approved_by: painel` e o aval.
5. O usuário edita os critérios: o painel grava com o hash, registra o ajuste e mostra os problemas, se houver.
6. O usuário pede um ajuste: o painel grava o `PED-NNN.md`.
7. O agente muda um arquivo: o painel avisa as abas, e as telas atualizam.

### Fluxos alternativos

- FA-01 (passo 1): a pasta não tem `vostok/config/main.yml` em nenhuma pasta acima. O comando para com a mensagem e o caminho procurado.
- FA-02 (passo 2): o binário foi compilado sem o painel. O comando explica como gerar o bundle e compilar de novo.
- FA-03 (passo 5): o arquivo mudou no disco. Responde 409, e o diálogo mostra "Seu texto" e "Versão no disco".
- FA-04 (passo 4): a aprovação falha nas conferências. O painel mostra a mensagem do `vostok` e não grava o aval.

## Casos de borda

| ID | Situação | Comportamento esperado | Requisitos |
|---|---|---|---|
| CB-01 | `--porta` ocupada | Para com a mensagem da porta ocupada | RF-01 |
| CB-02 | Caminho fora de `vostok/work/` ou com `..` na edição | 403, sem gravar | RF-09, RN-03 |
| CB-03 | Ação de pedido desconhecida | 400 | RF-11 |
| CB-04 | Corpo maior que 2 MB | Recusado | RF-09 |
| CB-05 | Pasta sem marca em `documents.yml` | Usa os tokens padrão | RF-04 |
| CB-06 | Duas abas abertas ao mesmo tempo | As duas recebem o `mudou` | RF-05 |
| CB-07 | Spec ou tarefa arquivada | Só leitura, sem "Editar" | RF-03, RF-09 |

## Contratos e interfaces

```text
vostok painel [--porta N] [--pasta DIR] [--sem-navegador]
```

```text
GET  /api/visao | /api/versao | /api/marca | /api/documentos | /api/configuracoes
GET  /api/specs/<ID> | /api/planos/<ID> | /api/tarefas/<SPEC>/<TASK>
GET  /api/fonte?caminho=          -> {"conteudo","hash"}
GET  /api/arquivo?caminho=        -> arquivo de vostok/
PUT  /api/arquivo?caminho=&hash=  -> 200 {"hash","mensagem","problemas"} | 409 {"hash","conteudo"} | 403
POST /api/aprovacoes?spec=&tarefas=
POST /api/pedidos?acao=&spec=&tarefa=   (corpo: texto do usuário)
WS   /api/ao-vivo                 -> "conectado" | "mudou"
Cabeçalho das ações: X-Painel-Token; token na página: <meta name="painel-token">
```

Campo no front-matter da spec e no `estado.yml` da tarefa:

```yaml
approved_by: chat   # ou painel
```

Linha de `vostok/work/avais.log` (separada por tabulação):

```text
2026-11-02T12:11:42Z	SPEC-027/TASK-05	<hash aprovado>	painel
```

O pedido segue o formato de `docs/estudos/painel-local/testes.md`. O front-matter tem `id`, `acao`, `spec`, `tarefa`, `estado`, `origem`, `criado_em` e `atualizado_em`, e o corpo tem as seções `## Pedido` e `## Resultado`. As ações aceitas são `aprovar-plano`, `planejar-spec`, `ajustar-spec`, `ajustar-plano`, `ajustar-tarefa`, `executar-tarefa`, `finalizar-tarefa`, `desbloquear-tarefa`, `finalizar-spec` e `responder-pergunta`.

## Componentes afetados

| Caminho | Ação | Motivo |
|---|---|---|
| `apps/cli/src/cli.rs` | alterar | RF-01: subcomando `painel` |
| `apps/cli/src/commands/mod.rs` | alterar | RF-01: registro do comando |
| `apps/cli/src/commands/painel.rs` | criar | RF-01, RF-12: comando, abertura e mensagem sem bundle |
| `apps/cli/src/painel/server.rs` | criar | RF-01, RF-08: servidor, token e roteamento |
| `apps/cli/src/painel/read.rs` | criar | RF-02, RF-04: API de leitura e marca |
| `apps/cli/src/painel/actions.rs` | criar | RF-06, RF-09, RF-10, RF-11: aprovação, edição, configurações e pedidos |
| `apps/cli/src/painel/live.rs` | criar | RF-05: WebSocket e consulta periódica |
| `apps/cli/src/lib.rs` | alterar | Registro do módulo `painel` |
| `apps/cli/src/commands/task.rs` | alterar | RF-06, RF-07: `Origin`, `approve_from_panel`, `approved_by`, `drop_approval` e `record_manual_at` |
| `apps/cli/src/commands/spec.rs` | alterar | RF-06, RF-07: aprovação com origem e `record_manual_at` |
| `apps/cli/src/commands/plan.rs` | alterar | RF-09: `record_manual_at` |
| `apps/cli/Cargo.toml` | alterar | Feature `painel` e dependências `tiny_http` e `sha1` |
| `apps/cli/build.rs` | alterar | RF-12: gerar o bundle quando há Node |
| `apps/cli/tests/common/golden.rs` | alterar | RNF-04: tipo na linha 507 e fixtures com `approved_by: chat` |
| `apps/cli/tests/painel.rs` | criar | Testes de integração do painel |
| `apps/painel/package.json` | criar | RF-03: app Vite com Preact com compat |
| `apps/painel/src/shared` | criar | RF-03, RF-05, RF-08: API, roteador, diálogo de edição e popover |
| `apps/painel/src/layout` | criar | RF-03: menu lateral e topo |
| `apps/painel/src/theme` | criar | RF-04: tokens do Atria e marca |
| `apps/painel/src/ui` | criar | RF-03: componentes copiados do Atria |
| `apps/painel/src/features/dashboard` | criar | RF-03: painel |
| `apps/painel/src/features/inbox` | criar | RF-03, RF-06, RF-11: pendências e ações |
| `apps/painel/src/features/specs` | criar | RF-03: lista de specs |
| `apps/painel/src/features/spec` | criar | RF-03, RF-09: spec e plano |
| `apps/painel/src/features/board` | criar | RF-03: quadros de tarefas |
| `apps/painel/src/features/task` | criar | RF-03, RF-09: tarefa |
| `apps/painel/src/features/documents` | criar | RF-03: documentos |
| `apps/painel/src/features/settings` | criar | RF-10: configurações |
| `.github/workflows/release.yml` | alterar | RF-13: Node, bundle e conferência do tamanho |

## Premissas e dependências

- O protótipo em `/home/victor/projetos/ai-hub-painel` (branch `spike/painel-local`) é referência. A implementação é reescrita com testes.
- Os componentes do Atria (`/home/victor/projetos/ishtar-sink/atria/packages/ui`) podem ser copiados, porque o Vostok usa a mesma licença AGPL-3.0.
- A SPEC-028 roda em paralelo e troca a regra `stack-02` para Preact. Esta spec não muda regras.
- A SPEC-030 vai conferir o aval gravado aqui, nos formatos desta spec.
- Node e pnpm estão disponíveis na CI e na máquina do usuário.

## Riscos

| Risco | Impacto | Mitigação |
|---|---|---|
| O binário passa do limite | médio | Conferência na release; embutir o bundle com gzip, se preciso (cerca de 108 KB a menos). |
| O agente aprova chamando o painel por fora | alto | Token de sessão na página e 403 sem ele; a trava fica na SPEC-030. |
| Edição pelo painel quebra o formato | médio | Registro do ajuste manual e problemas da validação mostrados na hora. |
| A SPEC-030 muda `task.rs` ao mesmo tempo | médio | A SPEC-030 começa depois desta ou combina os mesmos caminhos no plano. |
| `cargo install` sem Node perde o painel sem aviso | baixo | O `build.rs` avisa, e `vostok painel` explica como obter. |

## Casos de teste

| ID | Cenário | Passos | Resultado esperado | Requisitos |
|---|---|---|---|---|
| CT-01 | Abrir em duas pastas | `vostok painel --sem-navegador` no ai-hub e numa cópia | Portas diferentes, só em `127.0.0.1`, cada uma com a sua marca | RF-01, RF-04, RNF-02 |
| CT-02 | Pasta sem harness | `vostok painel --pasta /tmp` | Erro com o caminho procurado | RF-01 |
| CT-03 | Telas | Abrir cada rota do frontend com a massa de teste | Conteúdo dos arquivos, estados de vazio e erro | RF-02, RF-03, RNF-05 |
| CT-04 | Aprovar tarefa | POST de aprovação com token | `approved_by: painel`, linha no `avais.log` e `task adjust validate` passando | RF-06 |
| CT-05 | Aprovar sem token | POST sem `X-Painel-Token` | 403, nada gravado | RF-08 |
| CT-06 | Aprovação pela CLI | `vostok task approve approve` | `approved_by: chat`; os goldens atualizados passam | RF-07, RNF-04 |
| CT-07 | Ajuste derruba aprovação | Editar uma tarefa aprovada | Status `pending` e `approved_by` vazio | RF-07 |
| CT-08 | Editar com hash certo | PUT com o hash lido | Gravado, ajuste manual registrado e problemas devolvidos | RF-09 |
| CT-09 | Editar com hash velho | PUT depois de mudar o arquivo por fora | 409 com o conteúdo atual; arquivo intacto | RF-09, RN-06 |
| CT-10 | Editar arquivo proibido | PUT em `registro.md` ou fora de `vostok/work/` | 403 | RN-03 |
| CT-11 | Gravar configuração | Alterar uma opção na tela | O arquivo de `vostok/config/` muda e `vostok rules validate` passa | RF-10 |
| CT-12 | Pedido | POST de `ajustar-tarefa` e de uma ação desconhecida | `PED-NNN.md` no formato; 400 na desconhecida | RF-11 |
| CT-13 | Ao vivo | Mudar um arquivo de `vostok/work/` com o WebSocket aberto | `mudou` em menos de 1 s | RF-05, RNF-03 |
| CT-14 | Build sem Node | Compilar sem `dist/` e sem Node | Compila; `vostok painel` explica como obter o painel | RF-12 |
| CT-15 | Release | Build de release com a feature | Binário até 6,0 MB e tar.gz até 2,3 MB | RF-13, RNF-01 |

## Critérios de aceite

- [ ] CA-01: `vostok painel` abre o painel da pasta só em `127.0.0.1`, com as telas do protótipo e a marca da pasta, em várias pastas ao mesmo tempo (RF-01, RF-02, RF-03, RF-04, RNF-02, RNF-05)
- [ ] CA-02: A aprovação pelo painel grava `approved_by: painel` e o aval, passa na validação do `vostok` e é recusada sem token (RF-06, RF-07, RF-08)
- [ ] CA-03: Edição de `.md` e de configuração pelo painel registra o ajuste e não perde dados num conflito (RF-09, RF-10)
- [ ] CA-04: Pedidos são gravados no formato do estudo, e o painel atualiza sozinho quando os arquivos mudam (RF-05, RF-11, RNF-03)
- [ ] CA-05: O `cargo install` funciona com e sem Node, a release gera o bundle e o binário fica dentro de 6,0 MB e 2,3 MB (RF-12, RF-13, RNF-01, RNF-04)

## Referências

| Tipo | Referência | Uso |
|---|---|---|
| documentação | docs/estudos/painel-local.md | Arquitetura recomendada, limite do binário e specs propostas. |
| documentação | docs/estudos/painel-local/prototipo.md | Telas, marca, atualização ao vivo e retorno do usuário. |
| documentação | docs/estudos/painel-local/testes.md | Aprovação, edição, conflito e formato do pedido. |
| documentação | docs/estudos/painel-local/medicoes.md | Tamanhos e impacto de Node no build. |
| arquivo | /home/victor/projetos/ai-hub-painel | Código do protótipo, como referência. |
| arquivo | apps/cli/build.rs | Onde entra a geração do bundle. |
| arquivo | .github/workflows/release.yml | Onde entram o passo de Node e a conferência do tamanho. |
| arquivo | apps/cli/src/commands/task.rs | Aprovação, `drop_approval` e `record_manual`. |
| arquivo | /home/victor/projetos/ishtar-sink/atria/packages/ui | Componentes a copiar. |
| arquivo | vostok/config/documents.yml | Grupos de marca e tokens. |

## O que mudou

| # | Assunto | Estado no Original | Estado Refinado |
|---|---|---|---|
| 1 | Tipo | Não definido | feature |
| 2 | Build local | Não citado | `build.rs` gera o bundle quando há Node |
| 3 | Backend | `painel.rs` único | Módulo `apps/cli/src/painel/` dividido em servidor, leitura, ações e ao vivo |
| 4 | Fora do escopo | Trava e loop na SPEC-030 | Também regras, `notify`, massa de dados e ações que viram pedido |

## Registro de decisões

| # | Decisão | Alternativa descartada | Motivo |
|---|---|---|---|
| 1 | `build.rs` gera o bundle quando `dist/` falta e há Node | Gerar à mão ou versionar o `dist/` | resposta 1 |
| 2 | Backend dividido em `server`, `read`, `actions` e `live` | Um arquivo `painel.rs` como no spike | inferência A |
| 3 | Frontend com componentes divididos por feature, para tarefas paralelas | Uma pasta só | inferência B |
| 4 | Testes do frontend com Vitest e Testing Library | Só testes manuais | inferência C |
| 5 | A massa de dados fictícia fica fora | Versioná-la como fixture | inferência D |
| 6 | Configurações gravadas com o mesmo controle por hash | Gravar sem controle | inferência E |

## Pendências

| # | Pendência | Opções em aberto |
|---|---|---|
| - | Nenhuma | - |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
