<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** `apps/cli/src/commands/plan.rs` ganha `pub fn plan_outdated(harness: &Harness, spec_id: &str) -> bool`, que monta o `Workspace` do harness, localiza o plano (`plan_path`) e a spec refinada (`find_spec`) e devolve `plano existe && plan_is_outdated(plano, spec)`. A regra continua num lugar só, `plan_is_outdated`: `approved_hash` vazio ou `null`, ou `spec_hash` do plano diferente dele. `apps/cli/src/painel/read.rs` chama a função em `overview` (uma vez por spec aberta, com o `Harness` criado uma vez) e em `spec`, e grava o booleano `"plan_outdated"` logo depois de `"plan"`.

**Contrato com a TASK-02 (fixo).** CLI: `pub fn plan_outdated(harness: &Harness, spec_id: &str) -> bool`, verdadeiro só quando o plano existe e está desatualizado. API: `"plan_outdated"` ao lado de `"plan"` em cada spec de `GET /api/visao` e em `GET /api/specs/<ID>`. O campo `"plan"` continua com o mesmo significado (o arquivo do plano existe).

**Decisões.**
- *Onde a regra fica.* Escolhido: uma função pública fina sobre o `Workspace` privado, que reaproveita `plan_is_outdated`. Alternativas descartadas: repetir a comparação de hashes em `read.rs` ou no frontend (a regra divergiria da CLI, risco da spec e inferência B) e tornar públicos o `Workspace` e o método (expõe toda a configuração interna dos comandos `plan` por uma leitura só).
- *Erros viram falso.* Escolhido: spec fora de `refined`, configuração ilegível ou plano ausente devolvem `false`. Alternativa descartada: devolver `Result` e falhar a resposta, que derrubaria a visão geral inteira por uma spec. Segue o padrão de `commits`, que usa `unwrap_or_default`. Uma spec arquivada não tem plano e vem com `false`.
- *Formato do campo.* Escolhido: um booleano novo ao lado de `"plan"`. Alternativa descartada: trocar `"plan"` por um estado em texto (`sem`, `em_dia`, `desatualizado`), que quebraria o frontend atual antes da TASK-02.
- *Spec refined com plano antigo (CB-01).* Vem com `"plan_outdated":true`, pela regra da spec sem aprovação; a TASK-02 decide que, nesse caso, só as ações da spec aparecem (decisão 8).

**Riscos.** `Workspace::new` relê a configuração de tarefas a cada chamada; com dezenas de specs abertas, a visão geral fica um pouco mais lenta, sem mudança de resultado. O `init_layout` chamado por ele só grava o layout global na primeira vez.

**Referências da spec.** RF-01, CT-01, CB-01 (parte da API), CB-02, decisões 7 e 8, risco "A regra de plano desatualizado divergir entre o painel e a CLI".

**Arquivos em comum com a onda 1.** Nenhum: a TASK-03 muda `commands/spec.rs`, `commands/task.rs` e `tests/spec.rs`, e a TASK-04 muda `build.rs`. Atenção: a TASK-04 muda o `build.rs` da feature `painel`, usada pela Verificação desta tarefa; se as duas rodarem juntas e o `apps/painel/dist` estiver mais velho que `apps/painel/src`, o `cargo test --features painel` pode parar até rodar `pnpm --dir apps/painel build`.
