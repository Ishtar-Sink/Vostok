### Ler

- `vostok/work/specs/refined/SPEC-030-trava-de-aval-e-loop-de-pedidos-do-painel.md` (RF-01, RF-02, CB-02, CB-03, "Contratos e interfaces")
- `apps/cli/src/painel/actions.rs` (fn record_aval, fn request_markdown: formato do `avais.log` e do `PED-NNN.md`)
- `apps/cli/src/support.rs` (fn config_get, fn frontmatter_get)
- `apps/cli/src/harness.rs` (fn locate, fn path, fn config_file)
- `apps/cli/src/lib.rs`
- `packages/harness/config/main.yml`

### Mudar

- `apps/cli/src/aval.rs` (criar): leitura do `avais.log`, da chave `approval_lock`, do estado de um pedido e a conferência do aval.
- `apps/cli/src/lib.rs` (alterar): `pub mod aval;`.
- `packages/harness/config/main.yml` (alterar): chave `approval_lock: false`, com comentário.

### Passos

1. `apps/cli/src/aval.rs` (criar): escreva primeiro os testes unitários no `mod tests` do arquivo: `four_columns_read_as_aprovar`, `five_columns_keep_the_action`, `malformed_line_is_reported_not_matched`, `lock_is_off_without_the_key`, `lock_reads_true`, `request_state_reads_estado`. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --lib aval` falha (o módulo ainda não tem as funções).
2. `apps/cli/src/aval.rs` (criar): implemente, sem dependência nova:
   - `pub struct Aval { pub date: String, pub id: String, pub hash: String, pub origin: String, pub action: String }`;
   - `pub fn read(work: &Path) -> Result<(Vec<Aval>, Vec<String>)>`: lê `<work>/avais.log` (ausente = listas vazias), separa por tabulação; 4 colunas viram `action: "aprovar"`; 5 colunas mantêm a ação; outra contagem vai para a segunda lista como `linha N malformada` (CB-03);
   - `pub fn lock_enabled(harness: &Harness) -> Result<bool>`: `config_get(main.yml, "approval_lock")`, `true` só com o valor `true`; chave ausente = `false` (RF-01);
   - `pub fn require(harness: &Harness, id: &str, action: &str, hash: &str) -> Result<()>`: com a trava desligada, `Ok`; ligada, procura um aval com `origin == "painel"` e os três campos iguais; sem ele, erro: `sem o aval do painel para <ação> de <id> (hash <hash>); peça pelo painel (vostok painel)`, citando, se houver aval do mesmo ID e ação com outro hash, `o aval existente é do hash <h>` (RNF-03, CB-01, CB-07);
   - `pub fn has_panel_approval(work: &Path, id: &str, hash: &str) -> Result<bool>` (aval `aprovar` com o ID e o hash, para a validação do RF-09);
   - `pub fn request_state(work: &Path, request_id: &str) -> Result<(String, String)>`: `estado` e `acao` do front-matter de `<work>/pedidos/<ID>.md`;
   - `pub fn request_text_hash(work: &Path, request_id: &str) -> Result<String>`: SHA-256 (`sha256_hex`) do texto da seção `## Pedido`, sem espaços nas pontas.
   Confira: `cargo test --manifest-path apps/cli/Cargo.toml --lib aval` passa.
3. `apps/cli/src/lib.rs` (alterar): `pub mod aval;` em ordem alfabética entre os módulos.
4. `packages/harness/config/main.yml` (alterar): no fim do arquivo, um bloco comentado em português e `approval_lock: false`, explicando que `true` faz concluir, aprovar, arquivar e fazer commit exigirem o aval do painel. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test init_update` passa.
5. Confira: `cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings` sem avisos.
