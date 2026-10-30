---
name: execute-task
description: "Executa uma tarefa aprovada de uma especificação (spec), em work/tasks/: confere se ela pode começar, coleta o baseline antes de qualquer mudança, implementa os testes pedidos em Verificação e depois o código, seguindo os passos, sem commit nem stage, e termina com a tarefa em execução, pronta para ser finalizada. Uma tarefa ou uma onda por pedido; numa onda, o Coordenador dispara um subagente por tarefa, com a persona e o modelo dela. Use quando o usuário pedir para executar, implementar, fazer ou começar uma tarefa ou uma onda, citar um ID como TASK-03 para execução ou pedir para executar a onda N."
allowed-tools: Read Edit Write Glob Grep Bash
metadata:
  version: "1.1.0"
  author: "Victor Batalha"
  scripts: "vostok context load, vostok task status, vostok task load, vostok task start, vostok commit check"
  templates: "templates/responses/question.md templates/responses/blocker.md templates/responses/completion.md"
---

# Executar tarefa

Leva uma tarefa aprovada, ou as tarefas de uma onda, de `approved` a `in_progress`, coleta o estado anterior (baseline) e implementa cada tarefa, deixando-as em execução e prontas para serem finalizadas pela skill `finish-task`. Numa onda, o Coordenador dispara um subagente por tarefa, com a persona e o modelo do perfil dela, e sequencia as que mudam os mesmos arquivos.

## Persona

Esta skill atua como Desenvolvedor, mais UI/UX Designer nas tarefas com interface, carregada(s) no passo 1 pelo `vostok context load`. O chat segue como Coordenador, que escolhe o modelo, dispara os subagentes e consolida a resposta; cada subagente carrega a persona da tarefa no [pedido ao subagente](#pedido-ao-subagente). Sem subagentes, a própria sessão carrega a persona de cada tarefa. Ao terminar, inclusive numa parada, o chat volta à persona Coordenador; nenhum estado de persona ativa é gravado.

## Termos

- **Tarefa:** o arquivo `vostok/work/tasks/<SPEC_ID>/<TASK_ID>.md`.
- **Executável:** tarefa com status `approved` e todas as dependências (`depends_on`) com status `done`.
- **Baseline:** a saída e o código de saída dos comandos de "Baseline", gravados antes de qualquer mudança em `evidence/<TASK_ID>/baseline/`. A `finish-task` os executa de novo e compara, para achar regressões.
- **Registro:** a seção "Registro de execução" da tarefa, com uma linha por etapa, escrita só pelos comandos.
- **Onda:** as tarefas da spec com o mesmo `wave`, mostrado pelo `vostok task status`; podem rodar em paralelo.
- **Perfil e modelo:** o bloco "Modelo" do `vostok task load`: perfil, descrição e apelido de reserva da ferramenta em uso.
- **Arquivos em comum:** caminhos que aparecem em "Mudar" de mais de uma tarefa da onda.

## Regras

[execute-task-01]: Uma tarefa ou uma onda por pedido. Ao terminar, não comece outra sem um novo pedido do usuário.
[execute-task-02]: Só execute tarefas que o `vostok task start` aceitar. Não contorne uma recusa mudando o status, o hash ou as dependências.
[execute-task-03]: Colete o baseline antes de mudar qualquer arquivo do projeto.
[execute-task-04]: Siga "Mudar" e "Passos" da tarefa. Não amplie o escopo: o que for necessário e não estiver na tarefa vira pergunta ao usuário.
[execute-task-05]: Não faça commit nem stage ([agent/boundaries-01], [agent/boundaries-02]).
[execute-task-06]: Nunca edite o front-matter, o "Registro de execução" nem o "Histórico de ajustes" da tarefa. Só os comandos os alteram.
[execute-task-07]: Esta skill não valida nem conclui a tarefa e não chama a `finish-task` (RN-03). A finalização é um pedido separado do usuário.
[execute-task-08]: O chat segue como Coordenador: ele escolhe o modelo, dispara os subagentes e consolida a resposta, sem implementar nem testar. Cada subagente carrega a persona da tarefa (persona Coordenador, RF-25).
[execute-task-09]: O modelo vem do perfil da tarefa, nunca do usuário (RN-03): o Coordenador escolhe, entre os modelos que a ferramenta oferece, o que atende à descrição do perfil; se a ferramenta não deixa listar ou escolher, usa o apelido de reserva; com "modelo da sessão", segue com o modelo da sessão. O modelo usado vai no `vostok task start --model`.
[execute-task-10]: Tarefas da onda com arquivos em comum rodam em sequência, na ordem dos IDs, e o usuário é avisado (RF-16, FA-02). As demais rodam em paralelo, com no máximo 6 subagentes ao mesmo tempo (ou menos, se a ferramenta limitar); as excedentes esperam uma vaga.
[execute-task-11]: Sem subagentes ou sem escolha de modelo na ferramenta, as tarefas rodam uma por vez na própria sessão, com o modelo da sessão (FA-01).

## Checklist

Copie e marque o progresso:

```
- [ ] 1. Carregar o contexto
- [ ] 2. Identificar a tarefa ou a onda
- [ ] 3. Escolher o modelo
- [ ] 4. Conferir os arquivos em comum
- [ ] 5. Disparar os subagentes
- [ ] 6. Consolidar
- [ ] 7. Responder ao usuário
```

## Passos

### 1. Carregar o contexto

Execute:

```bash
vostok context load --skill execute-task --persona desenvolvedor
```

### 2. Identificar a tarefa ou a onda

As tarefas ficam em `vostok/work/tasks/<SPEC_ID>/`, e os IDs são numerados dentro de cada spec. Com tarefa citada, procure-a com Glob (`vostok/work/tasks/*/TASK-05.md`); ela é tratada como uma onda de uma tarefa:

- Tarefa citada que existe numa única spec: use essa.
- Tarefa citada que existe em mais de uma spec: **PARE** e pergunte de qual, com o template `question`.
- Sem citação ("executa a próxima"): leia o front-matter das tarefas e liste as executáveis. Uma só: use essa. Mais de uma: **PARE** e pergunte qual, com o template `question`, usando a [lista de tarefas executáveis](#lista-de-tarefas-executáveis). Nenhuma: informe as tarefas aprovadas com dependências pendentes e as pendentes de aprovação, e encerre.

Com onda citada ("executa a onda 2 da SPEC-014") ou "a próxima onda", execute:

```bash
vostok task status <SPEC_ID>
```

A onda é a citada ou a de menor número sem status `done`. As tarefas a executar são as dela com status `approved` e dependências `done`. As demais tarefas da onda ficam fora e são listadas na resposta. Nenhuma executável: **PARE** e informe, com a saída do `vostok task status`.

### 3. Escolher o modelo

Para cada tarefa a executar, execute `vostok task load <SPEC_ID> <TASK_ID>` e leia o bloco "Modelo". Siga, para a ferramenta de "Ferramenta", o que `docs/estudos/multiagentes.md` registra em "Escolha do modelo pelo Coordenador":

- A ferramenta lista os modelos: escolha o que atende à "Descrição" do perfil.
- Não lista nem deixa escolher: use o "Apelido de reserva".
- "Modelo da sessão": use o modelo da sessão.

Anote o nome do modelo escolhido: com o modelo da sessão, o nome dele, se o agente o conhece, ou o texto `modelo da sessão`. O pedido do usuário não muda o modelo ([execute-task-09]).

### 4. Conferir os arquivos em comum

Compare as listas "Mudar" das tarefas a executar e monte grupos: tarefas que dividem um caminho ficam no mesmo grupo, executado em sequência na ordem dos IDs; tarefas sem caminho em comum ficam sozinhas. Antes de disparar, avise o usuário numa linha por grupo em sequência:

```markdown
TASK-08 e TASK-13 mexem em `apps/cli/src/commands/task.rs`: rodam em sequência.
```

### 5. Disparar os subagentes

Para cada grupo, dispare um subagente por tarefa: em sequência dentro do grupo e com os grupos em paralelo, com no máximo 6 subagentes ao mesmo tempo, ou menos se o estudo registrar um limite menor para a ferramenta. As excedentes esperam uma vaga. Cada subagente recebe:

- o [pedido ao subagente](#pedido-ao-subagente);
- a persona da tarefa: o campo `persona` mostrado pelo `vostok task status`; `-` ou `null` vale `desenvolvedor`;
- o modelo do passo 3.

Se a ferramenta tem as definições de subagente das personas (`vostok-<persona>`), use a da persona.

**Antes de disparar o primeiro subagente**, o Coordenador inicia cada tarefa da onda, inclusive numa onda de uma tarefa, e só então dispara os subagentes:

```bash
vostok task start <SPEC_ID> <TASK_ID> --model <MODELO>
```

O `--model` grava no front-matter e no registro de execução o modelo escolhido no passo 3 (RF-19). O comando confere se a tarefa está aprovada, sem mudança depois da aprovação e com as dependências concluídas, muda o status para `in_progress` e coleta o baseline no mesmo comando, para que nenhum baseline contenha a mudança de outra tarefa. Se ele recusar, **PARE** e repasse a mensagem do comando com o template `blocker`:

| Recusa | Como resolver |
|---|---|
| Tarefa fora de `approved` | Aprovar a tarefa (skill `approve-task`), a pedido do usuário. |
| Tarefa mudou depois da aprovação | Registrar o ajuste (skill `adjust-task`) e aprovar de novo. |
| Dependência fora de `done` | Executar e concluir a dependência antes. |
| Caminho `criar` de Mudar já existe | O estado anterior já foi alterado: restaurar o estado anterior, com a decisão do usuário, e iniciar de novo. |

Sem subagentes (FA-01): a sessão também inicia todas as tarefas da onda antes de implementar a primeira (CB-05); depois executa cada tarefa na própria sessão, uma por vez, carregando a persona com `vostok context load --persona <persona>` e seguindo o roteiro do pedido, com o modelo da sessão.

### 6. Consolidar

Espere todos os subagentes. Para cada tarefa, confira com `vostok task status <SPEC_ID>` que ela está `in_progress`, confira no "Registro de execução" (`vostok task load`) a linha `baseline` e leia o resumo devolvido. Um subagente que parou com `blocker` ou `question` tem a mensagem repassada ao usuário, sem resolver por ele.

### 7. Responder ao usuário

Execute `vostok commit check` e responda com o template `completion`:

- **Resumo:** as tarefas foram implementadas e seguem `in_progress`.
- **O que foi feito:** uma linha por tarefa (ID, persona, modelo, resultado), os avisos de sequência do passo 4 e as tarefas da onda que ficaram fora.
- **Verificação:** o que foi feito nesta execução, com as linhas do "Registro de execução" de cada tarefa (início e baseline), e o que não foi verificado: os testes, a regressão e as evidências ficam para a `finish-task`.
- **Status das tarefas:** a saída de `vostok task status <SPEC_ID>`.
- **Próximos passos:** o pedido em linguagem natural para finalizar cada tarefa, por exemplo `- "finaliza a TASK-04" (skill finish-task): executar os testes, comparar com o baseline e concluir a tarefa`. Nunca indique um comando.

Esta skill não chama a `finish-task` nem roda `vostok task tests`, `vostok task regression` ou `vostok task evidence` (RN-03): a conclusão só começa quando o usuário pedir.

Esta resposta encerra a persona da skill: o chat volta à persona Coordenador.

## Roteiro do subagente

Os passos que o subagente segue para a sua tarefa, pelo [pedido ao subagente](#pedido-ao-subagente). Sem subagentes, a própria sessão os segue para cada tarefa.

### Carregar a tarefa

Execute `vostok task load <SPEC_ID> <TASK_ID>` e leia os arquivos de "Ler".

Tarefa com interface (lista "Mudar" com arquivos `.html`, `.css`, `.scss`, `.tsx`, `.jsx`, `.vue`, `.svelte` ou telas e templates visuais): execute `vostok context load --skill execute-task --persona ui-ux-designer` antes de implementar.

### Conferir o baseline

Antes de mudar qualquer arquivo, confirme no registro da tarefa (`vostok task load <SPEC_ID> <TASK_ID>`) a linha `baseline`. Sem ela, **PARE** e devolva ao Coordenador com o template `blocker`. O `vostok task baseline` só existe para tarefa iniciada antes da versão que coleta o baseline no `start`.

### Implementar os testes

Implemente primeiro os testes automatizados que a "Verificação" e os "Passos" pedem, para que comprovem a tarefa antes do código existir.

### Implementar o código

Siga os "Passos", na ordem, mudando só os arquivos de "Mudar". Se um passo exigir algo fora da tarefa, **PARE** e devolva a pergunta ao Coordenador, com o template `question`; ele a repassa ao usuário.

## Formatos de resposta

Respostas ao usuário usam os templates de `templates/responses/`. Formatos próprios desta skill:

### Lista de tarefas executáveis

```markdown
| Spec | ID | Título | Perfil | Onda |
|---|---|---|---|---|
| SPEC-002 | TASK-05 | Título curto | julgamento | 2 |
```

### Pedido ao subagente

```markdown
Você é o subagente da tarefa <TASK_ID> da <SPEC_ID>, com a persona <persona>.

1. Rode `vostok context load --persona <persona>` e `vostok context load --skill execute-task`.
2. A tarefa já foi iniciada pelo Coordenador com `--model <MODELO>` e o baseline já foi coletado: não rode `vostok task start` nem `vostok task baseline`, e não mude nenhum arquivo antes de conferir a linha `baseline` no registro.
3. Siga os passos "Carregar a tarefa", "Conferir o baseline", "Implementar os testes" e "Implementar o código" da skill `execute-task`, só para esta tarefa, sem commit nem stage e sem finalizar.
4. Devolva, em até 5 linhas: o resultado, os arquivos mudados e o que ficou em aberto; se precisar parar, devolva a mensagem no template `blocker` ou `question`.
```
