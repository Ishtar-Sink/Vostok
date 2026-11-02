### Ler

- `vostok/work/specs/refined/SPEC-029-painel-local-do-vostok.md` (RF-02, RF-04, CB-05, CB-07, "Contratos e interfaces")
- `apps/cli/src/painel/mod.rs` e `apps/cli/src/painel/read.rs` (esqueleto da TASK-02)
- `/home/victor/projetos/ai-hub-painel/apps/cli/src/commands/painel.rs` (fn overview a fn brand, linhas 294 a 590, e as funções livres de 595 a 735)
- `vostok/config/documents.yml` (grupos `brand`, `light` e `dark`)
- `/home/victor/projetos/ai-hub-painel/apps/painel/src/shared/api.ts` (os tipos que o frontend espera)

### Mudar

- `apps/cli/src/painel/read.rs` (alterar): rotas de leitura `visao`, `versao`, `marca`, `documentos`, `configuracoes`, `specs/<ID>`, `planos/<ID>`, `tarefas/<SPEC>/<TASK>`, `fonte` e `arquivo`, e os arquivos de `/arquivos/...`.

### Passos

1. `apps/cli/src/painel/read.rs` (alterar): escreva primeiro os testes unitários, cada um com uma pasta temporária com `vostok/` mínimo:
   - `visao` lista as specs abertas e arquivadas, a atividade e as perguntas;
   - `specs/<ID>` devolve o conteúdo e os metadados;
   - `tarefas/<SPEC>/<TASK>` devolve as partes da tarefa e as evidências;
   - `marca` sem grupo `brand` devolve os tokens padrão (CB-05);
   - `fonte` devolve o conteúdo e o SHA-256;
   - `arquivo` fora de `vostok/` ou com `..` é recusado;
   - spec arquivada vem com `archived: true` (CB-07).
   Confira: `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::read` falha.
2. `apps/cli/src/painel/read.rs` (alterar): implemente `route` portando do spike `overview`, `specs`, `archived`, `task_summaries`, `activity`, `questions`, `settings`, `documents`, `spec`, `plan`, `task`, `file`, `brand`, `source` e as funções livres de leitura (front-matter e YAML em JSON, resumo de tarefa, grupos YAML, arquivos de evidência, front-matter e comentários HTML removidos). Mantenha os nomes dos campos do JSON do spike, que o frontend já usa. Leia os arquivos a cada requisição, sem cache (RN-02). `versao` usa `live::version`, sem implementá-la aqui.
3. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::read` passa e `cargo clippy --manifest-path apps/cli/Cargo.toml --features painel --all-targets -- -D warnings` sem avisos.
