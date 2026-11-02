<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Uma worktree em `/home/victor/projetos/ai-hub-painel`, no branch `spike/painel-local`, recebe um comando `vostok painel` mínimo com duas implementações de servidor (features `painel-tiny` e `painel-axum`) e um app Vite com React e alguns componentes copiados do Atria, que também compila com Preact com compat. O `medir.sh` gera as 5 variantes e mede o binário como a release (`cargo build --release --locked` e `tar -czf`). O resultado vai para `docs/estudos/painel-local/medicoes.md`. Cobre PQ-02, PQ-03 e PQ-04; sustenta CA-03, CA-04 e CA-13.

**Decisões.** `tiny_http` (síncrono, sem runtime) contra `axum` com `tokio` mínimo, por ser o par mais leve contra o mais completo; `hyper` sozinho fica fora porque o `axum` já o inclui e mede o teto. O frontend parte de React (decisão 7 da spec) e compara só o empacotamento (React contra Preact com compat, gzip contra brotli). O `dist/` é embutido com `include_dir`, como o harness. A porta 0 deixa o sistema escolher uma livre, base para vários painéis (PQ-09).

**Riscos.** Builds de release lentos: a evidência tem `Tempo-limite: 1800`. O spike exige Node e pnpm, que a release não usa hoje: o documento registra esse impacto.

**Arquivos em comum na onda.** Nenhum com a TASK-01.

## O que foi feito

**Worktree do spike.** `/home/victor/projetos/ai-hub-painel`, branch `spike/painel-local`, criada a partir do commit `cb57802` da `main`. O código do spike fica sem commit e sem merge.

**Arquivos do spike (só na worktree):**
- `apps/painel/`: app Vite com React 19.3.1, a mesma versão do Atria. Contém `package.json`, `vite.config.ts`, `index.html`, `src/main.tsx`, `src/theme/` e `src/ui/` (button, card, page-header e badge, copiados do Atria sem os testes). Não lê nem copia nada de `secrets/`.
- `apps/painel/vite.config.ts`: com `PAINEL_PREACT=1`, aplica o alias de `react` e `react-dom` para `preact/compat`.
- `apps/painel/medir.sh`: gera o bundle de cada sabor, compila as cinco variantes com um `CARGO_TARGET_DIR` próprio e imprime uma linha por variante. `PAINEL_MEDICAO_DIR` escolhe a pasta de trabalho.
- `apps/cli/Cargo.toml`: features `painel-tiny` (`tiny_http`) e `painel-axum` (`axum` com `tokio` só com `rt` e `net`), e `include_dir` sobre `../painel/dist`. O perfil de release não mudou.
- `apps/cli/src/commands/painel.rs`, com registro em `apps/cli/src/cli.rs` e `apps/cli/src/commands/mod.rs`: comando `vostok painel [--porta N] [--sem-navegador]`, que escuta em `127.0.0.1`, serve o `dist/` embutido e responde `GET /api/specs`.

**Repositório principal.** `docs/estudos/painel-local/medicoes.md` (novo, sem rastreamento): tabela das variantes com a diferença para a base, comandos de reprodução, impacto de Node no build e o limite proposto.

**Como foi medido.** O `medir.sh` gera o bundle (React, e Preact com `PAINEL_PREACT=1`), compila cada variante com `cargo build --release --locked` e gera o tar.gz como a release (`tar -czf`). Versões: rustc 1.98.0, Node v24.19.0, pnpm 11.21.0, Vite 8.3.5, Preact 10.29.8. O spike foi executado pelo modelo haiku.

**Resultados** (bytes, binário e tar.gz): base 4 586 856 e 1 782 243; tiny+preact 5 102 136 e 1 956 582; tiny+react 5 302 120 e 2 016 614; axum+preact 5 639 192 e 2 157 510; axum+react 5 839 552 e 2 217 820. Tabela completa em `medicoes.md`.

**Decisões.**
- `tiny_http` (síncrono, sem runtime) contra `axum` com `tokio` mínimo, por ser o par mais leve contra o mais completo. `hyper` puro ficou de fora, porque o `axum` já o inclui.
- Variante recomendada: `tiny+preact`, a menor medida. A recomendação depende de testes de comportamento que o spike não fez.
- `dist/` embutido com `include_dir`, como o harness.
- Limite proposto: 5 300 000 bytes instalados e 2 000 000 bytes no tar.gz.

**Pontos de atenção.**
- O `include_dir` embute o bundle sem compressão. A diferença de cerca de 200 KB entre React e Preact no binário é o tamanho bruto do bundle (230 201 contra 30 209 bytes). As colunas gzip e brotli são só referência.
- O limite proposto deixa de fora também `tiny+react`, que fica 2 120 bytes acima do limite instalado e 16 614 bytes acima do compactado. Só `tiny+preact` cabe; as variantes `axum` ficam bem acima.
- A margem no compactado é de 44 KB, e a medição usou um app de 4 componentes. O limite deve ser revisto com o protótipo completo.
- Preact foi medido só em tamanho. Não há teste de comportamento.
- Node: `cargo install --path apps/cli` sem features não muda, porque a base não embute o painel. Com `painel-tiny` ou `painel-axum`, o build exige `apps/painel/dist/`; sem Node e sem `dist/`, ele falha. A release (`.github/workflows/release.yml`) precisaria de um passo de Node e pnpm antes do `cargo build --release --locked`; o spike não altera esse arquivo. Saídas: gerar o bundle na CI, ou versionar o `dist/` (cerca de 30 KB com Preact).
- Em `medicoes.md`, a linha "Commit da worktree" mostra `cb57802`, que é o commit de base; o código do spike não tem commit.

**Verificação.** Testes, regressão e evidências passaram; a regressão registrou um comando com acréscimo, e as evidências (2) estão em `TASK-02/evidencias/`.
