---
type: decision
title: "Decisões"
description: "Decisões estruturantes do projeto e o motivo de cada uma. Consulte antes de decidir algo que afete arquitetura, stack, dados, contratos ou convenções, e quando houver ambiguidade."
updated_at: 2026-11-02T13:19:32Z
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
- **Status:** ativa
- **Tags:** arquitetura, painel, servidor, mongodb, mcp, aval, pedidos
- **Origem:** Estudo da SPEC-027 (docs/estudos/painel-local.md); confirmado pelo Victor no refinamento da SPEC-028, em 2026-11-02.
- **Contexto:** A premissa final previa servidor HTTP self-hosted, MongoDB e MCP, que nunca saíram do papel. O estudo da SPEC-027 mostrou, com um protótipo usado pelo usuário, que um painel local sem servidor faz quase tudo o que o chat faz.
- **Decisão:** Sem servidor HTTP, MongoDB nem MCP. O vostok painel abre um painel local por pasta, só em 127.0.0.1, com os arquivos de vostok/ como banco; o painel grava só pela lógica da CLI, aprova com a origem e o aval em work/avais.log, e o que precisa do agente vira um pedido em work/pedidos/, processado por um chat em loop.
- **Alternativas descartadas:** Servidor HTTP self-hosted com MongoDB como único escritor e MCP para as ferramentas; pedido de aprovação processado pelo agente.
- **Consequências:** As regras de arquitetura e stack deixam de citar servidor, banco e MCP; as pastas apps/server, apps/mcp, apps/web e packages/contracts saem; o frontend fica em apps/painel, em Preact com compat; a trava de aval e o loop de pedidos vêm na SPEC-030.
- **Substitui:** Nenhuma
