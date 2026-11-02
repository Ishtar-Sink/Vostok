<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Tarefa documental, sem código: grava `docs/estudos/painel-local/levantamento.md`, que a TASK-05 consolida no estudo. Cobre PQ-05, PQ-06 e PQ-11 e sustenta CA-08, CA-11 e parte de CA-12.

**Decisões.** O inventário classifica as ações em três formas (CLI, pedido ao agente, fora do painel), conforme a inferência C da spec. A aprovação compara três alternativas (aprovação direta pela CLI com origem, pedido ao agente, aval em arquivo para a trava de aval) pelos critérios: gesto do usuário, impossível de forjar pelo agente, independente da ferramenta, custo. A remoção do servidor só é recomendada (decisão 1 da spec): nenhuma regra, memória ou pasta é alterada.

**Riscos.** Uma aprovação pelo painel que o agente consiga gravar sozinho anula as regras de `rules/agent/`; por isso o critério "não forjável" vem antes do custo.

**Arquivos em comum na onda.** Nenhum: a TASK-02 só grava `docs/estudos/painel-local/medicoes.md` e a worktree do spike.

## O que foi feito

**Arquivo criado.** `docs/estudos/painel-local/levantamento.md` (novo, ainda sem rastreamento no git). Nenhum outro arquivo do repositório, regra, memória ou pasta foi alterado.

**Como.** Leitura apenas de `vostok/context/skill-catalog.md` (22 skills: 21 padrão e 1 customizada), `apps/cli/src/cli.rs` (14 grupos de comando), `vostok/rules/agent/`, dos comandos de aprovação em `apps/cli/src/commands/spec.rs`, `plan.rs`, `task.rs` (`run_approve`, `approval_problems`, `approve`) e `commit.rs`, de `vostok/rules/project/architecture.md` e `stack.md`, das memórias `vostok-arquitetura` e `spec-trava-aval`, e de buscas `git grep` por citações ao servidor em `docs/mapa-vostok.html`, `README.md` e nas regras versionadas.

**Inventário (PQ-05).** Tabela de 35 linhas: 14 em `CLI` (5 skills: `approve-spec`, `approve-commit`, `approve-task`, `create-spec`, `customize-documents`; e 9 comandos), 17 em `pedido ao agente` (16 skills e a parte de escrita do commit liberado) e 4 em `fora do painel` (`init-harness` e 3 comandos internos). Comandos usados só dentro de uma skill entram na linha da skill.

**Aprovação (PQ-06).** Compara três alternativas: (a) o painel chama as funções de aprovação da CLI e grava a origem; (b) o painel grava um pedido e o agente aprova; (c) o painel grava um aval que a trava confere antes de `finish`, commit e aprovações. Recomenda (a) com o aval de (c): `approved_by: painel` (a CLI grava `chat`), um registro de aval só de acréscimo em `vostok/work/` com ID, hash aprovado, data e origem, e a conferência desse aval nos comandos travados. Descarta (b), porque o agente ficaria entre o gesto do usuário e a aprovação. O endpoint do painel escutaria só em `127.0.0.1` e exigiria um token de sessão entregue ao navegador na abertura.

**Remoção do servidor (PQ-11).** Tabela com uma linha por item: regras de `architecture.md` e `stack.md` que citam servidor, banco, MCP ou Node; a memória `vostok-arquitetura`; as pastas `apps/server`, `apps/mcp`, `apps/web` e `packages/contracts`; `docs/mapa-vostok.html`; e `README.md`. Só recomenda; nada foi alterado.

**Pontos de atenção.**
- Forja residual: um agente com shell e o mesmo usuário do sistema pode escrever na pasta. A alternativa (c) torna a forja visível e presa ao hash do conteúdo aprovado, mas não a impede. A trava fica para a spec própria, prevista na memória `spec-trava-aval`.
- Hoje `spec approve approve`, `task approve approve` e `plan approve mark-planned` gravam `status`, `approved_at` e `approved_hash` sem nenhum campo de origem; a CLI não distingue quem a chama.
- O `README.md` entrou na tabela de remoção (fase 2 nas linhas 14 e 26; pastas `mcp/` e `server/` na árvore, linhas 113 e 114). A tarefa não pedia isso.
- Em `architecture-02`, `-04`, `-07` e `stack-02` a recomendação é revisar, não remover.
- As pastas `apps/server`, `apps/mcp`, `apps/web` e `packages/contracts` estão vazias e fora do git.

**Verificação.** Testes, regressão e evidências passaram. A regressão registrou nenhum comando igual ao baseline e um com acréscimo; a evidência está em `TASK-01/evidencias/`.
