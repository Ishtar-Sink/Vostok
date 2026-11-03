<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Em `apps/cli/src/commands/spec.rs`, `drop_approval` passa a chamar `record_status(file, &refined_status)` depois de limpar os campos da aprovação, como `approve` já faz com `approved`. A função é o único ponto por onde os dois ajustes (`adjust record` e `adjust record-manual`) derrubam a aprovação, então os dois ganham a linha. O retorno antecipado para spec fora de `approved` fica como está: numa spec só refinada o status não muda e o histórico não ganha linha (CB-04). Em `apps/cli/src/commands/task.rs`, a constante `PANEL_WITHOUT_AVAL` vira `pub fn panel_without_aval(spec_id: &str) -> String`, que cita `work/avais/<SPEC>.log`; ela é usada pela validação da tarefa (com o ID da spec da tarefa) e pela da spec, em `spec.rs`.

**Decisões.**
- *Onde gravar o histórico.* Escolhido: dentro de `drop_approval`. Alternativa descartada: chamar `record_status` em `record` e em `record_manual`, que repetiria a chamada e deixaria um terceiro chamador futuro sem a linha. Critério: decisão 9 da spec (inferência D), um único lugar que muda o status.
- *Mensagem com o log da spec.* Escolhido: uma função com o ID da spec no caminho (`work/avais/SPEC-001.log`), para o usuário ver o arquivo exato. Alternativas descartadas: manter uma constante com `<SPEC>` literal, menos útil para quem lê a recusa; e montar o caminho por `Harness::approvals_dir()` (que segue `paths.approvals`), que tornaria a mensagem dependente da configuração e diferente do texto fixado na spec. Critério: RF-07 e decisão 10 (inferência E), o texto cita `work/avais/<SPEC>.log`.
- *Golden.* As fixtures dos scripts bash não gravam o histórico de status, e `normalize` (`apps/cli/tests/common/golden.rs`) já ignora as linhas `status_history:`; os casos `spec/0153`, `0155` e `0219`, que derrubam a aprovação, continuam iguais.

**Fora dos componentes da spec.** `apps/cli/tests/aval.rs`: a constante do teste compara o texto antigo da mensagem e precisa mudar; ganha o teste `panel_without_aval_cites_the_spec_log` (CT-09).

**Fora do escopo.** O comentário de `approval_lock` em `packages/harness/config/main.yml` e a regra `agent/boundaries-05` ainda citam `avais.log`; não são mensagem da CLI (RF-07 cobre a ajuda dos diálogos, a mensagem de sucesso e a validação da CLI) e não entram nesta tarefa. Os textos do painel ficam com a TASK-02.

**Riscos.** Baixo. Specs ajustadas antes desta versão continuam com o histórico sem a volta para refined; nada é reescrito.

**Referências da spec.** RF-06, RF-07, CT-08, CT-09 (parte da CLI), CB-04, causa raiz (`drop_approval` sem `record_status`; textos não atualizados na TASK-03 da SPEC-033), decisões 9 e 10.

**Arquivos em comum com a onda 1.** Nenhum: a TASK-01 toca `plan.rs`, `painel/read.rs` e `tests/painel.rs`, e a TASK-04, `build.rs`.
