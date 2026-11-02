### Ler

- `vostok/work/specs/refined/SPEC-027-estudo-do-painel-local-do-vostok-sem-servidor.md`
- `docs/estudos/painel-local/medicoes.md` (variante recomendada pela TASK-02)
- `/home/victor/projetos/ai-hub-painel/apps/painel/` e `/home/victor/projetos/ai-hub-painel/apps/cli/src/commands/painel.rs` (spike da TASK-02)
- `/home/victor/projetos/ishtar-sink/atria/packages/ui/src/` (`tabs`, `empty-state`, `loading-state`, `error-state`, `toolbar`, `progress-bar`, `brand-logo`, `icons`)
- `/home/victor/projetos/ishtar-sink/atria/packages/theme/src/`
- `vostok/config/documents.yml` (grupo `brand` e tokens `light`/`dark`)
- `apps/cli/src/commands/documents.rs` (derivação das cores da marca)
- `apps/cli/src/commands/task.rs` (fn de `task status`, formato de `estado.yml`)
- `vostok/work/tasks/SPEC-027/TASK-03/plano-exemplo.md` (plano de exemplo para a tela de plano)

### Mudar

- `docs/estudos/painel-local/prototipo.md` (criar): as telas, as decisões de visual e de atualização ao vivo e como reproduzir o protótipo (PQ-01, PQ-09, PQ-10, PQ-12).

### Passos

1. Na worktree `/home/victor/projetos/ai-hub-painel`, use a variante de servidor e de empacotamento recomendada em `docs/estudos/painel-local/medicoes.md`. Copie do Atria para `/home/victor/projetos/ai-hub-painel/apps/painel/src/ui/` os componentes listados em "Ler" que as telas usarem, sem os testes.
2. `/home/victor/projetos/ai-hub-painel/apps/cli/src/commands/painel.rs` (alterar, na worktree): acrescente a API de leitura, em JSON: `GET /api/visao` (specs por status, tarefas por status com `vostok task status`, planos abertos), `GET /api/specs/<ID>`, `GET /api/planos/<ID>`, `GET /api/tarefas/<SPEC>/<TASK>` (arquivos da pasta da tarefa e a lista de evidências) e `GET /api/arquivo?caminho=` só para arquivos dentro de `vostok/` (recuse `..` e caminhos absolutos). Acrescente `--pasta <dir>` (padrão: a pasta atual) e `GET /api/marca`, com o grupo `brand` e as cores de `light` e `dark` de `vostok/config/documents.yml`.
3. Atualização ao vivo (PQ-12): `GET /api/eventos` (Server-Sent Events) avisa quando um arquivo de `vostok/work/` muda. Implemente por consulta periódica da data de modificação (1 s) e meça, no mesmo spike, a alternativa com a crate `notify`: o tamanho a mais no binário (com o `medir.sh` da TASK-02) e o atraso até o aviso. Registre os dois números.
4. `/home/victor/projetos/ai-hub-painel/apps/painel/src/` (alterar): telas com os componentes do Atria e as rotas `/` (visão geral), `/specs/<ID>`, `/planos/<ID>`, `/tarefas/<SPEC>/<TASK>` (com `execucao`, `verificacao`, `criterios` e as evidências: texto e imagem) e o estado vazio. Os `.md` são exibidos como HTML, com tabelas. Marca (PQ-10): mapeie os tokens de cor de `@atria/theme` para as cores de `/api/marca` (cor principal, fundo, texto, linhas, estados) nos temas claro e escuro, e mostre o nome e o ícone da marca no topo; sem marca, use os tokens padrão do `documents.yml`. Siga a persona ui-ux-designer: contraste de 4,5:1, foco visível, estados de carregamento, vazio e erro, e layout que funciona em 1280 px e em 400 px.
5. Plano de exemplo: as specs da pasta não têm plano aberto. Para a tela de plano, crie uma cópia temporária do projeto (`T="$(mktemp -d)"; cp -r vostok "$T/"`), copie `vostok/work/tasks/SPEC-027/TASK-03/plano-exemplo.md` para `"$T/vostok/work/plans/SPEC-027.md"` e abra o painel com `--pasta "$T"`.
6. Gere o binário de release do spike com a variante escolhida, em `/home/victor/projetos/ai-hub-painel/apps/cli/target/release/vostok`. Abra dois painéis ao mesmo tempo, um em `/home/victor/projetos/ai-hub` e outro em `/home/victor/projetos/ishtar-sink/atria`, e confira que cada um imprime uma porta diferente, que `ss -ltnp | grep vostok` mostra só `127.0.0.1` e que cada um mostra a marca da própria pasta (PQ-09).
7. Abra o painel do ai-hub no navegador e **PARE**: mostre ao usuário a URL e as telas e peça para ele usar o protótipo, com o template `question`, perguntando se as telas, o visual e a atualização ao vivo estão bons ou o que muda. Aplique os ajustes pedidos antes de seguir.
8. `docs/estudos/painel-local/prototipo.md` (criar, no repositório principal): título `# Protótipo do painel local`, data, commit da worktree, a variante usada, as telas com as rotas, o mapeamento dos tokens do Atria para a marca, a comparação da atualização ao vivo (consulta periódica contra `notify`, com os números), o resultado dos dois painéis e o retorno do usuário no passo 7. Confira: `grep -n '^## ' docs/estudos/painel-local/prototipo.md`.
