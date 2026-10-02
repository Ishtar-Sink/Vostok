---
name: validate-task
description: "Valida e conclui uma tarefa em execução, em work/tasks/: executa os testes da Verificação, compara com o baseline para achar regressões, grava as evidências, confere os critérios de conclusão e marca a tarefa como done, e a spec como implemented na última tarefa. Bloqueia a tarefa depois do máximo de falhas ou numa regressão e a retoma com a decisão do usuário. Use ao fim da execução de uma tarefa (skill execute-task) ou quando o usuário pedir para validar, testar ou revalidar uma tarefa em andamento, ou para retomar uma tarefa bloqueada."
allowed-tools: Read Edit Write Glob Grep Bash
---

> Este é apenas um redirecionamento para a skill.
> Leia e siga o arquivo `packages/harness/skills/default/validate-task/SKILL.md`.
