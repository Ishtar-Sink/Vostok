### Ler

- `vostok/work/specs/refined/SPEC-033-operar-o-projeto-de-ponta-a-ponta-pelo-painel-v0-4-0.md` (RF-06, RF-07, RF-11, RN-01, RNF-03, RNF-04, FA-02, CB-05, CB-06, CT-07, CT-08, CT-12, CT-18)
- `vostok/work/tasks/SPEC-033/TASK-08/execucao.md` (rotas `POST /api/conferencias`, `POST /api/limpeza`, pedidos `liberar-commits` e `desfazer-commits`, `responder-pergunta` com `pergunta=`)
- `apps/painel/src/shared/attention.ts` e `apps/painel/src/shared/action-bar.tsx` (já com as ações da TASK-09)
- `apps/painel/src/features/task/task-page.tsx` (fn Evidence, fn EvidenceItem) e `apps/painel/src/features/task/task-page.test.tsx`
- `apps/painel/src/features/inbox/inbox-page.tsx` (fn RequestCommit, fn PendingList) e `apps/painel/src/features/inbox/inbox-page.test.tsx`
- `apps/painel/src/features/settings/settings-page.tsx` (fn SettingsPage, até a linha 140; fn sha256Hex) e `apps/painel/src/features/settings/settings-page.test.tsx`
- `apps/painel/src/ui/confirm-dialog/confirm-dialog.tsx`, `apps/painel/src/ui/dialog/dialog.tsx`, `apps/painel/src/ui/badge/badge.tsx`
- `apps/cli/src/commands/commit.rs` (struct Repo, fn open, fn has_git, fn ack_errors, fn changes, fn report, fn check)
- `apps/cli/src/painel/read.rs` (fn route, mod tests: fn project, fn get, fn body)

### Mudar

- `apps/painel/src/features/task/task-page.tsx` (alterar): conferência dos prints (aceitar e rejeitar) na seção Evidências
- `apps/painel/src/features/task/task-page.module.css` (alterar): linha de conferência sob o print
- `apps/painel/src/features/task/task-page.test.tsx` (alterar): testes da conferência
- `apps/painel/src/features/inbox/inbox-page.tsx` (alterar): commits apontados com "Liberar commits" e "Desfazer commits", este com confirmação
- `apps/painel/src/features/inbox/inbox-page.module.css` (alterar): bloco dos commits apontados
- `apps/painel/src/features/inbox/inbox-page.test.tsx` (alterar): testes dos commits e da resposta à pergunta
- `apps/painel/src/features/settings/settings-page.tsx` (alterar): "Limpar pedidos arquivados" com a lista e a confirmação
- `apps/painel/src/features/settings/settings-page.test.tsx` (alterar): testes da limpeza
- `apps/painel/src/shared/attention.ts` (alterar): ações `release-commits` e `undo-commits`
- `apps/painel/src/shared/action-bar.tsx` (alterar): pedidos `liberar-commits` e `desfazer-commits` e o `pergunta=` na resposta
- `apps/cli/src/commands/commit.rs` (alterar): `pub fn pending_commits`, as linhas do `commit check` sem recusar
- `apps/cli/src/painel/read.rs` (alterar): rota `GET /api/commits` e o teste dela

### Passos

Antes de começar, confira que a TASK-08 e a TASK-09 estão concluídas (`vostok task status SPEC-033`); se não, **PARE** e responda com o template `blocker`.

1. `apps/cli/src/commands/commit.rs` (alterar): acrescente `pub fn pending_commits(harness: &Harness) -> Result<Vec<String>>`: `Repo::open(harness)`; sem git ou sem a referência (`!repo.has_git() || !repo.ack.is_file()`), devolve vazio; com `ack_errors` não vazio, devolve essas linhas; senão, `repo.report(&repo.changes()?)`. Não grava nada (não chama `store`). Confira: `cargo build --manifest-path apps/cli/Cargo.toml --features painel`.
2. `apps/cli/src/painel/read.rs` (alterar): em `fn route`, `["api", "commits"]` devolve `{"linhas":[...]}` com `commit::pending_commits(&Harness::locate(&painel.project)?)` (erro vira lista vazia), cada linha por `json_string`. Em `mod tests`, acrescente `commits_route_is_empty_without_git`: no `project()`, sem `.git`, `GET /api/commits` devolve `{"linhas":[]}`. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::read`.
3. `apps/painel/src/features/task/task-page.test.tsx` (alterar): com `verificacao` contendo `` - `tela.png` (print): `cmd` ``, a evidência `tela.png` e o simulado respondendo `/arquivos/` com `new Response('img')`, acrescente:
   - `mostra o print a conferir com aceitar e rejeitar`: tarefa `in_progress`, registro sem conferência: o selo `a conferir` e os botões `getByRole('button', { name: 'Aceitar tela.png' })` e `Rejeitar tela.png`.
   - `aceita o print pelo painel`: clicar em "Aceitar tela.png" chama `fetch` com `'/api/conferencias?spec=SPEC-001&tarefa=TASK-02&print=tela.png&resultado=aceito'` e `method: 'POST'`; o `getByRole('status')` contém `conferido`.
   - `rejeita o print depois de confirmar`: "Rejeitar tela.png" abre `getByRole('alertdialog')`; "Cancelar" fecha sem `fetch` de conferência; de novo, "Rejeitar print" chama `resultado=rejeitado`.
   - `mostra o print já conferido sem os botões`: o registro tem `print tela.png conferido pelo usuário (sha256 <8 primeiros hex do SHA-256 de 'img'>)` (calcule no teste com `createHash('sha256')` de `node:crypto`): selo `conferido` e nenhum botão "Aceitar tela.png".
   - `mostra a recusa da conferência`: `/api/conferencias` responde `new Response('não é um print desta tarefa', { status: 422 })` e o `getByRole('alert')` contém a mensagem.
   - `não oferece conferência na tarefa concluída`: tarefa `done`: nenhum botão "Aceitar tela.png".
   Confira: falham antes do passo 4.
4. `apps/painel/src/features/task/task-page.tsx` e `task-page.module.css` (alterar):
   - `function printNames(verificacao: string): string[]`: os nomes dos itens `` - `<nome>` (print): `` da seção (regex `/^- `([^`]+)` \(print\):/gm`).
   - `function reviewedHash(registro: string, name: string): string | null`: o hash da última linha com `print <name> conferido pelo usuário (sha256 <hash>)`.
   - Em `TaskPage`, passe a `Evidence` os `prints` (`printNames(files.verificacao ?? '')`), o `registro`, `spec`, `task` e `reviewable = ['in_progress', 'validating'].includes(state.status)`.
   - Em `EvidenceItem`, quando o nome está em `prints`: calcule o SHA-256 da imagem (`fetch(fileUrl(path))`, `arrayBuffer`, `crypto.subtle.digest`, 8 primeiros hex) e compare com `reviewedHash`. Abaixo da imagem, uma linha `.review` com o `Badge` `conferido` (`success`) ou `a conferir` (`warning`) e, só com `reviewable` e não conferido, `Button variant="primary"` `aria-label={'Aceitar ' + nome}` com o texto "Aceitar" e `Button variant="secondary"` `aria-label={'Rejeitar ' + nome}` com o texto "Rejeitar". Aceitar chama `act('POST', '/api/conferencias?spec=..&tarefa=..&print=..&resultado=aceito')` (valores com `encodeURIComponent`); Rejeitar abre o `ConfirmDialog` com título `Rejeitar o print <nome>?`, descrição `A tarefa fica bloqueada até você decidir no painel, como numa falha da finalização.`, `confirmLabel="Rejeitar print"` e `cancelLabel="Cancelar"`, e só no confirmar chama `resultado=rejeitado`. Sucesso: `<p role="status">` com `Print <nome> conferido pelo painel.` ou `Print <nome> rejeitado; a tarefa ficou bloqueada.`; erro: `<p role="alert">` com o texto da resposta. Enquanto envia, os dois botões ficam `disabled`.
   - CSS: `.review { display: flex; flex-wrap: wrap; align-items: center; gap: 8px; }` e `.reviewMessage { margin: 0; color: var(--text-muted); font-size: var(--font-size-sm); }`.
   Confira: os testes do passo 3 passam e os antigos continuam passando.
5. `apps/painel/src/shared/attention.ts` (alterar): em `ActionKind`, `'release-commits' | 'undo-commits'`; em `ACTION_LABELS`, `'release-commits': { button: 'Liberar commits', tag: 'Commit', how: 'O painel grava um pedido de liberação com aval; o agente em loop libera os commits apontados pela skill approve-commit.' }` e `'undo-commits': { button: 'Desfazer commits', tag: 'Commit', how: 'O painel grava um pedido com aval; o agente em loop tira os commits novos da branch atual e mantém as alterações na pasta de trabalho.' }`. `apps/painel/src/shared/action-bar.tsx` (alterar): em `REQUEST_ACTION`, `'release-commits': 'liberar-commits'` e `'undo-commits': 'desfazer-commits'`; exporte `perform` como `export async function performAction`, trocando a chamada no `ActionDialog`; na resposta (`item.question`), o corpo passa a ser só o texto e a URL ganha `&pergunta=<id da pergunta>`. Confira: `pnpm --dir apps/painel exec tsc --noEmit`.
6. `apps/painel/src/features/inbox/inbox-page.test.tsx` (alterar): o simulado responde `/api/commits` com `json({ linhas: [...] })` (vazio nos testes que já existem) e acrescente:
   - `mostra os commits apontados com liberar e desfazer`: com `linhas: ['Commits novos:', 'abc1234 feat: x']`, a região `getByRole('region', { name: 'Commits sem liberação' })` mostra `abc1234 feat: x` e os botões "Liberar commits" e "Desfazer commits".
   - `libera os commits pelo pedido`: "Liberar commits" abre o diálogo; o último "Liberar commits" chama `/api/pedidos?acao=liberar-commits` com `method: 'POST'` e o corpo com as linhas.
   - `desfaz os commits só depois de confirmar`: "Desfazer commits" abre `getByRole('alertdialog')`; "Cancelar" fecha sem `fetch` de pedido; de novo e "Desfazer commits" no diálogo chama `/api/pedidos?acao=desfazer-commits`; o `getByRole('status')` mostra o `PED-` devolvido.
   - `esconde os commits quando não há nenhum`: `linhas: []` e `queryByRole('region', { name: 'Commits sem liberação' })` é `null`.
   - `responde a pergunta com o id dela`: escolher a opção `A` e clicar no último "Responder" chama `/api/pedidos?acao=responder-pergunta&spec=SPEC-001&pergunta=Q-1` com `body: 'A'`; com 409, o `getByRole('alert')` mostra o texto da resposta (CB-06).
   Confira: falham antes do passo 7.
7. `apps/painel/src/features/inbox/inbox-page.tsx` e `inbox-page.module.css` (alterar): crie `function PendingCommits({ version })`, com `useResource<{ linhas: string[] }>('/api/commits', version)`; carregando, com erro ou vazio não mostra nada. Com linhas, um `<section aria-labelledby>` com o `h2` "Commits sem liberação", a nota `Commits feitos sem liberação, apontados pela verificação de commits.`, a lista `<ul>` das linhas em monoespaçada e os botões `Button variant="primary"` "Liberar commits" (abre o `ActionDialog` com `{ key: 'release', kind: 'release-commits', spec: '', title: linhas.join('\n'), reason: '', href: '/pendencias' }`) e `Button variant="secondary"` "Desfazer commits" (abre o `ConfirmDialog` com título `Desfazer os commits apontados?`, descrição `Os commits novos saem da branch atual; as alterações ficam na pasta de trabalho.`, `confirmLabel="Desfazer commits"`, `cancelLabel="Cancelar"`; no confirmar, `performAction` com o item `undo-commits` e o texto das linhas, e o resultado num `<p role="status">` ou `<p role="alert">`). Mostre `PendingCommits` entre o `PageHeader` e a `PendingList`. CSS `.commits` no molde de `.item` (borda esquerda `var(--warning)`), `.commitLines` com `font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: var(--font-size-xs)`. Confira: os testes do passo 6 passam.
8. `apps/painel/src/features/settings/settings-page.test.tsx` (alterar): com o simulado respondendo `/api/configuracoes` com `[file]`, acrescente:
   - `lista os pedidos arquivados antes de apagar`: "Limpar pedidos arquivados" chama `POST /api/limpeza` (sem `confirmar`) e o `getByRole('dialog', { name: 'Limpar pedidos arquivados' })` lista os dois caminhos de `{ arquivos: [...] }`; nenhuma chamada com `confirmar`.
   - `apaga só depois de confirmar`: no diálogo, "Apagar 2 arquivos" chama `/api/limpeza?confirmar=sim` e o `getByRole('status')` mostra `2 arquivos apagados.`; "Cancelar" antes fecha sem essa chamada.
   - `diz que não há o que limpar`: `{ arquivos: [] }` mostra `Não há pedidos nem perguntas para limpar.` e nenhum botão "Apagar" (CB-05).
   - `mostra o erro da limpeza`: 500 com `falhou` mostra `getByRole('alert')` com `falhou`.
   Confira: falham antes do passo 9.
9. `apps/painel/src/features/settings/settings-page.tsx` (alterar): crie `function PruneRequests()`: `Button variant="secondary"` "Limpar pedidos arquivados" no `actions` do `PageHeader`; ao clicar, `act('POST', '/api/limpeza')` e abre o `Dialog` de título "Limpar pedidos arquivados" com a nota `Pedidos e perguntas já arquivados com a spec e concluídos sem spec há mais de 30 dias. Os avais nunca são apagados.`, a lista `<ul>` dos caminhos (ou o texto do vazio) e as ações "Cancelar" e, com itens, `Button variant="primary"` `Apagar N arquivos` (`1 arquivo` no singular), que chama `act('POST', '/api/limpeza?confirmar=sim')` e mostra `N arquivos apagados.` em `role="status"`. Erro em `role="alert"`. Enquanto envia, o botão mostra `Apagando…` e fica `disabled`. Confira: os testes do passo 8 passam.
10. `apps/painel` (conferir): `pnpm --dir apps/painel exec tsc --noEmit`. Confira: sem erros.
11. `apps/cli` (conferir): antes dos prints, compile o binário de release com as rotas da TASK-08 e a `GET /api/commits`: `cargo build --release --features painel --manifest-path apps/cli/Cargo.toml`. Confira: `apps/cli/target/release/vostok --version` responde.
