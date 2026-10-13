---
type: decision
title: "Decisões"
description: "Decisões estruturantes do projeto e o motivo de cada uma. Consulte antes de decidir algo que afete arquitetura, stack, dados, contratos ou convenções, e quando houver ambiguidade."
updated_at: 2026-10-06T10:19:43Z
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
