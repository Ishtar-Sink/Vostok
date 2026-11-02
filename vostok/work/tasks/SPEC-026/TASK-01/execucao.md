### Ler

- `vostok/work/specs/refined/SPEC-026-documentos-finais-com-a-identidade-visual-do-repositorio.md` (RF-04, RF-05, RF-09, RF-10, RF-11, RN-01 a RN-03, CB-01, CB-04 a CB-06, seção "Contratos e interfaces")
- `packages/harness/config/documents.yml` (grupos `light`, `dark` e `font`)
- `apps/cli/src/cli.rs` (enum Command e fn run)
- `apps/cli/src/commands/mod.rs`
- `apps/cli/src/commands/finalize.rs` (fn css_tokens, fn config_group, fn opens_group, fn config_value)
- `apps/cli/src/support.rs` (fn config_get)
- `apps/cli/src/embedded.rs` (fn file)
- `apps/cli/src/harness.rs` (struct Harness, fn config_file)
- `apps/cli/tests/common/mod.rs` (como os testes montam um projeto instalado)
- `docs/estudos/documentos-finais.md` (pares de contraste do estudo visual)

### Mudar

- `apps/cli/src/commands/documents.rs` (criar): subcomando `brand`, derivação das cores, contraste e gravação no `documents.yml`
- `apps/cli/src/commands/mod.rs` (alterar): registrar o módulo `documents`
- `apps/cli/src/cli.rs` (alterar): registrar o subcomando `documents`
- `packages/harness/config/documents.yml` (alterar): grupo `brand` vazio, com comentário
- `apps/cli/tests/documents.rs` (criar): testes `brand_*`

### Passos

1. `apps/cli/tests/documents.rs` (criar): escreva primeiro os testes, num projeto instalado como nos outros testes de `tests/`:
   - `brand_primary_rewrites_the_brand_tokens_of_both_themes` (CT-03): `brand --primary "#0f766e"` grava `brand.primary` e muda `primary`, `accent`, `brand_from`, `calm_from` e `bg` em `light` e `dark`; `success`, `warning`, `danger`, `info` e os `*_soft` deles ficam iguais (RN-01).
   - `brand_secondary_is_derived_or_taken` (CT-04, CB-05): sem `--secondary`, `brand_to` é a principal com o matiz girado em -60°; com `--secondary "#f59e0b"`, `brand_to` é `#f59e0b`; com a secundária igual à principal, aceita.
   - `brand_text_pairs_pass_aa_in_both_themes` (CT-08, CB-04, RNF-03): para `#facc15`, `#111111`, `#ff00ff` e `#0f766e`, calcula a razão de contraste (WCAG 2.x, luminância relativa) de `text` e `text_muted` sobre `bg` e `surface`, de `primary` e `accent` sobre `bg` e `surface` e de `primary_on` sobre `primary`, nos dois temas, e exige ≥ 4,5; para `#facc15`, a saída tem a linha de aviso de ajuste.
   - `brand_reset_restores_the_default_tokens` (CT-09): depois de um `brand`, `--reset` deixa `light` e `dark` iguais aos do `documents.yml` embutido e o grupo `brand` com valores vazios.
   - `brand_prints_each_token_before_and_after` (CT-10): a saída tem uma linha por token alterado, no formato `<tema>.<token>: <antes> -> <depois>`.
   - `brand_refuses_invalid_input` (CT-14, CB-01, CB-06): `--primary azul`, `--primary "#12"`, `--icon` inexistente, `--icon` de 300 KB e `--icon x.gif` saem com código 1, mensagem do motivo e o `documents.yml` igual byte a byte; com um `documents.yml` sem `bg`, a mensagem pede `vostok update`.
   - `brand_name_and_icon_are_recorded` (RF-06, RF-07): `--name "Atria" --icon logo.svg` grava `brand.name` e `brand.icon` (caminho relativo à raiz do projeto, pasta pai de `vostok/`).
   Confira que falham: `cargo test --manifest-path apps/cli/Cargo.toml --test documents`.
2. `packages/harness/config/documents.yml` (alterar): no fim, acrescente o grupo do contrato, com o comentário `# Identidade do repositório, gravada pelo \`vostok documents brand\`. Vazio usa a do harness.` e as chaves `name`, `icon` (com o comentário do contrato), `primary` e `secondary`, todas `""`.
3. `apps/cli/src/cli.rs` e `apps/cli/src/commands/mod.rs` (alterar): `pub mod documents;` em ordem alfabética e a variante `Documents(documents::DocumentsCommand)` com o doc `/// Identidade visual dos documentos finais.`, entre `Commit` e `Skill`, com `#[command(subcommand)]` e o braço em `fn run`.
4. `apps/cli/src/commands/documents.rs` (criar), com `DocumentsCommand::Brand(BrandArgs)` (`--primary`, `--secondary`, `--name`, `--icon` opcionais e `--reset` em conflito com os outros; sem nenhum, erro de uso):
   - cores: aceite `#rgb` e `#rrggbb` (sem diferenciar maiúsculas) e grave sempre `#rrggbb` minúsculo; converta entre hex, RGB e HSL com funções próprias, sem crate nova;
   - contraste: `fn contrast(a, b) -> f64` pela luminância relativa do WCAG 2.x;
   - `fn readable(color, backgrounds, step)`: muda a claridade HSL de 1 em 1 ponto (para baixo no claro, para cima no escuro) até ≥ 4,5 contra todos os fundos; devolve a cor e se houve ajuste;
   - derivação por tema, a partir da principal P e da secundária S (S = P com matiz -60° quando ausente), com os neutros padrão lidos do `documents.yml` embutido (`embedded::file("config/documents.yml")`):
     - neutros (`bg`, `bg_elevated`, `surface_muted`, `surface_hover`, `border`, `border_strong`, `text`, `text_muted`, `text_faint`, `code_bg`): mantêm a claridade e a saturação do padrão (saturação no máximo 40%) e trocam o matiz pelo de P; `surface` não muda;
     - claro: `primary` = readable(P, [bg, surface]); `primary_hover` = `primary` com claridade -8; `primary_soft` = matiz de P, saturação 70% da de P, claridade 93; `primary_on` = `#ffffff` se contraste ≥ 4,5 sobre `primary`, senão o `text` derivado; `accent` e `accent_soft` igual, a partir de S; `calm_from` e `calm_to` = matiz de P e de S com claridade 96 e saturação 80%; `shadow_*` = os padrão com o `rgba` trocado pelo RGB de P escurecido para claridade 20;
     - escuro: `primary` = readable(P com claridade mínima 65, [bg, surface]) subindo a claridade; `primary_hover` = +8; `primary_soft` = claridade 20, saturação 60% da de P; `primary_on` = o `bg` derivado; `accent` e `accent_soft` igual, a partir de S; `calm_from` e `calm_to` com claridade 14 e saturação 50%; `shadow_*` = os padrão;
     - `brand_from` = P e `brand_to` = S nos dois temas, sem ajuste (só preenchimento);
     - garanta `text` e `text_muted` ≥ 4,5 sobre `bg` e `surface` com `readable`;
   - para cada ajuste, imprima `aviso: <tema>.<token> ajustado de <antes> para <depois> para contraste 4,5:1`;
   - gravação: reescreva só as linhas `  <token>: "..."` dos grupos `light` e `dark` e as chaves do grupo `brand`, preservando comentários, ordem e as demais linhas; acrescente o grupo `brand` no fim se faltar; nada é gravado se alguma validação falhar;
   - `--name` e `--icon` só gravam `brand.name` e `brand.icon`; o ícone precisa existir em relação à raiz do projeto, ter extensão `svg`, `png` ou `webp` e até 256 KB;
   - `--reset`: copia `light` e `dark` do embutido e grava `brand` com valores vazios;
   - saída: `<tema>.<token>: <antes> -> <depois>` por token alterado e `Identidade gravada em vostok/config/documents.yml`.
   Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test documents`.
5. Confira: `cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings` e `cargo fmt --manifest-path apps/cli/Cargo.toml --check`.
