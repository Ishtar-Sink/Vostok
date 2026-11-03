### Baseline

- `pnpm --dir apps/painel exec vitest run src/features/board --reporter=verbose 2>&1 | grep -E '✓|✗|×' | sed 's/ [0-9]*ms$//' | sort | grep -v 'cor de status\|título com aspas'`
- `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::read 2>&1 | grep -E '^test ' | sort | grep -v 'quoted_task_title'`
- `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib support:: 2>&1 | grep -E '^test ' | sort`

### Verificação

```bash
# evidência: quadro-testes.txt
pnpm --dir apps/painel exec vitest run src/features/board --reporter=verbose
pnpm --dir apps/painel exec tsc --noEmit
# evidência: titulo-sem-escape.txt
cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::read
cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib support::
```

### Evidências

- `quadro-testes.txt` (automática, da verificação)
  - Prova: as colunas Em execução, Em validação, Bloqueadas e Concluídas têm a bolinha de cor, e o título com aspas aparece como veio (RF-14, CT-15).
  - Trecho: `mostra a cor de status nas colunas do fluxo` e `mostra o título com aspas como veio` com ✓, junto dos testes que já existiam.
  - Falha: algum teste com × ou erro do `tsc`.
- `titulo-sem-escape.txt` (automática, da verificação)
  - Prova: a API do painel entrega o título e a descrição do `estado.yml` sem as barras de escape, no quadro da spec e na página da tarefa (RF-14, CT-15).
  - Trecho: `test painel::read::tests::quoted_task_title_comes_without_escapes ... ok` e `test result: ok` nas duas execuções.
  - Falha: `FAILED` no teste novo ou em um que já existia.
- `quadro-1440.png` (print): `PAINEL_BIN=apps/cli/target/release/vostok bash apps/painel/scripts/print.sh /specs/SPEC-032/quadro "$VOSTOK_EVIDENCE/quadro-1440.png" 1440,900`
  - Prova: o quadro real a 1440x900 mostra as seis colunas, a bolinha de cor em "Em validação" e o título com aspas da SPEC-032 sem barras (RF-14, CT-15).
  - Trecho: as colunas Para aprovar, Aprovadas, Em execução, Em validação, Bloqueadas e Concluídas lado a lado, a bolinha verde-azulada em "Em validação" e, em Concluídas, o cartão `Separar a coluna "Em validação" no quadro`.
  - Falha: "Concluídas" cortada ou fora da tela, "Em validação" sem bolinha ou `\"` no título do cartão.
- `tarefa-sem-escape.png` (print): `PAINEL_BIN=apps/cli/target/release/vostok bash apps/painel/scripts/print.sh /tarefas/SPEC-033/TASK-12 "$VOSTOK_EVIDENCE/tarefa-sem-escape.png" 1440,900`
  - Prova: a página da tarefa mostra a descrição com aspas do `estado.yml` sem barras de escape (RF-14).
  - Trecho: o subtítulo `Seis colunas a 1440 px, títulos sem barras e cor em "Em validação" (DT-017).`
  - Falha: `\"Em validação\"` no subtítulo.
