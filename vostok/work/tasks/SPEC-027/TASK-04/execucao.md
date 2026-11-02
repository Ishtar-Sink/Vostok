### Ler

- `vostok/work/specs/refined/SPEC-027-estudo-do-painel-local-do-vostok-sem-servidor.md`
- `docs/estudos/painel-local/levantamento.md` (inventário e aprovação recomendada, da TASK-01)
- `docs/estudos/painel-local/prototipo.md` (da TASK-03)
- `/home/victor/projetos/ai-hub-painel/apps/cli/src/commands/painel.rs`
- `apps/cli/src/commands/task.rs` (fn run_approve, fn approval_problems, fn approve; `adjust record-manual`)
- `apps/cli/src/commands/spec.rs` (`adjust record-manual`, `approve validate`)
- `vostok/rules/agent/` (regras de aprovação)

### Mudar

- `docs/estudos/painel-local/testes.md` (criar): resultados dos testes de aprovação, edição, edição simultânea e loop pelo protótipo (PQ-06, PQ-07, PQ-08).

### Passos

1. Prepare um projeto de teste, para não mexer nas tarefas reais: `T=/home/victor/projetos/ai-hub-painel-teste; rm -rf "$T"; mkdir -p "$T"; cp -r /home/victor/projetos/ai-hub/vostok "$T/"; git -C "$T" init -q; git -C "$T" add -A; git -C "$T" -c user.name=v -c user.email=v@v commit -qm base`. Na cópia, deixe uma tarefa pendente para aprovar (por exemplo, volte o `status` de `vostok/work/tasks/SPEC-027/TASK-05/estado.yml` para `pending`, se ela já tiver sido aprovada).
2. Escolha das alternativas, já desenhadas na TASK-01: implemente no spike a aprovação recomendada em `levantamento.md` (`## Aprovação pelo painel`). A aprovação do painel usa a mesma lógica de `vostok task approve approve` e grava a origem, conforme a recomendação. Edição: `PUT /api/arquivo?caminho=` grava um `.md` de `vostok/work/` só se o hash enviado for igual ao do arquivo no disco; senão, responde 409 com o conteúdo atual (controle otimista). Depois de gravar, o painel chama o `adjust record-manual` correspondente (spec, plano ou tarefa).
3. Teste de aprovação (PQ-06): aprove a tarefa pendente pelo painel aberto em `"$T"` e rode `vostok task approve validate` e `vostok task status SPEC-027` na cópia. Registre se a aprovação passa nos comandos e como a origem ficou gravada.
4. Teste de edição (PQ-01, PQ-07): edite pelo painel um critério de `criterios.md` da tarefa pendente e confira que `vostok task adjust record-manual SPEC-027 TASK-05` registra a edição e que `vostok task adjust validate SPEC-027 TASK-05` passa.
5. Teste de edição simultânea (PQ-07): abra o mesmo arquivo no painel, altere-o no disco por fora (como faria o agente) e salve pelo painel. O esperado é o 409 com o conteúdo novo, sem perda. Registre também o caso inverso: o agente edita depois do painel e o evento `GET /api/eventos` atualiza a tela.
6. Pedido ao agente e loop (PQ-08): defina o formato do pedido gravado pelo painel numa pasta `vostok/work/pedidos/` (um arquivo por pedido, com a ação, os argumentos, a data e o estado `pendente`, `em_andamento`, `feito` ou `recusado`). Implemente `POST /api/pedidos` no spike. Num chat do Claude Code aberto em `"$T"`, rode `/loop 1m` com a instrução de ler `vostok/work/pedidos/`, executar o pedido pendente pela skill correspondente e marcar o estado. Grave pelo painel um pedido simples (por exemplo, "ajustar a TASK-05: acrescentar um critério") e registre o tempo até o estado `feito`. **PARE** antes do teste e peça ao usuário para abrir o chat em loop, com o template `question`.
7. Para o Codex, registre, sem testar, o que muda: como ele vigiaria a pasta e o que não tem equivalente.
8. `docs/estudos/painel-local/testes.md` (criar, no repositório principal): título `# Testes do painel local`, data, commit da worktree, uma seção por teste (passos 3 a 7) com o comando, o resultado e a conclusão, e o formato do pedido em bloco de código. Confira: `grep -n '^## ' docs/estudos/painel-local/testes.md`.
9. Apague o projeto de teste: `rm -rf /home/victor/projetos/ai-hub-painel-teste`.
