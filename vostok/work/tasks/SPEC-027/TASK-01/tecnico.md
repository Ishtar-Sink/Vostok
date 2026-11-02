<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Tarefa documental, sem código: grava `docs/estudos/painel-local/levantamento.md`, que a TASK-05 consolida no estudo. Cobre PQ-05, PQ-06 e PQ-11 e sustenta CA-08, CA-11 e parte de CA-12.

**Decisões.** O inventário classifica as ações em três formas (CLI, pedido ao agente, fora do painel), conforme a inferência C da spec. A aprovação compara três alternativas (aprovação direta pela CLI com origem, pedido ao agente, aval em arquivo para a trava de aval) pelos critérios: gesto do usuário, impossível de forjar pelo agente, independente da ferramenta, custo. A remoção do servidor só é recomendada (decisão 1 da spec): nenhuma regra, memória ou pasta é alterada.

**Riscos.** Uma aprovação pelo painel que o agente consiga gravar sozinho anula as regras de `rules/agent/`; por isso o critério "não forjável" vem antes do custo.

**Arquivos em comum na onda.** Nenhum: a TASK-02 só grava `docs/estudos/painel-local/medicoes.md` e a worktree do spike.
