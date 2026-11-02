<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Uma worktree em `/home/victor/projetos/ai-hub-painel`, no branch `spike/painel-local`, recebe um comando `vostok painel` mínimo com duas implementações de servidor (features `painel-tiny` e `painel-axum`) e um app Vite com React e alguns componentes copiados do Atria, que também compila com Preact com compat. O `medir.sh` gera as 5 variantes e mede o binário como a release (`cargo build --release --locked` e `tar -czf`). O resultado vai para `docs/estudos/painel-local/medicoes.md`. Cobre PQ-02, PQ-03 e PQ-04; sustenta CA-03, CA-04 e CA-13.

**Decisões.** `tiny_http` (síncrono, sem runtime) contra `axum` com `tokio` mínimo, por ser o par mais leve contra o mais completo; `hyper` sozinho fica fora porque o `axum` já o inclui e mede o teto. O frontend parte de React (decisão 7 da spec) e compara só o empacotamento (React contra Preact com compat, gzip contra brotli). O `dist/` é embutido com `include_dir`, como o harness. A porta 0 deixa o sistema escolher uma livre, base para vários painéis (PQ-09).

**Riscos.** Builds de release lentos: a evidência tem `Tempo-limite: 1800`. O spike exige Node e pnpm, que a release não usa hoje: o documento registra esse impacto.

**Arquivos em comum na onda.** Nenhum com a TASK-01.
