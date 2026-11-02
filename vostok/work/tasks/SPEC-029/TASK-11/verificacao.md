### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`
- `grep -c '^      - name:' .github/workflows/release.yml` (muda): a release ganha os passos de Node, do bundle e do tamanho

### Verificação

```bash
# evidência: integracao-testes.txt
cargo test --manifest-path apps/cli/Cargo.toml --features painel --test painel
cargo test --manifest-path apps/cli/Cargo.toml --features painel
cargo test --manifest-path apps/cli/Cargo.toml
cargo clippy --manifest-path apps/cli/Cargo.toml --features painel --all-targets -- -D warnings
```

Resultado esperado: os testes do painel, a suíte com e sem a feature e o clippy passam.

### Evidências

- `integracao-testes.txt` (automática, da verificação)
  - Prova: dois painéis ao mesmo tempo, cada um com a sua marca e só em `127.0.0.1`, e o fluxo completo de aprovar, editar, pedir e receber o aviso (RF-01, CT-01, RNF-02, RNF-04).
  - Trecho: os testes de `--test painel` com `ok` e a suíte com e sem a feature com `test result: ok`.
  - Falha: `FAILED`.
  - Espera: `test result: ok`
- `tamanho.txt` (automática): `pnpm --dir apps/painel build >/dev/null && cargo build -q --release --manifest-path apps/cli/Cargo.toml --features painel && T="$(mktemp -d)" && tar -czf "$T/vostok.tar.gz" -C apps/cli/target/release vostok && stat -c '%n %s' apps/cli/target/release/vostok "$T/vostok.tar.gz"`
  - Prova: o binário de release com o painel cabe no limite (RF-13, CT-15, RNF-01).
  - Trecho: o tamanho do `vostok` até 6000000 e o do tar.gz até 2300000.
  - Falha: um dos números acima do limite.
  - Tempo-limite: 600
- `painel-final.png` (print): `PAINEL_BIN=apps/cli/target/release/vostok bash apps/painel/scripts/print.sh / "$VOSTOK_EVIDENCE/painel-final.png" 1280,900`
  - Prova: o painel completo, servido pelo binário novo, com a marca da pasta e os dados reais (RF-01, RF-03, RF-04).
  - Trecho: o painel do ai-hub com os indicadores e as specs em andamento.
  - Falha: página provisória, sem marca ou com erro de API.
  - Requer: `apps/cli/tests/painel.rs`
