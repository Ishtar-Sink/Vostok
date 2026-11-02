### Ler

- `vostok/work/specs/refined/SPEC-029-painel-local-do-vostok.md` (RF-03, RF-04, RNF-05, "Contratos e interfaces")
- `/home/victor/projetos/ai-hub-painel/apps/painel/package.json`, `vite.config.ts`, `tsconfig.json`, `index.html`
- `/home/victor/projetos/ai-hub-painel/apps/painel/src/main.tsx`, `app.css`, `css-modules.d.ts`
- `/home/victor/projetos/ai-hub-painel/apps/painel/src/shared/` (api.ts, router.ts, brand.ts, markdown.tsx, edit-dialog.tsx, ref-popover.tsx, references.ts, toc.tsx, back-link.tsx, status.ts)
- `/home/victor/projetos/ai-hub-painel/apps/painel/src/features/inbox/attention.ts` e `inbox.tsx` (ActionBar e o diálogo de ação, que passam a `shared/`)
- `/home/victor/projetos/ai-hub-painel/apps/painel/src/layout/` e `/home/victor/projetos/ai-hub-painel/apps/painel/src/theme/`
- `/home/victor/projetos/ai-hub-painel/apps/painel/src/ui/` (componentes copiados do Atria)
- `docs/estudos/painel-local/prototipo.md` (telas, marca e retorno do usuário)

### Mudar

- `apps/painel/package.json` (criar): Vite, Preact com compat, `marked`, TypeScript, Vitest e Testing Library, com os scripts `dev`, `build` e `test`.
- `apps/painel/pnpm-lock.yaml` (criar): lockfile do pnpm.
- `apps/painel/index.html` (criar): página do app.
- `apps/painel/vite.config.ts` (criar): aliases de Preact, proxy de `/api` e `/arquivos` para `PAINEL_API` no modo de desenvolvimento e configuração do Vitest.
- `apps/painel/tsconfig.json` (criar): TypeScript estrito.
- `apps/painel/scripts/print.sh` (criar): print de uma rota com a API de `PAINEL_BIN`.
- `apps/painel/src/main.tsx` (criar): app com tema, marca, menu e roteamento para as páginas das features.
- `apps/painel/src/app.css` (criar): estilos globais.
- `apps/painel/src/shared` (criar): API, roteador, marca, markdown, referências, índice, diálogo de edição, `attention.ts` e `action-bar.tsx`.
- `apps/painel/src/layout` (criar): menu lateral e topo.
- `apps/painel/src/theme` (criar): tokens do Atria.
- `apps/painel/src/ui` (criar): componentes copiados do Atria.
- `apps/painel/src/features/dashboard/dashboard-page.tsx` (criar): página provisória.
- `apps/painel/src/features/inbox/inbox-page.tsx` (criar): página provisória.
- `apps/painel/src/features/specs/specs-page.tsx` (criar): página provisória.
- `apps/painel/src/features/spec/spec-page.tsx` (criar): página provisória.
- `apps/painel/src/features/board/tasks-page.tsx` (criar): página provisória.
- `apps/painel/src/features/task/task-page.tsx` (criar): página provisória.
- `apps/painel/src/features/documents/documents-page.tsx` (criar): página provisória.
- `apps/painel/src/features/settings/settings-page.tsx` (criar): página provisória.
- `apps/painel/src/features/dashboard/dashboard-page.module.css` (criar): arquivo provisório da feature, preenchido pela tarefa de telas.
- `apps/painel/src/features/inbox/inbox-page.module.css` (criar): arquivo provisório da feature, preenchido pela tarefa de telas.
- `apps/painel/src/features/specs/specs-page.module.css` (criar): arquivo provisório da feature, preenchido pela tarefa de telas.
- `apps/painel/src/features/spec/spec-page.module.css` (criar): arquivo provisório da feature, preenchido pela tarefa de telas.
- `apps/painel/src/features/board/board.tsx` (criar): arquivo provisório da feature, preenchido pela tarefa de telas.
- `apps/painel/src/features/board/board.module.css` (criar): arquivo provisório da feature, preenchido pela tarefa de telas.
- `apps/painel/src/features/task/task-page.module.css` (criar): arquivo provisório da feature, preenchido pela tarefa de telas.
- `apps/painel/src/features/documents/documents-page.module.css` (criar): arquivo provisório da feature, preenchido pela tarefa de telas.
- `apps/painel/src/features/settings/settings-page.module.css` (criar): arquivo provisório da feature, preenchido pela tarefa de telas.
- `apps/painel/src/features/settings/schema.ts` (criar): arquivo provisório da feature, preenchido pela tarefa de telas.
- `apps/painel/src/features/settings/yaml.ts` (criar): arquivo provisório da feature, preenchido pela tarefa de telas.
- `apps/painel/src/features/settings/fields.tsx` (criar): arquivo provisório da feature, preenchido pela tarefa de telas.
- `apps/painel/src/features/settings/fields.module.css` (criar): arquivo provisório da feature, preenchido pela tarefa de telas.
- `.gitignore` (alterar): `apps/painel/node_modules/` e `apps/painel/dist/`.

### Passos

1. `apps/painel/package.json`, `index.html`, `tsconfig.json` e `vite.config.ts` (criar): a partir do spike, com o nome `@vostok/painel`. O Preact com compat passa a ser fixo, sem a variável `PAINEL_PREACT`. Acrescente `vitest`, `@testing-library/preact` e `jsdom`. Scripts: `dev` (`vite`), `build` (`tsc --noEmit && vite build`) e `test` (`vitest run`). No `vite.config.ts`, com `PAINEL_API` definida, faça o proxy de `/api` (com `ws: true`) e de `/arquivos` para esse endereço. Rode `pnpm --dir apps/painel install`. Confira: `pnpm --dir apps/painel exec tsc --noEmit` roda.
2. `apps/painel/src/theme`, `ui`, `layout` e `app.css` (criar): copie do spike os tokens, os componentes do Atria e o layout, com os testes de cada componente quando houver. Sem testes dos componentes copiados, escreva um teste de render do `Button` e do `Dialog` em `ui/`. Mantenha os ajustes de espaçamento do spike.
3. `apps/painel/src/shared` (criar): copie `api.ts`, `router.ts`, `brand.ts`, `markdown.tsx`, `references.ts`, `ref-popover.tsx`, `toc.tsx`, `back-link.tsx`, `status.ts` e `edit-dialog.tsx`. Mova para `shared/attention.ts` o `attention.ts` do spike, com `specActions` e `taskActions`. Mova para `shared/action-bar.tsx` o `ActionBar` e o `ActionDialog` de `inbox.tsx`. Assim nenhuma feature importa outra. Escreva os testes `router.test.ts` (cada rota da tabela da spec vira a página certa, inclusive `/planos/<ID>`) e `attention.test.ts` (uma tarefa `pending` gera "Aprovar tarefa"; uma `in_progress` gera "Pedir finalização").
4. `apps/painel/src/features/<feature>/<feature>-page.tsx` (criar): uma página provisória por feature, com o componente e as props do spike (`DashboardPage`, `InboxPage`, `SpecsPage`, `SpecPage`, `TasksPage`, `TaskPage`, `DocumentsPage`, `SettingsPage`). Cada página mostra só o `PageHeader` com o título da tela. Crie também, vazios ou com uma exportação mínima, os demais arquivos provisórios listados em Mudar, para as tarefas de telas só os alterarem. Nomes de arquivo: `dashboard-page.tsx`, `inbox-page.tsx`, `specs-page.tsx`, `spec-page.tsx`, `board/tasks-page.tsx`, `task-page.tsx`, `documents-page.tsx` e `settings-page.tsx`.
5. `apps/painel/src/main.tsx` (criar): o `App` do spike, importando só de `shared/`, `layout/` e das páginas do passo 4. O app aplica a marca de `/api/marca`, o tema claro ou escuro, o menu lateral com as specs em andamento e a contagem de pendências.
6. `apps/painel/scripts/print.sh` (criar): `print.sh <rota> <arquivo> [largura,altura]`. O script:
   - sobe `"$PAINEL_BIN" painel --sem-navegador --porta <livre>` na raiz do projeto;
   - sobe o `vite` em outra porta livre, com `PAINEL_API=http://127.0.0.1:<porta da API>`;
   - espera os dois responderem;
   - tira o print com o Chromium do Playwright, `--headless=new` e `--virtual-time-budget=3000`;
   - encerra os dois processos pelo PID, mesmo em erro (`trap`).
   Confira: `PAINEL_BIN=/home/victor/projetos/ai-hub-painel/apps/cli/target/release/vostok bash apps/painel/scripts/print.sh / /tmp/x.png` gera a imagem.
7. `.gitignore` (alterar): acrescente `apps/painel/node_modules/` e `apps/painel/dist/`.
8. Confira: `pnpm --dir apps/painel exec tsc --noEmit`, `pnpm --dir apps/painel test` e `pnpm --dir apps/painel build` passam. O `dist/` não entra no git (`git status --short apps/painel | grep dist` vazio).
