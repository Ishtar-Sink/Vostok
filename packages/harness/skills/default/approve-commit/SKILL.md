---
name: approve-commit
description: "Faz commits a pedido do usuário, já liberados, e libera ou desfaz os commits apontados pela verificação de commits (check-commits.sh): commits feitos sem liberação, pelo agente ou pelo próprio usuário. Liberar grava o estado atual do repositório como referência; desfazer remove os commits novos da branch atual mantendo as alterações na pasta de trabalho. Use quando o usuário pedir para fazer commit, ou para liberar, aprovar, aceitar, desfazer ou reverter os commits apontados."
allowed-tools: Read Bash(packages/harness/skills/default/approve-commit/scripts/*) Bash(packages/harness/scripts/default/*)
metadata:
  version: "1.0.0"
  author: "Victor Batalha"
  scripts: "scripts/default/context/load-extra-context.sh scripts/default/git/check-commits.sh skills/default/approve-commit/scripts/approve-commit.sh"
  templates: "templates/responses/question.md templates/responses/completion.md"
---

# Commits

Único caminho do agente para commitar: o pedido de commit do usuário é a liberação, e o commit é feito e liberado no mesmo passo. Também registra a decisão do usuário sobre os commits feitos sem liberação: liberá-los ou desfazê-los. Até essa decisão, o `check-commits.sh` recusa e o agente não conclui nenhuma resposta ([agent/boundaries-04]).

## Termos

- **Referência:** o estado das refs locais (HEAD, branches e tags) na última liberação, gravado em `packages/harness/.state/commit-ack`, fora do git.
- **Commit novo:** commit alcançável hoje e não alcançável pela referência. Inclui `--amend`, rebase e commits em outras branches.
- **Commit removido:** commit da referência que saiu do histórico, por `reset`, rebase ou branch apagada.
- **Código de liberação:** identifica o estado mostrado ao usuário. Se o repositório mudar depois, o script recusa a ação.

## Regras

[approve-commit-01]: Só execute esta skill a pedido do usuário, numa mensagem dele posterior à que mostrou os commits. Nunca libere nem desfaça na mesma resposta em que os commits foram apontados.
[approve-commit-02]: O pedido tem que dizer o que fazer: liberar ou desfazer. "Ok", "segue" ou um pedido sobre outro assunto não são liberação; na dúvida, pergunte (passo 3).
[approve-commit-03]: Nunca edite nem apague o arquivo de referência. Só o script o altera.
[approve-commit-04]: Desfaça só pelo script. Se ele recusar, não use `git reset`, `git revert` nem outro comando: mostre o motivo e deixe a decisão com o usuário.
[approve-commit-05]: Commite só pelo `--commit` e só quando a mensagem do usuário pedir commit. Nunca use `git commit`, `git commit-tree` nem outro comando que crie commits.
[approve-commit-06]: O commit tem só o que o usuário pediu. Não coloque em stage nem passe caminhos fora do pedido; na dúvida sobre o escopo, pergunte antes de commitar.

## Checklist

Pedido de commit: siga [Fazer commits](#fazer-commits). Pedido de liberar ou desfazer, copie e marque o progresso:

```
- [ ] 1. Carregar o contexto
- [ ] 2. Mostrar os commits
- [ ] 3. Confirmar a decisão
- [ ] 4. Liberar ou desfazer
- [ ] 5. Responder ao usuário
```

## Passos

### 1. Carregar o contexto

Execute:

```bash
packages/harness/scripts/default/context/load-extra-context.sh --skill approve-commit
```

### 2. Mostrar os commits

Execute:

```bash
packages/harness/skills/default/approve-commit/scripts/approve-commit.sh --show
```

O script lista os commits novos e removidos e o código de liberação. Sem commits, responda que não há nada a liberar e encerre.

### 3. Confirmar a decisão

Compare a lista com a que o usuário viu. Execute o passo 4 sem perguntar só se as duas forem iguais e o pedido disser liberar ou desfazer.

Nos demais casos, **PARE** e pergunte com o template `question`, mostrando a lista e as opções: liberar todos ou desfazer os novos.

### 4. Liberar ou desfazer

Execute, com o código do passo 2:

```bash
packages/harness/skills/default/approve-commit/scripts/approve-commit.sh --approve <código>
packages/harness/skills/default/approve-commit/scripts/approve-commit.sh --revert <código>
```

O `--revert` desfaz os commits novos da branch atual com `git reset --mixed`: as alterações continuam na pasta de trabalho, fora de stage. Ele recusa commits já enviados ao remoto, commits removidos e commits fora da branch atual. Se recusar, **PARE** e responda com o template `blocker`, com a mensagem do script.

### 5. Responder ao usuário

Execute `packages/harness/scripts/default/git/check-commits.sh` e confirme que ele passa. Responda com o template `completion`:

- **O que foi feito:** os commits liberados ou desfeitos, com hash e assunto.
- **Verificação:** a saída do `check-commits.sh`.

## Fazer commits

Um commit por chamada, com a mensagem pela entrada padrão. Para commits atômicos, chame uma vez por grupo de arquivos:

```bash
packages/harness/skills/default/approve-commit/scripts/approve-commit.sh --commit caminho/a caminho/b <<'MSG'
tipo(escopo): mensagem
MSG
```

Com caminhos, relativos à raiz do projeto, o script commita só eles, com o conteúdo atual da pasta de trabalho, inclusive arquivos novos e removidos. Sem caminhos, commita o que está em stage, para quando um arquivo precisa ser dividido entre commits. O script recusa se já houver commits sem liberação: nesse caso, siga os passos 2 a 5 antes. Se ele recusar por outro motivo, **PARE** e responda com o template `blocker`.

Depois do último commit, execute `packages/harness/scripts/default/git/check-commits.sh` e responda com o template `completion`, com os commits feitos (hash e mensagem).
