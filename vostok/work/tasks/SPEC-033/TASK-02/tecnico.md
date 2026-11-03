<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** `apps/cli/src/commands/pedidos.rs` ganhou `Requests` e `Questions`, que conhecem a pasta configurada (TASK-01), a subpasta de cada estado e a migração. Todo código que montava `work/pedidos/<ID>.md` ou lia `work/perguntas/` passou a usá-los: os comandos `vostok pedidos`, o painel (`read.rs` e `actions.rs`) e a trava do commit (`commit.rs` com `aval.rs`).

**Decisões (perfil julgamento: alternativas e critérios).**
- *Onde fica a verdade do estado.* Escolhido: o front-matter (`estado`, `status`) continua sendo a verdade e a pasta é a apresentação (RN-06, decisão 11). A transição grava o front-matter e o histórico e só depois faz o `rename`; uma interrupção entre os dois deixa o arquivo na pasta antiga com o estado novo, e a próxima transição dele o acha pelo `find` nas quatro pastas. Alternativa descartada: renomear primeiro, porque a pasta diria um estado que o arquivo ainda não tem. Critério: nenhum leitor do front-matter vê um estado que não foi gravado.
- *Quando migrar.* Escolhido: no `open`, em todo comando, só para arquivos soltos na raiz e para subpastas cujo nome não está na configuração (formato antigo, FA-04, e nomes trocados, CB-07). As quatro pastas configuradas são só listadas, então `listar pendente` lê só `pendente/` (RNF-01). Alternativas descartadas: um comando de migração explícito (o usuário teria de lembrar de rodar, e a spec pede a migração na primeira execução) e conferir o estado de todos os arquivos em todo comando (quebraria o RNF-01). Critério: custo proporcional ao que está fora do lugar, e nenhum trabalho com as pastas já organizadas.
- *Estado desconhecido.* O arquivo fica onde está e nada é apagado: a migração nunca perde um pedido.
- *Formato do histórico.* Seção `## Histórico` entre `## Pedido` e `## Resultado`, numa tabela `| Data | De | Para |`. É o contrato que a TASK-06 lê para a `GET /api/atividades` (RF-12). Alternativa descartada: lista solta, mais difícil de ler com a mesma função das tabelas de ajustes.
- *Próximo ID.* Calculado pelos nomes de arquivo das quatro pastas e da raiz, sem abrir arquivos, para dois pedidos não terem o mesmo número depois que o primeiro sai de `pendente/`.
- *Painel.* Constrói um `Harness { root }` a partir de `painel.root` em vez de guardar a configuração no `Painel`, para pegar uma troca de configuração sem reiniciar o painel. Com configuração inválida, a leitura mostra listas vazias e a gravação responde 500 com a mensagem do `folders()`.

**Arquivos fora dos componentes da tarefa.** `aval.rs` (componente da TASK-03, mesma onda 2): só a assinatura de `request_state` e `request_text_hash`, que passaram a receber o caminho. `actions.rs` (componente da TASK-08, onda 3, que depende desta): a gravação do pedido em `pendente/`, sem ela o painel gravaria no formato antigo. `commit.rs` (de nenhuma tarefa): a conferência do pedido na trava de commit. Os testes `tests/pedidos.rs` e `tests/painel.rs`.

**Arquivos em comum com a onda 2.** `apps/cli/src/aval.rs`, `apps/cli/src/painel/actions.rs` e `apps/cli/src/painel/read.rs` também mudam na TASK-03 (avais por spec); `apps/cli/tests/painel.rs` também. A execução da TASK-02 e da TASK-03 foi em sequência, pelo Coordenador.

**Feito.**
- `apps/cli/src/commands/pedidos.rs`: `Requests` e `Questions` com a pasta por estado, a transição por `rename` (depois de gravar o front-matter e o histórico), a seção `## Histórico`, o próximo ID e a migração. `list`, `iniciar`, `concluir`, `recusar` e `vigiar` usam os dois. O `vigiar` passou a citar, na instrução ao `codex`, o caminho do pedido encontrado, relativo ao projeto, e não mais `vostok/work/pedidos/`.
- `apps/cli/src/painel/read.rs`: atividades dos pedidos de todas as subpastas; perguntas abertas lidas só da subpasta de abertas. Se a configuração for inválida, as listas saem vazias.
- `apps/cli/src/painel/actions.rs`: `REQUESTS_DIR`, `write_request` e `next_request_id` foram removidos. O pedido novo é gravado em `pendente/` pelo `create`, com o próximo ID de todas as subpastas. Na falha do aval, o caminho devolvido é removido.
- `apps/cli/src/aval.rs`: `request_state` e `request_text_hash` recebem o caminho do arquivo.
- `apps/cli/src/commands/commit.rs`: `require_request` acha o pedido pelo `Requests::find`, com a mesma mensagem citando a pasta configurada.
- Testes em `apps/cli/tests/pedidos.rs` (19 no total, 7 novos `folders_*`): `folders_start_and_finish_move_the_file`, `folders_record_the_history`, `folders_list_pending_reads_only_pendente`, `folders_migrate_loose_requests`, `folders_migrate_renamed_folders`, `folders_follow_the_config` e `folders_keep_unknown_states_in_place`.
- Testes unitários em `pedidos.rs`: `questions_migrate_loose_files` e `questions_move_to_answered`.
- Testes do painel: `next_request_id_follows_the_highest_of_every_folder` (renomeado de `next_request_id_follows_the_highest`) e `pedidos_e_perguntas_vem_das_subpastas`, que cobre um pedido em `pedidos/feito/` e uma pergunta em `perguntas/aberta/`. O teste de ponta a ponta de `tests/painel.rs` lê o pedido em `pedidos/pendente/`.
- **Mudança fora do planejado em `apps/cli/tests/trava_plano_commit.rs`.** O helper `write_request` passou a gravar o pedido na subpasta do seu estado e a apagar as cópias das outras subpastas. Motivo: gravar o pedido solto fazia a migração deixar duplicatas entre as iterações do teste `commit_refuses_a_finished_request`.

**Verificação.** Testes, regressão e evidências passaram na primeira tentativa. A regressão comparou quatro comandos com o baseline e não achou diferença; um comando ganhou um acréscimo (o teste novo). As evidências ficaram em `TASK-02/evidencias/` (2 arquivos), e a saída dos testes em `validation/tests.out`.

**Pontos de atenção.**
- O registro de execução tem um `check` reprovado às 17:52:50Z e aprovado em 17:53:15Z: a reprovação foi o `commit_refuses_a_finished_request`, corrigido pela mudança no helper de `trava_plano_commit.rs` descrita acima.
- Uma pasta configurada fora de `work/` não é vista pelo aviso ao vivo do painel (`live.rs` vigia só `work/`).
- A migração roda em todo comando, mas só mexe em arquivos soltos e em subpastas fora da configuração. Um arquivo com estado desconhecido fica onde está e não é apagado.
- As skills que citam `work/pedidos/` dependem da TASK-11.
- Fixtures de teste soltas, como `perguntas/P-1.md` e `pedidos/PED-001.md`, continuam válidas porque passam pela migração. Novos testes devem gravar nas subpastas do estado.

**Referências da spec.** RF-08, RF-12 (formato do histórico), RF-16, RN-06, RNF-01, FA-04, CB-07, CT-09, CT-17, CT-19 (parte dos pedidos), decisões 5, 8 e 11.
