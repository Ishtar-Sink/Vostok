<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Tudo entra em `apps/cli/src/painel/actions.rs`, no mesmo padrão das ações atuais:
- **Pedidos novos** em `REQUEST_ACTIONS`: `criar-spec`, `refinar-spec`, `executar-onda`, `liberar-commits`, `desfazer-commits` (16 ao todo). Os dois de commit entram em `LOCKED_ACTIONS` com o aval do texto do pedido (`AvalTarget::Request`), como o `fazer-commit` (RN-02).
- **Ações diretas**, sem passar pelo agente (RN-01): `POST /api/conferencias` (conferir print) e `POST /api/limpeza` (listar e, com `confirmar=sim`, apagar).
- **`responder-pergunta`** continua um pedido, mas antes move a pergunta de `aberta/` para `respondida/` e grava a resposta nela.

**Decisões.**
- **Conferência de print.** `review_print_from_panel` fica em `task.rs`, ao lado de `approve_from_panel`, e grava a linha `conferência` com o mesmo `print_review` do `--reviewed` e a origem `painel` na coluna do resultado. Assim o `vostok task evidence` seguinte a reconhece sem mudança. Alternativa descartada: escrever o registro direto em `actions.rs`, que duplicaria `record_step` e violaria a regra de que o registro é só do `vostok`. O print rejeitado bloqueia a tarefa (FA-02), e ela é retomada pelo `desbloquear-tarefa` atual.
- **Prints que mudam entre execuções.** A regra da SPEC-020 continua valendo (teste `evidence_print_changed_after_the_review_is_pending_again`): se o comando do print gera outra imagem na finalização, ela precisa de nova conferência. Alternativa descartada: não regerar o print já conferido, que quebraria essa regra.
- **Liberar e desfazer exigem aval também na CLI.** `commit.rs` aceita `liberar-commits` no `commit approve` e exige `desfazer-commits` no `commit revert --pedido` com a trava ligada. Sem isso, o pedido com aval do painel seria recusado pelo `commit approve`, que hoje aceita só `fazer-commit`, e o desfazer não teria trava (RN-02). Alternativa descartada: reaproveitar o `fazer-commit`, contrariando a decisão 12 da spec.
- **Onda no texto.** `executar-onda` leva a onda na query e a grava como `Onda N.` no `## Pedido`, sem campo novo no front-matter, para manter o formato dos pedidos já gravados (decisão 11). Alternativa descartada: um campo `onda`, que mudaria o formato lido pelo painel e pelo `vostok pedidos`.
- **Resposta concorrente.** O `rename` de `aberta/` para `respondida/` é o ponto atômico. Quem chega depois recebe 409 e não cria pedido (CB-06). Alternativa descartada: conferir o hash da pergunta, que não evita a corrida entre duas abas.
- **Limpeza.** A rota chama as funções do `vostok pedidos prune` da TASK-05, sem lógica própria (RNF-03).

**Riscos.**
- A ordem com a TASK-05 é obrigatória: a TASK-08 não depende dela no plano, mas chama o `prune` dela.
- O frontend atual (`apps/painel/src/shared/action-bar.tsx`) não envia `pergunta=`; até a TASK-10, responder pelo painel dá 400.

**Arquivos em comum.**
- Na onda 3, `apps/cli/src/commands/pedidos.rs` também é mudado pela TASK-05 e pela TASK-07: execute a TASK-08 depois da TASK-05.
- `actions.rs` pode ter sido tocado pela TASK-02 e pela TASK-03 (onda 2), conforme o passo 4.
- `commit.rs`, `task.rs` e `trava_plano_commit.rs` ficam fora dos componentes do plano.

**Referências da spec.** RF-03, RF-05, RF-06, RF-07, RF-11, RN-01, RN-02, RN-06, CT-03, CT-07, CT-08, CB-01, CB-06, FA-02; decisões 9, 11 e 12.
