### Ler

- `vostok/work/specs/refined/SPEC-029-painel-local-do-vostok.md` (RF-07, RN-01, CT-06, CT-07, "Contratos e interfaces")
- `apps/cli/src/commands/task.rs` (fn run_approve, fn approve, fn drop_approval, fn record_manual; arquivo com mais de 5 mil linhas: leia por busca)
- `apps/cli/src/commands/spec.rs` (fn approve, fn record_manual e a função que derruba a aprovação, perto da linha 947)
- `apps/cli/src/commands/plan.rs` (fn record_manual)
- `apps/cli/tests/common/golden.rs` (fn normalize, fn replace_volatile e a linha 507)
- `/home/victor/projetos/ai-hub-painel/apps/cli/src/commands/task.rs` (linhas 240 a 300 e fn approve, referência do spike)
- `/home/victor/projetos/ai-hub-painel/apps/cli/src/commands/spec.rs` (linhas 1195 a 1230)
- `/home/victor/projetos/ai-hub-painel/apps/cli/src/commands/plan.rs` (linhas 95 a 110)

### Mudar

- `apps/cli/src/commands/task.rs` (alterar): `Origin`, `APPROVED_BY_FIELD`, `approve` com origem, `approve_from_panel`, `record_manual_at` e `drop_approval` limpando `approved_by`.
- `apps/cli/src/commands/spec.rs` (alterar): `approve` com origem e hash devolvido, `approve_from_panel`, `record_manual_at` e a queda da aprovação limpando `approved_by`.
- `apps/cli/src/commands/plan.rs` (alterar): `record_manual_at`.
- `apps/cli/tests/common/golden.rs` (alterar): ignorar a linha `approved_by: chat` na comparação e anotar o tipo na linha 507.
- `apps/cli/tests/task.rs` (alterar): testes da origem e da queda da aprovação.
- `apps/cli/tests/spec.rs` (alterar): teste da origem na aprovação da spec.

### Passos

1. `apps/cli/tests/task.rs` (alterar): escreva primeiro os testes `approve_records_origin_chat` (depois de `vostok task approve approve`, o `estado.yml` tem `approved_by: chat`) e `adjust_drops_approved_by` (um ajuste manual numa tarefa aprovada volta a `pending` com `approved_by: null`), usando os ajudantes de projeto do próprio arquivo (fn approved_task). Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test task -- approved_by approve_records_origin` falha.
2. `apps/cli/tests/spec.rs` (alterar): teste `spec_approve_records_origin_chat` (o front-matter da refinada aprovada tem `approved_by: chat`). Confira: falha.
3. `apps/cli/src/commands/task.rs` (alterar): antes de `pub fn run`, crie `pub const APPROVED_BY_FIELD: &str = "approved_by";` e `pub enum Origin { Chat, Panel }` com `pub fn as_str(&self) -> &'static str` (`"chat"` e `"painel"`), como no spike (linhas 240 a 260). Em `fn approve`, receba `origin: Origin` e grave `APPROVED_BY_FIELD` com `origin.as_str()` em cada arquivo; `run_approve` passa `Origin::Chat`. Em `fn drop_approval`, grave `null` em `APPROVED_BY_FIELD` quando o campo existir. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test task -- approved_by approve_records_origin` passa.
4. `apps/cli/src/commands/task.rs` (alterar): crie `pub fn approve_from_panel(project: &Path, spec: &str, ids: &[String]) -> Result<Vec<(String, String)>>` que faz as mesmas conferências de `task approve approve` (todas ou nenhuma) e aprova com `Origin::Panel`, devolvendo `(id, hash aprovado)`; e `pub fn record_manual_at(project: &Path, spec: &str, task: &str) -> Result<Vec<String>>`, que registra o ajuste manual e devolve os problemas do `adjust validate` (vazio quando válido). Siga o spike (linhas 262 a 300). A origem nunca vira argumento de linha de comando (RN-01).
5. `apps/cli/src/commands/spec.rs` (alterar): `fn approve(specs, spec_id, origin: Origin) -> Result<String>` grava `approved_by` e devolve o hash; a chamada da CLI usa `Origin::Chat` e descarta o hash. Crie `pub fn approve_from_panel(project: &Path, spec_id: &str) -> Result<String>` e `pub fn record_manual_at(project: &Path, spec_id: &str) -> Result<Vec<String>>`. Na queda da aprovação (perto da linha 947), limpe também `approved_by`. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test spec -- approve_records_origin` passa.
6. `apps/cli/src/commands/plan.rs` (alterar): `pub fn record_manual_at(project: &Path, spec_id: &str) -> Result<Vec<String>>`, pelo `Workspace::new(Harness::locate(project)?)`, como no spike (linha 100).
7. `apps/cli/tests/common/golden.rs` (alterar): em `fn normalize`, remova as linhas `approved_by: chat` do texto comparado, com um comentário dizendo que os scripts bash nunca gravaram o campo; na linha 507, troque `(b'a'..=b'f').contains(b)` por `(b'a'..=b'f').contains::<u8>(b)`. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test task --test spec -- golden` passa.
8. Confira: `cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings` sem avisos.
