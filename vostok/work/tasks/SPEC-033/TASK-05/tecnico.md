<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.**
- **Arquivamento.** `finalize.rs` ganhou `communication_files`, que reúne os arquivos de comunicação da spec: pedidos feito e recusado, perguntas aberta e respondida, o log `<approvals>/<SPEC>.log` e a lista `in_flight` com os pedidos pendentes e em andamento, que só são citados. Os arquivos são copiados para `<archive>/<SPEC>/` pelo `copy_to_archive`, depois das tarefas. Os originais são removidos por último, no `archive`, depois da remoção da pasta de conteúdo. A remoção só acontece depois da cópia.
- **Limpeza.** `pedidos.rs` ganhou `vostok pedidos prune [--confirmar]` e duas funções públicas, `prune_candidates` e `prune_delete`, para que o painel use a mesma regra sem passar pelo agente. A ligação ao painel fica para TASK-08 e TASK-10.

**Decisões e alternativas.**
1. **Destino no arquivo morto:** `pedidos/`, `perguntas/` e `avais/` sob `<archive>/<SPEC>/`, com o último trecho da pasta configurada (`paths.requests`, `paths.questions`, `paths.approvals`, RF-16), sem subpasta de estado.
   - Alternativa descartada: manter as subpastas de estado. O estado já está no front-matter (RN-06), e o arquivo morto não tem mais transição. Pastas planas simplificam o `prune`, a numeração das perguntas (TASK-04) e a leitura das atividades (TASK-06).
   - Critério: um leitor só precisa de `<archive>/*/pedidos/*.md`.
2. **Pedidos em andamento não são arquivados.** A spec é arquivada durante o pedido `finalizar-spec`, que está `em_andamento`. Movê-lo quebraria o `vostok pedidos concluir` da `process-requests` logo depois.
   - Alternativas descartadas:
     - arquivar todos, o que quebra o pedido em curso;
     - recusar o archive com pedido aberto da spec, o que impede finalizar pelo painel;
     - mover no `concluir` quando a spec já está arquivada, o que mexe nas transições da TASK-02 e no CT-09.
   - Escolha: arquivar só `feito` e `recusado`, citar na saída os que ficaram na pasta de trabalho e deixar o `prune` limpar os concluídos de spec já arquivada (motivo `spec arquivada`, sem regra de idade).
   - Critério: nenhum fluxo do painel quebra, e nada fica para sempre na pasta de trabalho.
3. **Perguntas vão todas, abertas ou respondidas.** Uma pergunta aberta de uma spec arquivada não tem mais o que destravar. Alternativa descartada: deixá-las na pasta de trabalho, onde ficariam em "Aguardando você" sem efeito.
4. **Avais copiados byte a byte e depois removidos.** Nenhuma linha é reescrita (RN-05, DT-014). Alternativa descartada: um `rename` isolado antes da cópia das tarefas, que deixaria o log fora do lugar se a cópia falhasse depois.
5. **O `prune` nunca toca em `.log` nem em `paths.approvals`** (RNF-03). Sem `--confirmar`, só lista (decisão 7 da spec). O limite de 30 dias é a constante `PRUNE_DAYS`, porque a spec fixa o valor. Ele é comparado como texto ISO-8601 com `utc_datetime_days_ago`, sem dependência nova de datas. Para pergunta, usa `answered_at` quando existe e `created_at` quando não existe.
6. **Saída do archive sem mudança quando não há nada a levar.** A linha de resumo e a lista de pedidos em andamento só aparecem quando há arquivo, e o `golden_finalize` continuou igual.
7. **Visibilidade das constantes de estado.** `PENDING`, `IN_PROGRESS`, `DONE` e `REFUSED` de `pedidos.rs` passaram a `pub`, para que `finalize.rs` use os mesmos nomes de estado. Não estava no plano original.

**Riscos.**
- Apagar o arquivo morto tira da atividade o histórico desses pedidos. Por isso o `prune` lista e cita as specs antes de apagar, e só apaga com `--confirmar`.
- O `vostok avais <SPEC>` da TASK-03 lê `<approvals>/<SPEC>.log`. Depois do arquivamento, os avais da spec ficam em `<archive>/<SPEC>/avais/`. Se a TASK-03 não ler o arquivo morto, o comando responde vazio para specs arquivadas. Isso é esperado, porque elas não pedem mais aval. Esta tarefa não verificou essa leitura.

**Feito.**
- `apps/cli/src/commands/finalize.rs`: arquivamento dos pedidos, perguntas e avais da spec, com a linha de resumo e a lista de pedidos em andamento (1 arquivo, 113 inserções e 3 remoções).
- `apps/cli/src/commands/pedidos.rs`: comando `vostok pedidos prune [--confirmar]`, constante `PRUNE_DAYS = 30`, `PruneItem`, `prune_candidates` e `prune_delete`, e a tabela `| Arquivo | Spec | Motivo |`.
- `apps/cli/src/support.rs`: `format_utc`, `utc_datetime_days_ago` público, e `utc_datetime` passou a chamar `format_utc` (1 arquivo, 14 inserções e 2 remoções).
- `apps/cli/tests/finalize.rs`: helper `write_file` e teste `archive_leva_pedidos_perguntas_e_avais_da_spec` (129 inserções).
- `apps/cli/tests/pedidos.rs`: helpers `write_front`, `prune_request` e `prune_fixture` (o nome `write_front` porque `write_at` já existia da TASK-02) e quatro testes `prune_*`.
- Testes do arquivamento e do `prune` passaram. A regressão rodou os dois comandos do baseline com o mesmo resultado, e as evidências foram gravadas em `TASK-05/evidencias/`, com dois arquivos. Tudo passou na primeira tentativa.

**Pontos de atenção.**
- O `cargo clippy -D warnings` e o `cargo fmt --check` do passo 6 fazem parte da Verificação e passaram no `vostok task tests` (saída em `evidencias/validation/tests.out`).
- Pedidos em andamento de uma spec arquivada ficam na pasta de trabalho até serem concluídos. Depois disso, o `prune` os limpa com o motivo `spec arquivada`.
- `prune --confirmar` apaga os arquivos listados sem volta. Os avais ficam e nunca são apagados.
- As pastas `pedidos/` e `perguntas/` do arquivo morto são removidas quando ficam vazias.

**Arquivos fora dos componentes do plano.** `apps/cli/src/support.rs`, para o cálculo do limite de 30 dias, e os testes `apps/cli/tests/finalize.rs` e `apps/cli/tests/pedidos.rs`. Também `apps/cli/src/commands/pedidos.rs`, nas constantes de estado tornadas públicas (decisão 7).

**Arquivos em comum na onda 3.**
- `apps/cli/src/commands/pedidos.rs` e `apps/cli/tests/pedidos.rs` são alterados também pela TASK-07 (disparo).
- `apps/cli/src/commands/finalize.rs` e `apps/cli/tests/finalize.rs` são alterados também pela TASK-06 (linha `archived` no histórico de status).

O Coordenador executou essas tarefas em sequência.

**Referências da spec.** RF-10, RF-11, RF-16, RNF-03, RN-05, RN-06, CB-02, CB-05, CT-11, CT-12; decisões 6 e 7; contrato de `vostok pedidos prune [--confirmar]`.
