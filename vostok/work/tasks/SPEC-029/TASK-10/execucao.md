### Ler

- `vostok/work/specs/refined/SPEC-029-painel-local-do-vostok.md` (RF-03, RF-10, "Contratos e interfaces")
- `docs/estudos/painel-local/prototipo.md` (telas e retorno do usuário)
- `/home/victor/projetos/ai-hub-painel/apps/painel/src/features/settings/settings-page.tsx`
- `/home/victor/projetos/ai-hub-painel/apps/painel/src/features/settings/settings-page.module.css`
- `/home/victor/projetos/ai-hub-painel/apps/painel/src/features/settings/schema.ts`
- `/home/victor/projetos/ai-hub-painel/apps/painel/src/features/settings/yaml.ts`
- `/home/victor/projetos/ai-hub-painel/apps/painel/src/features/settings/fields.tsx`
- `/home/victor/projetos/ai-hub-painel/apps/painel/src/features/settings/fields.module.css`
- `apps/painel/src/shared/` (api.ts, attention.ts, action-bar.tsx e edit-dialog.tsx, da TASK-03)
- `apps/painel/src/ui/` (componentes do Atria)

### Mudar

- `apps/painel/src/features/settings/settings-page.tsx` (alterar): opções de `vostok/config/` por seção, com nomes da interface, controles por tipo, cartão "Avançado" e barra de salvar ligada.
- `apps/painel/src/features/settings/schema.ts` (alterar): nome da interface e controle de cada chave.
- `apps/painel/src/features/settings/yaml.ts` (alterar): leitura e escrita do YAML preservando comentários.
- `apps/painel/src/features/settings/fields.tsx` (alterar): controles por tipo.
- `apps/painel/src/features/settings/fields.module.css` (alterar): estilos dos controles.
- `apps/painel/src/features/settings/settings-page.module.css` (alterar): estilos da página.

### Passos

1. Escreva primeiro os testes com Vitest e Testing Library (o `yaml.ts` lê e reescreve um YAML com blocos, listas e comentários sem perder o que não mudou; o `schema.ts` dá nome da interface a cada chave mapeada; uma chave sem mapeamento vai para "Avançado"; "Salvar" chama `PUT /api/configuracao` com o hash e mostra o 409 com os dois textos). Use `fetch` simulado com os JSON do contrato da spec. Confira: `pnpm --dir apps/painel exec vitest run src/features/settings` falha.
2. Troque as páginas provisórias da TASK-03 pela implementação do spike (`features/settings/settings-page.tsx`, `features/settings/settings-page.module.css`, `features/settings/schema.ts`, `features/settings/yaml.ts`, `features/settings/fields.tsx`, `features/settings/fields.module.css`). Importe ações, diálogos e pendências só de `shared/` e componentes só de `ui/`, nunca de outra feature. Mantenha os estados de carregamento, vazio e erro e o foco visível (RNF-05).
3. `apps/painel/src/features/settings/settings-page.tsx` (alterar): ligue o botão "Salvar". Ele chama `act('PUT', '/api/configuracao?caminho=<arquivo>&hash=<hash lido>', <yaml reescrito>)` de `shared/api`. Num 409, mostre "Seu texto" e "Versão no disco" com o `shared/edit-dialog`. Os problemas devolvidos aparecem na barra de salvar.
4. Confira: `pnpm --dir apps/painel exec tsc --noEmit`, os testes e `pnpm --dir apps/painel build` passam; tire os prints e confira com o painel do spike que mostram dados reais.
