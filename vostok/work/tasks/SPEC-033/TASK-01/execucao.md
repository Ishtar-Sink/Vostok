### Ler

- `vostok/work/specs/refined/SPEC-033-operar-o-projeto-de-ponta-a-ponta-pelo-painel-v0-4-0.md` (RF-13, RF-16, RN-06, CB-07, CB-08, CT-19, CT-20 e o bloco `config/main.yml` de "Contratos e interfaces")
- `packages/harness/config/main.yml`
- `apps/cli/src/harness.rs`
- `apps/cli/src/support.rs` (fn key_lines, fn config_get, fn config_set e o comentário "Subconjunto de YAML" no topo da seção de configuração)
- `apps/cli/src/aval.rs` (fn lock_enabled, exemplo de chave ausente lida como padrão)
- `apps/cli/src/commands/update.rs` (fn merge_configs, fn merge_keys: como as chaves novas chegam aos projetos instalados)
- `apps/cli/tests/base.rs` (testes paths_are_resolved_from_main_yml, config_reads_scalars_lists_and_blocks, config_set_preserves_the_rest_of_the_file)

### Mudar

- `packages/harness/config/main.yml` (alterar): `paths.requests`, `paths.questions`, `paths.approvals`, a seção `folders` e a chave `request_dispatch`, com os valores padrão do contrato
- `apps/cli/src/harness.rs` (alterar): leitura das pastas de comunicação, dos nomes de subpasta com padrão em português, validação de nomes repetidos ou inválidos e leitura de `request_dispatch`
- `apps/cli/src/support.rs` (alterar): `key_lines` passa a montar o caminho completo da chave (`folders.requests.pending`), porque o leitor só conhece dois níveis
- `apps/cli/tests/base.rs` (alterar): testes da leitura em três níveis, dos padrões, da configuração trocada e das recusas

### Passos

1. `apps/cli/tests/base.rs` (alterar): acrescente os testes abaixo, usando `Project::new()` e reescrevendo `vostok/config/main.yml` quando o caso pedir. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test base -- communication_ config_reads_three_levels` falha (não compila) antes da mudança.
   - `config_reads_three_levels`: arquivo `a:\n  b:\n    c: 1\n  d: 2\ne: 3\n`; `config_get` lê `a.b.c` = `1`, `a.d` = `2`, `e` = `3`; `config_set(&file, "a.b.c", "9")` troca só essa linha.
   - `communication_paths_default_to_portuguese`: `main.yml` do harness embutido sem as chaves novas (apague as linhas de `requests`, `questions`, `approvals`, o bloco `folders` e `request_dispatch` com `fs::write` de um `main.yml` reduzido a `paths:\n  root: ../\n`); `requests_dir()` = `<harness>/work/pedidos`, `questions_dir()` = `<harness>/work/perguntas`, `approvals_dir()` = `<harness>/work/avais`, `folders()` com `pendente`, `em_andamento`, `feito`, `recusado`, `aberta`, `respondida` e `sem-spec.log`, e `request_dispatch()` = `""`.
   - `communication_paths_follow_the_config` (CT-19): `main.yml` com `paths.requests: ./work/requests`, `paths.questions: ./work/questions`, `paths.approvals: ./work/approvals` e `folders` em inglês (`pending`, `in_progress`, `done`, `refused`, `open`, `answered`, `no-spec.log`); os métodos devolvem esses valores.
   - `communication_shipped_config_has_the_defaults`: com o `main.yml` embutido (sem reescrever), os mesmos valores do teste de padrão, provando que o arquivo distribuído traz as chaves.
   - `communication_folders_reject_repeated_names` (CT-20, CB-08): `folders.requests.pending: x` e `folders.requests.done: x`; `folders()` falha com uma mensagem que contém `folders.requests`, `pending`, `done` e `x`. Repita com `folders.questions.open` e `answered` iguais.
   - `communication_folders_reject_unsafe_names`: valor vazio (`pending: ""`), com `/` (`a/b`) ou `..` falha citando a chave.
   - `communication_dispatch_accepts_only_known_agents`: `request_dispatch: codex` e `claude` são aceitos; `request_dispatch: gemini` falha citando `request_dispatch` e os valores aceitos.
2. `apps/cli/src/support.rs` (alterar): em `fn key_lines`, troque o `top: String` por uma pilha `Vec<(usize, String)>` de (indentação, chave): antes de empilhar a chave da linha, desempilhe enquanto o topo tem indentação maior ou igual à da linha; o `path` é a junção com `.` das chaves da pilha mais a da linha. Uma linha com indentação 0 esvazia a pilha. Atualize o comentário do topo da seção ("em até dois níveis") para "em níveis aninhados (`pai.filho.neto`)". Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test base -- config_` passa, inclusive `config_reads_three_levels`.
3. `packages/harness/config/main.yml` (alterar): em `paths`, depois de `bugs`, acrescente `requests: ./work/pedidos      # pedidos do painel ao agente, numa subpasta por estado`, `questions: ./work/perguntas  # perguntas do agente ao usuário, numa subpasta por status` e `approvals: ./work/avais      # avais do painel, um arquivo por spec`. No fim do arquivo, acrescente um comentário e o bloco `folders` exatamente como no contrato da spec (`requests` com `pending: pendente`, `in_progress: em_andamento`, `done: feito`, `refused: recusado`; `questions` com `open: aberta`, `answered: respondida`; `approvals_without_spec: sem-spec.log`), dizendo que os nomes são só de pasta e que o estado gravado nos arquivos não muda (RN-06). Depois, o comentário e `request_dispatch: ""   # "" (desligado) | codex | claude`, dizendo que, ligado, o `vostok painel` chama o agente para cada pedido, um por vez. Confira: `communication_shipped_config_has_the_defaults` passa.
4. `apps/cli/src/harness.rs` (alterar): acrescente, com os padrões em constantes `DEFAULT_*`:
   - `pub struct RequestFolders { pub pending, pub in_progress, pub done, pub refused: String }`, `pub struct QuestionFolders { pub open, pub answered: String }` e `pub struct Folders { pub requests: RequestFolders, pub questions: QuestionFolders, pub approvals_without_spec: String }`, todas com `#[derive(Debug, Clone, PartialEq, Eq)]`.
   - `fn path_or(&self, key: &str, default: &str) -> PathBuf`: `config_get` de `paths.<key>`; ausente ou vazio usa `default`; resolve com `self.resolve`.
   - `pub fn requests_dir(&self) -> PathBuf`, `pub fn questions_dir(&self) -> PathBuf` e `pub fn approvals_dir(&self) -> PathBuf`, com os padrões `./work/pedidos`, `./work/perguntas` e `./work/avais`.
   - `pub fn folders(&self) -> Result<Folders>`: lê cada `folders.<grupo>.<estado>` e `folders.approvals_without_spec`, ausente ou vazio vira o padrão (chave ausente: projeto da v0.3 sem `vostok update`); recusa valor com `/`, `\`, `.` ou `..` com `bail!("folders.{grupo}.{estado}: nome de pasta inválido '{valor}'")`; recusa dois estados do mesmo grupo com o mesmo nome com `bail!("folders.{grupo}: os estados {a} e {b} usam a mesma pasta '{nome}'")`. Uma chave presente com valor vazio explícito (`""`) é inválida, não padrão: distinga pelo `config_get` que devolve `Ok("")`.
   - `pub fn request_dispatch(&self) -> Result<String>`: ausente ou `""` devolve `""`; `codex` ou `claude` são devolvidos; outro valor falha com `request_dispatch: agente '{valor}' desconhecido; use "", codex ou claude`.
   Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test base -- communication_` passa.
5. `apps/cli` (conferir): `cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings` sem avisos; `cargo test --manifest-path apps/cli/Cargo.toml --test base --test init_update` sem falhas (o `vostok update` acrescenta as chaves novas aos projetos instalados pelo `merge_keys`). Não edite `vostok/config/main.yml`: ele é atualizado pelo `vostok update` num commit de atualização do harness instalado, como foi com `approval_lock`.
