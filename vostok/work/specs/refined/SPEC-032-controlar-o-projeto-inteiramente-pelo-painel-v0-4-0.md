---
id: SPEC-032
title: "Controlar o projeto inteiramente pelo painel (v0.4.0)"
status: planned
author: "Victor Batalha"
created_at: 2026-11-03T15:03:32Z
updated_at: 2026-11-03T15:44:28Z
tags: []
type: feature
source: "draft/SPEC-032-controlar-o-projeto-inteiramente-pelo-painel-v0-4-0.md"
refined_at: 2026-11-03T15:31:18Z
adjustments: 0
adjusted_at: null
content_hash: 48652d2c84a0a23bd3cd3ff663cdb6f0fa8b23098420828187c1b4b31e5018b0
approved_at: 2026-11-03T15:37:18Z
approved_hash: 48652d2c84a0a23bd3cd3ff663cdb6f0fa8b23098420828187c1b4b31e5018b0
approved_by: chat
---

# Controlar o projeto inteiramente pelo painel (v0.4.0) - Especificação Refinada

Primeira parte da v0.4.0, que leva o controle do projeto inteiramente para o painel. Esta spec entrega um estudo das técnicas para operar sem o chat e o que já está claro: a coluna "Em validação" no quadro, pendências restritas ao que trava o agente, a atividade recente com seletor e o menu "Atividades" com filtros. A operação de ponta a ponta pelo painel fica para uma spec seguinte, a partir do estudo.

## Objetivo

Hoje é difícil acompanhar pelo painel o que está acontecendo no projeto: a atividade recente só mostra o registro das tarefas abertas, cortado em 20 linhas e com rolagem interna, e as pendências mostram tarefas em execução que não esperam nada do usuário. Ao mesmo tempo, o objetivo da v0.4.0 é operar o projeto inteiramente pelo painel, sem interagir com o chat, o que exige decidir como o agente conversa com o painel. O resultado esperado é um painel que mostra com fidelidade o andamento e as decisões pendentes, e um estudo que define, com medições, como chegar à operação de ponta a ponta.

## Escopo

### Dentro do escopo

- Estudo preliminar, como primeira tarefa, das técnicas para operar o projeto pelo painel sem o chat e dos dados de status e de atividade.
- Coluna "Em validação" no quadro de tarefas.
- Sinal de "implementação conferida" gravado pelo `vostok task check`, e pendência de finalização só com esse sinal.
- Atividade recente do painel inicial com 5 itens por padrão, seletor de 5, 10 e 15 e "Ver todas".
- Menu "Atividades" na sidebar, com todas as atividades e filtros por tipo e por spec.
- Atividades montadas a partir de todos os dados que os arquivos já gravam.

### Fora do escopo

- Operar o projeto de ponta a ponta pelo painel (criar e refinar specs e os demais processos): vai para a spec seguinte, a partir do estudo (decisão 1).
- Gravar as perguntas do agente para o painel, reorganizar os arquivos de comunicação em pastas por status e um chat dentro do painel: decididos no estudo e implementados na spec seguinte.
- Gravar eventos novos só para a atividade: esta spec usa os dados que já existem (RN-03).
- Operar pelo painel as skills do próprio harness (report-harness-bug, suggest-harness-*, validate-harness-bug, create-skill) e as de configuração inicial (init-harness, adjust-rules, customize-documents) (resposta 2).
- Tarefas no formato antigo de arquivo único (DT-015).
- Publicar a v0.4.0: a versão sobe no fim da spec seguinte.

## Requisitos funcionais

| ID | Requisito |
|---|---|
| RF-01 | O estudo deve ser gravado em `docs/estudos/painel-ponta-a-ponta.md` e cobrir: as técnicas para operar sem o chat, com o pedido em loop como caminho preferido e a viabilidade de um chat dentro do painel; o inventário dos processos das skills que o painel deve operar, sem as do harness e as de configuração inicial; como spec, plano, tarefa, pedido e pergunta registram hoje as mudanças de status e onde posicionar as que faltam em fluxos existentes; como o agente grava uma pergunta para o painel; a organização dos arquivos de comunicação, com o custo de leitura de cada opção, incluindo pastas por status; o custo de cada fonte de atividade; e a proposta de escopo da spec seguinte. |
| RF-02 | O quadro de tarefas deve ter a coluna "Em validação", com as tarefas `validating`, entre "Em execução" (só `in_progress`) e "Bloqueadas". |
| RF-03 | O `vostok task check` deve gravar no `estado.yml` da tarefa `check_result` (`passed` ou `failed`) e `checked_at`, e uma linha `check` no registro de execução. O ajuste de uma tarefa em execução e o `vostok task resume` devem limpar `check_result`. |
| RF-04 | A pendência de finalização deve aparecer só para tarefa `in_progress` com `check_result: passed` e todas as dependências concluídas. A tela da tarefa deve continuar oferecendo "Pedir finalização" para tarefa `in_progress` ou `validating`. |
| RF-05 | A atividade recente do painel inicial deve mostrar 5 atividades por padrão, sem rolagem interna, com um seletor de 5, 10 e 15 ao lado do título "Atividade recente" e um link "Ver todas" no fim da lista, que leva a `/atividades`. |
| RF-06 | O painel deve ter a rota `/atividades` e o item "Atividades" na sidebar, com todas as atividades da mais nova para a mais antiga e filtros por tipo (spec, plano, tarefa, pedido e aval) e por spec. |
| RF-07 | A CLI deve montar as atividades a partir dos dados já gravados: registro de execução das tarefas abertas e arquivadas, histórico de ajustes das specs e dos planos, pedidos (criação e último estado) e `avais.log`, e servi-las em `GET /api/atividades` com `limite`, `tipo` e `spec`. |

## Requisitos não funcionais

| ID | Requisito | Como verificar |
|---|---|---|
| RNF-01 | `GET /api/atividades?limite=15` responde em até 500 ms num projeto com 1.000 atividades. | Teste com fixture de 1.000 linhas, medindo o tempo da rota. |
| RNF-02 | O painel inicial pede só as atividades que mostra: no máximo 15 por requisição. | Teste do frontend que confere o `limite` da chamada. |
| RNF-03 | Seletor e filtros acessíveis por teclado, com rótulo. | Teste do frontend com `getByRole` e `getByLabelText`. |
| RNF-04 | O estudo traz medições (contagens de arquivos lidos, tempos ou tamanhos), não só opiniões, para cada opção comparada. | Revisão do estudo pelo usuário. |

## Regras de negócio

| ID | Regra |
|---|---|
| RN-01 | Pendência é o que trava o agente e precisa de decisão do usuário. Tarefa em execução sem a conferência passada não é pendência. |
| RN-02 | `check_result` e `checked_at` só são gravados pelo `vostok`: o `estado.yml` continua fora do alcance da edição pelo painel. |
| RN-03 | As atividades desta spec vêm só de dados que os arquivos já gravam; nenhum evento novo é gravado só para a atividade. |
| RN-04 | Os tipos de atividade são spec, plano, tarefa, pedido e aval. |

## Fluxos

### Fluxo principal

1. O agente executa a tarefa e roda `vostok task check`, que grava `check_result: passed` e a linha `check` no registro.
2. O painel recebe o aviso de mudança, mostra a tarefa em "Em execução" e a inclui nas pendências como pronta para finalizar.
3. O usuário pede a finalização; o agente roda `vostok task tests`, que leva a tarefa a `validating`, e o quadro a mostra em "Em validação".
4. Cada passo aparece na atividade recente do painel inicial, com as 5 mais novas, e no menu "Atividades", filtrável por tipo e por spec.

### Fluxos alternativos

- **FA-01 (passo 1):** a conferência falha; `check_result: failed`, a tarefa não vira pendência e o agente corrige.
- **FA-02 (passo 2):** a tarefa é ajustada depois da conferência; `check_result` é limpo e ela sai das pendências até uma nova conferência.
- **FA-03 (passo 4):** o usuário escolhe 10 ou 15 no seletor; a lista recarrega com essa quantidade, ainda sem rolagem interna.

## Casos de borda

| ID | Situação | Comportamento esperado | Requisitos |
|---|---|---|---|
| CB-01 | Tarefa `in_progress` sem `check_result` (iniciada antes desta versão) | Não aparece nas pendências; a tela da tarefa mantém "Pedir finalização" | RF-03, RF-04 |
| CB-02 | Menos de 5 atividades no projeto | Mostra as que há, com o seletor e o "Ver todas" | RF-05 |
| CB-03 | Linha malformada no `avais.log` | Ignorada na atividade, sem derrubar a rota | RF-07 |
| CB-04 | Spec arquivada | As atividades das tarefas arquivadas aparecem em `/atividades` | RF-06, RF-07 |
| CB-05 | Filtro sem resultado | Mensagem de lista vazia, com o filtro ainda visível | RF-06 |

## Contratos e interfaces

```yaml
# estado.yml da tarefa, gravado pelo vostok task check
check_result: passed   # passed | failed
checked_at: 2026-11-03T15:00:00Z
```

```text
GET /api/atividades?limite=15&tipo=tarefa&spec=SPEC-032
[{"date":"2026-11-03","type":"tarefa","spec":"SPEC-032","target":"TASK-01","step":"check","result":"passed","detail":"..."}]
```

Rota do painel: `/atividades`; item da sidebar "Atividades".

## Componentes afetados

| Caminho | Ação | Motivo |
|---|---|---|
| `docs/estudos/painel-ponta-a-ponta.md` | criar | Estudo preliminar (RF-01) |
| `apps/painel/src/shared/status.ts` | alterar | Coluna "Em validação" em `BOARD_COLUMNS` (RF-02) |
| `apps/cli/src/commands/task.rs` | alterar | `check_result` e `checked_at` no `vostok task check`, limpos no ajuste e no resume (RF-03) |
| `apps/painel/src/shared/attention.ts` | alterar | Pendência de finalização só com a conferência passada (RF-04) |
| `apps/painel/src/features/dashboard/dashboard-page.tsx` | alterar | Atividade recente com 5, seletor e "Ver todas" (RF-05) |
| `apps/painel/src/features/activity/activity-page.tsx` | criar | Página `/atividades` com filtros (RF-06) |
| `apps/painel/src/shared/router.ts` | alterar | Rota `/atividades` (RF-06) |
| `apps/painel/src/layout/sidebar.tsx` | alterar | Item "Atividades" (RF-06) |
| `apps/painel/src/shared/api.ts` | alterar | Chamada a `/api/atividades` (RF-05, RF-06) |
| `apps/cli/src/painel/read.rs` | alterar | Fontes de atividade e rota `GET /api/atividades` (RF-07) |
| `apps/cli/src/painel/server.rs` | alterar | Roteamento de `/api/atividades` (RF-07) |

## Premissas e dependências

- O `vostok task check` já é rodado pela `execute-task` ao fim da implementação, então o sinal aparece sem mudar a skill.
- Os pedidos guardam só o último estado (`atualizado_em`); a atividade de um pedido mostra a criação e o estado atual, não cada transição.
- O histórico de ajustes das specs e dos planos tem data em cada linha.

## Riscos

| Risco | Impacto | Mitigação |
|---|---|---|
| O estudo muda o desenho de algo já entregue nesta spec | médio | Esta spec só entrega o que não depende do estudo; o resto vai para a spec seguinte |
| Ler todas as fontes de atividade a cada requisição fica lento | médio | RNF-01 mede com 1.000 atividades; o estudo apura o custo de cada fonte |
| `check_result` velho depois de uma mudança de código sem ajuste formal | baixo | Limpo no ajuste e no resume; a finalização roda os testes de novo de qualquer forma |
| Atividade de pedidos incompleta por guardar só o último estado | baixo | Premissa registrada; o estudo avalia gravar as transições na spec seguinte |

## Casos de teste

| ID | Cenário | Passos | Resultado esperado | Requisitos |
|---|---|---|---|---|
| CT-01 | Estudo | Ler `docs/estudos/painel-ponta-a-ponta.md` | Os sete temas do RF-01 cobertos, com medições, e a proposta da spec seguinte | RF-01 |
| CT-02 | Coluna "Em validação" | Abrir o quadro com uma tarefa `validating` | A tarefa aparece em "Em validação", não em "Em execução" | RF-02 |
| CT-03 | Conferência passada | Rodar `vostok task check` numa tarefa que passa | `check_result: passed`, `checked_at` e a linha `check` no registro | RF-03 |
| CT-04 | Conferência falha | Rodar `vostok task check` numa tarefa que falha | `check_result: failed` | RF-03 |
| CT-05 | Ajuste limpa a conferência | Ajustar uma tarefa em execução já conferida | `check_result` some do `estado.yml` | RF-03 |
| CT-06 | Pendência só com conferência | Ter uma tarefa `in_progress` sem e outra com `check_result: passed` | Só a segunda aparece nas pendências; as duas têm "Pedir finalização" na tela da tarefa | RF-04 |
| CT-07 | Atividade recente | Abrir o painel inicial com 20 atividades e trocar o seletor | 5, depois 10 e 15 itens, sem rolagem interna, e "Ver todas" leva a `/atividades` | RF-05 |
| CT-08 | Menu "Atividades" | Abrir `/atividades` e filtrar por tipo e por spec | Lista da mais nova para a mais antiga, filtrada | RF-06 |
| CT-09 | Fontes de atividade | Projeto com registro, ajustes de spec e plano, pedido e aval | `GET /api/atividades` traz os cinco tipos | RF-07 |
| CT-10 | Desempenho | Fixture com 1.000 atividades | `limite=15` em até 500 ms | RF-07 |

## Critérios de aceite

- [ ] CA-01: o estudo está publicado em `docs/estudos/` com os sete temas e a proposta da spec seguinte, revisado pelo usuário (RF-01).
- [ ] CA-02: o quadro separa "Em execução" de "Em validação" (RF-02).
- [ ] CA-03: uma tarefa em execução só aparece nas pendências depois de passar no `vostok task check` (RF-03, RF-04).
- [ ] CA-04: a atividade recente mostra 5 itens por padrão, troca para 10 e 15 e leva a `/atividades` (RF-05).
- [ ] CA-05: o menu "Atividades" lista specs, planos, tarefas, pedidos e avais, com filtros por tipo e por spec (RF-06, RF-07).

## Referências

| Tipo | Referência | Uso |
|---|---|---|
| arquivo | `apps/painel/src/shared/status.ts` | `BOARD_COLUMNS` junta `in_progress` e `validating` |
| arquivo | `apps/painel/src/shared/attention.ts` | Regra atual da pendência de finalização |
| arquivo | `apps/cli/src/painel/read.rs` | Atividade só do registro das tarefas abertas, `ACTIVITY_LIMIT` de 20; leitura de `work/perguntas/` |
| arquivo | `apps/cli/src/commands/pedidos.rs` | `vostok pedidos` lê todos os arquivos a cada rodada |
| arquivo | `apps/cli/src/painel/actions.rs` | As 11 ações de pedido e as 4 travadas |
| arquivo | `apps/cli/src/commands/task.rs` | `vostok task check` grava só `validation/check.out`, sem resultado |
| memória | DEC-003 | Painel local, arquivos como banco e pedidos processados por um chat em loop |
| arquivo | `docs/debitos-tecnicos.md` | DT-015, tarefas no formato antigo |

## O que mudou

| # | Assunto | Estado no Original | Estado Refinado |
|---|---|---|---|
| 1 | Divisão da v0.4.0 | Uma spec com tudo | Esta spec entrega o estudo e os itens claros; a operação de ponta a ponta vai para a seguinte |
| 2 | Processos a operar pelo painel | Todos os que se fazem pelo agente | Sem as skills do harness e as de configuração inicial |
| 3 | Pendência de tarefa em execução | Aviso constante | Só depois de passar no `vostok task check` |
| 4 | Entidades | "Todas" | Spec, plano, tarefa, pedido e pergunta; aval como evento |
| 5 | Atividade recente | Inclui tudo se já houver os dados, a depender do custo | Todos os dados já gravados; o estudo mede o custo |

## Registro de decisões

| # | Decisão | Alternativa descartada | Motivo |
|---|---|---|---|
| 1 | Duas specs: estudo e itens claros agora, operação de ponta a ponta depois | Uma spec só, ajustando as tarefas depois do estudo | resposta 1 |
| 2 | Fora da v0.4.0: skills do harness e de configuração inicial | Todas as 22 skills | resposta 2 |
| 3 | Pendência de finalização a partir do `vostok task check` passado | Sinal gravado pelo agente; decidir no estudo | resposta 3 |
| 4 | Pedido em loop como caminho preferido, com a viabilidade do chat no painel no estudo | Chat no painel como caminho principal | conversa do rascunho |
| 5 | Entidades: spec, plano, tarefa, pedido e pergunta; aval como evento | Aval e commit como entidades com status | inferência A |
| 6 | Filtros por tipo e por spec | Só por tipo | inferência B |
| 7 | Gravar perguntas do agente fica para a spec seguinte | Nesta spec (inferência C original) | consequência da decisão 1 |
| 8 | Sem migração dos arquivos de comunicação | Migrar pedidos e perguntas existentes | inferência D: as pastas estão vazias |
| 9 | Versão 0.4.0 sobe no fim da spec seguinte | Subir nesta spec | inferência E, ajustada pela decisão 1 |
| 10 | Atividades com todos os dados já gravados; o estudo mede o custo | Esperar o estudo para decidir as fontes | inferência F |

## Pendências

| # | Pendência | Opções em aberto |
|---|---|---|
| - | Nenhuma | - |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
