### Ler

- `vostok/work/specs/refined/SPEC-027-estudo-do-painel-local-do-vostok-sem-servidor.md`
- `apps/cli/Cargo.toml`
- `apps/cli/src/cli.rs` (enum dos comandos, para registrar `painel`)
- `apps/cli/src/commands/mod.rs`
- `apps/cli/src/embedded.rs` (como o harness é embutido com `include_dir`)
- `.github/workflows/release.yml` (build `--release --locked` e o `tar -czf` das releases)
- `/home/victor/projetos/ishtar-sink/atria/packages/ui/src/button/`, `card/`, `page-header/`, `badge/` (componentes de exemplo)
- `/home/victor/projetos/ishtar-sink/atria/packages/theme/src/`
- `/home/victor/projetos/ishtar-sink/atria/apps/web/package.json` e `vite.config.ts` (versões de React e Vite)

### Mudar

- `docs/estudos/painel-local/medicoes.md` (criar): tabela das medições do binário e do bundle, os comandos para reproduzir e o limite proposto (PQ-02, PQ-03, PQ-04).

### Passos

1. Crie a worktree do spike, fora da `main`: `git worktree add -b spike/painel-local /home/victor/projetos/ai-hub-painel main`. Nada desta worktree é commitado nem mergeado (CA-13). Confira: `git -C /home/victor/projetos/ai-hub-painel branch --show-current` imprime `spike/painel-local`.
2. `/home/victor/projetos/ai-hub-painel/apps/painel/` (criar, na worktree): app Vite com React na mesma versão do Atria (`apps/web/package.json`), com `package.json`, `vite.config.ts`, `index.html` e `src/main.tsx`. Copie de `/home/victor/projetos/ishtar-sink/atria/packages/theme/src` para `src/theme/` e de `packages/ui/src` os componentes `button`, `card`, `page-header` e `badge` para `src/ui/` (só os `.tsx` e `.module.css`, sem os testes). A página `src/main.tsx` mostra um `PageHeader` "Vostok", um `Card` com a lista de specs vinda de `GET /api/specs` e um `Badge` por status. Não leia nem copie nada de `secrets/`. Confira: `pnpm --dir /home/victor/projetos/ai-hub-painel/apps/painel install && pnpm --dir /home/victor/projetos/ai-hub-painel/apps/painel build` gera `dist/`.
3. Variante Preact: no `vite.config.ts`, quando `PAINEL_PREACT=1`, aplique o alias `react` e `react-dom` para `preact/compat` (dependência `preact`). Confira: `PAINEL_PREACT=1 pnpm --dir /home/victor/projetos/ai-hub-painel/apps/painel build` também gera `dist/`.
4. `/home/victor/projetos/ai-hub-painel/apps/cli/Cargo.toml` (alterar, na worktree): duas features opcionais e exclusivas, `painel-tiny` (`tiny_http`) e `painel-axum` (`axum` com `tokio` só com `rt` e `net`), e `include_dir` para `../painel/dist`. Release com o mesmo perfil de hoje (sem mudar `[profile]`).
5. `/home/victor/projetos/ai-hub-painel/apps/cli/src/commands/painel.rs` (criar, na worktree) e o registro em `cli.rs` e `mod.rs`: `vostok painel [--porta N] [--sem-navegador]`, escuta em `127.0.0.1` (porta 0 por padrão: o sistema escolhe uma livre), imprime `Painel em http://127.0.0.1:<porta>`, serve o `dist/` embutido em `/` e responde `GET /api/specs` com o ID, o título e o status de cada spec de `vostok/work/specs/refined/` e `draft/`, em JSON. Uma implementação por feature, com a mesma interface. Sem `--sem-navegador`, abre o navegador com `xdg-open`/`open`. Confira: `cargo run --manifest-path /home/victor/projetos/ai-hub-painel/apps/cli/Cargo.toml --features painel-tiny -- painel --sem-navegador` imprime a URL, e `curl -s <url>/api/specs` devolve JSON.
6. `/home/victor/projetos/ai-hub-painel/apps/painel/medir.sh` (criar, na worktree): para cada variante (`base` sem painel, `tiny+react`, `tiny+preact`, `axum+react`, `axum+preact`), gera o bundle se preciso, roda `cargo build --release --locked` (com `--features` correspondente) num `CARGO_TARGET_DIR` próprio da variante e imprime uma linha `variante | binário (bytes) | tar.gz (bytes) | bundle (bytes) | bundle gzip (bytes) | bundle brotli (bytes, ou -)`. O tar.gz é gerado como na release: `tar -czf <tmp> -C <dir> vostok`. Imprime antes a versão do `rustc` e o commit da worktree. Confira: `bash /home/victor/projetos/ai-hub-painel/apps/painel/medir.sh` termina com código 0 e imprime 5 linhas de variante.
7. `docs/estudos/painel-local/medicoes.md` (criar, no repositório principal): título `# Medições do painel local`, data, versão do `vostok`, commit da worktree, a tabela do passo 6 com a diferença de cada variante para a `base`, os comandos para reproduzir e a seção `## Limite proposto` (PQ-04): o limite de tamanho instalado e compactado para o painel, justificado pelos números e pelo tamanho atual (4,6 MB instalado, cerca de 1,7 MB compactado na release), e a variante recomendada. Registre também o impacto de exigir Node no `cargo install --path apps/cli` e na release (PQ-03) e as duas saídas: gerar o bundle na CI ou embutir o `dist/` já gerado. Confira: `grep -n '^## ' docs/estudos/painel-local/medicoes.md`.
