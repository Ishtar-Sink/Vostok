<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** A versão da CLI veio do `version` do `[package]` em `apps/cli/Cargo.toml` (o `clap` a imprime em `--version`). Seguindo o commit `f5416e2` (subida para 0.3.0), mudaram três arquivos: `apps/cli/Cargo.toml`, `apps/cli/Cargo.lock` (versionado no git, atualizado pelo `cargo build`) e o teste `ct01_version_is_0_1_0` em `apps/cli/tests/base.rs`, que fixa a saída de `vostok --version`.

**Decisões.**
- O `Cargo.lock` e o teste entraram em "Mudar", além do `Cargo.toml` do plano: sem eles, o lock fica divergente e o teste da versão falha. Alternativa descartada: mudar só o `Cargo.toml`, que quebraria `--test base`.
- O nome do teste (`ct01_version_is_0_1_0`) ficou como estava, para o baseline não perder a linha. Alternativa descartada: renomear para a versão nova, que apareceria como regressão.
- O `README.md` (links da release) e o `vostok/.manifest` ficaram de fora: o primeiro muda na publicação da release, o segundo pelo `vostok update`.

**Riscos.** Nenhum relevante: a tarefa rodou por último (onda 5), depois de TASK-06, TASK-07, TASK-09, TASK-10, TASK-11 e TASK-12, e não divide arquivos com nenhuma outra tarefa da onda.

**Feito.** `version` do `[package]` passou de `0.3.0` a `0.4.0` em `apps/cli/Cargo.toml`; o `cargo build` atualizou só a linha da versão do `vostok` no `Cargo.lock`; o teste `ct01_version_is_0_1_0` passou a esperar `vostok 0.4.0` e falhou antes da troca da versão, como esperado. Testes, regressão (1 comando igual ao baseline e a mudança esperada da versão, com o motivo) e evidências (`versao.txt` com `vostok 0.4.0` e `versao-testes.txt` com 36 testes ok) passaram. A execução foi na sessão do Coordenador, com o modelo da sessão, por ser a única tarefa da onda.

**Pontos de atenção.**
- A tarefa esperou a SPEC-034, criada depois da demonstração, para a versão 0.4.0 sair com as correções do painel.
- Com a SPEC-034, o `build.rs` reconstrói o `dist` do painel mais antigo que o código; o `release.yml` já roda `pnpm --dir apps/painel build` antes do `cargo build`, então a publicação não muda.

**Referências da spec.** RF-15, CT-16, CA-08.
