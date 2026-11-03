### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test base 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`
- `cargo test --manifest-path apps/cli/Cargo.toml --test init_update 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`

### Verificação

```bash
cargo build --manifest-path apps/cli/Cargo.toml
# evidência: skills-testes.txt
cargo test --manifest-path apps/cli/Cargo.toml --test base
cargo test --manifest-path apps/cli/Cargo.toml --test init_update
```

Resultado esperado: o build e as duas suítes passam com as skills alteradas embutidas.

### Evidências

- `skills-testes.txt` (automática, da verificação)
  - Prova: as oito skills alteradas continuam válidas no pacote embutido: chamam o `vostok context load`, têm o `allowed-tools` e não citam scripts (base de RF-02 e RF-04).
  - Trecho: `test ct13_skills_and_skill_templates_call_vostok ... ok` e `test result: ok`.
  - Falha: `FAILED` ou `test result: FAILED`.
  - Espera: `test ct13_skills_and_skill_templates_call_vostok ... ok`
- `skill-process-requests.diff` (diff): `packages/harness/skills/default/process-requests/SKILL.md`
  - Prova: a skill process-requests abre a pergunta em vez de recusar, atende o pedido de resposta (recusando quando o de origem foi recusado) e mapeia as ações novas (RF-02, RF-03, CT-02, CB-01).
  - Trecho: as regras process-requests-04, 05 e 09 novas e as linhas `criar-spec`, `refinar-spec`, `executar-onda`, `liberar-commits` e `desfazer-commits` no mapa, e a versão menor subida.
  - Falha: o texto ainda manda recusar o pedido quando a skill pergunta.
- `skill-create-spec.diff` (diff): `packages/harness/skills/default/create-spec/SKILL.md`
  - Prova: a skill create-spec por um pedido, grava a pergunta com `vostok perguntas abrir` (RF-04, CT-04).
  - Trecho: a regra create-spec-07, e a versão menor subida.
  - Falha: a pergunta continua só no chat.
- `skill-refine-spec.diff` (diff): `packages/harness/skills/default/refine-spec/SKILL.md`
  - Prova: a skill refine-spec por um pedido, grava a rodada com `vostok perguntas abrir`, e o limite de 2 rodadas vale pelo painel (RF-04, CT-04, RN-03).
  - Trecho: a regra refine-spec-09 e o acréscimo no passo 5, e a versão menor subida.
  - Falha: a rodada continua só no chat ou sem o limite.
- `skill-adjust-spec.diff` (diff): `packages/harness/skills/default/adjust-spec/SKILL.md`
  - Prova: a skill adjust-spec por um pedido, grava a rodada com `vostok perguntas abrir` (RF-04, CT-04).
  - Trecho: a regra adjust-spec-09, e a versão menor subida.
  - Falha: a rodada continua só no chat.
- `skill-adjust-plan.diff` (diff): `packages/harness/skills/default/adjust-plan/SKILL.md`
  - Prova: a skill adjust-plan por um pedido, grava a rodada com `vostok perguntas abrir` (RF-04, CT-04).
  - Trecho: a regra adjust-plan-09, e a versão menor subida.
  - Falha: a rodada continua só no chat.
- `skill-adjust-task.diff` (diff): `packages/harness/skills/default/adjust-task/SKILL.md`
  - Prova: a skill adjust-task por um pedido, grava a rodada com `vostok perguntas abrir` (RF-04, CT-04).
  - Trecho: a regra adjust-task-13, e a versão menor subida.
  - Falha: a rodada continua só no chat.
- `skill-finish-task.diff` (diff): `packages/harness/skills/default/finish-task/SKILL.md`
  - Prova: a skill finish-task aceita como do usuário a conferência de print feita no painel (RF-06, RN-01, FA-02).
  - Trecho: a nova finish-task-09 e o caminho do pedido em "Prints sem conferência", e a versão menor subida.
  - Falha: a regra ainda diz que o pedido do painel não confere print.
- `skill-approve-commit.diff` (diff): `packages/harness/skills/default/approve-commit/SKILL.md`
  - Prova: a skill approve-commit atende `liberar-commits` e `desfazer-commits` com `--pedido` (RF-07, RN-02).
  - Trecho: a approve-commit-01 com os dois pedidos e o `--pedido` no `commit revert`, e a versão menor subida.
  - Falha: sem os pedidos novos ou sem o `--pedido` no desfazer.
- `skill-instalada.txt` (automática): `T="$(mktemp -d)" && cd "$T" && git init -q && cargo run -q --manifest-path "$OLDPWD/apps/cli/Cargo.toml" -- init --yes >/dev/null && cargo run -q --manifest-path "$OLDPWD/apps/cli/Cargo.toml" -- context load --skill process-requests | grep -n 'perguntas abrir\|aguardando a pergunta\|refinar-spec\|desfazer-commits'`
  - Prova: um projeto novo recebe a process-requests com o destino das perguntas e as ações novas (RF-02, RF-03).
  - Trecho: linhas com `vostok perguntas abrir`, `aguardando a pergunta`, `refinar-spec` e `desfazer-commits`.
  - Falha: saída vazia (o `grep` não acha nada e o comando termina com erro).
  - Espera: `aguardando a pergunta`
- `pedido-que-pergunta.txt` (manual): a saída de um chat do Claude Code que roda `/process-requests` sobre um pedido `refinar-spec` gravado pelo painel numa spec em rascunho com dúvidas, mostrando o `vostok perguntas abrir`, o pedido `feito` com "aguardando a pergunta P-NNN" e a pergunta gravada na pasta de abertas; depois, a resposta no painel e a nova rodada da refine-spec pelo pedido de resposta.
  - Prova: a skill que pergunta não trava nem recusa o pedido, e a resposta pelo painel retoma a skill de origem (RF-02, RF-03, RF-04, CT-02, CT-04).
  - Trecho: o `vostok perguntas abrir PED-NNN`, o resultado `aguardando a pergunta P-NNN` e o pedido de resposta atendido pela refine-spec.
  - Falha: pergunta feita no chat, pedido `recusado` por ter perguntado ou pedido de resposta sem rodar a refine-spec.
