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
- `detect-saida.txt` (automática): `T="$(mktemp -d)"; cp -r packages/harness/. "$T/vostok/"; mkdir -p "$T/src" "$T/node_modules/lib"; printf ':root {\n  --color-gray: #888888;\n  --color-primary: #0f766e;\n}\n' > "$T/src/theme.css"; printf "module.exports = { theme: { colors: {\n  brand: '#f59e0b',\n} } };\n" > "$T/tailwind.config.js"; printf -- '--color-primary: #123456;\n' > "$T/node_modules/lib/x.css"; M="$PWD/apps/cli/Cargo.toml"; cd "$T" && cargo run --quiet --manifest-path "$M" -- documents detect`
  - Prova: a detecção rodando num repositório de exemplo com um CSS, um tema do Tailwind e uma dependência em `node_modules` (RF-02, CT-01, CT-02).
  - Trecho: `src/theme.css:3  --color-primary  #0f766e` e `tailwind.config.js:2  brand  #f59e0b` antes de `src/theme.css:2  --color-gray  #888888`, sem nenhuma linha de `node_modules`.
  - Falha: comando desconhecido, a cor `#123456` de `node_modules` na lista ou o cinza antes das cores de marca.
