<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Porta para a `main`, com testes, o que o spike fez em `task.rs`, `spec.rs` e `plan.rs`: `Origin`, `approved_by`, `approve_from_panel` e `record_manual_at`. Cobre RF-07, CT-06 e CT-07; atende RN-01.

**Decisões.** A origem é um tipo Rust, nunca argumento de linha de comando (RN-01), para o agente não conseguir gravar `painel` pela CLI. Nos golden, a linha `approved_by: chat` é ignorada na comparação, porque os casos vêm dos scripts bash, que nunca gravaram o campo; regravar as fixtures (15 casos em blobs) foi descartado por apagar a equivalência que elas provam. A anotação `contains::<u8>` evita a ambiguidade que o crate `ascii` do `tiny_http` cria quando a feature `painel` está ligada.

**Riscos.** `task.rs` tem mais de 5 mil linhas: mudar só as funções citadas. A SPEC-030 vai mexer nos mesmos arquivos e deve começar depois desta tarefa.

**Arquivos em comum na onda.** Nenhum.

## O que foi feito

**`task.rs`:**
- Novos `APPROVED_BY_FIELD` e `Origin` (`Chat` grava `"chat"` e `Panel` grava `"painel"`).
- `approve` recebe a origem, e o `run_approve` passa `Origin::Chat`.
- `drop_approval` grava `approved_by: null` quando o campo existe.
- Novas `approve_from_panel(project, spec, ids) -> Result<Vec<(String, String)>>` e `record_manual_at(project, spec, task) -> Result<Vec<String>>`.

**`spec.rs`:**
- `approve(specs, spec_id, origin) -> Result<String>`, e a CLI descarta o hash.
- Novas `approve_from_panel` e `record_manual_at`, que devolvem os problemas da validação.
- A queda da aprovação limpa o `approved_by`.

**`plan.rs`:** nova `record_manual_at`, que devolve os problemas da validação.

**Testes:**
- `tests/common/golden.rs` passa a ignorar as linhas `approved_by: chat` e `approved_by: null`, porque os scripts bash nunca gravaram o campo e a queda da aprovação grava `null`. A tarefa previa só `chat`.
- A linha 507 usa `contains::<u8>`.
- Testes novos: `approve_records_origin_chat` e `adjust_drops_approved_by` em `tests/task.rs`, e `spec_approve_records_origin_chat` em `tests/spec.rs`.

**Decisões fora do previsto:**
- O `record_manual_at` de spec e plano devolve `Vec<String>`, como pedia a tarefa, e não `()`, como no spike.

**Pontos de atenção:**
- O teste da spec usa `python3` com `tests/fixtures/fill-refined.py`, que já existia.
- O `cargo fmt` rodou no crate inteiro, mas só mudou arquivos das tarefas da onda.
- A SPEC-030 mexe nos mesmos arquivos e começa depois desta tarefa.

**Verificação:**
- Os testes passaram, inclusive `golden_task` e `golden_spec`.
- A regressão não encontrou diferença: as três linhas a mais são os testes novos.
- A evidência é `aprovacao-origem.txt`.
