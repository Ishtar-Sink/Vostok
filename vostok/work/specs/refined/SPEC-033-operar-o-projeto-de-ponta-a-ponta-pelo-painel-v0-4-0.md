---
id: SPEC-033
title: "Operar o projeto de ponta a ponta pelo painel (v0.4.0)"
status: implemented
author: "Victor Batalha"
created_at: 2026-11-03T15:35:49Z
updated_at: 2026-11-03T22:52:19Z
tags: []
type: feature
source: "draft/SPEC-033-operar-o-projeto-de-ponta-a-ponta-pelo-painel-v0-4-0.md"
refined_at: 2026-11-03T17:02:00Z
adjustments: 2
adjusted_at: 2026-11-03T17:07:12Z
content_hash: 921a43c210c626df40d808c4dc73f6f3b2ee2e7d58ca88d6cd58962f9ca4bcd3
approved_at: 2026-11-03T17:07:12Z
approved_hash: 921a43c210c626df40d808c4dc73f6f3b2ee2e7d58ca88d6cd58962f9ca4bcd3
approved_by: chat
---

# Operar o projeto de ponta a ponta pelo painel (v0.4.0) - Especificação Refinada

Segunda parte da v0.4.0: operar o projeto inteiro pelo painel, sem interagir com o chat. O agente passa a fazer perguntas pelo painel, criar e refinar specs viram pedidos, os arquivos de comunicação ficam em pastas por estado e são arquivados com a spec, e as lacunas das skills apontadas pelo estudo da SPEC-032 são fechadas. A versão sobe para 0.4.0.

## Objetivo

Permitir que o usuário opere todo o ciclo do projeto pelo painel: criar, refinar, ajustar e aprovar specs, planos e tarefas, executar e finalizar, fazer commits e responder às perguntas do agente, sem abrir o chat. Hoje, toda skill que pergunta faz o pedido ser recusado, e os arquivos de pedidos, perguntas e avais só se acumulam, o que confunde o usuário e gasta contexto do agente (estudo `docs/estudos/painel-ponta-a-ponta.md`).

## Escopo

### Dentro do escopo

- Perguntas do agente pelo painel: `vostok perguntas abrir`, a `process-requests` abrindo pergunta em vez de recusar e as skills de conversa usando esse destino quando rodam por um pedido.
- Pedidos novos: `criar-spec`, `refinar-spec` e `executar-onda`, com os botões no painel.
- Conferência de prints pelo painel e liberação ou desfazimento dos commits apontados pelo `vostok commit check`.
- Pastas por estado para pedidos e perguntas, com a transição por `rename`.
- Avais num arquivo por spec, consultados pelo `vostok avais <ID>`.
- Pedidos, perguntas e avais arquivados junto com a spec pelo `vostok finalize archive`.
- `vostok pedidos prune`, também no painel, com a lista e a confirmação.
- Histórico de status das specs e dos pedidos, alimentando a `GET /api/atividades`.
- Disparo sob demanda do agente pelo painel, desligado por padrão.
- Os detalhes visuais do quadro registrados no DT-017.
- A versão 0.4.0.

### Fora do escopo

- Chat dentro do painel (avaliado no estudo e descartado na v0.4.0).
- Skills do próprio harness (report-harness-bug, suggest-harness-*, validate-harness-bug, create-skill) e as de configuração inicial (init-harness, adjust-rules, customize-documents).
- O aviso das tarefas no formato antigo (DT-015).
- Servidor, banco de dados ou MCP (DEC-003).

## Requisitos funcionais

| ID | Requisito |
|---|---|
| RF-01 | `vostok perguntas abrir <PEDIDO>` deve gravar uma pergunta `P-NNN.md` na subpasta de perguntas abertas, com o texto pela entrada padrão (perguntas e opções no formato do template `question`), o pedido de origem, a spec e a tarefa. |
| RF-02 | Quando a skill de um pedido para com uma pergunta, a `process-requests` deve abrir a pergunta e concluir o pedido com o resultado "aguardando a pergunta P-NNN", em vez de recusá-lo. |
| RF-03 | A ação `responder-pergunta` do painel deve gravar a resposta na pergunta, movê-la para a subpasta de respondidas e criar o pedido de resposta; a `process-requests` atende esse pedido rodando de novo a skill do pedido de origem, com o texto original e as respostas. |
| RF-04 | As skills de conversa (create-spec, refine-spec, adjust-spec, adjust-plan, adjust-task), quando rodam por um pedido, devem gravar a rodada de perguntas pelo `vostok perguntas abrir` em vez de perguntar no chat. |
| RF-05 | O painel deve ter os pedidos `criar-spec` (com o texto do assunto), `refinar-spec` e `executar-onda` (com a spec e a onda), com os botões "Nova spec" na lista de specs, "Refinar" na spec em rascunho e "Executar onda" na spec planejada. |
| RF-06 | O painel deve permitir conferir os prints de uma tarefa (aceitar ou rejeitar), gravando a conferência como o `vostok task evidence --reviewed` e a origem `painel`. |
| RF-07 | O painel deve permitir liberar ou desfazer os commits apontados pelo `vostok commit check`, pelos pedidos `liberar-commits` e `desfazer-commits` à skill `approve-commit`, com aval; desfazer pede confirmação na tela. |
| RF-08 | Pedidos devem ficar numa subpasta por estado (pendente, em andamento, feito, recusado) e perguntas numa subpasta por status (aberta, respondida); mudar de estado é mover o arquivo. |
| RF-09 | Os avais devem ficar num arquivo `<SPEC-NNN>.log` por spec na pasta de avais, só de acréscimo, e os sem spec num arquivo à parte; `vostok avais <ID>` mostra os avais de uma spec ou tarefa. |
| RF-10 | `vostok finalize archive` deve mover para o arquivo da spec os pedidos, as perguntas e o arquivo de avais dela. |
| RF-11 | `vostok pedidos prune` deve listar os pedidos e as perguntas arquivados e os concluídos sem spec com mais de 30 dias, e só apagá-los com `--confirmar`; o painel oferece a mesma limpeza nas Configurações, com a lista e a confirmação na tela. |
| RF-12 | As mudanças de status da spec (approved, planned, implemented, archived) e as transições de cada pedido devem ganhar uma linha de histórico, gravada pelos comandos que já as fazem, e aparecer na `GET /api/atividades`. |
| RF-13 | Com `request_dispatch: codex` ou `claude` em `config/main.yml`, o `vostok painel` deve disparar o agente para cada pedido gravado, um por vez, como o `vostok pedidos vigiar`; vazio (padrão) não dispara nada. |
| RF-14 | O quadro deve mostrar as seis colunas numa tela de 1440 px, os títulos com aspas sem barras de escape e a coluna "Em validação" com cor de status (DT-017). |
| RF-15 | A versão da CLI deve subir para 0.4.0. |
| RF-16 | As pastas de pedidos, perguntas e avais, os nomes das subpastas de cada estado e o nome do arquivo de avais sem spec devem vir de `config/main.yml` (`paths.requests`, `paths.questions`, `paths.approvals` e a seção `folders`), com os nomes em português como padrão; a CLI, o painel e as skills usam só a configuração. |

## Requisitos não funcionais

| ID | Requisito | Como verificar |
|---|---|---|
| RNF-01 | `vostok pedidos listar pendente` lê só a pasta `pendente/`. | Teste com pedidos nos quatro estados, contando os arquivos lidos ou conferindo que os outros não aparecem. |
| RNF-02 | O agente nunca precisa abrir os arquivos de avais: a consulta é pelo `vostok avais`. | Regra no contexto e teste do comando. |
| RNF-03 | O `prune` nunca apaga avais nem nada sem `--confirmar`. | Teste do comando sem e com a opção. |
| RNF-04 | Botões, diálogos de pergunta e confirmações acessíveis por teclado, com rótulo. | Testes do frontend com `getByRole`. |

## Regras de negócio

| ID | Regra |
|---|---|
| RN-01 | O painel é o usuário: uma resposta, uma conferência de print ou uma confirmação feita no painel vale como a do chat, com a origem `painel`. |
| RN-02 | Pedidos que mudam o código ou o histórico (finalizar, aprovar plano, commit, liberar e desfazer commits) continuam exigindo aval (trava da SPEC-030). |
| RN-03 | O limite de 2 rodadas da refine-spec vale também pelo painel, contando as perguntas abertas. |
| RN-04 | O disparo sob demanda nunca roda dois agentes ao mesmo tempo. |
| RN-05 | Avais são só de acréscimo: mover o arquivo inteiro para o arquivo morto não reescreve linhas. |
| RN-06 | O nome de uma pasta é só apresentação: o estado gravado no front-matter não muda com a configuração, então trocar os nomes não muda o significado dos arquivos. |

## Fluxos

### Fluxo principal

1. O usuário clica em "Nova spec" e escreve o assunto; o painel grava o pedido `criar-spec` em `pedidos/pendente/`.
2. O agente em loop (ou o disparo sob demanda) atende o pedido pela `process-requests`, que roda a create-spec.
3. O usuário clica em "Refinar"; a refine-spec grava a rodada de perguntas com `vostok perguntas abrir` e o pedido é concluído aguardando a pergunta.
4. A pergunta aparece em "Aguardando você"; o usuário responde no painel e o pedido de resposta é criado.
5. O agente roda de novo a refine-spec com as respostas e gera a refinada.
6. O usuário segue pelo painel: aprova, planeja, aprova o plano e as tarefas, executa ondas, confere prints, finaliza tarefas e a spec e faz os commits.
7. Ao arquivar a spec, os pedidos, as perguntas e os avais dela vão para o arquivo morto.

### Fluxos alternativos

- FA-01: nenhum agente rodando e disparo desligado: o pedido fica em `pendente/` e o painel mostra que aguarda o agente.
- FA-02: o usuário rejeita um print: a tarefa fica bloqueada, como na finish-task.
- FA-03: a segunda rodada de perguntas da refine-spec ainda deixa dúvidas: elas vão para as pendências da refinada (RN-03).
- FA-04: projeto com pedidos ou `avais.log` no formato antigo: a primeira execução da versão nova move os pedidos para as pastas de estado e divide o `avais.log` por spec, uma vez.

## Casos de borda

| ID | Situação | Comportamento esperado | Requisitos |
|---|---|---|---|
| CB-01 | Pergunta respondida depois que o pedido de origem foi recusado | O pedido de resposta é recusado com o motivo | RF-03 |
| CB-02 | Pedido sem spec (um commit avulso) | Fica na pasta de trabalho até o `prune`, depois de 30 dias concluído | RF-10, RF-11 |
| CB-03 | Aval de uma linha malformada no `avais.log` antigo | Vai para `sem-spec.log` sem quebrar a migração | RF-09 |
| CB-04 | Disparo ligado com o agente fora do PATH | O painel avisa e não grava o pedido como recusado | RF-13 |
| CB-05 | `prune` sem nada a apagar | Diz que não há o que limpar | RF-11 |
| CB-06 | Duas abas do painel respondendo à mesma pergunta | A segunda recebe 409 e a pergunta mantém a primeira resposta | RF-03 |
| CB-07 | Nomes de pasta trocados na configuração com arquivos já gravados | A primeira execução move os arquivos para as pastas novas, como na migração do formato antigo | RF-16 |
| CB-08 | Dois estados configurados com o mesmo nome de pasta | O `vostok` recusa a configuração e diz quais estados colidem | RF-16 |

## Contratos e interfaces

```text
vostok perguntas abrir PED-004 <<EOF   # grava <questions>/<open>/P-001.md
vostok avais SPEC-033                   # avais da spec, mais novos primeiro
vostok pedidos prune [--confirmar]
```

```yaml
# <questions>/<open>/P-001.md (front-matter)
id: P-001
pedido: PED-004
spec: SPEC-033
task: ""
status: aberta        # aberta | respondida
created_at: 2026-11-04T10:00:00Z
```

```yaml
# config/main.yml (valores padrão)
paths:
  requests: ./work/pedidos
  questions: ./work/perguntas
  approvals: ./work/avais
folders:
  requests:                # subpasta de cada estado do pedido
    pending: pendente
    in_progress: em_andamento
    done: feito
    refused: recusado
  questions:               # subpasta de cada status da pergunta
    open: aberta
    answered: respondida
  approvals_without_spec: sem-spec.log
request_dispatch: ""       # "" (desligado) | codex | claude
```

Com o padrão, as pastas ficam `work/pedidos/{pendente,em_andamento,feito,recusado}/`, `work/perguntas/{aberta,respondida}/`, `work/avais/<SPEC-NNN>.log` e `work/avais/sem-spec.log`; um projeto em inglês pode usar, por exemplo, `requests/{pending,in_progress,done,refused}/`. Ações novas de pedido: `criar-spec`, `refinar-spec`, `executar-onda`, `liberar-commits`, `desfazer-commits`; ações diretas: `conferir-print`, `limpar-pedidos`.

## Componentes afetados

| Caminho | Ação | Motivo |
|---|---|---|
| `apps/cli/src/commands/perguntas.rs` | criar | `vostok perguntas abrir` (RF-01) |
| `apps/cli/src/commands/pedidos.rs` | alterar | Pastas por estado, histórico, `prune` e disparo reaproveitado (RF-08, RF-11, RF-12, RF-13) |
| `apps/cli/src/aval.rs` | alterar | Avais por spec, migração e `vostok avais` (RF-09) |
| `apps/cli/src/commands/finalize.rs` | alterar | Arquivar pedidos, perguntas e avais com a spec (RF-10) |
| `apps/cli/src/commands/spec.rs` | alterar | Linha de histórico nas mudanças de status (RF-12) |
| `apps/cli/src/commands/mod.rs` | alterar | Registro dos comandos novos |
| `apps/cli/src/painel/actions.rs` | alterar | Ações novas, conferência de print, limpeza e disparo (RF-05, RF-06, RF-07, RF-11, RF-13) |
| `apps/cli/src/painel/read.rs` | alterar | Leitura das pastas por estado e histórico nas atividades (RF-08, RF-12) |
| `apps/cli/Cargo.toml` | alterar | Versão 0.4.0 (RF-15) |
| `packages/harness/skills/default/process-requests/SKILL.md` | alterar | Abrir pergunta em vez de recusar (RF-02, RF-03) |
| `packages/harness/skills/default/create-spec/SKILL.md` | alterar | Destino das perguntas por pedido (RF-04) |
| `packages/harness/skills/default/refine-spec/SKILL.md` | alterar | Destino das perguntas por pedido (RF-04) |
| `packages/harness/skills/default/adjust-spec/SKILL.md` | alterar | Destino das perguntas por pedido (RF-04) |
| `packages/harness/skills/default/adjust-plan/SKILL.md` | alterar | Destino das perguntas por pedido (RF-04) |
| `packages/harness/skills/default/adjust-task/SKILL.md` | alterar | Destino das perguntas por pedido (RF-04) |
| `packages/harness/skills/default/finish-task/SKILL.md` | alterar | Conferência de print feita no painel (RF-06) |
| `packages/harness/skills/default/approve-commit/SKILL.md` | alterar | Liberar e desfazer por pedido (RF-07) |
| `packages/harness/config/main.yml` | alterar | Chave `request_dispatch` e os caminhos e nomes de pasta configuráveis (RF-13, RF-16) |
| `apps/cli/src/harness.rs` | alterar | Leitura e validação de `paths.requests`, `paths.questions`, `paths.approvals` e `folders` (RF-16) |
| `apps/painel/src/features/specs/specs-page.tsx` | alterar | Botão "Nova spec" (RF-05) |
| `apps/painel/src/features/spec/spec-page.tsx` | alterar | Botões "Refinar" e "Executar onda" (RF-05) |
| `apps/painel/src/features/task/task-page.tsx` | alterar | Conferência de prints (RF-06) |
| `apps/painel/src/features/inbox/inbox-page.tsx` | alterar | Perguntas e commits apontados (RF-03, RF-07) |
| `apps/painel/src/features/settings/settings-page.tsx` | alterar | "Limpar pedidos arquivados" (RF-11) |
| `apps/painel/src/features/board/board.tsx` | alterar | Título sem barras de escape (RF-14) |
| `apps/painel/src/features/board/board.module.css` | alterar | Seis colunas a 1440 px e cor de "Em validação" (RF-14) |
| `apps/painel/src/features/board/board.test.tsx` | alterar | Teste do título com aspas (RF-14) |
| `apps/painel/src/shared/attention.ts` | alterar | Ações novas (RF-05, RF-07) |

## Premissas e dependências

- O estudo `docs/estudos/painel-ponta-a-ponta.md` (SPEC-032) é a base das decisões.
- Um agente roda em loop (`/process-requests` no Claude Code ou `vostok pedidos vigiar` no Codex), ou o disparo sob demanda está ligado.
- O disparo com Claude usa `claude -p` em modo não interativo, como o `codex exec` do `vigiar`.
- As pastas `pedidos/` e `perguntas/` estão vazias neste repositório e no app da RD; o `avais.log` existente precisa de migração.

## Riscos

| Risco | Impacto | Mitigação |
|---|---|---|
| O disparo sob demanda roda um agente com permissão de escrita sozinho | Mudanças sem o usuário acompanhar | Desligado por padrão, um agente por vez, e a trava de aval continua valendo |
| Migração do `avais.log` atribuir um aval à spec errada | A trava aceita ou recusa errado | A spec vem do ID do aval; linhas sem spec vão para `sem-spec.log`; teste com o log real do ai-hub |
| `prune` apaga o histórico de atividades | Perda de rastro do que aconteceu | Lista antes, só apaga com confirmação e nunca apaga avais |
| Skills de conversa com dois destinos de pergunta | Uma skill pergunta no chat durante um pedido e trava o loop | Regra única na `process-requests` e teste de cenário por skill |

## Casos de teste

| ID | Cenário | Passos | Resultado esperado | Requisitos |
|---|---|---|---|---|
| CT-01 | Abrir pergunta | `vostok perguntas abrir PED-001` com o texto | `perguntas/aberta/P-001.md` com o pedido, a spec e `status: aberta` | RF-01 |
| CT-02 | Pedido que pergunta | Processar um pedido cuja skill pergunta | O pedido fica `feito` com "aguardando a pergunta P-NNN" | RF-02 |
| CT-03 | Responder pergunta | Responder no painel | A pergunta vai para `respondida/` e um pedido de resposta é criado | RF-03 |
| CT-04 | Skill de conversa por pedido | Rodar a refine-spec por pedido | As perguntas são gravadas, não feitas no chat | RF-04 |
| CT-05 | Nova spec e refinar | Clicar em "Nova spec" e em "Refinar" | Pedidos `criar-spec` e `refinar-spec` gravados | RF-05 |
| CT-06 | Executar onda | Clicar em "Executar onda" | Pedido `executar-onda` com a spec e a onda | RF-05 |
| CT-07 | Conferir print | Aceitar um print no painel | A evidência fica conferida com a origem `painel` | RF-06 |
| CT-08 | Liberar e desfazer commits | Pedir as duas ações no painel | Pedidos com aval; desfazer pede confirmação | RF-07 |
| CT-09 | Pastas por estado | Iniciar e concluir um pedido | O arquivo passa de `pendente/` a `em_andamento/` e a `feito/` | RF-08, RNF-01 |
| CT-10 | Avais por spec | Aprovar uma tarefa e rodar `vostok avais SPEC-NNN` | O aval está em `avais/SPEC-NNN.log` e aparece no comando | RF-09, RNF-02 |
| CT-11 | Arquivar com a spec | `vostok finalize archive` | Pedidos, perguntas e avais da spec no arquivo morto | RF-10 |
| CT-12 | Prune | Rodar sem e com `--confirmar`, e pelo painel | Sem a opção só lista; com ela apaga; avais ficam | RF-11, RNF-03 |
| CT-13 | Histórico de status | Aprovar uma spec e concluir um pedido | Linhas novas no histórico e na `GET /api/atividades` | RF-12 |
| CT-14 | Disparo | Ligar `request_dispatch` e gravar um pedido | O agente é chamado uma vez; desligado, não é | RF-13 |
| CT-15 | Quadro | Print a 1440x900 | Seis colunas visíveis, título sem barras e cor em "Em validação" | RF-14 |
| CT-16 | Versão | `vostok --version` | `vostok 0.4.0` | RF-15 |
| CT-17 | Migração | Rodar a versão nova num projeto com o formato antigo | Pedidos nas pastas de estado e `avais.log` dividido por spec | RF-08, RF-09 |
| CT-19 | Pastas configuráveis | Trocar os nomes em `folders` e `paths` para inglês e criar, iniciar e concluir um pedido, abrir e responder uma pergunta e aprovar uma tarefa | Os arquivos vão para as pastas configuradas; o painel e o `vostok avais` os leem | RF-16 |
| CT-20 | Nomes repetidos | Configurar dois estados com a mesma pasta | O `vostok` recusa e cita os estados | RF-16 |
| CT-18 | Acessibilidade | Testes dos botões, do diálogo de pergunta e das confirmações | Todos achados por papel e rótulo | RNF-04 |

## Critérios de aceite

- [ ] CA-01: uma spec pode ser criada e refinada só pelo painel, com as perguntas do agente respondidas lá (RF-01, RF-02, RF-03, RF-04, RF-05).
- [ ] CA-02: prints são conferidos e commits liberados ou desfeitos pelo painel (RF-06, RF-07).
- [ ] CA-03: pedidos e perguntas ficam em pastas por estado, e os avais num arquivo por spec, consultados pelo `vostok avais` (RF-08, RF-09).
- [ ] CA-04: arquivar a spec leva os pedidos, as perguntas e os avais dela, e o `prune` limpa os arquivados com confirmação, também pelo painel (RF-10, RF-11).
- [ ] CA-05: as mudanças de status das specs e dos pedidos aparecem nas atividades (RF-12).
- [ ] CA-06: com o disparo ligado, um pedido é atendido sem loop rodando; desligado, nada é disparado (RF-13).
- [ ] CA-07: o quadro mostra as seis colunas a 1440 px, sem barras nos títulos e com cor em "Em validação" (RF-14).
- [ ] CA-08: `vostok --version` responde `vostok 0.4.0` (RF-15).
- [ ] CA-09: com os nomes de pasta trocados na configuração, pedidos, perguntas e avais vão para as pastas configuradas e o painel continua funcionando (RF-16).

## Referências

| Tipo | Referência | Uso |
|---|---|---|
| Estudo | `docs/estudos/painel-ponta-a-ponta.md` | Técnicas, processos, status, perguntas, arquivos e a proposta desta spec |
| Decisão | DEC-003 | Painel local, arquivos como banco e pedidos em arquivo |
| Código | `apps/cli/src/commands/pedidos.rs` | Listagem, transições e o `vigiar`, base do disparo |
| Código | `apps/cli/src/painel/actions.rs` | `REQUEST_ACTIONS` e `LOCKED_ACTIONS` atuais |
| Código | `apps/cli/src/aval.rs` | Leitura do `avais.log` e da `approval_lock` |
| Código | `apps/cli/src/painel/read.rs` | Leitura de perguntas e a `GET /api/atividades` |
| Código | `apps/cli/src/commands/finalize.rs` | `fn archive`, que passa a levar pedidos, perguntas e avais |
| Débito | `docs/debitos-tecnicos.md` (DT-017) | Detalhes visuais do quadro |
| Spec | SPEC-032 | Primeira parte da v0.4.0 e a origem do estudo |

## O que mudou

| # | Assunto | Estado no Original | Estado Refinado |
|---|---|---|---|
| 1 | Técnica | Pedido em loop preferido; chat no painel a avaliar | Pedido em loop como base e disparo sob demanda opcional; chat fora |
| 2 | Arquivos de comunicação | Pastas por status como ideia | Pastas por estado, avais por spec, arquivamento com a spec e `prune` |
| 3 | Processos | Todos os do agente, sem os do harness e de configuração | Os mesmos, com as lacunas do estudo: perguntas, prints, commits e execução por onda |
| 4 | Status | Os que o estudo apontar | Histórico das specs e dos pedidos |
| 5 | Débitos | Não citados | DT-017 entra; DT-015 fica fora |
| 6 | Nomes das pastas | Fixos em português | Configuráveis em `config/main.yml`, com o português como padrão |

## Registro de decisões

| # | Decisão | Alternativa descartada | Motivo |
|---|---|---|---|
| 1 | Disparo sob demanda entra, desligado por padrão | Ficar só com o loop | Resposta do usuário; elimina a espera e o custo ocioso do loop |
| 2 | Chat dentro do painel fica fora da v0.4.0 | Fazer o chat agora | Recomendação do estudo: é o mais caro e o loop basta |
| 3 | DT-017 entra; DT-015 fica fora | Incluir os dois ou nenhum | Resposta do usuário |
| 4 | Uma spec só | Dividir em duas | Resposta do usuário; o plano separa em ondas |
| 5 | Pastas por estado com transição por `rename` | Índice ou leitura de todos os arquivos | Inferência do estudo: o estado fica visível no caminho e a listagem lê só a pasta |
| 6 | Avais por spec, arquivados com ela | Um arquivo por mês | Inferência do estudo, pedida pelo usuário: o agente nunca abre mais que os avais de uma spec |
| 7 | `prune` com 30 dias para pedidos sem spec e confirmação obrigatória | Apagar sem confirmação ou por idade sempre | Inferência do estudo: é destrutivo e o arquivo morto fica fora do git |
| 8 | Migração única do formato antigo na primeira execução | Manter os dois formatos | Inferência: a v0.1.x está em uso no app da RD, e os formatos antigos precisam continuar abrindo |
| 9 | A conferência de print pelo painel vale como a do usuário | Manter a conferência só no chat | Inferência: o painel é o usuário (RN-01); muda a regra [finish-task-09] |
| 10 | Pastas, subpastas por estado e arquivo de avais sem spec configuráveis, com o português como padrão | Nomes fixos em português | Pedido do usuário: o harness pode ser usado em outro idioma |
| 12 | Duas ações de pedido, `liberar-commits` e `desfazer-commits`, com confirmação na tela para desfazer | Uma ação só com a opção | Resposta do usuário à pendência 1 |
| 11 | O estado no front-matter continua com os valores atuais, e só o nome da pasta é configurável | Traduzir também os valores de estado | Inferência: mantém a compatibilidade com os pedidos já gravados e com o painel da v0.3 |

## Pendências

| # | Pendência | Opções em aberto |
|---|---|---|
| - | Nenhuma | - |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
| 1 | 2026-11-03 | agente | Nova RF-16: pastas de pedidos, perguntas e avais, subpastas por estado e arquivo de avais sem spec configuráveis em config/main.yml, com português como padrão; RF-01, RF-03, RF-08 e RF-09 sem nomes fixos; nova RN-06; CB-07, CB-08, CT-19, CT-20 e CA-09; contrato da configuração; harness.rs em componentes; decisões 10 e 11 |
| 2 | 2026-11-03 | agente | Pendência 1 resolvida: duas ações, liberar-commits e desfazer-commits, citadas no RF-07; decisão 12 |
