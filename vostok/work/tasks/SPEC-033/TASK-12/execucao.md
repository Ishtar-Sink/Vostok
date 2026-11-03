### Ler

- `vostok/work/specs/refined/SPEC-033-operar-o-projeto-de-ponta-a-ponta-pelo-painel-v0-4-0.md` (RF-14, CT-15)
- `docs/debitos-tecnicos.md` (DT-017)
- `apps/painel/src/features/board/board.tsx`
- `apps/painel/src/features/board/board.module.css`
- `apps/painel/src/features/board/board.test.tsx`
- `apps/painel/src/shared/status.ts` (BOARD_COLUMNS, TONES)
- `apps/painel/src/layout/layout.module.css` (.shell, .page: largura da sidebar e padding do conteúdo)
- `apps/cli/src/painel/read.rs` (fn yaml_json, fn task_summary, fn task, fn spec, mod tests: fn project, fn get, fn body, teste task_returns_parts_state_and_evidence_without_subfolders)
- `apps/cli/src/support.rs` (fn config_get, fn unquote, fn frontmatter_get, fn unescape, fn yaml_escape)

### Mudar

- `apps/painel/src/features/board/board.tsx` (alterar): bolinha de cor de status no título das colunas do fluxo, inclusive "Em validação"
- `apps/painel/src/features/board/board.module.css` (alterar): colunas mais estreitas para as seis caberem a 1440 px e as cores da bolinha por tom
- `apps/painel/src/features/board/board.test.tsx` (alterar): testes da bolinha de cor e do título com aspas
- `apps/cli/src/painel/read.rs` (alterar): valores entre aspas duplas do `estado.yml` lidos desfazendo o escape do YAML, no quadro e na página da tarefa
- `apps/cli/src/support.rs` (alterar): a função que desfaz o escape do YAML passa a ser pública para o `read.rs`

### Passos

1. `apps/cli/src/painel/read.rs` (alterar): no `mod tests`, acrescente o teste `quoted_task_title_comes_without_escapes`. Com `let (dir, painel) = project();`, grave pelo `write(dir.path(), "vostok/work/tasks/SPEC-001/TASK-02/estado.yml", ...)` o texto em string crua do Rust:

   ```rust
   r#"id: TASK-02
   title: "Separar a coluna \"Em validação\" no quadro"
   description: "Cor em \"Em validação\" e barra \\ literal"
   status: pending
   wave: 1
   "#
   ```

   (sem o recuo do bloco, cada linha começando na coluna 0). Com `let spec = body(&get(&painel, "/api/specs/SPEC-001"));` e `let task = body(&get(&painel, "/api/tarefas/SPEC-001/TASK-02"));`, confira com `assert!` que as duas contêm `r#""title":"Separar a coluna \"Em validação\" no quadro""#`, que `task` contém `r#""description":"Cor em \"Em validação\" e barra \\ literal""#` e que nenhuma das duas contém `r#"\\\"Em"#` (barra de escape sobrando). Confira: `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::read::tests::quoted_task_title_comes_without_escapes` falha antes da mudança.
2. `apps/cli/src/support.rs` (alterar): renomeie `fn unescape` para `pub fn yaml_unescape`, com uma linha de doc dizendo que desfaz o escape de `yaml_escape` dentro de aspas duplas, e troque a chamada em `fn frontmatter_get`. Confira: `cargo build --manifest-path apps/cli/Cargo.toml --features painel` sem erro.
3. `apps/cli/src/painel/read.rs` (alterar): importe `yaml_unescape` de `crate::support`. Crie `fn yaml_scalar(raw: &str) -> String`: com `raw.trim()` entre aspas duplas (`len() >= 2`, começa e termina com `"`), devolve `yaml_unescape` do miolo; entre aspas simples, devolve o miolo com `''` trocado por `'`; sem aspas, devolve o texto até o primeiro ` #` (comentário), sem espaços nas pontas. Crie `fn estado_pairs(file: &Path) -> Vec<(String, String)>` com as linhas de primeiro nível de `yaml_json` (sem as que começam com espaço ou `#`, `split_once(':')`), com o valor por `yaml_scalar`. `yaml_json` passa a ser `json_object(&estado_pairs(file))`. Em `task_summary`, leia `estado_pairs(&estado)` uma vez e monte cada chave de `keys` com o valor achado ou `""`, no lugar do `config_get` por chave. O `config_get` continua importado: a `fn new` o usa para as pastas das specs. Confira: o teste do passo 1 passa e `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::read` passa inteiro.
4. `apps/painel/src/features/board/board.test.tsx` (alterar): no `describe('Board')`, acrescente `mostra a cor de status nas colunas do fluxo`: renderize `<Board tasks={[]} />` e confira, pelo `section[data-column]`, que as colunas `running`, `validating`, `blocked` e `done` têm um `[data-tone]` no `header` com os tons `info`, `accent`, `warning` e `success`, que `pending` e `approved` não têm, e que o elemento tem `aria-hidden="true"`. Acrescente também `mostra o título com aspas como veio`: uma tarefa com `title: 'Separar a coluna "Em validação" no quadro'` aparece com `getByText('Separar a coluna "Em validação" no quadro')`. Confira: o primeiro falha antes do passo 5 (o segundo passa já hoje e protege o texto do passo 3).
5. `apps/painel/src/features/board/board.tsx` (alterar): acima de `Board`, declare `const COLUMN_TONE: Record<string, 'info' | 'accent' | 'warning' | 'success'> = { running: 'info', validating: 'accent', blocked: 'warning', done: 'success' };`. No `h3` de cada coluna, antes do `column.label`, renderize `{COLUMN_TONE[column.id] ? <span className={styles.dot} data-tone={COLUMN_TONE[column.id]} aria-hidden="true" /> : null}`. Confira: o teste do passo 4 passa.
6. `apps/painel/src/features/board/board.module.css` (alterar): apague as quatro regras `.column[data-column=...] .columnHeader h3::before` e acrescente `.dot { display: inline-block; width: 8px; height: 8px; margin-right: 8px; border-radius: 50%; vertical-align: 1px; }` e `.dot[data-tone='info'] { background: var(--info); }`, `.dot[data-tone='accent'] { background: var(--accent); }`, `.dot[data-tone='warning'] { background: var(--warning); }`, `.dot[data-tone='success'] { background: var(--success); }`. Em `.board`, troque `grid-auto-columns: minmax(220px, 1fr)` por `grid-auto-columns: minmax(160px, 1fr)`; em `.cardTop`, acrescente `flex-wrap: wrap;` (o id com a spec, na página de tarefas, quebra em vez de vazar). Confira: o print `quadro-1440.png` mostra as seis colunas sem rolagem lateral.
7. `apps/painel` (conferir): `pnpm --dir apps/painel exec tsc --noEmit`. Confira: sem erros.
8. `apps/cli` (conferir): antes dos prints, compile o binário de release com a leitura nova: `cargo build --release --features painel --manifest-path apps/cli/Cargo.toml`. Confira: `apps/cli/target/release/vostok --version` responde.
