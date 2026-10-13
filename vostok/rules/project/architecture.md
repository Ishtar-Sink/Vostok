# Arquitetura

Define como o código do projeto é organizado entre aplicações, pacotes e camadas.

[project/architecture-01]: Monorepo: aplicações em `apps/` (`cli`, `web`, `server`, `mcp`) e código compartilhado em `packages/` (`harness`, `contracts`).
[project/architecture-02]: Equilibre escalabilidade e complexidade: DDD leve, com o código organizado por domínio (spec, tarefa, memória, evento) e camadas só onde há regra de negócio.
[project/architecture-03]: Não crie abstração, camada ou padrão sem uma necessidade presente.
[project/architecture-04]: Frontend organizado por feature (componentes, hooks e chamadas de API juntos), sem camadas de domínio e infraestrutura separadas.
[project/architecture-05]: O servidor HTTP é o único que grava no banco; web e MCP só usam a API dele.
[project/architecture-06]: Tipos e contratos compartilhados entre web, server e MCP ficam em `packages/contracts`.
[project/architecture-07]: O harness é o produto distribuído: a CLI (`apps/cli`, em Rust) empacota no binário o conteúdo de `packages/harness` (skills, templates, regras e configurações padrão), o instala nos projetos com um comando de inicialização interativo e executa os comandos que hoje são scripts (ex.: criar specs).
[project/architecture-08]: Este projeto também usa o harness instalado pela CLI; `packages/harness` passa a ser só a fonte do que é empacotado.
