---
type: issue
title: "Problemas"
description: "Problemas já encontrados no projeto, com causa e solução. Consulte ao encontrar um erro, antes da primeira tentativa de correção."
updated_at: 2026-10-14T20:44:14Z
---

# Problemas

Problemas já encontrados no projeto, com causa e solução. Consulte ao encontrar um erro, antes da primeira tentativa de correção.

Entradas criadas por `add-memory.sh` e consultadas por `search-memory.sh`. Não carregue este arquivo inteiro.

## ISS-001: Tarefa ficou bloqueada por regressão mesmo com a diferença aceita pelo usuário

- **Data:** 2026-10-14
- **Status:** resolvido
- **Tags:** vostok, task, regressão, baseline, resume, aceite
- **Categoria:** ferramenta
- **Assinatura:** a tarefa TASK-NN ficou bloqueada por regressão. PARE: responda com o template `blocker`
- **Ambiente:** `vostok` 0.1.0; caso real na SPEC-001 TASK-02 de `~/rd/app` (vírgula na linha 52 do `package.json` muda a saída de `grep -n '"e2e:cart:' package.json`).
- **Causa:** o `vostok task resume` só gravava a decisão do usuário como texto, e o `vostok task regression` sempre comparava com o baseline: uma diferença aceita como esperada bloqueava a tarefa de novo, sem fim (SPEC-007).
- **Solução:** a partir da SPEC-007, aceitar a diferença com `vostok task resume <SPEC_ID> <TASK_ID> --accept-regression <N> "decisão"`, usando o número mostrado no bloqueio (`- [N] comando`). O aceite vale só para a saída exata; se ela mudar, a regressão bloqueia de novo. Não edite baseline nem front-matter.
