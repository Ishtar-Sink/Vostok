<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.**
- `spec.rs` oferece `pub fn record_status(file, status)`, que acrescenta `<data> <status>` à chave `status_history` do front-matter da spec, com as entradas separadas por `; `.
- Cada comando que já grava um status chama essa função logo depois de gravá-lo:
  - `spec approve` grava `approved`;
  - `plan approve mark-planned` grava `planned`, em `plan.rs`;
  - `task finish` da última tarefa grava `implemented`, em `task.rs`;
  - `finalize archive` grava `archived` no `spec.md` arquivado, em `finalize.rs`.
- A `GET /api/atividades`, em `read.rs`, ganha a fonte `status_rows`, das specs abertas e arquivadas. A `request_rows` passa a gerar uma linha por transição do histórico que a TASK-02 grava no pedido, lendo também `<archive>/*/pedidos/` (TASK-05).

**Decisões.**
1. **Uma função em `spec.rs`, chamada pelos quatro comandos.** Alternativa descartada: escrever a linha em cada comando, o que repetiria o formato quatro vezes e deixaria os leitores à mercê de diferenças. Por isso `plan.rs`, `task.rs` e `finalize.rs` entram em "Mudar", fora dos componentes do plano: são eles que gravam `planned`, `implemented` e `archived`, e a RF-12 pede a linha "gravada pelos comandos que já as fazem".
2. **O histórico fica no front-matter, não numa seção do corpo.** O `content_hash` da spec é o sha256 do corpo inteiro (`fn content_hash` em `spec.rs` e a conferência em `plan.rs`). Uma linha no corpo depois da aprovação faria o `plan create`/`approve` acusar "spec alterada depois da aprovação" e a detecção de edição manual disparar.
   - Alternativa descartada: uma seção "Histórico de status" excluída do hash, que exigiria mudar o cálculo em dois lugares e um filtro de várias linhas nas fixtures.
   - Alternativa descartada: um arquivo à parte por spec, que pediria um caminho novo na configuração e no arquivamento.
   - Custo aceito: o histórico fica numa linha só, menos legível à mão, mas o painel o mostra por linha.
3. **A normalização das fixtures (`tests/common/golden.rs`) ignora a linha `status_history:`**, como já ignora `approved_by`. Os scripts bash nunca a gravaram, e o resto do arquivo continua sendo comparado.
4. **Pedidos sem histórico** (gravados antes da TASK-02 ou migrados) mantêm as duas linhas de hoje, criação e estado atual. Assim a página não perde nada do que já mostrava.

**Riscos.**
- O formato exato do histórico do pedido é definido pela TASK-02. O passo 5 lê o que ela grava, por isso a TASK-06 depende dela.
- Uma spec arquivada antes desta versão não tem `status_history` e só aparece pelos ajustes.

**Arquivos em comum na onda 3.** `apps/cli/src/commands/finalize.rs` e `apps/cli/tests/finalize.rs` são alterados também pela TASK-05, no arquivamento de pedidos, perguntas e avais. O Coordenador executa as duas em sequência. `read.rs` não é dividido com outra tarefa da onda.

**Referências da spec.** RF-12, CT-13; estudo, seções "Status das entidades" e "Fontes de atividade".
