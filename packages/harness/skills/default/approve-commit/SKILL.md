---
name: approve-commit
description: "Libera ou desfaz, a pedido do usuário, os commits apontados pela verificação de commits (check-commits.sh): commits feitos sem liberação, pelo agente ou pelo próprio usuário. Liberar grava o estado atual do repositório como referência; desfazer remove os commits novos da branch atual mantendo as alterações na pasta de trabalho. Use quando o usuário pedir para liberar, aprovar, aceitar, desfazer ou reverter os commits apontados."
allowed-tools: Read Bash(packages/harness/skills/default/approve-commit/scripts/*) Bash(packages/harness/scripts/default/*)
metadata:
  version: "1.0.0"
  author: "Victor Batalha"
  requires: "rules/agent/boundaries.md"
  scripts: "scripts/default/git/check-commits.sh skills/default/approve-commit/scripts/approve-commit.sh"
  templates: "templates/responses/question.md templates/responses/completion.md"
---

# Liberar commits

Registra a decisão do usuário sobre os commits feitos sem liberação: liberá-los ou desfazê-los. Até essa decisão, o `check-commits.sh` recusa e o agente não conclui nenhuma resposta ([agent/boundaries-04]).

## Termos

- **Referência:** o estado das refs locais (HEAD, branches e tags) na última liberação, gravado em `packages/harness/.state/commit-ack`, fora do git.
- **Commit novo:** commit alcançável hoje e não alcançável pela referência. Inclui `--amend`, rebase e commits em outras branches.
- **Commit removido:** commit da referência que saiu do histórico, por `reset`, rebase ou branch apagada.
- **Código de liberação:** identifica o estado mostrado ao usuário. Se o repositório mudar depois, o script recusa a ação.

## Regras

[approve-commit-01]: Só execute esta skill a pedido do usuário, numa mensagem dele posterior à que mostrou os commits. Nunca libere nem desfaça na mesma resposta em que os commits foram apontados.
[approve-commit-02]: O pedido tem que dizer o que fazer: liberar ou desfazer. "Ok", "segue" ou um pedido sobre outro assunto não são liberação; na dúvida, pergunte (passo 2).
[approve-commit-03]: Nunca edite nem apague o arquivo de referência. Só o script o altera.
[approve-commit-04]: Desfaça só pelo script. Se ele recusar, não use `git reset`, `git revert` nem outro comando: mostre o motivo e deixe a decisão com o usuário.

## Checklist

Copie e marque o progresso:

```
- [ ] 1. Mostrar os commits
- [ ] 2. Confirmar a decisão
- [ ] 3. Liberar ou desfazer
- [ ] 4. Responder ao usuário
```

## Passos

### 1. Mostrar os commits

Execute:

```bash
packages/harness/skills/default/approve-commit/scripts/approve-commit.sh --show
```

O script lista os commits novos e removidos e o código de liberação. Sem commits, responda que não há nada a liberar e encerre.

### 2. Confirmar a decisão

Compare a lista com a que o usuário viu. Execute o passo 3 sem perguntar só se as duas forem iguais e o pedido disser liberar ou desfazer.

Nos demais casos, **PARE** e pergunte com o template `question`, mostrando a lista e as opções: liberar todos ou desfazer os novos.

### 3. Liberar ou desfazer

Execute, com o código do passo 1:

```bash
packages/harness/skills/default/approve-commit/scripts/approve-commit.sh --approve <código>
packages/harness/skills/default/approve-commit/scripts/approve-commit.sh --revert <código>
```

O `--revert` desfaz os commits novos da branch atual com `git reset --mixed`: as alterações continuam na pasta de trabalho, fora de stage. Ele recusa commits já enviados ao remoto, commits removidos e commits fora da branch atual. Se recusar, **PARE** e responda com o template `blocker`, com a mensagem do script.

### 4. Responder ao usuário

Execute `packages/harness/scripts/default/git/check-commits.sh` e confirme que ele passa. Responda com o template `completion`:

- **O que foi feito:** os commits liberados ou desfeitos, com hash e assunto.
- **Verificação:** a saída do `check-commits.sh`.
