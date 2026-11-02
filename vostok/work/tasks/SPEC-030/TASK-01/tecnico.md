<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Cria `apps/cli/src/aval.rs`, um módulo sem estado que lê `vostok/work/avais.log`, a chave `approval_lock` de `main.yml` e o front-matter de um pedido. Os comandos (TASK-06 e TASK-07) só chamam `aval::require`. Cobre RF-01, RF-02, CT-02, CB-02 e CB-03.

**Decisões.** O hash do aval é o campo já gravado pelo `vostok` em cada item (`approved_hash` da tarefa, `content_hash` do plano e da spec), e não um SHA-256 recalculado do arquivo, porque o arquivo muda durante a execução (registro, estado) sem mudar o conteúdo aprovado. No pedido de commit, o hash é o do texto de `## Pedido`, que o loop não altera; o arquivo inteiro foi descartado porque o estado do pedido muda. Linha malformada não derruba a leitura: ela é ignorada na conferência e listada, para a validação acusar.

**Riscos.** Confundir a ação das linhas antigas: as de 4 colunas valem só como `aprovar`, nunca como aval de finalização (RNF-02).

**Arquivos em comum na onda.** Nenhum.


**O que foi feito.** `apps/cli/src/aval.rs` traz:
- `Aval`, `read` (linhas de 4 colunas viram `aprovar`; linha malformada vai para a lista de problemas);
- `lock_enabled`, `require` (recusa citando a ação, o ID, o hash e, se existir, o hash do aval antigo);
- `has_panel_approval`, `request_state` e `request_text_hash`.

`lib.rs` registra o módulo, e `packages/harness/config/main.yml` ganhou `approval_lock: false`, comentado.

**Verificação.** Os 8 testes de `aval` e os de `init_update` passaram, com clippy sem avisos; a regressão ficou igual ao baseline. A evidência é `aval-leitura.txt`.

**Na execução.** Os testes foram escritos junto com o código.
