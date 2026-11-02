### Ler

- `vostok/work/specs/refined/SPEC-026-documentos-finais-com-a-identidade-visual-do-repositorio.md` (RF-06, RF-07, RF-08, RF-12, RNF-01, RNF-02, RNF-04, RNF-05, CB-02, CB-03, CB-08, seção "Contratos e interfaces")
- `apps/cli/src/commands/finalize.rs` (fn header_values, fn css_tokens, fn config_group, fn base64, fn font_faces)
- `packages/harness/templates/finalize/executive.html` (`.brand`, cabeçalho e rodapé)
- `packages/harness/templates/finalize/technical.html` (`.brand`, cabeçalho e rodapé)
- `packages/harness/templates/finalize/evidence.html` (`.brand`, tabela e `#p-cobertura`)
- `apps/cli/tests/finalize.rs` (fn built_folder_project_with, fn built_documents, testes `visual_*` e fn golden_finalize)
- `docs/specs/SPEC-025/tecnico.html` (o defeito da coluna de IDs aparece no documento de evidências da SPEC-025)

### Mudar

- `apps/cli/src/commands/finalize.rs` (alterar): valores `BRAND_NAME` e `BRAND_MARK`, ícone embutido e erro de ícone ausente
- `packages/harness/templates/finalize/executive.html` (alterar): nome e marca do topo por variável
- `packages/harness/templates/finalize/technical.html` (alterar): nome e marca do topo por variável
- `packages/harness/templates/finalize/evidence.html` (alterar): nome e marca do topo por variável e IDs da cobertura sem quebra
- `apps/cli/tests/finalize.rs` (alterar): testes `brand_*` e `coverage_*`

### Passos

1. `apps/cli/tests/finalize.rs` (alterar): escreva primeiro os testes, com `built_folder_project_with` acrescentando ao `vostok/config/documents.yml` o grupo `brand` do contrato:
   - `brand_name_replaces_vostok_in_the_header` (CT-05, CB-08): com `name: "Atria & Co"`, os três documentos têm `<span class="brand">` com `Atria &amp; Co` e o rodapé ainda cita `Vostok`.
   - `brand_icon_is_embedded_in_the_header` (CT-06, RNF-02): com `icon: "logo.svg"` (um SVG de 1 KB na raiz do projeto), os três documentos têm `<img class="brand-mark" alt="" src="data:image/svg+xml;base64,` no topo e nenhum `<i></i>` na marca; o teste `visual_documents_have_no_external_requests` continua passando.
   - `brand_missing_icon_fails_the_build` (CB-02): com `icon: "sumiu.svg"`, o build sai com código 1 e a mensagem cita `sumiu.svg`.
   - `brand_large_or_unknown_icon_fails_the_build` (CB-03, RNF-05): ícone de 300 KB e ícone `.gif` saem com código 1 e a mensagem do limite (256 KB) ou das extensões (`svg`, `png`, `webp`).
   - `brand_empty_keeps_the_vostok_identity` (CT-07, RF-08): sem o grupo e com o grupo vazio, o topo tem `<i></i>Vostok`, como hoje.
   - `coverage_ids_do_not_wrap` (CT-11, RF-12): o CSS do documento de evidências tem `white-space: nowrap` para a primeira coluna das tabelas de `#p-cobertura`.
   Confira que falham: `cargo test --manifest-path apps/cli/Cargo.toml --test finalize -- brand_ coverage_`.
2. `apps/cli/src/commands/finalize.rs` (alterar): leia o grupo `brand` com `config_group` (ausente vale vazio) e acrescente a `fn header_values`:
   - `BRAND_NAME`: o `name` escapado para HTML, ou `Vostok` se vazio;
   - `BRAND_MARK`: com `icon` vazio, `<i></i>`; senão, o arquivo em relação à raiz do projeto (pasta pai de `Harness::root`) embutido como `<img class="brand-mark" alt="" src="data:<tipo>;base64,<base64>">`, com `image/svg+xml`, `image/png` ou `image/webp` pela extensão e `fn base64` existente; ausente, acima de 256 KB ou de outra extensão é erro do build, com o caminho e o motivo.
   Templates instalados sem as variáveis seguem como hoje (os goldens não mudam, RNF-04).
3. `packages/harness/templates/finalize/executive.html`, `technical.html` e `evidence.html` (alterar): troque `<span class="brand"><i></i>Vostok</span>` pela marca seguida do nome, com as variáveis `BRAND_MARK` e `BRAND_NAME` na sintaxe de variável dos templates (cerquilha e chaves), dentro do mesmo `<span class="brand">`; acrescente `.brand img { width: 18px; height: 18px; object-fit: contain; }` ao lado de `.brand i`; mantenha `Vostok` no rodapé (RN-04). Em `evidence.html`, acrescente `#p-cobertura th:first-child, #p-cobertura td:first-child { white-space: nowrap; }`. Nenhuma cor literal nova (RNF-01).
4. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test finalize` (inclui os goldens e `visual_colors_come_only_from_documents_yml`) e `cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings`.
5. Gere os dois prints de Evidências e confira que o nome e o ícone aparecem no topo nos dois temas e que `RNF-01` fica numa linha só.
