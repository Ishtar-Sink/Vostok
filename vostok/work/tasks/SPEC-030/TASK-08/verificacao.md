### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test init_update --test context_memory_rules 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`

### Verificação

```bash
cargo build --manifest-path apps/cli/Cargo.toml
# evidência: skill-pedidos-testes.txt
cargo test --manifest-path apps/cli/Cargo.toml --test init_update --test context_memory_rules
```

Resultado esperado: build e testes passam com a skill nova no pacote.

### Evidências

- `skill-pedidos-testes.txt` (automática, da verificação)
  - Prova: o pacote com a skill nova instala e carrega o contexto sem erro (RF-14).
  - Trecho: `test result: ok` nas duas suítes.
  - Falha: `FAILED`.
  - Espera: `test result: ok`
- `skill-instalada.txt` (automática): `T="$(mktemp -d)" && cd "$T" && git init -q && cargo run -q --manifest-path "$OLDPWD/apps/cli/Cargo.toml" -- init --yes >/dev/null && cargo run -q --manifest-path "$OLDPWD/apps/cli/Cargo.toml" -- context load --skill process-requests | head -40`
  - Prova: um projeto novo recebe a skill e o contexto dela carrega (RF-14).
  - Trecho: o nome `process-requests` e o mapa das ações para as skills.
  - Falha: `skill desconhecida` ou erro.
  - Requer: `packages/harness/skills/default/process-requests/SKILL.md`
- `loop-pedido.txt` (manual): a saída de um chat do Claude Code com `/loop 1m /process-requests` processando um pedido `ajustar-tarefa` gravado pelo painel até `feito` ou `recusado`, com o `## Resultado` do pedido.
  - Prova: o loop processa um pedido real do painel (RF-14, CT-15, RN-04).
  - Trecho: o pedido em `feito` ou `recusado` com o resultado escrito.
  - Falha: pedido parado em `pendente` ou `em_andamento`.
