---
name: validate-task
description: "Valida uma tarefa em execução, em work/tasks/: executa os testes da Verificação, compara com o baseline para achar regressões, bloqueia a tarefa depois do máximo de falhas ou numa regressão e a retoma com a decisão do usuário. Use ao fim da execução de uma tarefa (skill execute-task) ou quando o usuário pedir para validar, testar ou revalidar uma tarefa em andamento, ou para retomar uma tarefa bloqueada."
allowed-tools: Read Edit Write Glob Grep Bash
metadata:
  version: "1.0.0"
  author: "Victor Batalha"
  requires: "rules/agent/anti-loop.md config/tasks.yml"
  scripts: "scripts/default/tasks/load-task.sh scripts/default/memory/search-memory.sh skills/default/validate-task/scripts/validate-task.sh"
  templates: "templates/responses/question.md templates/responses/blocker.md templates/responses/completion.md"
---

# Validar tarefa

Comprova uma tarefa implementada com os testes da "Verificação" e a comparação com o baseline, parando para o usuário decidir sempre que ela ficar bloqueada.

## Termos

- **Tarefa:** o arquivo `packages/harness/work/tasks/<SPEC_ID>/<TASK_ID>.md`, com status `in_progress`, `validating` ou `blocked`.
- **Tentativa:** uma execução de `--tests` com falha. Ao atingir `max_attempts` (`config/tasks.yml`), a tarefa fica `blocked`.
- **Regressão:** um comando de "Baseline" com código de saída ou saída diferente do baseline coletado no início.
- **Bloqueada:** tarefa com status `blocked`. Só volta com `--resume`, depois de uma decisão do usuário.
- **Registro:** a seção "Registro de execução" da tarefa, com uma linha por etapa, escrita só pelos scripts.

## Regras

[validate-task-01]: Só os scripts gravam status, resultados, tentativas e o registro. Nunca os edite à mão.
[validate-task-02]: Tarefa bloqueada: **PARE** e responda com o template `blocker`. Não corrija, não retome e não valide de novo sem a decisão do usuário.
[validate-task-03]: Execute `--resume` só com uma decisão do usuário dada numa mensagem dele, e registre essa decisão no comando.
[validate-task-04]: Corrija falhas de testes no código da tarefa, nunca na "Verificação", no "Baseline" ou nos testes que comprovam a tarefa. Mudar o que comprova a tarefa é um ajuste (skill `adjust-task`), a pedido do usuário.
[validate-task-05]: Não faça commit nem stage ([agent/boundaries-01], [agent/boundaries-02]).

## Checklist

Copie e marque o progresso:

```
- [ ] 1. Identificar a tarefa
- [ ] 2. Executar os testes
- [ ] 3. Verificar regressões
- [ ] 4. Gravar as evidências e concluir
- [ ] 5. Responder ao usuário
```

## Passos

### 1. Identificar a tarefa

Vindo da skill `execute-task`, use a tarefa que acabou de ser implementada. A pedido do usuário, procure a tarefa citada com Glob (`packages/harness/work/tasks/*/TASK-06.md`); sem citação, use a única com status `in_progress`, `validating` ou `blocked`, ou pergunte qual com o template `question`.

Tarefa `blocked` com decisão do usuário nesta mensagem: aplique o que ele definiu (correção, ajuste ou aceite da diferença) e execute:

```bash
packages/harness/skills/default/validate-task/scripts/validate-task.sh --resume <SPEC_ID> <TASK_ID> "decisão do usuário, em uma linha"
```

Tarefa `blocked` sem decisão: **PARE** e responda com o template `blocker`, com as falhas do registro e de `evidence/<TASK_ID>/validation/`.

### 2. Executar os testes

Execute:

```bash
packages/harness/skills/default/validate-task/scripts/validate-task.sh --tests <SPEC_ID> <TASK_ID>
```

O script executa o bloco `bash` de "Verificação" a partir da raiz do projeto e para no primeiro comando com falha. Se falhar:

1. Busque o erro na memória (`search-memory.sh issue <trecho estável da mensagem>`), conforme [agent/anti-loop-01].
2. Corrija o código e execute `--tests` de novo.
3. Se não houver o que corrigir no código da tarefa (a falha depende do ambiente ou da própria "Verificação"), execute `--tests` de novo mesmo assim, até o script bloquear a tarefa. Não pare antes: só uma tarefa bloqueada recebe a decisão do usuário pelo `--resume`.
4. Se o script bloquear a tarefa, **PARE** e responda com o template `blocker`, com as falhas, as tentativas e o comando que falhou (linha "comando com falha" da saída).

### 3. Verificar regressões

Execute:

```bash
packages/harness/skills/default/validate-task/scripts/validate-task.sh --regression <SPEC_ID> <TASK_ID>
```

O script executa de novo os comandos de "Baseline" e compara com o baseline. Se houver diferença, a tarefa fica bloqueada: **PARE** e responda com o template `blocker`, com os comandos e as diferenças. As opções incluem corrigir a regressão e aceitar a diferença como esperada.

### 4. Gravar as evidências e concluir

Execute `--evidence` e `--finish` quando existirem no script. Até lá, informe que a validação parou nos testes e na regressão.

### 5. Responder ao usuário

Responda com o template `completion`:

- **O que foi feito:** o status da tarefa, as linhas do registro desta execução e os caminhos das saídas em `evidence/<TASK_ID>/`.
- **Verificação:** os comandos executados e o resultado de cada etapa.
