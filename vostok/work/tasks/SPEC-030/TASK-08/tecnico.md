<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Cria a skill `process-requests` no pacote (`packages/harness/skills/default/`), com a linha do catálogo e a entrada do mapa de contexto. A instrução parte da usada no teste do estudo e troca a edição do arquivo pelos comandos `vostok pedidos`. Cobre RF-14 e CT-15; atende RN-03 e RN-04.

**Decisões.** A instrução vira skill (inferência F), e não um texto num documento, porque assim ela é carregada com o contexto e o `/loop` só chama `/process-requests`. O arquivo é escrito à mão no pacote, porque o `vostok skill create` deste repositório grava em `vostok/` e não em `packages/harness/`. O CT-15 é manual: depende de um chat real em loop.

**Riscos.** Os redirecionamentos para Claude Code, Codex e Cursor são gerados pelo `vostok update` nos projetos; confira-os no teste manual.

**Arquivos em comum na onda.** Nenhum.

**O que foi feito.**
- `packages/harness/skills/default/process-requests/SKILL.md` (versão 1.1.0) traz:
  - o mapa das 11 ações para as skills;
  - as regras de um pedido por vez, de nunca editar o pedido nem gravar aval, e de recusar com o motivo;
  - o resultado pela entrada padrão de `vostok pedidos concluir` e `recusar`.
- A linha da skill entrou no catálogo, e a entrada `process-requests` (com `config/tasks.yml`) no mapa de contexto.

**Ajuste 1 (pedido do usuário).** A skill passou a armar o loop sozinha. `/process-requests <intervalo>` chama a skill `loop` com `<intervalo> /process-requests` e faz a primeira rodada; sem intervalo, faz uma rodada. A regra `[process-requests-08]` impede que o loop se arme de novo. No Codex, a skill indica o `vostok pedidos vigiar`, que vem na TASK-09.

**Verificação.**
- O build e as suítes `init_update` (26) e `context_memory_rules` (42) passaram, e a regressão ficou igual ao baseline.
- Num projeto novo, o `context load --skill process-requests` sai com código 0, e o `vostok init` instala a skill, o redirecionamento e o catálogo.
- O `loop-pedido.txt`, anexado pelo usuário, mostra o loop armado por `/process-requests 1m` e o PED-002 (`ajustar-tarefa` na TASK-09) recusado com o motivo. A TASK-09 não mudou.

**Na execução.** O `skill-instalada.txt` mostra o contexto carregado, mas não o texto da skill, porque o `context load` não imprime o `SKILL.md`. A instalação foi conferida à parte, num projeto novo.
