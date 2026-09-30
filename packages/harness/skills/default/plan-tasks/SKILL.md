---
name: plan-tasks
description: "Planeja as tarefas de uma especificação (spec) aprovada, com status approved. Cria um plano para revisão em work/plans/, com a visão geral, a tabela de tarefas, as dependências, os IDs cobertos e o nível de esforço sugerido de cada tarefa, calculado por script. Use quando o usuário pedir para planejar, propor, granularizar ou quebrar em tarefas uma spec aprovada. Não use para ajustar um plano existente (skill adjust-plan) nem para aprová-lo (skill approve-plan)."
allowed-tools: Read Edit Glob Bash(packages/harness/skills/default/plan-tasks/scripts/*) Bash(packages/harness/scripts/default/*)
metadata:
  version: "1.0.0"
  author: "Victor Batalha"
  requires: "rules/agent/template-usage.md config/tasks.yml"
  scripts: "scripts/default/context/load-extra-context.sh scripts/default/specs/list-specs.sh skills/default/plan-tasks/scripts/plan-tasks.sh"
  templates: "templates/plan/plan.md"
---

# Planejar tarefas

Propõe, a partir de uma spec aprovada, um plano de tarefas pequenas, ordenadas e rastreáveis, com um nível de esforço sugerido para cada uma. O plano é só um documento de revisão.

## Termos

- **Aprovada:** a spec em `refined/` com status `approved`. É a única que pode ser planejada.
- **Plano:** o arquivo `packages/harness/work/plans/<SPEC_ID>.md`. É removido quando aprovado pela skill `approve-plan`.
- **Plano atualizado:** plano com `spec_hash` igual ao `approved_hash` da spec. Com hash diferente, o plano está **desatualizado** e é recriado.
- **Tarefa:** uma unidade de trabalho entregável sozinha, com ID `TASK-NN` sequencial a partir de `TASK-01`.
- **Fatos de esforço:** o que o agente sabe sobre a tarefa: arquivos a criar e a alterar e se ela muda um contrato público, depende de inferência ou cobre um risco alto.
- **Nível:** sugestão de esforço (`simples`, `moderada`, `complexa` ou `critica`), calculado pelo script a partir dos fatos. Nunca indica um modelo.

## Regras

[plan-tasks-01]: Só planeje specs com status `approved`.
[plan-tasks-02]: Nunca altere um plano atualizado. Um pedido para planejar uma spec que já tem plano atualizado é informado e para; ajustes são feitos pela skill `adjust-plan` só quando o usuário pedir.
[plan-tasks-03]: O planejamento de uma spec nunca altera o plano de outra.
[plan-tasks-04]: Nunca preencha a coluna "Nível" à mão. Só o `--score` a grava.
[plan-tasks-05]: Nunca edite o front-matter, a seção "Níveis de esforço" nem o "Histórico de ajustes" do plano.
[plan-tasks-06]: Todo ID da spec que precisa de cobertura (RF e CT; PQ em `study`; S em `analysis`) é coberto por pelo menos uma tarefa.
[plan-tasks-07]: Não crie arquivos de tarefa. Eles só existem depois que o usuário aprova o plano (skill `approve-plan`).

## Checklist

Copie e marque o progresso:

```
- [ ] 1. Carregar o contexto
- [ ] 2. Identificar a spec
- [ ] 3. Criar o plano
- [ ] 4. Preencher o plano
- [ ] 5. Calcular os níveis
- [ ] 6. Validar
- [ ] 7. Responder ao usuário
```

## Passos

### 1. Carregar o contexto

Execute:

```bash
packages/harness/scripts/default/context/load-extra-context.sh rules/agent/template-usage.md config/tasks.yml
```

### 2. Identificar a spec

Liste as specs aprovadas com `packages/harness/scripts/default/specs/list-specs.sh approved` e os planos existentes em `packages/harness/work/plans/`. Uma spec aprovada é **candidata** se não tem plano ou se o plano dela está desatualizado.

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
packages/harness/skills/default/plan-tasks/scripts/plan-tasks.sh --create <SPEC_ID>
```

O script cria o plano a partir do template e recusa specs que não estão aprovadas ou que mudaram depois da aprovação. Um plano desatualizado é recriado com aviso; informe o aviso ao usuário no passo 7. Se o script recusar, **PARE** e responda com o template `blocker`, com a mensagem do script.

### 4. Preencher o plano

Leia a spec aprovada inteira e o plano criado. Substitua as variáveis `#{...}`:

- **`#{PLAN_OVERVIEW}`:** um ou dois parágrafos em prosa: como o trabalho foi dividido, a ordem sugerida e o que pode ser feito em paralelo.
- **`#{PLAN_TASKS}`:** uma linha por tarefa, na [tabela de tarefas](#tabela-de-tarefas).
- **`#{PLAN_FACTS}`:** uma linha por tarefa, na [tabela de fatos](#tabela-de-fatos).

Para dividir o trabalho:

- Cada tarefa é entregável e verificável sozinha, com um objetivo único.
- Use os "Componentes afetados", os "Fluxos" e os "Casos de teste" da spec para agrupar mudanças relacionadas. Uma tarefa não mistura partes independentes.
- Prefira mais tarefas simples a poucas complexas. Uma tarefa com muitos arquivos ou muitos IDs cobertos provavelmente deve ser dividida.
- As dependências formam uma ordem sem ciclos: uma tarefa só depende das que precisam existir antes dela.

### 5. Calcular os níveis

Execute:

```bash
packages/harness/skills/default/plan-tasks/scripts/plan-tasks.sh --score <SPEC_ID>
```

O script grava o nível de cada tarefa na coluna "Nível". Se ele recusar por fato inválido, corrija a tabela de fatos e execute de novo.

### 6. Validar

Execute:

```bash
packages/harness/skills/default/plan-tasks/scripts/plan-tasks.sh --validate <SPEC_ID>
```

O script lista os problemas sem alterar nada. Corrija-os, execute o `--score` de novo se mudou fatos ou tarefas e valide outra vez. Se falhar pela segunda vez, **PARE** e responda com o template `blocker`.

### 7. Responder ao usuário

Responda com o template `completion`:

- **O que foi feito:** o caminho do plano e o [resumo do plano](#resumo-do-plano). Se o plano foi recriado, informe o aviso do script.
- **Verificação:** os comandos executados e o resultado.
- **Pendências:** informe que o plano pode ser ajustado (skill `adjust-plan`) ou aprovado (skill `approve-plan`), e que o nível é só uma sugestão de esforço.

## Formatos de resposta

Respostas ao usuário usam os templates de `templates/responses/`. Formatos próprios desta skill:

### Tabela de tarefas

`Tipo` é `tarefa`, `historia` ou `bug`. Sem dependências, use `-`. Deixe `-` em "Nível": o `--score` o preenche.

```markdown
| TASK-01 | Título curto | O que a tarefa entrega, em uma frase. | tarefa | - | - | RF-01, CT-01 |
| TASK-02 | Título curto | O que a tarefa entrega, em uma frase. | historia | - | TASK-01 | RF-02, CT-02 |
```

### Tabela de fatos

Arquivos a criar e a alterar são números inteiros; os demais campos são `sim` ou `não`.

```markdown
| TASK-01 | 1 | 2 | não | não | não |
| TASK-02 | 0 | 3 | sim | não | sim |
```

### Resumo do plano

A visão geral em prosa, a tabela de tarefas com os níveis gravados e a descrição de cada nível usado, lida da seção "Níveis de esforço" do plano.

```markdown
Visão geral do plano.

| ID | Título | Nível | Dependências | Cobre |
|---|---|---|---|---|
| TASK-01 | Título curto | simples | - | RF-01, CT-01 |

- **simples:** descrição do nível.
```
