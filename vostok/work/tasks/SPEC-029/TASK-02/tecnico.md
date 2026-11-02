<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Cria o módulo `apps/cli/src/painel/` atrás da feature `painel`: `mod.rs` com os tipos comuns, `server.rs` com o `tiny_http`, e esqueletos de `read.rs`, `live.rs` e `actions.rs` com assinaturas fixas, para a onda 2 implementar cada um sem tocar no `server.rs`. Cria o comando `vostok painel` e a geração do bundle no `build.rs`. Cobre RF-01, RF-08, RF-12, CT-02, CT-05 e CT-14.

**Decisões.**
- `tiny_http` com uma thread por requisição, no lugar de `axum` com `tokio`: o estudo mediu cerca de 537 KB a menos, e o painel atende poucas conexões locais.
- Os esqueletos com assinatura fixa permitem paralelismo na onda 2. Deixar cada tarefa registrar a própria rota no `server.rs` foi descartado, porque criaria um arquivo em comum.
- O `build.rs` gera o bundle quando há `pnpm` e só liga `painel_bundle` com `dist/` pronto (decisão 1 da spec). Falhar sem Node foi descartado, porque quebraria o `cargo install` de quem não tem Node.
- A feature não é padrão, para o `cargo test` sem feature continuar igual.

**Riscos.** Escutar em `0.0.0.0` por engano: o teste confere o endereço. Token previsível: vem de `/dev/urandom`.

**Arquivos em comum na onda.** Nenhum.

## O que foi feito

**Configuração e build:**
- `Cargo.toml`: feature `painel` com `tiny_http` e `sha1` opcionais. O `Cargo.lock` foi atualizado junto.
- `build.rs`:
  - com a feature e sem `apps/painel/dist/index.html`, roda `pnpm install --frozen-lockfile` e `pnpm build` quando há `pnpm`;
  - se não houver `pnpm` ou a geração falhar, emite `cargo:warning`;
  - liga o `cfg(painel_bundle)` só com o `dist/` pronto.

**Módulo `apps/cli/src/painel/`:**
- `mod.rs`: `Painel`, `Reply`, `session_token` (16 bytes de `/dev/urandom`) e os ajudantes de JSON, texto, consulta, decodificação e segmento seguro.
- `server.rs`:
  - `tiny_http` em `127.0.0.1`, com uma thread por requisição;
  - POST e PUT exigem `X-Painel-Token` (403 sem ele) e aceitam corpo de até 2 MB; outros métodos recebem 405;
  - o `index.html` sai com `<meta name="painel-token">`;
  - sem bundle, responde 503 com a instrução para gerá-lo.
- Esqueletos com assinatura fixa para a onda 2:
  - `read::route` devolve `None`;
  - `live::version` e `live::handle` respondem 501;
  - `actions::act` responde 501;
  - `parts` traz os segmentos a partir de `api`.

**Comando `vostok painel`:**
- `commands/painel.rs`, `cli.rs`, `mod.rs` e `lib.rs` criam `vostok painel [--porta N] [--pasta DIR] [--sem-navegador]`.
- O comando acha o harness subindo as pastas e abre o navegador.
- Sem a feature, o subcomando avisa que o binário foi compilado sem o painel.
- Uma porta ocupada dá erro citando a porta.

**Testes:** 16 unitários no módulo e 10 de integração em `tests/painel_server.rs`, que não dependem de `tests/common`.

**Pontos de atenção:**
- O caminho sem bundle (`cfg(not(painel_bundle))`) compila, mas não rodou nos testes, porque o `dist/` já existia.
- A SPEC-028 roda em paralelo, e nenhum arquivo é comum entre as duas.

**Verificação:**
- Os testes passaram, assim como o build e o clippy com e sem a feature.
- A regressão não encontrou diferença: o `--help` só ganhou a linha do `painel`.
- As evidências são `servidor-testes.txt` e `sem-pasta.txt` (código 1 fora de um projeto).
