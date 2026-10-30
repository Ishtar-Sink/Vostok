---
name: plan-tasks
description: "Planeja as tarefas de uma especificação (spec) aprovada, com status approved. Cria um plano para revisão em work/plans/, com a visão geral, a tabela de tarefas, as dependências, os IDs cobertos e o nível de esforço sugerido de cada tarefa, calculado pelo vostok. Use quando o usuário pedir para planejar, propor, granularizar ou quebrar em tarefas uma spec aprovada. Não use para ajustar um plano existente (skill adjust-plan) nem para aprová-lo (skill approve-plan)."
allowed-tools: Read Edit Glob Bash(vostok *)
metadata:
  version: "1.0.0"
  author: "Victor Batalha"
  scripts: "vostok context load, vostok spec list, vostok plan create, vostok plan score, vostok plan validate"
  templates: "templates/plan/plan.md"
---

# Planejar tarefas

Propõe, a partir de uma spec aprovada, um plano de tarefas pequenas, ordenadas e rastreáveis, com a persona e o perfil de execução de cada uma. O plano é só um documento de revisão.

## Persona

Esta skill atua como Planejador, carregada(s) no passo 1 pelo `vostok context load`. Ao terminar, inclusive numa parada, o chat volta à persona Coordenador; nenhum estado de persona ativa é gravado.

## Termos

- **Aprovada:** a spec em `refined/` com status `approved`. É a única que pode ser planejada.
- **Plano:** o arquivo `vostok/work/plans/<SPEC_ID>.md`. É removido quando aprovado pela skill `approve-plan`.
- **Plano atualizado:** plano com `spec_hash` igual ao `approved_hash` da spec. Com hash diferente, o plano está **desatualizado** e é recriado.
- **Tarefa:** uma unidade de trabalho entregável sozinha, com ID `TASK-NN` sequencial a partir de `TASK-01`.
- **Fatos de esforço:** o que se sabe sobre a tarefa: arquivos a criar e a alterar, que o `vostok plan score` calcula pelos componentes da tarefa, e se ela muda um contrato público, depende de inferência ou cobre um risco alto, que o agente informa.
- **Perfil:** o que a tarefa exige do modelo (`mecanica`, `padrao` ou `julgamento`), calculado pelo comando a partir dos fatos, pelo que a tarefa faz. Nunca indica um modelo.
- **Persona:** o nome da persona que executa a tarefa, como `desenvolvedor`, ou `-`.
- **Onda:** o número da onda da tarefa, calculado pelo `vostok plan score` a partir das dependências (1 sem dependências; senão, 1 mais a maior onda das dependências). Tarefas da mesma onda podem rodar em paralelo.

## Regras

[plan-tasks-01]: Só planeje specs com status `approved`.
[plan-tasks-02]: Nunca altere um plano atualizado. Um pedido para planejar uma spec que já tem plano atualizado é informado e para; ajustes são feitos pela skill `adjust-plan` só quando o usuário pedir.
[plan-tasks-03]: O planejamento de uma spec nunca altera o plano de outra.
[plan-tasks-04]: Nunca preencha à mão a coluna "Perfil", a coluna "Onda" nem a seção "Arquivos em comum por onda". Só o `vostok plan score` as grava.
[plan-tasks-05]: Nunca edite o front-matter, a seção "Perfis de execução" nem o "Histórico de ajustes" do plano.
[plan-tasks-06]: Todo ID da spec que precisa de cobertura (RF e CT em `feature`, `improvement` e `fix`; PQ em `study`; S em `analysis`) é coberto por pelo menos uma tarefa. CB pode ser citado em "Cobre" como cobertura opcional.
[plan-tasks-07]: Não crie arquivos de tarefa. Eles só existem depois que o usuário aprova o plano (skill `approve-plan`).
[plan-tasks-08]: Indique na coluna "Persona" a persona que executa cada tarefa, pela [tabela de personas](#tabela-de-personas).
[plan-tasks-09]: Divida o trabalho para permitir paralelismo, mesmo que o usuário execute em sequência: dependência só quando uma tarefa precisa do resultado de outra, e tarefas que podem rodar juntas não dividem arquivos sem necessidade (RF-15).

## Checklist

Copie e marque o progresso:

```
- [ ] 1. Carregar o contexto
- [ ] 2. Identificar a spec
- [ ] 3. Criar o plano
- [ ] 4. Preencher o plano
- [ ] 5. Calcular os perfis
- [ ] 6. Validar
- [ ] 7. Responder ao usuário
```

## Passos

### 1. Carregar o contexto

Execute:

```bash
vostok context load --skill plan-tasks --persona planejador
```

### 2. Identificar a spec

Liste as specs aprovadas com `vostok spec list approved` e os planos existentes em `vostok/work/plans/`. Uma spec aprovada é **candidata** se não tem plano ou se o plano dela está desatualizado.

Use a spec sem perguntar em qualquer um destes casos:

- O usuário a cita por ID, título ou arquivo.
- O pedido vem logo depois de a spec ser aprovada nesta conversa.
- Existe uma única candidata, mesmo que outras specs aprovadas já tenham plano.

Nos demais casos:

1. Nenhuma candidata: **PARE** e informe que não há specs aprovadas sem plano atualizado.
2. Mais de uma: **PARE** e pergunte qual planejar, com o template `question`, com uma opção por candidata (ID e título). Aceite também resposta livre. Não crie nada antes da resposta.

Spec citada com outro status: **PARE** e informe o status atual. `draft` ou `refined` precisam ser refinadas e aprovadas antes; `planned` já tem tarefas, e mudá-la exige um novo ciclo.

Spec citada com plano atualizado: **PARE** e informe que o plano já existe, com o caminho, sem alterá-lo. Não inicie um ajuste ([plan-tasks-02]).

### 3. Criar o plano

Execute:

```bash
vostok plan create <SPEC_ID>
```

O comando cria o plano a partir do template e recusa specs que não estão aprovadas ou que mudaram depois da aprovação. Um plano desatualizado é recriado com aviso; informe o aviso ao usuário no passo 7. Se o comando recusar, **PARE** e responda com o template `blocker`, com a mensagem do comando.

### 4. Preencher o plano

Leia a spec aprovada inteira e o plano criado. Substitua as variáveis `#{...}`:

- **`#{PLAN_OVERVIEW}`:** um ou dois parágrafos em prosa: como o trabalho foi dividido, a ordem sugerida e o que pode ser feito em paralelo.
- **`#{PLAN_TASKS}`:** uma linha por tarefa, na [tabela de tarefas](#tabela-de-tarefas).
- **`#{PLAN_COMPONENTS}`:** uma linha por tarefa, na [tabela de componentes](#tabela-de-componentes), com os caminhos de "Componentes afetados" da spec que ela assume, ou `-`. Todo componente da spec fica em pelo menos uma tarefa.
- **`#{PLAN_FACTS}`:** uma linha por tarefa, na [tabela de fatos](#tabela-de-fatos).

Para dividir o trabalho:

- Cada tarefa é entregável e verificável sozinha, com um objetivo único.
- Use os "Componentes afetados", os "Fluxos" e os "Casos de teste" da spec para agrupar mudanças relacionadas. Uma tarefa não mistura partes independentes.
- Prefira mais tarefas simples a poucas complexas. Uma tarefa com muitos arquivos ou muitos IDs cobertos provavelmente deve ser dividida.
- As dependências formam uma ordem sem ciclos: uma tarefa só depende das que precisam existir antes dela.
- Separe as frentes independentes em tarefas sem dependência entre si, para caberem na mesma onda.
- Evite que duas tarefas sem dependência entre si mudem o mesmo arquivo; quando for inevitável, diga na visão geral quais tarefas o Coordenador vai sequenciar.

### 5. Calcular os perfis

Execute:

```bash
vostok plan score <SPEC_ID>
```

O comando grava o perfil de cada tarefa na coluna "Perfil", a onda na coluna "Onda" e, na seção "Arquivos em comum por onda", os arquivos que duas tarefas da mesma onda assumem (essas tarefas devem rodar em sequência). Se ele recusar por fato inválido, corrija a tabela de fatos e execute de novo.

### 6. Validar

Execute:

```bash
vostok plan validate <SPEC_ID>
```

O comando lista os problemas sem alterar nada, como "componente da spec sem tarefa". Corrija-os, execute o `vostok plan score` de novo se mudou fatos ou tarefas e valide outra vez. Se falhar pela segunda vez, **PARE** e responda com o template `blocker`.

### 7. Responder ao usuário

Responda com o template `completion`:

- **O que foi feito:** o caminho do plano e o [resumo do plano](#resumo-do-plano). Se o plano foi recriado, informe o aviso do comando.
- **Verificação:** os comandos executados e o resultado.
- **Pendências:** informe que o plano pode ser ajustado (skill `adjust-plan`) ou aprovado (skill `approve-plan`), e que o perfil é a sugestão de modelo da tarefa.

Esta resposta encerra a persona da skill: o chat volta à persona Coordenador.

## Formatos de resposta

Respostas ao usuário usam os templates de `templates/responses/`. Formatos próprios desta skill:

### Tabela de tarefas

`Tipo` é `tarefa`, `historia` ou `bug`. Em "Persona", o nome da persona que executa a tarefa (como `desenvolvedor`) ou `-`. Sem dependências, use `-`. Deixe `-` em "Perfil" e em "Onda": o `vostok plan score` os preenche.

```markdown
| TASK-01 | Título curto | O que a tarefa entrega, em uma frase. | tarefa | desenvolvedor | - | - | - | RF-01, CT-01 |
| TASK-02 | Título curto | O que a tarefa entrega, em uma frase. | historia | desenvolvedor | - | TASK-01 | - | RF-02, CT-02 |
```

### Tabela de personas

```markdown
| Persona | Quando usar |
|---|---|
| desenvolvedor | Implementação de código, configuração e testes. Padrão. |
| ui-ux-designer | Tarefa com interface: telas, componentes visuais ou textos de interface. |
| arquiteto | Estudo, desenho ou documento técnico, sem código de produção. |
| qa | Tarefa só de testes, validação ou evidências. |
```

### Tabela de componentes

Uma linha por tarefa, com os caminhos separados por vírgula, ou `-` quando a tarefa não assume componente da spec.

```markdown
| TASK-01 | `src/a.rs`, `src/b.rs` |
| TASK-02 | - |
```

### Tabela de fatos

Deixe `-` em "Arquivos a criar" e "Arquivos a alterar": o `vostok plan score` os calcula pelos componentes da tarefa. Informe só os demais campos, `sim` ou `não`.

```markdown
| TASK-01 | - | - | não | não | não |
| TASK-02 | - | - | sim | não | sim |
```

### Resumo do plano

A visão geral em prosa, a tabela de tarefas com os perfis e as ondas gravados, a lista dos arquivos em comum por onda (ou a informação de que não há nenhum) e a descrição de cada perfil usado, lida da seção "Perfis de execução" do plano.

```markdown
Visão geral do plano.

| ID | Título | Persona | Perfil | Dependências | Onda | Cobre |
|---|---|---|---|---|---|---|
| TASK-01 | Título curto | desenvolvedor | mecanica | - | 1 | RF-01, CT-01 |

Arquivos em comum por onda (executar em sequência):

- onda 1: `caminho` (TASK-01, TASK-02)

- **mecanica:** descrição do perfil.
```
