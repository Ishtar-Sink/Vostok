<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.**
- `spec.rs` ganhou `pub fn record_status(file, status)`, que acrescentou `<data> <status>` à chave `status_history` do front-matter da spec, com as entradas separadas por `; `. A constante `STATUS_HISTORY_FIELD` guarda o nome da chave.
- Cada comando que já grava um status chama essa função logo depois de gravá-lo:
  - `spec approve` grava `approved`;
  - `plan approve mark-planned` grava `planned`, em `plan.rs`;
  - `task finish` da última tarefa grava `implemented`, em `task.rs`;
  - `finalize archive` grava `archived` no `spec.md` arquivado, em `finalize.rs`.
- A `GET /api/atividades`, em `read.rs`, ganhou a fonte `status_rows`, que lê o histórico das specs abertas e arquivadas. `request_rows` passou a gerar uma linha por transição do histórico que a TASK-02 grava no pedido, e passou a ler também `<archive>/*/<pasta de pedidos>/` (TASK-05). Pedido sem histórico mantém a criação e o estado atual.
- `tests/common/golden.rs` passou a descartar a linha `status_history:` na normalização, como já fazia com `approved_by`.

**Decisões.**
1. **Uma função em `spec.rs`, chamada pelos quatro comandos.** Foi descartada a alternativa de gravar a linha em cada comando, que repetiria o formato quatro vezes e deixaria os leitores à mercê de diferenças. Por isso `plan.rs`, `task.rs` e `finalize.rs` foram alterados, mesmo fora dos componentes do plano: são eles que gravam `planned`, `implemented` e `archived`.
2. **O histórico ficou no front-matter, não numa seção do corpo.** O `content_hash` da spec é o sha256 do corpo inteiro (`content_hash` em `spec.rs` e a conferência em `plan.rs`). Uma linha no corpo depois da aprovação faria o plano acusar "spec alterada depois da aprovação" e dispararia a detecção de edição manual. Foram descartadas uma seção "Histórico de status" excluída do hash, que exigiria mudar o cálculo em dois lugares e um filtro de várias linhas nas fixtures, e um arquivo à parte por spec, que pediria um caminho novo na configuração e no arquivamento. O custo aceito é um histórico numa linha só, menos legível à mão; o painel o mostra por linha.
3. **A normalização das fixtures ignora a linha `status_history:`.** Os scripts bash nunca a gravaram, e o resto do arquivo continua sendo comparado.
4. **Pedidos sem histórico** (gravados antes da TASK-02 ou migrados) mantêm as duas linhas de antes, criação e estado atual, para que a página não perdesse nada do que já mostrava.

**Riscos.**
- O formato do histórico do pedido foi definido pela TASK-02. A leitura em `request_rows` depende do que ela grava, por isso a TASK-06 dependia da TASK-02.
- Uma spec arquivada antes desta versão não tem `status_history` e só aparece pelos ajustes.

**Arquivos em comum na onda 3.** `apps/cli/src/commands/finalize.rs` e `apps/cli/tests/finalize.rs` também foram alterados pela TASK-05, no arquivamento de pedidos, perguntas e avais. O Coordenador executou as duas em sequência. `read.rs` não foi dividido com outra tarefa da onda.

**Feito.**
- Alterados `spec.rs`, `plan.rs`, `task.rs`, `finalize.rs`, `painel/read.rs`, `tests/common/golden.rs` e os testes de `spec`, `plan`, `task` e `finalize`.
- Testes novos dos comandos: `approve_records_the_status_history`, `mark_planned_records_the_status_history`, `finish_records_implemented_in_the_status_history` e `archive_records_archived_in_the_status_history`.
- Testes novos do painel: `atividades_trazem_o_historico_de_status_das_specs`, `atividades_trazem_as_transicoes_dos_pedidos` e `atividades_leem_os_pedidos_arquivados`.
- Verificação, regressão (5 comandos iguais ao baseline) e evidências passaram na primeira tentativa. As evidências estão em `TASK-06/evidencias/` (2 arquivos).
- **Desvio:** a fixture `running_task(&[])` prevista no passo 1 não tinha comandos de Baseline, e `task baseline` falhava. O teste `finish_records_implemented_in_the_status_history` usa `running_task(&[("a.txt", "a\n")])`.

**Referências da spec.** RF-12, CT-13; estudo, seções "Status das entidades" e "Fontes de atividade".

**Pontos de atenção.**
- A fixture do teste de `finish` diverge do passo 1 do `execucao.md` (ver Desvio acima).
- O `cargo clippy --features painel --all-targets -- -D warnings` e o `cargo fmt --check` do passo 7 fazem parte da Verificação e passaram no `vostok task tests` (saída em `evidencias/validation/tests.out`).
- A linha `status_history:` fica fora da comparação das fixtures, então a presença dela é verificada só pelos testes dedicados.
- Specs arquivadas antes desta versão não têm o histórico de status.
