### Ler

- `vostok/work/specs/refined/SPEC-026-documentos-finais-com-a-identidade-visual-do-repositorio.md` (RF-02, RN-05, CT-01, CT-02, CB-07)
- `apps/cli/src/commands/documents.rs` (criado na TASK-01: DocumentsCommand)
- `apps/cli/tests/documents.rs` (criado na TASK-01: como o projeto de teste é montado)
- `apps/cli/src/harness.rs` (raiz do projeto: pasta pai de `Harness::root`)

### Mudar

- `apps/cli/src/commands/documents.rs` (alterar): subcomando `detect`
- `apps/cli/tests/documents.rs` (alterar): testes `detect_*`

### Passos

1. `apps/cli/tests/documents.rs` (alterar): escreva primeiro os testes:
   - `detect_lists_css_and_tailwind_colors_by_relevance` (CT-01): com `src/theme.css` contendo `--color-gray: #888888;` e `--color-primary: #0f766e;` e `tailwind.config.js` contendo `brand: '#f59e0b'`, a saída lista `src/theme.css:<linha>  --color-primary  #0f766e` e `tailwind.config.js:<linha>  brand  #f59e0b` antes do cinza.
   - `detect_ignores_generated_folders_and_secrets` (CT-02, RN-05): cores em `node_modules/x.css`, `target/x.css`, `.git/x.css`, `vostok/x.css`, `.env` e `.env.local` não aparecem.
   - `detect_lists_at_most_ten_candidates` (CB-07): com 14 cores, lista 10 e a linha `... e mais 4 cores`.
   - `detect_without_candidates_says_so` (RF-03): sem cores, a saída é `Nenhuma cor encontrada no repositório.` e o código é 0.
   Confira que falham: `cargo test --manifest-path apps/cli/Cargo.toml --test documents -- detect_`.
2. `apps/cli/src/commands/documents.rs` (alterar): variante `Detect` em `DocumentsCommand` e `fn detect`:
   - percorra a raiz do projeto (pasta pai de `Harness::root`) com `std::fs`, pulando as pastas `.git`, `node_modules`, `target`, `dist`, `build`, `vostok` e qualquer pasta que comece com `.`, e os arquivos `.env*`, `*.pem`, `*.key`, `*.p12`, `*.crt`;
   - leia só `*.css`, `*.scss`, `*.less` e `tailwind.config.{js,cjs,mjs,ts}`, até 1 MB cada;
   - capture, por linha, `--<nome>: <cor>` (CSS), `$<nome>: <cor>` (SCSS), `@<nome>: <cor>` (Less) e `<nome>: '<cor>'` ou `"<cor>"` (Tailwind), com `<cor>` em `#rgb` ou `#rrggbb`;
   - ordene: nome com `primary`, `brand` ou `accent` primeiro, depois as demais, e dentro de cada grupo pelo caminho e pela linha; remova repetições de nome e cor;
   - imprima `<caminho relativo>:<linha>  <nome>  <cor normalizada>` para as 10 primeiras e, se houver mais, `... e mais N cores`.
   Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test documents`.
3. Confira: `cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings`.
