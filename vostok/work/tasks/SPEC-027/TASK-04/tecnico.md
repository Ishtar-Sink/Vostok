<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Os testes rodam numa cópia do projeto em `/home/victor/projetos/ai-hub-painel-teste`, com o binário do spike, para não alterar as tarefas reais. O spike ganha a aprovação recomendada pela TASK-01, a gravação de `.md` com controle otimista por hash (409 com o conteúdo atual) seguida do `adjust record-manual`, e a fila de pedidos em `vostok/work/pedidos/`. O resultado vai para `docs/estudos/painel-local/testes.md`. Cobre PQ-06, PQ-07 e PQ-08; sustenta CA-06, CA-09 e CA-10.

**Decisões.** Controle otimista por hash em vez de trava de arquivo: o agente não respeitaria uma trava do painel, mas o painel sempre confere o hash. A edição pelo painel é registrada como edição manual, como o harness já faz com edições à mão. O pedido ao agente é um arquivo por pedido, independente da ferramenta; o loop é testado no Claude Code e só descrito para o Codex (inferência E da spec).

**Riscos.** O teste do loop exige um chat aberto pelo usuário: a tarefa para e pede isso. A evidência do loop é manual.

**Arquivos em comum na onda.** Nenhum: é a única tarefa da onda 3.

## O que foi feito

**Spike** (`/home/victor/projetos/ai-hub-painel`, sem commit).
- `task.rs`:
  - `Origin` (`Chat` ou `Panel`, nunca vindo da linha de comando) e `APPROVED_BY_FIELD`;
  - `approve(&files, origin)` grava `approved_by`;
  - `approve_from_panel` e `record_manual_at`, que devolve os problemas do `adjust validate`;
  - `drop_approval` também limpa o `approved_by`.
- `spec.rs`: `approve_from_panel` e `record_manual_at`.
- `plan.rs`: `record_manual_at`.
- `painel.rs`:
  - `act` (POST e PUT, só com o `X-Painel-Token`, um token de 16 bytes aleatórios colocado no `index.html`);
  - `PUT /api/arquivo?caminho=&hash=`, com controle otimista por SHA-256 e resposta 409 com o conteúdo e o hash atuais;
  - `GET /api/fonte`;
  - `POST /api/aprovacoes?spec=&tarefas=`, que aprova e acrescenta o aval a `work/avais.log`;
  - `POST /api/pedidos?acao=&spec=&tarefa=`, que grava `work/pedidos/PED-NNN.md`.
- Testes: 10 unitários no `painel.rs`.
- Frontend:
  - `shared/api.ts` (`act`) e `shared/edit-dialog.tsx`, com o conflito lado a lado e os problemas da validação;
  - botões "Editar" na tarefa e na spec;
  - `ActionDialog` com aprovação direta e pedidos para as demais ações.

**Repositório principal.** `docs/estudos/painel-local/testes.md` (novo): aprovação, edição, edição simultânea, pedido e loop, e Codex.

**Como foi testado.**
- Numa cópia descartável do projeto, com git próprio, apagada no fim.
- O painel rodou na porta 4710, e as ações foram feitas pela interface com cliques automáticos (Chrome headless).
- As validações usaram o `vostok` do spike e o 0.2.10 instalado.

**Resultados.**
- **Aprovação:** a TASK-05 ficou `approved` com `approved_by: painel`. As duas versões validam, e uma ação sem token é recusada (403).
- **Edição dos critérios:** o ajuste manual foi registrado e passa no `adjust validate`.
- **Conflito:** o painel recebeu 409, e nada se perdeu.
- **Aviso ao vivo:** latência média de 338 ms.
- **Loop:** o `PED-001` foi `recusado` em 1 min 30 s, por critérios sem comprovação deixados pelo próprio teste. O `PED-002` ficou `feito` em 1 min 18 s, com ajuste 5 de origem `agente`.

**Decisões.**
- **Aprovação:** direta no mesmo processo, nunca por pedido, conforme a recomendação da TASK-01.
- **Concorrência:** controle otimista por hash no lugar de trava de arquivo.
- **Pedidos:** um arquivo por pedido, com estado no front-matter.
- **Respostas ao agente:** vão como pedido `responder-pergunta`.

**Pontos de atenção.**
- A CLI passa a gravar `approved_by: chat`, o que exige atualizar 15 casos dos testes golden (2 testes). Fora isso, 517 testes passam.
- Com `--features painel-tiny`, os testes de integração não compilam: o crate `ascii` do `tiny_http` torna ambígua a comparação de `u8` em `tests/common/golden.rs:507`.
- O aval ainda não é conferido pela CLI. A conferência é a trava de aval, que pede spec própria.
- Pedidos não carregam aval.
- O loop do Claude Code virou um job agendado de 1 min ligado à sessão. O tempo é dominado por esse intervalo.

**Verificação.** Testes, regressão (1 comando igual ao baseline) e evidências passaram. As 2 evidências (`testes.diff` e `pedido-loop.txt`, anexada pelo usuário) estão em `TASK-04/evidencias/`.
