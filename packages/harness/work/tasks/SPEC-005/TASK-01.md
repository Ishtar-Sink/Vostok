---
id: TASK-01
spec: SPEC-005
title: "Base do crate vostok"
effort: critica
depends_on: []
covers: [RF-01, RF-02, RF-14, RF-15, RF-24, CT-01, CT-02, CT-11]
type: tarefa
labels: [SPEC-005, critica]
status: done
content_hash: 5e8660008dbec3f81b4bac31b06212a876c4207f53b772449d12041689c96e39
adjustments: 0
adjusted_at: null
approved_at: 2026-10-13T11:19:56Z
approved_hash: 0049517dbd45718fa970df564c6c5c7736f253b3b14760cf66395e78c4a69090
attempts: 0
tests_result: passed
regression_result: passed
evidence_result: passed
---

# TASK-01: Base do crate vostok

## Execução

### Ler

- `packages/harness/work/specs/refined/SPEC-005-migracao-do-harness-para-rust.md`: RF-01, RF-02, RF-14, RF-15, RF-24, CT-01, CT-02, CT-11, CB-02, CB-07, RN-01, RNF-01 a RNF-07 e a tabela de subcomandos em "Contratos e interfaces".
- `packages/harness/scripts/lib/config.sh`, `frontmatter.sh`, `markdown.sh`, `text.sh` e `context.sh`: utilitários a reproduzir.
- `packages/harness/config/main.yml` e os demais `config/*.yml`: chaves lidas pelos comandos.
- `packages/harness/rules/project/architecture.md`, `stack.md`, `code-style.md`, `testing.md` e `naming-convention.md`: regras do projeto (Rust idiomático, clean code, testes de cenários de erro).

### Mudar

- `apps/cli/Cargo.toml` (criar): crate `vostok` 0.1.0, edição 2021, dependências mínimas (`clap` com derive, `serde`, `serde_yaml`, `sha2`, `include_dir`, `anyhow`; `tempfile` e `assert_cmd` só em dev).
- `apps/cli/src/main.rs` (criar): ponto de entrada, mapeia erros para saída 1 com a mensagem em pt-br.
- `apps/cli/src/cli.rs` (criar): definição clap com `--version` e todos os grupos (`context`, `memory`, `rules`, `spec`, `plan`, `task`, `commit`, `skill`, `finalize`, `init`, `update`) e seus subcomandos, conforme a tabela da spec.
- `apps/cli/src/harness.rs` (criar): localização do harness (`vostok/config/main.yml`, subindo até a raiz; nesta fase também aceita `packages/harness/config/main.yml` deste repositório) e resolução de `paths`.
- `apps/cli/src/support.rs` (criar): utilitários comuns: leitura de YAML de configuração com erro que cita arquivo e chave, front-matter, tabelas Markdown, templates com variáveis no formato `#` + `{NOME}` e SHA-256.
- `apps/cli/src/embedded.rs` (criar): conteúdo distribuível de `packages/harness` embutido com `include_dir` (RN-01), sem `work/`, `.state/`, `tests/`, `scripts/` nem entradas de memória.
- `apps/cli/src/commands/mod.rs` (criar): declara os módulos dos grupos.
- `apps/cli/src/commands/context.rs` (criar): esqueleto do grupo `context`; cada subcomando termina com "não implementado" e saída 1.
- `apps/cli/src/commands/memory.rs` (criar): esqueleto do grupo `memory`; cada subcomando termina com "não implementado" e saída 1.
- `apps/cli/src/commands/rules.rs` (criar): esqueleto do grupo `rules`; cada subcomando termina com "não implementado" e saída 1.
- `apps/cli/src/commands/spec.rs` (criar): esqueleto do grupo `spec`; cada subcomando termina com "não implementado" e saída 1.
- `apps/cli/src/commands/plan.rs` (criar): esqueleto do grupo `plan`; cada subcomando termina com "não implementado" e saída 1.
- `apps/cli/src/commands/task.rs` (criar): esqueleto do grupo `task`; cada subcomando termina com "não implementado" e saída 1.
- `apps/cli/src/commands/commit.rs` (criar): esqueleto do grupo `commit`; cada subcomando termina com "não implementado" e saída 1.
- `apps/cli/src/commands/skill.rs` (criar): esqueleto do grupo `skill`; cada subcomando termina com "não implementado" e saída 1.
- `apps/cli/src/commands/finalize.rs` (criar): esqueleto do grupo `finalize`; cada subcomando termina com "não implementado" e saída 1.
- `apps/cli/src/commands/init.rs` (criar): esqueleto do grupo `init`; cada subcomando termina com "não implementado" e saída 1.
- `apps/cli/src/commands/update.rs` (criar): esqueleto do grupo `update`; cada subcomando termina com "não implementado" e saída 1.
- `apps/cli/tests/base.rs` (criar): testes da base.
- `apps/cli/tests/common/mod.rs` (criar): helper de projeto temporário reaproveitado pelas outras tarefas.
- `.gitignore` (alterar): ignora `apps/cli/target/`.

### Passos

1. Crie o crate com `cargo new --bin apps/cli --name vostok` e ajuste o `Cargo.toml` (versão 0.1.0, dependências da lista).
2. Escreva primeiro `tests/base.rs` e `tests/common/mod.rs`: CT-01 (`vostok --version` imprime `0.1.0`), CT-02 (arquivos embutidos: contém skills padrão, templates, regras, contexto e configurações; não contém `work/`, `.state/`, `tests/` nem `scripts/`), CT-11 (subcomando numa subpasta do projeto funciona; fora de um projeto falha com mensagem que indica `vostok init`), CB-07 (chave obrigatória ausente cita arquivo e chave) e erros de YAML inválido.
3. Implemente `harness.rs`, `support.rs` e `embedded.rs`. Os utilitários seguem o comportamento das libs bash citadas em "Ler"; não deixe caminho fixo no código (RF-15).
4. Implemente `cli.rs` com todos os grupos e subcomandos da tabela da spec, com os argumentos dos scripts atuais, e os esqueletos em `commands/`. Cada grupo fica num arquivo próprio para as tarefas da onda 2 não disputarem arquivos.
5. Acrescente `apps/cli/target/` ao `.gitignore` sem mudar as outras linhas.
6. Rode `cargo test`, `cargo clippy -- -D warnings` e `cargo fmt --check`.

### Baseline

- `packages/harness/tests/run-tests.sh > /dev/null && echo "suíte sem falhas"`

### Verificação

```bash
cargo test --manifest-path apps/cli/Cargo.toml
cargo clippy --manifest-path apps/cli/Cargo.toml -- -D warnings
cargo fmt --manifest-path apps/cli/Cargo.toml --check
cargo run --quiet --manifest-path apps/cli/Cargo.toml -- --version   # 0.1.0
```

### Evidências

- `cargo-test.txt` (automática): `cargo test --manifest-path apps/cli/Cargo.toml`
- `versao.txt` (automática): `cargo run --quiet --manifest-path apps/cli/Cargo.toml -- --version`
- `ajuda.txt` (automática): `cargo run --quiet --manifest-path apps/cli/Cargo.toml -- --help`

### Concluída quando

- [x] O binário `vostok` 0.1.0 compila e imprime a versão (RF-01, RF-24, CT-01).

  > Como comprovar: abrir `versao.txt` e ver `0.1.0`.

- [x] O conteúdo distribuível está embutido, sem work, estado, testes nem scripts (RF-02, CT-02).

  > Como comprovar: no `cargo-test.txt`, o teste CT-02 passa.

- [x] O harness é localizado a partir de subpastas, e fora de um projeto o erro indica `vostok init` (RF-14, CT-11).

  > Como comprovar: no `cargo-test.txt`, o teste CT-11 passa.

- [x] A configuração é lida dos YAML, e chave ausente cita arquivo e chave (RF-15, CB-07).

  > Como comprovar: no `cargo-test.txt`, o teste de chave ausente passa.

- [x] Todos os grupos e subcomandos da spec estão registrados.

  > Como comprovar: abrir `ajuda.txt` e ver os 11 grupos.

- [x] Clippy e fmt passam (RNF-03).

  > Como comprovar: os comandos da verificação terminam sem erro.

## Descrição

### Resumo executivo

Cria a base do novo programa do Vostok, que substituirá os scripts atuais: o executável, a leitura das configurações e o conteúdo do harness embutido. Esforço crítico, por definir a estrutura que todas as outras tarefas usam. Risco médio, mitigado por testes desde o início. Desbloqueia toda a reescrita em paralelo.

### Detalhes técnicos

Crate `vostok` em `apps/cli` (inferência B), com clap e um módulo por grupo de comandos já registrado como esqueleto, para as 6 tarefas da onda 2 trabalharem em paralelo sem conflito. Conteúdo embutido com `include_dir` conforme RN-01. Localização do harness por `vostok/config/main.yml` (RF-14), aceitando `packages/harness` só enquanto este projeto não migra (TASK-11). Cobre RF-01, RF-02, RF-14, RF-15, RF-24, CT-01, CT-02 e CT-11.

### Card

- **Título:** Base do crate vostok
- **Tipo:** tarefa
- **Descrição:** Cria o crate Rust `vostok` com a CLI, a leitura de configuração, os utilitários comuns e o conteúdo do harness embutido.
- **Critérios de aceite:**
  - [ ] Versão 0.1.0.
  - [ ] Conteúdo embutido correto.
  - [ ] Localização do harness.
  - [ ] Grupos registrados.
  - [ ] Clippy e fmt sem erros.
- **Rótulos:** SPEC-005, critica
- **Estimativa:** critica
- **Bloqueada por:** -

## Registro de execução

| # | Data | Etapa | Resultado | Detalhe |
|---|---|---|---|---|
| 1 | 2026-10-13 | início | in_progress | aprovada em 2026-10-13T11:19:56Z |
| 2 | 2026-10-13 | baseline | coletado | 1 comando(s); códigos de saída: 0 |
| 3 | 2026-10-13 | testes | passed | Verificação sem falhas; saída em validation/tests.out |
| 4 | 2026-10-13 | regressão | passed | 1 comando(s) iguais ao baseline |
| 5 | 2026-10-13 | evidências | passed | 3 evidência(s) em evidence/TASK-01/ |
| 6 | 2026-10-13 | conclusão | done | testes, regressão e evidências passed; critérios de conclusão marcados |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
