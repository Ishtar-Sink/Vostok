<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Cria `apps/painel` a partir do spike, já fixo em Preact com compat. A tarefa traz o tema, os componentes do Atria, o layout, o roteador, a API compartilhada e uma página provisória por feature. Cobre RF-03, RF-04 e CT-03.

**Decisões.**
- **Ações em `shared/`:** `attention.ts` e o `ActionBar` passam de `features/inbox` para `shared/`. No spike, a spec, a tarefa e o painel importavam a feature de pendências, o que obrigaria as tarefas de telas a rodar em sequência.
- **Páginas provisórias:** cada página fica no caminho e com o nome final. As tarefas de telas trocam só o próprio arquivo, sem tocar no roteador nem no `main.tsx`.
- **Proxy no modo de desenvolvimento:** o `vite` encaminha `/api` para um painel que já roda (`PAINEL_API`). Assim as telas são testadas e fotografadas com dados reais antes de a API nova ficar pronta. Simular a API com dados fixos foi descartado, porque eles divergiriam do contrato real.
- **Dois binários nos prints:** os prints desta onda usam o binário do spike (`PAINEL_BIN`), que segue o mesmo contrato. A TASK-11 refaz os prints com o binário novo.

**Riscos.** O spike em `/home/victor/projetos/ai-hub-painel` precisa existir até a TASK-11. O `dist/` nunca entra no git.

**Arquivos em comum na onda.** Nenhum.

## O que foi feito

**Configuração do app (`apps/painel`):**
- Vite com Preact com compat, via `@preact/preset-vite`, com os tipos por `@types/react`.
- TypeScript estrito, `marked`, Vitest, `@testing-library/preact` e `jsdom`.
- Scripts `dev`, `build` (`tsc --noEmit && vite build`) e `test`.
- No modo de desenvolvimento, o `vite.config.ts` faz o proxy de `/api` (com `ws`) e de `/arquivos` para o endereço em `PAINEL_API`.

**Código:**
- **`theme/`, `ui/`, `layout/` e `app.css`:** copiados do spike.
- **`shared/`:**
  - api, roteador, marca, markdown, referências, popover, índice, voltar, status e diálogo de edição;
  - `attention.ts` e `action-bar.tsx` (com `action-bar.module.css`), que saíram de `features/inbox`, para nenhuma feature importar outra.
- **`main.tsx`:** importa só de `shared/`, `layout/` e das páginas.
- **Arquivos provisórios das features:**
  - as 8 páginas mostram só o `PageHeader`;
  - `board.tsx`, `schema.ts`, `yaml.ts` e `fields.tsx` têm só `export {};`;
  - os `.module.css` estão vazios.

**Outros arquivos:**
- `scripts/print.sh`:
  - sobe o `PAINEL_BIN` e o `vite` em portas livres;
  - usa o Chromium do Playwright em `~/.cache/ms-playwright`, ou o de `PAINEL_CHROMIUM`;
  - encerra os dois processos por `trap`.
- `.gitignore`: acrescenta `apps/painel/node_modules/` e `apps/painel/dist/`.

**Testes:** `router.test.ts`, `attention.test.ts`, `button.test.tsx` e `dialog.test.tsx`, com 30 casos.

**Pontos de atenção:**
- No teste do `Button`, a checagem de clique em botão desabilitado saiu, porque o jsdom dispara o evento mesmo assim. Ficaram `disabled` e `aria-busy`.
- Os prints desta onda e da próxima usam o binário do spike, e a TASK-11 os refaz com o binário novo.

**Verificação:**
- `tsc --noEmit`, os testes e o build passaram.
- A regressão só registrou a mudança esperada: a pasta `painel` passa a existir.
- A evidência `frontend-base.txt` foi gravada, e o print `base-claro.png` foi conferido pelo usuário.
