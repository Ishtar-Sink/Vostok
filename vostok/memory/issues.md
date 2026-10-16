---
type: issue
title: "Problemas"
description: "Problemas já encontrados no projeto, com causa e solução. Consulte ao encontrar um erro, antes da primeira tentativa de correção."
updated_at: 2026-10-16T13:48:13Z
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

## ISS-002: golden_commit falha quando o hash do commit não tem dígito

- **Data:** 2026-10-16
- **Status:** contornado
- **Tags:** teste, golden, hash, intermitente, commit
- **Categoria:** teste
- **Assinatura:** 2 caso(s) do grupo 'commit' diferem das fixtures ... Commit feito e liberado: <HEX> ... obtido ... Commit feito e liberado: ffcbacc
- **Ambiente:** `cargo test` do `apps/cli`, teste `golden_commit` (`apps/cli/tests/common/golden.rs`, normalização de hashes).
- **Causa:** a normalização só troca por `<HEX>` sequências hexadecimais com pelo menos um dígito, para não trocar palavras; um hash curto só com letras de `a` a `f` (cerca de 0,1% dos commits) escapa e o caso difere.
- **Solução:** é intermitente: rodar os testes de novo resolve, porque o hash muda. Correção definitiva pendente: normalizar o hash na linha `Commit feito e liberado:` mesmo sem dígito.
