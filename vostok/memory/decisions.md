---
type: decision
title: "Decisões"
description: "Decisões estruturantes do projeto e o motivo de cada uma. Consulte antes de decidir algo que afete arquitetura, stack, dados, contratos ou convenções, e quando houver ambiguidade."
updated_at: 2026-11-12T13:40:21Z
---

# Decisões

Decisões estruturantes do projeto e o motivo de cada uma. Consulte antes de decidir algo que afete arquitetura, stack, dados, contratos ou convenções, e quando houver ambiguidade.

Entradas criadas por `add-memory.sh` e consultadas por `search-memory.sh`. Não carregue este arquivo inteiro.

## DEC-001: Nome do produto e da CLI: Vostok

- **Data:** 2026-09-28
- **Status:** ativa
- **Tags:** nome, produto, cli, ishtar-sink, convenção
- **Origem:** Conversa com o Victor em 2026-09-28; proposta confirmada com "pode registrar".
- **Contexto:** O projeto, até então chamado ai-hub, faz parte do ecossistema Ishtar Sink e precisava de um nome simples, fácil de lembrar e natural para brasileiros, que também servisse de nome para a CLI.
- **Decisão:** O produto se chama Vostok e a CLI é `vostok`. Vostok foi a nave do primeiro voo tripulado, e o nome significa "oriente": o harness orienta cada agente, em qualquer ferramenta, pelo mesmo caminho de spec, aprovação, plano e tarefas.
- **Alternativas descartadas:** Korolev e Baikonur (soviéticos); Ikora, Zavala, Warmind e Rasputin (Destiny); Sputnik, Soyuz e Mir; Houston e Rover (americanos); Orrery, Spica, Órbita, Barycenter e Ephemeris (astronomia).
- **Consequências:** O nome vale para repositório, CLI e pacotes. O npm `vostok` é um pacote ativo, então o pacote usa uma variação (`vostok-cli` ou `@ishtar-sink/vostok`). Por vir da exploração espacial e não ser um corpo celeste, o nome precisa ser registrado em `Ishtar-Sink/.github` como leitura mais ampla do tema de astronomia.
- **Substitui:** Nenhuma

## DEC-002: Linguagem do harness: Rust, com Node quando fizer sentido

- **Data:** 2026-10-06
- **Status:** ativa
- **Tags:** linguagem, rust, node, bash, harness, stack, migração
- **Origem:** Conversa com o Victor em 2026-10-06, no refinamento da SPEC-003; proposta confirmada com "ok".
- **Contexto:** O harness é escrito em bash 3.2 com awk e sed do BSD, por portabilidade. Os scripts novos, como os da finalização em HTML na SPEC-003, ficam mais difíceis de manter nessas ferramentas.
- **Decisão:** A linguagem do harness será Rust. O Node é usado nos casos em que fizer sentido. A migração será feita numa spec própria; até lá, os scripts novos continuam em bash, inclusive os da SPEC-003.
- **Alternativas descartadas:** Continuar em bash indefinidamente; usar Node como linguagem principal.
- **Consequências:** Specs anteriores à migração mantêm o bash. A spec de migração precisa resolver a distribuição dos binários e o Rust na Mac de testes, que hoje não tem `cargo`. Templates e configurações (`.yml` e templates HTML com variáveis) devem continuar sem depender da linguagem dos scripts.
- **Substitui:** Nenhuma

## DEC-003: Painel local no lugar do servidor

- **Data:** 2026-11-02
- **Status:** substituída
- **Tags:** arquitetura, painel, servidor, mongodb, mcp, aval, pedidos
- **Origem:** Estudo da SPEC-027 (docs/estudos/painel-local.md); confirmado pelo Victor no refinamento da SPEC-028, em 2026-11-02.
- **Contexto:** A premissa final previa servidor HTTP self-hosted, MongoDB e MCP, que nunca saíram do papel. O estudo da SPEC-027 mostrou, com um protótipo usado pelo usuário, que um painel local sem servidor faz quase tudo o que o chat faz.
- **Decisão:** Sem servidor HTTP, MongoDB nem MCP. O vostok painel abre um painel local por pasta, só em 127.0.0.1, com os arquivos de vostok/ como banco; o painel grava só pela lógica da CLI, aprova com a origem e o aval em work/avais.log, e o que precisa do agente vira um pedido em work/pedidos/, processado por um chat em loop.
- **Alternativas descartadas:** Servidor HTTP self-hosted com MongoDB como único escritor e MCP para as ferramentas; pedido de aprovação processado pelo agente.
- **Consequências:** As regras de arquitetura e stack deixam de citar servidor, banco e MCP; as pastas apps/server, apps/mcp, apps/web e packages/contracts saem; o frontend fica em apps/painel, em Preact com compat; a trava de aval e o loop de pedidos vêm na SPEC-030.
- **Substitui:** Nenhuma

## DEC-004: Comunicação do painel por estado e avais por spec

- **Data:** 2026-11-03
- **Status:** substituída
- **Tags:** arquitetura, painel, pedidos, perguntas, aval, avais
- **Origem:** SPEC-033 (operar o projeto de ponta a ponta pelo painel); proposta confirmada pelo Victor com "ok, registrar" em 2026-11-03.
- **Contexto:** A DEC-003 definiu o painel local com os arquivos de `vostok/` como banco, o aval em `work/avais.log` e os pedidos em `work/pedidos/`. Na SPEC-033, os pedidos, as perguntas e os avais passaram a se acumular, e o agente precisou perguntar pelo painel.
- **Decisão:** Mantém o painel local da DEC-003: sem servidor HTTP, MongoDB nem MCP, só em 127.0.0.1, gravando só pela lógica da CLI. Pedidos ficam em subpastas por estado (pendente, em andamento, feito, recusado) e perguntas em aberta e respondida, com o front-matter como verdade e a pasta como apresentação. O aval fica num log por spec em `work/avais/<SPEC>.log`, só de acréscimo, com os sem spec à parte, consultado pelo `vostok avais`. As pastas e os nomes vêm de `config/main.yml`. Pedidos, perguntas e avais são arquivados com a spec.
- **Alternativas descartadas:** Manter o log único e as pastas soltas: crescem sem fim e fazem o agente ler o que já foi resolvido. Estado só na pasta: uma interrupção no meio do `rename` perderia o estado.
- **Consequências:** O formato antigo é migrado na primeira execução, sem apagar nada. A trava de aval lê o log da spec. A `process-requests` lê só `pendente/`. O comentário de `approval_lock` em `config/main.yml` e a regra `agent/boundaries-05` ainda citam `avais.log` e precisam ser atualizados.
- **Substitui:** DEC-003

## DEC-005: Painel somente leitura e aval no chat

- **Data:** 2026-11-08
- **Status:** substituída
- **Tags:** arquitetura, painel, somente-leitura, aval, chat, pedidos
- **Origem:** SPEC-042 (painel somente leitura a partir da versão pré painel), resposta 1A do refinamento; proposta confirmada pelo Victor com "ok" em 2026-11-08.
- **Contexto:** Com o painel interativo da DEC-004, o consumo de tokens subiu, sobretudo pelo loop da process-requests: cerca de 24% de uma sessão de ~/rd/app e cerca de 93% de um dia do ai-hub. A SPEC-042 reconstruiu a main a partir da v0.2.10 e lançou a 0.5.0.
- **Decisão:** O painel local continua sem servidor HTTP, MongoDB nem MCP e só em 127.0.0.1, mas é somente leitura: responde só às rotas GET de leitura e ao WebSocket /api/ao-vivo, com 405 para qualquer outro método e sem token de sessão. O trabalho é feito só pelo chat, e o aval é o pedido explícito do usuário no chat, como na v0.2.10: sem pedidos, perguntas, avais em arquivo, approval_lock nem loop de pedidos.
- **Alternativas descartadas:** Manter a DEC-004 parcial, com o painel interativo e o loop sob demanda: o custo de tokens continuaria. Aval com origem chat gravado em arquivo: mantém a trava e os arquivos sem ganho, porque o pedido no chat já é o aval.
- **Consequências:** Saem os comandos vostok pedidos, vostok perguntas e vostok avais, o módulo de aval, as skills process-requests e ask-agent, os ramos por pedido das skills e as chaves da comunicação. O vostok update da 0.5.0 remove de uma instalação da 0.4.0 as skills, as chaves e as pastas de pedidos, perguntas e avais, listando cada uma. O painel mostra specs, tarefas, ondas, tarefas que faltam e documentos, e se atualiza sozinho.
- **Substitui:** DEC-004

## DEC-006: Painel com escrita só na conferência de evidências

- **Data:** 2026-11-12
- **Status:** ativa
- **Tags:** arquitetura, painel, aval, conferência, evidências, somente-leitura
- **Origem:** Estudo de viabilidade da conferência de evidências pelo painel, em 2026-11-12; proposta confirmada pelo Victor com "pode registrar a decisão".
- **Contexto:** A DEC-005 deixou o painel somente leitura porque o loop da process-requests consumia muitos tokens. O usuário quer conferir, recusar e retirar a conferência de evidências pelo painel, continuando a conduzir o trabalho pelo chat. Essas ações não precisam do agente: são as mesmas operações do `vostok task evidence --reviewed`, `--rejected` e `--undo`.
- **Decisão:** O painel continua local, só em 127.0.0.1, sem servidor externo, banco nem MCP, e sem agente. Ele aceita escrita só para conferir (com observação opcional), recusar (com motivo) e retirar a conferência de evidências, pela mesma lógica da CLI, com token de sessão e conferência de `Host` e `Origin`. Todas as demais rotas continuam somente leitura, com 405. Não há pedidos em arquivo, perguntas nem loop: o trabalho e os demais avais continuam no chat.
- **Alternativas descartadas:** Manter a DEC-005: a conferência continuaria só pelo chat, sem ganho de tokens, porque a escrita no painel não passa pelo agente. Gravar a conferência no front-matter da evidência: prints e vídeos não têm onde guardar e a escrita mudaria o sha256 que valida a conferência. Voltar aos pedidos em arquivo da DEC-004: traria de volta o custo do loop.
- **Consequências:** O servidor do painel ganha rotas POST restritas às três ações, com token e conferência de origem; a lógica de conferência da CLI vira função chamada pelo comando e pelo painel; as skills `execute-task` e `finish-task` passam a considerar as conferências feitas no painel antes de perguntar. Uma recusa pelo painel devolve a tarefa para execução na hora, e o agente vê isso no próximo comando.
- **Substitui:** DEC-005
