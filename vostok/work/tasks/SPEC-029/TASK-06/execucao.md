### Ler

- `vostok/work/specs/refined/SPEC-029-painel-local-do-vostok.md` (RF-06, RF-09, RF-10, RF-11, RN-02 a RN-06, CB-02 a CB-04, FA-03, FA-04, "Contratos e interfaces")
- `apps/cli/src/painel/mod.rs` e `apps/cli/src/painel/actions.rs` (esqueleto da TASK-02)
- `apps/cli/src/commands/task.rs` (fn approve_from_panel, fn record_manual_at, da TASK-01)
- `apps/cli/src/commands/spec.rs` (fn approve_from_panel, fn record_manual_at)
- `apps/cli/src/commands/plan.rs` (fn record_manual_at)
- `/home/victor/projetos/ai-hub-painel/apps/cli/src/commands/painel.rs` (fn act, fn save, fn approve, fn record_aval, fn request, fn write_markdown, fn edited, fn request_markdown, fn next_request_id, fn is_id, e os testes de 1083 a 1180)
- `docs/estudos/painel-local/testes.md` (formato do pedido e resultados esperados)
- `vostok/rules/project/` e `vostok/config/` (o que a gravação de configuração pode tocar)
- `apps/cli/src/commands/rules.rs` (fn validate, fn map_problems, fn category_problems)
- `apps/cli/src/painel/server.rs` (fn dispatch, linha 91, e o teste action_with_token_reaches_the_actions)

### Mudar

- `apps/cli/src/painel/actions.rs` (alterar): `PUT /api/arquivo` (edição com hash e registro manual), `PUT /api/configuracao` (configuração com hash), `POST /api/aprovacoes` (aprovação com aval) e `POST /api/pedidos` (pedido ao agente).

- `apps/cli/src/commands/rules.rs` (alterar): expor `pub fn validate_at(project: &Path) -> Result<Vec<String>>`, com os mesmos problemas de `vostok rules validate`, sem encerrar o processo.
- `apps/cli/src/painel/server.rs` (alterar): encaminhar os GETs de `/arquivos/...` para `read::route` e ajustar o teste `action_with_token_reaches_the_actions` à ação real.

### Passos

1. `apps/cli/src/painel/actions.rs` (alterar): escreva primeiro os testes unitários, numa pasta temporária com uma spec e uma tarefa válidas:
   - PUT com o hash certo grava e devolve o hash novo e os problemas (CT-08);
   - PUT com hash velho devolve 409 com o conteúdo atual e não toca no arquivo (CT-09);
   - PUT em `registro.md`, `README.md`, `estado.yml` ou fora de `vostok/work/` devolve 403 (CT-10, CB-02);
   - corpo acima de 2 MB é recusado (CB-04);
   - aprovação de tarefa grava `approved_by: painel` e uma linha no `avais.log`, e uma tarefa inválida não grava aval (CT-04, FA-04);
   - pedido `ajustar-tarefa` gera o `PED-001.md` no formato do estudo, e uma ação desconhecida dá 400 (CT-12, CB-03);
   - configuração com hash certo grava em `vostok/config/`, e caminho fora de `vostok/config/` dá 403 (CT-11).
   Confira: `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::actions` falha.
2. Edição (RF-09): porte `save`, `write_markdown` e `edited` do spike. Aceite só `.md` sob `vostok/work/`. Recuse `registro.md`, `README.md` e `estado.yml` (RN-03). O hash é SHA-256 do conteúdo. Depois de gravar, chame o `record_manual_at` de tarefa, plano ou spec refinada e devolva `{"hash","mensagem","problemas"}`. O 409 devolve `{"hash","conteudo"}`, e gravar por cima só acontece com um novo PUT usando o hash atual (RN-06).
3. Configuração (RF-10): crie `PUT /api/configuracao?caminho=&hash=`, com o mesmo controle de hash, aceitando só `.yml` sob `vostok/config/`. Depois de gravar, rode a validação de regras do `vostok` (a mesma função de `vostok rules validate`) e devolva os problemas.
4. Aprovação (RF-06): porte `approve` e `record_aval`. `spec=` sem `tarefas=` aprova a spec por `spec::approve_from_panel`, e com `tarefas=` aprova as tarefas por `task::approve_from_panel`. Para cada item aprovado, acrescente ao `vostok/work/avais.log` a linha `<data UTC>\t<SPEC>[/<TASK>]\t<hash>\tpainel`, com o arquivo aberto em modo de acréscimo (RN-05). Se a aprovação falhar, responda 422 com a mensagem do `vostok` e não grave aval.
5. Pedidos (RF-11): porte `request`, `request_markdown`, `next_request_id` e `is_id`, com as 10 ações da spec. `aprovar-tarefa` e `aprovar-spec` dão 400 (RN-04). O arquivo segue o front-matter e as seções de `docs/estudos/painel-local/testes.md`.
6. `apps/cli/src/commands/rules.rs` (alterar): crie `pub fn validate_at(project: &Path) -> Result<Vec<String>>` que monta a lista com `map_problems` e `category_problems`, como `fn validate`, e faça `fn validate` usar essa função para imprimir e encerrar. Em `actions.rs`, depois de gravar a configuração, devolva os problemas de `rules::validate_at`. Teste: uma configuração que quebra o mapa de contexto devolve o problema. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test context_memory_rules` continua passando.
7. `apps/cli/src/painel/server.rs` (alterar): em `dispatch`, encaminhe também os GETs com `parts[0] == "arquivos"` para `read::route`, mantendo o resto do bundle. Troque o teste `action_with_token_reaches_the_actions` para conferir que um POST com token chega a `actions::act` (por exemplo, `POST /api/pedidos` sem `acao` responde 400, não 403). Acrescente o teste de que `GET /arquivos/<caminho de vostok/>` não cai no bundle.
8. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::actions` passa e `cargo clippy --manifest-path apps/cli/Cargo.toml --features painel --all-targets -- -D warnings` sem avisos.
