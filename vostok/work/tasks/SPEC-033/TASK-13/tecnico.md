<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** A versão da CLI vem do `version` do `[package]` em `apps/cli/Cargo.toml` (o `clap` a imprime em `--version`). Seguindo o commit `f5416e2` (subida para 0.3.0), mudam três arquivos: `apps/cli/Cargo.toml`, `apps/cli/Cargo.lock` (versionado no git, atualizado pelo `cargo build`) e o teste `ct01_version_is_0_1_0` em `apps/cli/tests/base.rs`, que fixa a saída de `vostok --version`.

**Decisões.**
- O `Cargo.lock` e o teste entram em "Mudar", além do `Cargo.toml` do plano: sem eles, o lock fica divergente e o teste da versão falha. Alternativa descartada: mudar só o `Cargo.toml`, que quebraria `--test base`.
- O nome do teste (`ct01_version_is_0_1_0`) fica como está, para o baseline não perder a linha. Alternativa descartada: renomear para a versão nova, que apareceria como regressão.
- O `README.md` (links da release) e o `vostok/.manifest` ficam de fora: o primeiro muda na publicação da release, o segundo pelo `vostok update`.

**Riscos.** Nenhum relevante: a tarefa roda por último (onda 5), depois de TASK-06, TASK-07, TASK-09, TASK-10, TASK-11 e TASK-12, e não divide arquivos com nenhuma outra tarefa da onda.

**Referências da spec.** RF-15, CT-16, CA-08.
