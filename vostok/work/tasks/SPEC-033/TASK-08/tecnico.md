<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Tudo entrou em `apps/cli/src/painel/actions.rs`, no mesmo padrão das ações que já existiam:
- **Pedidos novos** em `REQUEST_ACTIONS`: `criar-spec`, `refinar-spec`, `executar-onda`, `liberar-commits` e `desfazer-commits` (16 ao todo). `liberar-commits` e `desfazer-commits` entraram em `LOCKED_ACTIONS` com o aval do texto do pedido (`AvalTarget::Request`), como `fazer-commit` (RN-02).
- **Ações diretas**, sem passar pelo agente (RN-01): `POST /api/conferencias` (conferir print) e `POST /api/limpeza` (listar e, com `confirmar=sim`, apagar).
- **`responder-pergunta`** continua sendo um pedido, mas antes move a pergunta de `aberta/` para `respondida/` e grava a resposta nela.

**Decisões.**
- **Conferência de print.** `review_print_from_panel` ficou em `task.rs`, ao lado de `approve_from_panel`. Grava a linha `conferência` com o mesmo `print_review` do `--reviewed` e a origem `painel` na coluna do resultado, para que o `vostok task evidence` seguinte a reconheça sem mudança. Alternativa descartada: gravar o registro direto em `actions.rs`, o que duplicaria `record_step` e violaria a regra de que o registro é só do `vostok`. O print rejeitado bloqueia a tarefa (FA-02), que volta pelo `desbloquear-tarefa` existente.
- **Prints que mudam entre execuções.** A regra da SPEC-020 continua valendo (teste `evidence_print_changed_after_the_review_is_pending_again`): se o comando do print gera outra imagem na finalização, ela precisa de nova conferência. Alternativa descartada: não regerar o print já conferido, o que quebraria essa regra.
- **Liberar e desfazer exigem aval também na CLI.** `commit approve` passou a aceitar `liberar-commits`. `commit revert` ganhou `--pedido` e, com a trava ligada, exige um `desfazer-commits` com aval. Sem isso, o pedido com aval do painel seria recusado pelo `commit approve`, que aceitava só `fazer-commit`, e o desfazer não teria trava (RN-02). Alternativa descartada: reaproveitar `fazer-commit`, contrariando a decisão 12 da spec.
- **Onda no texto.** `executar-onda` grava a onda como `Onda N.` no início do `## Pedido`, sem campo novo no front-matter, para manter o formato dos pedidos já gravados (decisão 11). Alternativa descartada: um campo `onda`, que mudaria o formato lido pelo painel e pelo `vostok pedidos`.
- **Resposta concorrente.** O `rename` de `aberta/` para `respondida/` é o ponto atômico: quem chega depois recebe 409 e não cria pedido (CB-06). Alternativa descartada: conferir o hash da pergunta, que não evita a corrida entre duas abas.
- **Limpeza.** A rota chama `prune_candidates` e `prune_delete` do `vostok pedidos prune` (TASK-05), sem lógica própria (RNF-03).

**Feito.**
- Pedidos `criar-spec`, `refinar-spec`, `executar-onda`, `liberar-commits` e `desfazer-commits` aceitos, com a validação de campos em `request_fields`: `criar-spec` exige texto (400); `refinar-spec` e `executar-onda` exigem `spec` (400); `executar-onda` exige onda inteira maior que zero (400).
- `responder-pergunta` exige `pergunta` com ID válido e texto não vazio (400). Pergunta inexistente dá 404. Pergunta já respondida, ou `rename` que falha com NotFound, dá 409. Ao responder, a pergunta vai para `respondida/` com status `respondida`, `answered_at` e a seção `## Resposta`. O pedido criado usa a spec e a tarefa da pergunta.
- Rotas diretas `POST /api/conferencias` (erro 422 com mensagem) e `POST /api/limpeza` (erro 500 com mensagem).
- Testes do painel acrescentados ou ajustados conforme o plano. As checagens genéricas passaram a contar 16 ações e deixaram de incluir `responder-pergunta`, que tem teste próprio. Testes de `trava_plano_commit.rs` cobrem a liberação e o desfazimento por pedido.
- Mensagens de `require_request` passaram de "o commit exige" para "a operação exige".
- **Desvios.**
  1. Em vez de criar uma quinta cópia privada de `frontmatter_put` (já havia quatro), a de `spec.rs` virou `pub(crate)` e é usada no painel.
  2. O teste `commit_revert_needs_the_undo_request` usa `locked_git_project_with_a_base` (um commit inicial já liberado), porque o desfazer precisa da branch existente na liberação.
  3. Ajuste 1, aprovado pelo usuário: o "Espera" de `acoes-painel-testes.txt` perdeu o prefixo `test `. Os testes do painel saem com o caminho do módulo (`painel::actions::tests::...`), e o texto nunca casava. A primeira rodada de evidências deu erro por causa disso; depois do ajuste, testes, regressão e evidências passaram.
- **Verificação.** Testes: passed. Regressão: passed (3 comandos iguais ao baseline). Evidências: passed (2 em `TASK-08/evidencias/`). Clippy com `-D warnings` sem avisos.

**Riscos.**
- A TASK-05 foi concluída antes desta, porque a rota de limpeza chama o `prune` dela. As funções do `prune` já eram públicas, então `pedidos.rs` não precisou de mudança adicional.
- Liberar e desfazer commits continuam exigindo aval. A ação de limpeza só apaga com `confirmar=sim`; sem ele, apenas lista.

**Pontos de atenção.**
- O frontend atual (`apps/painel/src/shared/action-bar.tsx`) não envia `pergunta=`. Responder pelo painel dá 400 até a TASK-10, como previsto no plano.
- Um print conferido que for regerado em outra execução precisa de nova conferência (regra da SPEC-020).

**Arquivos alterados.**
- `apps/cli/src/painel/actions.rs`: 1 file changed, 432 insertions(+), 11 deletions(-)
- `apps/cli/src/commands/task.rs`: 1 file changed, 48 insertions(+)
- `apps/cli/src/commands/commit.rs`: 1 file changed, 24 insertions(+), 14 deletions(-)
- `apps/cli/tests/trava_plano_commit.rs`: 1 file changed, 89 insertions(+)
- `apps/cli/src/commands/pedidos.rs`: 1 file changed, 232 insertions(+), 21 deletions(-)
- `apps/cli/src/commands/spec.rs`: `frontmatter_put` passou a `pub(crate)` (desvio 1), sem outra mudança.

**Referências da spec.** RF-03, RF-05, RF-06, RF-07, RF-11, RN-01, RN-02, RN-06, CT-03, CT-07, CT-08, CB-01, CB-06, FA-02; decisões 9, 11 e 12.
