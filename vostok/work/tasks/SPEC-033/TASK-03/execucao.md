### Ler

- `vostok/work/specs/refined/SPEC-033-operar-o-projeto-de-ponta-a-ponta-pelo-painel-v0-4-0.md` (RF-09, RF-16, RN-05, RNF-02, FA-04, CB-03, CT-10, CT-17, CT-19, riscos e decisões 6 e 8)
- `docs/estudos/painel-ponta-a-ponta.md` (seção "Arquivos de comunicação", parte do `avais.log`)
- `vostok/work/tasks/SPEC-033/TASK-01/tecnico.md` (a API do `Harness`: `approvals_dir`, `folders().approvals_without_spec`)
- `apps/cli/src/aval.rs`
- `apps/cli/src/harness.rs`
- `apps/cli/src/commands/mod.rs`
- `apps/cli/src/cli.rs`
- `apps/cli/src/commands/pedidos.rs` (exemplo de comando curto com `Harness::from_current_dir` e tabela Markdown na saída)
- `apps/cli/src/commands/spec.rs` (fn panel_approval_error)
- `apps/cli/src/commands/task.rs` (chamada de `aval::has_panel_approval` perto de `PANEL_WITHOUT_AVAL`)
- `apps/cli/src/painel/actions.rs` (fn record_aval, const AVAIS_LOG, testes task_approval_records_the_panel_origin_and_the_aval, avais_log_only_receives_appends, aval_lines, last_aval_fields e os que conferem `AVAIS_LOG`)
- `apps/cli/src/painel/read.rs` (fn approval_rows, fn activity_project dos testes, teste avais_malformados_sao_ignorados)
- `apps/cli/tests/aval.rs`
- `apps/cli/tests/trava_plano_commit.rs` (fn add_aval)
- `apps/cli/tests/painel.rs` (leitura de `vostok/work/avais.log` no teste approve_edit_request_and_live_update_end_to_end)
- `packages/harness/.gitignore`

### Mudar

- `apps/cli/src/aval.rs` (alterar): um log por spec na pasta de avais, o de sem spec à parte, a gravação só de acréscimo, a leitura de uma spec ou de todas e a migração do `avais.log`
- `apps/cli/src/commands/avais.rs` (criar): o comando `vostok avais <ID>`
- `apps/cli/src/commands/mod.rs` (alterar): `pub mod avais;`
- `apps/cli/src/cli.rs` (alterar): o subcomando `Avais` e o despacho para `avais::run`
- `apps/cli/src/commands/spec.rs` (alterar): `has_panel_approval` recebe o `Harness`
- `apps/cli/src/commands/task.rs` (alterar): `has_panel_approval` recebe o `Harness`
- `apps/cli/src/painel/actions.rs` (alterar): `record_aval` grava pelo `aval::append` (componente da TASK-08, onda 3)
- `apps/cli/src/painel/read.rs` (alterar): `approval_rows` lê todos os logs pelo `aval::read_all` (componente da TASK-02 e da TASK-06)
- `packages/harness/.gitignore` (alterar): `work/avais/`, local de cada clone como o `avais.log`
- `apps/cli/tests/aval.rs` (alterar): avais por spec, sem spec, migração e o `vostok avais`
- `apps/cli/tests/painel.rs` (alterar): o aval da aprovação é lido em `work/avais/SPEC-001.log`

### Passos

1. `apps/cli/tests/aval.rs` (alterar): acrescente os testes abaixo (os atuais continuam: o `add_aval` deles grava no `avais.log` antigo e passa a exercitar a migração). Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test aval -- by_spec_` falha antes da mudança.
   - `by_spec_task_approval_goes_to_the_spec_log` (CT-10): com a trava ligada e o aval do painel gravado pelo `vostok::aval::append` em `SPEC-001`, o `vostok task finish` aceita; o arquivo `vostok/work/avais/SPEC-001.log` tem a linha, e `vostok/work/avais.log` não existe.
   - `by_spec_avais_command_lists_newest_first` (CT-10, RNF-02): dois avais de `SPEC-001` (um da spec, um de `SPEC-001/TASK-01`) e um de `SPEC-002`; `vostok avais SPEC-001` mostra a tabela `| Data | ID | Ação | Origem | Hash |` com os dois de `SPEC-001`, o mais novo primeiro, sem o de `SPEC-002`; `vostok avais SPEC-001/TASK-01` mostra só o da tarefa.
   - `by_spec_avais_command_without_avais_says_so`: `vostok avais SPEC-009` termina com 0 e `Nenhum aval de SPEC-009.`; `vostok avais x/y/z` falha com `ID inválido`.
   - `by_spec_legacy_log_is_split_once` (CT-17, FA-04, CB-03): `avais.log` antigo com linhas de `SPEC-001`, `SPEC-001/TASK-01`, `SPEC-002`, `PED-004` (commit), uma linha malformada (`curta\tlinha`) e uma em branco; depois de `vostok avais SPEC-001`, `avais/SPEC-001.log` tem as duas linhas de `SPEC-001` sem nenhuma mudança de bytes, `avais/SPEC-002.log` a dele, `avais/sem-spec.log` a do `PED-004` e a malformada, e o `avais.log` não existe; um segundo comando não duplica nada.
   - `by_spec_folders_follow_the_config` (CT-19, parte dos avais): `paths.approvals: ./work/approvals` e `folders.approvals_without_spec: no-spec.log`; o aval vai para `work/approvals/SPEC-001.log`, o de `PED-001` para `work/approvals/no-spec.log`, e o `vostok avais SPEC-001` os lê.
2. `apps/cli/src/aval.rs` (alterar): substitua o `AVAIS_LOG` por:
   - `const LEGACY_LOG: &str = "avais.log";` lido em `harness.root.join("work")`, o lugar do formato antigo.
   - `fn spec_of(id: &str) -> Option<&str>`: a parte antes da primeira `/` quando ela é `SPEC-` seguido só de dígitos.
   - `fn log_of(harness, id) -> Result<PathBuf>`: `approvals_dir()/<SPEC>.log`, ou `approvals_dir()/<folders()?.approvals_without_spec>` sem spec.
   - `fn parse(content, avais, malformed)`: a leitura de colunas atual (4 ou 5), sem mudar o formato da linha.
   - `pub fn migrate_legacy(harness) -> Result<()>`: se `work/avais.log` existe, acrescenta cada linha não vazia, como está, ao log da spec do ID da segunda coluna (linha malformada ou sem spec vai ao log sem spec, CB-03), com `OpenOptions::append`; só depois de todas gravadas, apaga o `avais.log`. Falha de gravação devolve o erro e mantém o `avais.log`.
   - `pub fn append(harness, id, hash, origin, action) -> Result<()>`: `migrate_legacy`, cria a pasta e acrescenta `data\tid\thash\torigem\tação` ao log do ID.
   - `pub fn read_for(harness, id) -> Result<Vec<Aval>>`: `migrate_legacy` e lê só o log do ID.
   - `pub fn read_all(harness) -> Result<(Vec<Aval>, Vec<String>)>`: `migrate_legacy` e lê todos os `*.log` da pasta, em ordem de nome.
   `require` e `has_panel_approval(harness, id, hash)` passam a usar `read_for`. A mensagem de recusa do `require` termina com `; consulte com vostok avais {id}`. Reescreva os testes unitários do módulo para o `Harness` temporário (`harness_with_main`): os nomes atuais continuam, e `missing_log_reads_empty` lê a pasta sem logs. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --lib aval::`.
3. `apps/cli/src/commands/spec.rs` e `apps/cli/src/commands/task.rs` (alterar): troque `aval::has_panel_approval(&work, ...)` por `aval::has_panel_approval(&<harness>, ...)` e remova a variável `work` que sobrar. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test aval`.
4. `apps/cli/src/commands/avais.rs` (criar): `#[derive(Debug, clap::Args)] pub struct AvaisArgs { pub id: String }` e `pub fn run(args: AvaisArgs) -> Result<()>`: valida o ID (`SPEC-NNN`, `SPEC-NNN/TASK-NN` ou `PED-NNN`; senão `bail!("ID inválido '{id}': use SPEC-NNN, SPEC-NNN/TASK-NN ou PED-NNN")`), lê `aval::read_for`; para uma spec, mostra os avais do ID e dos IDs que começam com `<ID>/`; para os outros, só o ID exato; ordena pela data, do mais novo para o mais antigo, e imprime a tabela `| Data | ID | Ação | Origem | Hash |`, ou `Nenhum aval de {id}.` quando vazia. `apps/cli/src/commands/mod.rs` (alterar): `pub mod avais;` antes de `pub mod bug;`, na ordem alfabética. `apps/cli/src/cli.rs` (alterar): importe `avais`, acrescente `/// Avais do painel de uma spec, tarefa ou pedido.\n Avais(avais::AvaisArgs),` depois de `Pedidos` e `Command::Avais(args) => avais::run(args),` no `run`. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test aval -- by_spec_`.
5. `apps/cli/src/painel/actions.rs` (alterar): `record_aval` chama `aval::append(&Harness { root: painel.root.clone() }, id, hash, "painel", action)`; remova `AVAIS_LOG`. Nos testes, troque `painel.work.join(AVAIS_LOG)` pelo log da spec (`painel.work.join("avais/SPEC-001.log")`; no de commit, `avais/sem-spec.log`), inclusive em `aval_lines` e `last_aval_fields`; `avais_log_only_receives_appends` grava a `linha anterior` no `avais/SPEC-001.log`. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::actions`.
6. `apps/cli/src/painel/read.rs` (alterar): `approval_rows` usa `aval::read_all(&Harness { root: self.painel.root.clone() })`. As fixtures `vostok/work/avais.log` dos testes ficam e passam pela migração. `apps/cli/tests/painel.rs` (alterar): leia `vostok/work/avais/SPEC-001.log`. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::read` e `--features painel --test painel`.
7. `packages/harness/.gitignore` (alterar): depois de `work/avais.log`, acrescente `work/avais/`. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test base`.
8. `apps/cli` (conferir): `cargo clippy --manifest-path apps/cli/Cargo.toml --features painel --all-targets -- -D warnings` sem avisos e `cargo test --manifest-path apps/cli/Cargo.toml --test trava_plano_commit --test spec --test task` sem falhas (o `add_aval` antigo de `trava_plano_commit` passa pela migração; nenhum `golden_*` grava avais).
