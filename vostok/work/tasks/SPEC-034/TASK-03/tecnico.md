<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Feito.** `drop_approval` passou a gravar a volta para `refined` no `status_history`, e a validação de aprovação do painel passou a citar `work/avais/<SPEC>.log` pela função `panel_without_aval`, no lugar da constante antiga. Foram acrescentados três testes de histórico em `apps/cli/tests/spec.rs` e um teste de mensagem em `apps/cli/tests/aval.rs`. Verificação, regressão (5 comandos iguais ao baseline) e evidências passaram.

**Abordagem.** Em `apps/cli/src/commands/spec.rs`, `drop_approval` passou a chamar `record_status(file, &refined_status)` depois de limpar os campos da aprovação, como `approve` já fazia com `approved`. Essa função é o único ponto pelo qual os dois ajustes (`adjust record` e `adjust record-manual`) derrubam a aprovação, então os dois passaram a gravar a linha. O retorno antecipado para spec fora de `approved` foi mantido: numa spec só refinada o status não muda e o histórico não ganhou linha (CB-04). Em `apps/cli/src/commands/task.rs`, a constante `PANEL_WITHOUT_AVAL` virou `pub fn panel_without_aval(spec_id: &str) -> String`, usada pela validação da tarefa (com o ID da spec da tarefa) e pela validação da spec, em `spec.rs`.

**Decisões.**
- *Onde gravar o histórico.* Escolhido: dentro de `drop_approval`. Descartado: chamar `record_status` em `record` e em `record_manual`, o que repetiria a chamada e deixaria um futuro chamador sem a linha. Critério: decisão 9 da spec (inferência D), um único lugar que muda o status.
- *Mensagem com o log da spec.* Escolhido: função com o ID da spec no caminho, para o usuário ver o arquivo exato. Descartados: constante com `<SPEC>` literal (menos útil para quem lê a recusa) e caminho montado por `Harness::approvals_dir()` (dependeria da configuração e divergiria do texto fixado na spec). Critério: RF-07 e decisão 10 (inferência E).
- *Golden.* As fixtures dos scripts bash não gravam o histórico de status, e `normalize` (`apps/cli/tests/common/golden.rs`) já ignora as linhas `status_history:`; os casos `spec/0153`, `0155` e `0219`, que derrubam a aprovação, continuaram iguais.

**Fora dos componentes da spec.** `apps/cli/tests/aval.rs` precisou mudar, porque a constante do teste comparava o texto antigo da mensagem. Ganhou o teste `panel_without_aval_cites_the_spec_log` (CT-09), que confere que cada recusa cita `work/avais/SPEC-001.log` e não cita `avais.log`.

**Pontos de atenção.**
- Ficaram fora desta tarefa o comentário de `approval_lock` em `packages/harness/config/main.yml` e a regra `agent/boundaries-05`, que ainda citam `avais.log`. Não são mensagem da CLI (RF-07 cobre a ajuda dos diálogos, a mensagem de sucesso e a validação da CLI), então não entraram aqui.
- Os únicos `avais.log` que restaram em `apps/cli/src` são dados de teste do log antigo, usados na migração, e não mensagens.
- Specs ajustadas antes desta versão continuam com o histórico sem a volta para `refined`; nada foi reescrito.
- A tarefa foi executada em paralelo com a TASK-01. `cargo fmt` rodou no crate inteiro, mas o diff final ficou restrito aos arquivos desta tarefa.
- Risco baixo: correção pequena e isolada.

**Arquivos em comum com a onda 1.** Nenhum: a TASK-01 tocou `plan.rs`, `painel/read.rs` e `tests/painel.rs`, e a TASK-04, `build.rs`.

**Referências da spec.** RF-06, RF-07, CT-08, CT-09 (parte da CLI), CB-04, causa raiz (`drop_approval` sem `record_status`; textos não atualizados na TASK-03 da SPEC-033), decisões 9 e 10.
