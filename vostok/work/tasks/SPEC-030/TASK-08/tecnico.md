<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Cria a skill `process-requests` no pacote (`packages/harness/skills/default/`), com a linha do catálogo e a entrada do mapa de contexto. A instrução parte da usada no teste do estudo e troca a edição do arquivo pelos comandos `vostok pedidos`. Cobre RF-14 e CT-15; atende RN-03 e RN-04.

**Decisões.** A instrução vira skill (inferência F), e não um texto num documento, porque assim ela é carregada com o contexto e o `/loop` só chama `/process-requests`. O arquivo é escrito à mão no pacote, porque o `vostok skill create` deste repositório grava em `vostok/` e não em `packages/harness/`. O CT-15 é manual: depende de um chat real em loop.

**Riscos.** Os redirecionamentos para Claude Code, Codex e Cursor são gerados pelo `vostok update` nos projetos; confira-os no teste manual.

**Arquivos em comum na onda.** Nenhum.
