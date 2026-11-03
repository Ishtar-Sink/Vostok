### Ler

- `vostok/work/specs/refined/SPEC-033-operar-o-projeto-de-ponta-a-ponta-pelo-painel-v0-4-0.md` (RF-15, CT-16)
- `apps/cli/Cargo.toml` (linha `version` do `[package]`)
- `apps/cli/Cargo.lock` (bloco `name = "vostok"`)
- `apps/cli/tests/base.rs` (teste ct01_version_is_0_1_0)

### Mudar

- `apps/cli/Cargo.toml` (alterar): `version = "0.3.0"` passa a `version = "0.4.0"`.
- `apps/cli/Cargo.lock` (alterar): a versão do pacote `vostok` acompanha o `Cargo.toml` (o arquivo é versionado).
- `apps/cli/tests/base.rs` (alterar): o teste da versão espera `vostok 0.4.0`.

### Passos

1. `apps/cli/tests/base.rs` (alterar): em `fn ct01_version_is_0_1_0`, troque `.stdout("vostok 0.3.0\n");` por `.stdout("vostok 0.4.0\n");`. Não renomeie o teste. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test base -- ct01_version` falha (a versão ainda é 0.3.0).
2. `apps/cli/Cargo.toml` (alterar): na seção `[package]`, troque `version = "0.3.0"` por `version = "0.4.0"`. Não mude nenhuma dependência.
3. `apps/cli/Cargo.lock` (alterar): execute `cargo build --manifest-path apps/cli/Cargo.toml`; o Cargo atualiza sozinho o bloco `name = "vostok"` para `version = "0.4.0"`. Confira com `git diff apps/cli/Cargo.lock`: só essa linha mudou. Se outra linha mudou, desfaça-a com `git checkout -p apps/cli/Cargo.lock` e mantenha só a da versão.
4. `apps/cli` (conferir): `apps/cli/target/debug/vostok --version` imprime `vostok 0.4.0` e `cargo test --manifest-path apps/cli/Cargo.toml --test base` passa sem falhas.
5. Não mude o `README.md` (links da release) nem o `vostok/.manifest`: o README é atualizado na publicação da release e o manifesto pelo `vostok update`.
