### Baseline

- `pnpm --dir apps/painel exec vitest run src/features/task src/features/inbox src/features/settings --reporter=verbose 2>&1 | grep -E '✓|✗|×' | sed 's/ [0-9]*ms$//' | sort | grep -v 'print\|commits\|pergunta com o id\|limpar\|apagar\|limpeza'`
- `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::read 2>&1 | grep -E '^test ' | sort | grep -v 'commits_route'`
- `pnpm --dir apps/painel exec tsc --noEmit`

### Verificação

```bash
# evidência: painel-conferencia-testes.txt
pnpm --dir apps/painel exec vitest run src/features/task src/features/inbox src/features/settings src/shared --reporter=verbose
pnpm --dir apps/painel exec tsc --noEmit
# evidência: commits-rota-testes.txt
cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::read
```

### Evidências

- `painel-conferencia-testes.txt` (automática, da verificação)
  - Prova: aceitar e rejeitar prints pela rota de conferência, commits apontados com liberar e desfazer (este só depois da confirmação), resposta com o id da pergunta e a limpeza que lista antes e apaga só ao confirmar, tudo achado por papel e rótulo (RF-06, RF-07, RF-11, CT-07, CT-08, CT-12, CT-18).
  - Trecho: os testes novos da tarefa, das pendências e das configurações com ✓, junto dos que já existiam.
  - Falha: algum com ×, um teste antigo quebrado ou erro do `tsc`.
- `commits-rota-testes.txt` (automática, da verificação)
  - Prova: a rota `GET /api/commits` responde a lista do `commit check` sem recusar nem gravar (RF-07).
  - Trecho: `test painel::read::tests::commits_route_is_empty_without_git ... ok` e `test result: ok`.
  - Falha: `FAILED` no teste novo ou em um antigo.
- `configuracoes.png` (print): `PAINEL_BIN=apps/cli/target/release/vostok bash apps/painel/scripts/print.sh /configuracoes "$VOSTOK_EVIDENCE/configuracoes.png" 1280,900`
  - Prova: o botão "Limpar pedidos arquivados" nas Configurações reais (RF-11, CT-12).
  - Trecho: o botão secundário "Limpar pedidos arquivados" à direita do título "Harness desta pasta".
  - Falha: sem o botão ou fora do estilo dos botões do painel.
- `pendencias.png` (print): `PAINEL_BIN=apps/cli/target/release/vostok bash apps/painel/scripts/print.sh /pendencias "$VOSTOK_EVIDENCE/pendencias.png" 1280,900`
  - Prova: as pendências reais com as perguntas abertas e, quando o `commit check` aponta commits, o bloco "Commits sem liberação" com os dois botões (RF-07, CT-08).
  - Trecho: o cabeçalho "Aguardando você" e a lista; com commits apontados, o bloco com as linhas e "Liberar commits" e "Desfazer commits".
  - Falha: página com erro ou bloco de commits vazio com botões.
- `conferencia-prints.png` (print): `PAINEL_BIN=apps/cli/target/release/vostok bash apps/painel/scripts/print.sh /tarefas/SPEC-033/TASK-10#evidencias "$VOSTOK_EVIDENCE/conferencia-prints.png" 1280,6500`
  - Prova: a página real desta tarefa, na finalização, mostra os prints já gerados (`configuracoes.png` e `pendencias.png`) a conferir, com "Aceitar" e "Rejeitar" (RF-06, CT-07).
  - Trecho: o selo "a conferir" e os botões "Aceitar" e "Rejeitar" sob as imagens na seção Evidências.
  - Falha: prints sem selo, sem botões ou "conferido" antes da conferência.
