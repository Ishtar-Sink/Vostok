### Ler

- `vostok/work/specs/refined/SPEC-029-painel-local-do-vostok.md` (RF-01, RF-08, RF-12, FA-01, FA-02, CB-01, "Contratos e interfaces")
- `apps/cli/src/cli.rs`
- `apps/cli/src/commands/mod.rs`
- `apps/cli/src/lib.rs`
- `apps/cli/Cargo.toml`
- `apps/cli/build.rs`
- `apps/cli/src/harness.rs` (fn locate)
- `/home/victor/projetos/ai-hub-painel/apps/cli/src/commands/painel.rs` (fn run, struct Painel, fn act, fn responder, fn asset, fn with_token, fn session_token, fn abrir_navegador e o módulo do servidor `tiny`, linhas 940 a 1080)
- `/home/victor/projetos/ai-hub-painel/apps/cli/Cargo.toml` (features `painel-tiny`)
- `docs/estudos/painel-local.md` (seção "Arquitetura recomendada")

### Mudar

- `apps/cli/Cargo.toml` (alterar): feature `painel` com `tiny_http` e `sha1` opcionais.
- `apps/cli/build.rs` (alterar): gerar o bundle de `apps/painel` quando `dist/` falta e há Node, e ligar o cfg `painel_bundle` só com `dist/` pronto.
- `apps/cli/src/cli.rs` (alterar): subcomando `painel`.
- `apps/cli/src/commands/mod.rs` (alterar): `pub mod painel;`.
- `apps/cli/src/commands/painel.rs` (criar): argumentos, localização do harness, abertura do navegador e mensagem sem bundle.
- `apps/cli/src/lib.rs` (alterar): `pub mod painel;` com `#[cfg(feature = "painel")]`.
- `apps/cli/src/painel/mod.rs` (criar): `Painel`, `Reply`, token e ajudantes comuns (JSON, texto, consulta, decodificação, segmento seguro).
- `apps/cli/src/painel/server.rs` (criar): servidor `tiny_http`, bundle com o token, conferência do `X-Painel-Token` e despacho.
- `apps/cli/src/painel/read.rs` (criar): esqueleto `route`, que devolve `None`.
- `apps/cli/src/painel/live.rs` (criar): esqueleto `version` e `handle`, que responde 501.
- `apps/cli/src/painel/actions.rs` (criar): esqueleto `act`, que responde 501.

### Passos

1. `apps/cli/Cargo.toml` (alterar): `tiny_http = { version = "0.12", optional = true }`, `sha1 = { version = "0.10", optional = true }` e `[features] painel = ["dep:tiny_http", "dep:sha1"]`, sem ligar por padrão. Confira: `cargo build --manifest-path apps/cli/Cargo.toml` sem a feature continua igual.
2. `apps/cli/build.rs` (alterar): com a feature `painel` (`CARGO_FEATURE_PAINEL`), se `../painel/dist/index.html` não existe e `pnpm` está no PATH, rode `pnpm --dir ../painel install --frozen-lockfile` e `pnpm --dir ../painel build`; se a geração falhar ou não houver `pnpm`, emita `cargo:warning=painel sem bundle: ...` e siga. Só com `dist/index.html` presente, emita `cargo:rustc-cfg=painel_bundle`. Declare `cargo::rustc-check-cfg=cfg(painel_bundle)` e `rerun-if-changed` de `../painel/src`, `../painel/index.html` e `../painel/package.json`. Decisão: o `build.rs` não falha sem Node (RF-12, CT-14).
3. `apps/cli/src/painel/mod.rs` (criar): `pub struct Reply { pub status: u16, pub content_type: &'static str, pub body: Vec<u8> }`; `pub struct Painel { pub project: PathBuf, pub root: PathBuf, pub work: PathBuf, pub token: String }` com `Painel::new(&Harness)`; ajudantes `json`, `text`, `not_found`, `json_string`, `query_value`, `percent_decode`, `safe_segment`, portados do spike; `session_token()` com 16 bytes de `/dev/urandom` em hex. Declare `pub mod server; pub mod read; pub mod live; pub mod actions;`. Testes unitários: `session_token` com 32 caracteres hex e dois tokens diferentes; `safe_segment` recusa `..` e `/`.
4. `apps/cli/src/painel/read.rs`, `live.rs` e `actions.rs` (criar): esqueletos com as assinaturas fixas que as tarefas da onda 2 implementam, sem mudar: `pub fn route(painel: &Painel, parts: &[&str], query: &str) -> Option<Reply>` (devolve `None`); `pub fn version(work: &Path) -> u128` (devolve 0) e `pub fn handle(painel: &Painel, request: tiny_http::Request)` (responde 501); `pub fn act(painel: &Painel, method: &str, parts: &[&str], query: &str, body: &[u8]) -> Reply` (501).
5. `apps/cli/src/painel/server.rs` (criar): `pub fn serve(painel: Painel, port: u16, announce: impl FnOnce(String)) -> Result<()>` escutando em `127.0.0.1:<port>` (0 = porta livre), uma thread por requisição. Despacho: `GET /api/ao-vivo` → `live::handle`; outros `GET /api/...` → `read::route` ou 404; `POST` e `PUT` → confere `X-Painel-Token` igual a `painel.token` (403 sem ele, RF-08), lê o corpo com limite de 2 MB e chama `actions::act`; demais `GET` → arquivo do bundle, com `index.html` para rotas do app e o token injetado em `<meta name="painel-token" content="...">`. Bundle por `include_dir!` só com `#[cfg(painel_bundle)]`; sem ele, a página responde texto explicando como gerar o bundle (FA-02). Testes unitários do despacho com `Reply`: POST sem token é 403; GET de rota da API desconhecida é 404; `index.html` sai com o token.
6. `apps/cli/src/commands/painel.rs` (criar), `cli.rs`, `commands/mod.rs` e `lib.rs` (alterar): `vostok painel [--porta N] [--pasta DIR] [--sem-navegador]`. Com a feature: `Harness::locate` a partir de `--pasta` ou da pasta atual (FA-01: erro com o caminho procurado), `Painel::new`, imprime `Painel de <pasta> em http://127.0.0.1:<porta>/` e abre o navegador (`xdg-open`, `open` ou `cmd /c start`), salvo `--sem-navegador`. Sem a feature, o subcomando existe e responde que o binário foi compilado sem o painel. Porta ocupada (CB-01 da spec, CB-01 do plano): erro com a porta.
7. `apps/cli/tests/painel_server.rs` (criar): testes de integração com `--features painel` que sobem `vostok painel --sem-navegador --porta 0` numa cópia mínima com `vostok/config/main.yml`, leem a URL impressa e conferem: escuta só em `127.0.0.1`, POST sem token 403 (CT-05); e `vostok painel --pasta /` termina com erro citando `vostok/config/main.yml` (CT-02).
8. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel`, `cargo test --manifest-path apps/cli/Cargo.toml --features painel --test painel_server` e `cargo build --manifest-path apps/cli/Cargo.toml` sem a feature passam; `cargo clippy --manifest-path apps/cli/Cargo.toml --features painel --all-targets -- -D warnings` sem avisos.
