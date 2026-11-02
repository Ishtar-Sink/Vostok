<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Troca as páginas provisórias de `task` e `documents` pelas do spike, com o diálogo de edição de `shared/`. Cobre RF-03, RF-09 e CT-03.

**Decisões.** A ordem da tarefa (executivo, técnico, critérios e evidências antes de execução, verificação e registro) é a aprovada pelo usuário no protótipo. O "Editar" não aparece no registro, que o servidor também recusa (RN-03), nem numa tarefa concluída.

**Riscos.** Imagens de evidência grandes: são servidas por `/arquivos/...`, sem embutir no JSON.

**Arquivos em comum na onda.** Nenhum: cada tarefa de telas muda só as próprias pastas de feature, e as ações compartilhadas já estão em `shared/` (TASK-03).


## O que foi feito

**O que mudou:**
- `features/task/task-page.tsx` e `features/documents/documents-page.tsx`, com os CSS de cada um.
- As ações e a lista do que espera o usuário vêm de `shared/`.
- **Testes:** 8, que falharam antes do código.

**Ponto de atenção:**
- Numa rodada, o `tsc` acusou um erro em `board/tasks-page.tsx`, de outra tarefa em andamento. O erro sumiu na rodada seguinte.

**Verificação:**
- `tsc`, vitest e build passaram.
- A regressão não encontrou diferença.
- O teste gravou `task-09-testes.txt`, e os prints `tarefa.png` e `documentos.png` foram conferidos pelo usuário.
