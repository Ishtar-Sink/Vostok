### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test base 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort | grep -v 'communication_\|config_reads_three_levels'`
- `cargo test --manifest-path apps/cli/Cargo.toml --test init_update 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`

### Verificação

```bash
# evidência: config-testes.txt
cargo test --manifest-path apps/cli/Cargo.toml --test base -- communication_ config_
cargo test --manifest-path apps/cli/Cargo.toml --test init_update
cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings
```

### Evidências

- `config-testes.txt` (automática, da verificação)
  - Prova: as pastas de pedidos, perguntas e avais, os nomes das subpastas e o arquivo de avais sem spec vêm de `config/main.yml`, com o português como padrão, e nomes repetidos ou inválidos são recusados citando os estados (RF-16, CT-19, CT-20, CB-08).
  - Trecho: `test communication_paths_default_to_portuguese ... ok`, `test communication_paths_follow_the_config ... ok`, `test communication_folders_reject_repeated_names ... ok`, `test communication_dispatch_accepts_only_known_agents ... ok` e `test config_reads_three_levels ... ok`.
  - Falha: algum desses testes com `FAILED`, erro de compilação, falha no `init_update` ou aviso do clippy.
