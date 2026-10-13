---
name: finish-task
description: "Finaliza uma tarefa em execução, em work/tasks/, só a pedido do usuário: executa os testes da Verificação, compara com o baseline para achar regressões, grava as evidências, confere os critérios de conclusão e marca a tarefa como done, e a spec como implemented na última tarefa. Bloqueia a tarefa depois do máximo de falhas ou numa regressão e a retoma com a decisão do usuário. Use quando o usuário pedir para finalizar, concluir, validar, testar ou revalidar uma tarefa em andamento, ou para retomar uma tarefa bloqueada."
allowed-tools: Read Edit Write Glob Grep Bash
metadata:
  version: "1.0.0"
  author: "Victor Batalha"
  scripts: "vostok context load, vostok task load, vostok memory search, vostok task resume, vostok task tests, vostok task regression, vostok task evidence, vostok task finish"
  templates: "templates/responses/question.md templates/responses/blocker.md templates/responses/completion.md"
---

# Finalizar tarefa

Comprova e conclui uma tarefa implementada, só a pedido do usuário: testes da "Verificação", comparação com o baseline, evidências e critérios de conclusão, parando para o usuário decidir sempre que ela ficar bloqueada.

## Termos

- **Tarefa:** o arquivo `packages/harness/work/tasks/<SPEC_ID>/<TASK_ID>.md`, com status `in_progress`, `validating` ou `blocked`.
- **Tentativa:** uma execução de `vostok task tests` com falha. Ao atingir `max_attempts` (`config/tasks.yml`), a tarefa fica `blocked`.
- **Regressão:** um comando de "Baseline" com código de saída ou saída diferente do baseline coletado no início.
- **Bloqueada:** tarefa com status `blocked`. Só volta com `vostok task resume`, depois de uma decisão do usuário.
- **Registro:** a seção "Registro de execução" da tarefa, com uma linha por etapa, escrita só pelos comandos.
- **Evidência:** item de "Evidências", gravado em `evidence/<TASK_ID>/<arquivo>`. A automática é a saída de um comando; a manual é um arquivo que o usuário anexa.
- **Critério de conclusão:** item de "Concluída quando", com a forma de comprová-lo. O agente marca `[x]` só depois de conferir a comprovação.

## Regras

[finish-task-01]: Só os comandos gravam status, resultados, tentativas e o registro. Nunca os edite à mão.
[finish-task-02]: Tarefa bloqueada: **PARE** e responda com o template `blocker`. Não corrija, não retome e não valide de novo sem a decisão do usuário.
[finish-task-03]: Execute `vostok task resume` só com uma decisão do usuário dada numa mensagem dele, e registre essa decisão no comando.
[finish-task-04]: Corrija falhas de testes no código da tarefa, nunca na "Verificação", no "Baseline" ou nos testes que comprovam a tarefa. Mudar o que comprova a tarefa é um ajuste (skill `adjust-task`), a pedido do usuário.
[finish-task-05]: Não faça commit nem stage ([agent/boundaries-01], [agent/boundaries-02]).
[finish-task-06]: Marque `[x]` num critério de conclusão só depois de conferir a comprovação dele. Nunca marque para destravar o `vostok task finish`.
[finish-task-07]: Nunca crie nem edite o arquivo de uma evidência manual. Ele é anexado pelo usuário.
[finish-task-08]: Só finalize a pedido do usuário, uma tarefa por pedido. Ao terminar a implementação, a skill `execute-task` não finaliza por conta própria.

## Checklist

Copie e marque o progresso:

```
- [ ] 1. Carregar o contexto
- [ ] 2. Identificar a tarefa
- [ ] 3. Executar os testes
- [ ] 4. Verificar regressões
- [ ] 5. Gravar as evidências
- [ ] 6. Conferir os critérios de conclusão
- [ ] 7. Concluir
- [ ] 8. Responder ao usuário
```

## Passos

### 1. Carregar o contexto

Execute:

```bash
vostok context load --skill finish-task
```

### 2. Identificar a tarefa

Procure a tarefa citada com Glob (`packages/harness/work/tasks/*/TASK-06.md`); sem citação, use a única com status `in_progress`, `validating` ou `blocked`; havendo mais de uma, pergunte qual com o template `question`.

Tarefa `blocked` com decisão do usuário nesta mensagem: aplique o que ele definiu (correção, ajuste ou aceite da diferença) e execute:

```bash
vostok task resume <SPEC_ID> <TASK_ID> "decisão do usuário, em uma linha"
```

Tarefa `blocked` sem decisão: **PARE** e responda com o template `blocker`, com as falhas do registro e de `evidence/<TASK_ID>/validation/`.

### 3. Executar os testes

Execute:

```bash
vostok task tests <SPEC_ID> <TASK_ID>
```

O `vostok task tests` executa o bloco `bash` de "Verificação" a partir da raiz do projeto e para no primeiro comando com falha. Se falhar:

1. Busque o erro na memória (`vostok memory search issue <trecho estável da mensagem>`), conforme [agent/anti-loop-01].
2. Corrija o código e execute `vostok task tests` de novo.
3. Se não houver o que corrigir no código da tarefa (a falha depende do ambiente ou da própria "Verificação"), execute `vostok task tests` de novo mesmo assim, até o comando bloquear a tarefa. Não pare antes: só uma tarefa bloqueada recebe a decisão do usuário pelo `vostok task resume`.
4. Se o comando bloquear a tarefa, **PARE** e responda com o template `blocker`, com as falhas, as tentativas e o comando que falhou (linha "comando com falha" da saída).

### 4. Verificar regressões

Execute:

```bash
vostok task regression <SPEC_ID> <TASK_ID>
```

O `vostok task regression` executa de novo os comandos de "Baseline" e compara com o baseline. Se houver diferença, a tarefa fica bloqueada: **PARE** e responda com o template `blocker`, com os comandos e as diferenças. As opções incluem corrigir a regressão e aceitar a diferença como esperada.

### 5. Gravar as evidências

Execute:

```bash
vostok task evidence <SPEC_ID> <TASK_ID>
```

O comando grava a saída de cada evidência automática e confere as manuais:

- Evidência automática com erro: corrija a causa no código da tarefa e execute `vostok task evidence` de novo. Se o erro estiver no próprio comando da evidência, **PARE** e responda com o template `blocker`: mudar a evidência é um ajuste (skill `adjust-task`).
- Evidência manual sem arquivo: **PARE** e peça ao usuário, com o template `question`, que anexe cada arquivo no caminho listado pelo comando. Com os arquivos, execute `vostok task evidence` de novo.

### 6. Conferir os critérios de conclusão

Para cada critério de "Concluída quando", faça a comprovação descrita em "Como comprovar". Marque `[x]` só nos comprovados. Um critério que depende de algo que esta execução não alcança (por exemplo, um cenário de outra tarefa) fica sem marcar: **PARE** e responda com o template `blocker`, citando o critério.

### 7. Concluir

Execute:

```bash
vostok task finish <SPEC_ID> <TASK_ID>
```

O comando exige testes, regressão e evidências `passed` e todos os critérios marcados, grava `done` e, se esta for a última tarefa da spec, muda a spec para `implemented`. Se ele recusar, volte à etapa que falta.

### 8. Responder ao usuário

Responda com o template `completion`:

- **O que foi feito:** o status da tarefa (e o da spec, se mudou), as linhas do registro desta execução e os caminhos das evidências em `evidence/<TASK_ID>/`.
- **Verificação:** os comandos executados e o resultado de cada etapa.
