### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test documents 2>&1 | grep -E '^test ' | grep -v detect_ | sort`

### Verificação

```bash
# evidência: detect-tests.txt
cargo test --manifest-path apps/cli/Cargo.toml --test documents -- detect_
cargo test --manifest-path apps/cli/Cargo.toml --test documents
cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings
```

Resultado esperado: os testes `detect_` e os `brand_` passam, e o clippy termina sem avisos.

### Evidências

- `detect-tests.txt` (automática, da verificação)
  - Prova: a detecção lista as cores por relevância, ignora pastas geradas e segredos e limita a 10 (RF-02, CT-01, CT-02, CB-07).
  - Trecho: as linhas `test detect_... ok`.
  - Falha: `FAILED` ou ausência dos testes `detect_`.
  - Espera: `detect_ignores_generated_folders_and_secrets ... ok`
- `detect-saida.txt` (automática): `cargo run --quiet --manifest-path apps/cli/Cargo.toml -- documents detect`
  - Prova: a detecção rodando neste repositório, que tem as cores dos protótipos em `docs/estudos/` (RF-02).
  - Trecho: linhas `<arquivo>:<linha>  <nome>  <cor>`, sem nenhum caminho de `vostok/` ou `target/`.
  - Falha: comando desconhecido ou caminhos de pastas ignoradas.
