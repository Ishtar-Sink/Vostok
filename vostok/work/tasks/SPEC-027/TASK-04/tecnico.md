<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Os testes rodam numa cópia do projeto em `/home/victor/projetos/ai-hub-painel-teste`, com o binário do spike, para não alterar as tarefas reais. O spike ganha a aprovação recomendada pela TASK-01, a gravação de `.md` com controle otimista por hash (409 com o conteúdo atual) seguida do `adjust record-manual`, e a fila de pedidos em `vostok/work/pedidos/`. O resultado vai para `docs/estudos/painel-local/testes.md`. Cobre PQ-06, PQ-07 e PQ-08; sustenta CA-06, CA-09 e CA-10.

**Decisões.** Controle otimista por hash em vez de trava de arquivo: o agente não respeitaria uma trava do painel, mas o painel sempre confere o hash. A edição pelo painel é registrada como edição manual, como o harness já faz com edições à mão. O pedido ao agente é um arquivo por pedido, independente da ferramenta; o loop é testado no Claude Code e só descrito para o Codex (inferência E da spec).

**Riscos.** O teste do loop exige um chat aberto pelo usuário: a tarefa para e pede isso. A evidência do loop é manual.

**Arquivos em comum na onda.** Nenhum: é a única tarefa da onda 3.
