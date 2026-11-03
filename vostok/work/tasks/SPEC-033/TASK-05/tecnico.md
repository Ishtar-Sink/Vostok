<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.**
- **Arquivamento.** O `vostok finalize archive` ganha, em `finalize.rs`, a coleta dos arquivos de comunicação da spec. Eles são copiados para `<archive>/<SPEC>/` junto com as tarefas, pelo mesmo `copy_to_archive`, e os originais só são removidos depois de a cópia inteira dar certo. É o mesmo esquema de hoje: em falha, o arquivo morto é desfeito e nada some.
- **Limpeza.** O `vostok pedidos prune [--confirmar]` fica em `pedidos.rs`, com duas funções públicas, `prune_candidates` e `prune_delete`, para a ação `limpar-pedidos` do painel (TASK-08 e TASK-10) chamar a mesma regra sem passar pelo agente.

**Decisões e alternativas.**
1. **Destino no arquivo morto:** `pedidos/`, `perguntas/` e `avais/` sob `<archive>/<SPEC>/`, com o último trecho de `paths.requests`, `paths.questions` e `paths.approvals` (RF-16), sem subpasta de estado.
   - Alternativa descartada: manter as subpastas de estado. O estado já está no front-matter (RN-06) e o arquivo morto não tem mais transição. Pastas planas simplificam o `prune`, a numeração das perguntas (TASK-04) e a leitura das atividades (TASK-06).
   - Critério: um leitor só precisa de `<archive>/*/pedidos/*.md`.
2. **Pedidos em andamento não são arquivados.** A spec é arquivada justamente durante o pedido `finalizar-spec`, que está `em_andamento`. Movê-lo faria o `vostok pedidos concluir` da `process-requests` falhar logo depois.
   - Alternativas descartadas:
     - arquivar todos, o que quebra o pedido em curso;
     - recusar o archive com pedido aberto da spec, o que impede finalizar pelo painel;
     - mover no `concluir` quando a spec já está arquivada, o que mexe nas transições da TASK-02 e no CT-09.
   - Escolha: arquivar só `feito` e `recusado`, avisar na saída quais ficaram e deixar o `prune` limpar os concluídos de spec já arquivada (motivo `spec arquivada`, sem regra de idade).
   - Critério: nenhum fluxo do painel quebra e nada fica para sempre na pasta de trabalho.
3. **Perguntas vão todas, abertas ou respondidas.** Uma pergunta aberta de uma spec arquivada não tem mais o que destravar. Alternativa descartada: deixar as abertas, que ficariam em "Aguardando você" sem efeito.
4. **Avais copiados byte a byte e depois removidos**, dentro da mesma transação do arquivamento. Nenhuma linha é reescrita (RN-05, DT-014). Alternativa descartada: `rename` isolado antes da cópia das tarefas, que deixaria o log fora do lugar se a cópia falhasse depois.
5. **O `prune` nunca toca em `.log` nem em `paths.approvals`** (RNF-03). Sem `--confirmar`, só lista (decisão 7 da spec). O limite de 30 dias é a constante `PRUNE_DAYS`, porque a spec fixa o valor. Ele é comparado como texto ISO-8601 com `utc_datetime_days_ago`, sem dependência nova de datas. Para pergunta, o limite usa `answered_at` quando houver (TASK-08) e `created_at` quando não houver.
6. **Saída do archive sem mudança** quando a spec não tem pedidos, perguntas nem avais, para o `golden_finalize` continuar igual.

**Riscos.**
- Apagar o arquivo morto tira da atividade o histórico desses pedidos. Por isso o `prune` lista e cita as specs antes.
- O `vostok avais <SPEC>` da TASK-03 lê `<approvals>/<SPEC>.log`. Depois do arquivamento, os avais da spec ficam em `<archive>/<SPEC>/avais/`. Se a TASK-03 não ler o arquivo morto, o comando passa a responder vazio para specs arquivadas, o que é esperado porque elas não pedem mais aval.

**Arquivos fora dos componentes do plano.** `apps/cli/src/support.rs`, para o cálculo do limite de 30 dias, e os testes `apps/cli/tests/finalize.rs` e `apps/cli/tests/pedidos.rs`.

**Arquivos em comum na onda 3.**
- `apps/cli/src/commands/pedidos.rs` e `apps/cli/tests/pedidos.rs` são alterados também pela TASK-07 (disparo).
- `apps/cli/src/commands/finalize.rs` e `apps/cli/tests/finalize.rs` são alterados também pela TASK-06 (linha `archived` no histórico de status).

O Coordenador executa essas tarefas em sequência.

**Referências da spec.** RF-10, RF-11, RF-16, RNF-03, RN-05, RN-06, CB-02, CB-05, CT-11, CT-12; decisões 6 e 7; contrato de `vostok pedidos prune [--confirmar]`.
