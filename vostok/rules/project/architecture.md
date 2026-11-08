# Arquitetura

Define como o código do projeto é organizado entre aplicações, pacotes e camadas.

[project/architecture-01]: Monorepo: a CLI em `apps/cli`, o frontend do painel local em `apps/painel` e a fonte do harness em `packages/harness`.
[project/architecture-02]: Equilibre escalabilidade e complexidade: DDD leve, com o código organizado por domínio (spec, tarefa, memória, aval) e camadas só onde há regra de negócio.
[project/architecture-03]: Não crie abstração, camada ou padrão sem uma necessidade presente.
[project/architecture-04]: Frontend organizado por feature (componentes, hooks e chamadas à API local do `vostok` juntos), sem camadas de domínio e infraestrutura separadas.
[project/architecture-05]: Os arquivos de `vostok/` são a fonte de verdade e o único armazenamento: o painel local grava só pelas funções e validações da CLI.
[project/architecture-06]: Removida em 2026-11-02.
[project/architecture-07]: O harness é o produto distribuído: a CLI (`apps/cli`, em Rust) empacota no binário o conteúdo de `packages/harness` e o bundle do painel local, instala o harness nos projetos com um comando de inicialização interativo, executa os comandos que hoje são scripts (ex.: criar specs) e abre o painel local por pasta com `vostok painel`.
[project/architecture-08]: Este projeto também usa o harness instalado pela CLI; `packages/harness` passa a ser só a fonte do que é empacotado.
