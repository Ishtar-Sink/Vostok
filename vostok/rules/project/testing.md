# Testes

Define o que precisa de teste e como testar.

[project/testing-01]: Toda mudança de comportamento tem teste automatizado.
[project/testing-02]: O `cargo test` do `apps/cli` passa no Linux e no macOS.
[project/testing-03]: Ferramentas: `cargo test` no Rust e Vitest (com Testing Library na web) no TypeScript.
[project/testing-04]: Os testes não cobrem só os cenários felizes: os cenários de erro, borda e entrada inválida são exercitados principalmente.
[project/testing-05]: Print de tela, inclusive de evidência `(print)`, usa o Playwright por padrão: `apps/painel/scripts/print.sh <rota> <arquivo.png> [largura,altura]`, que chama `scripts/print.mjs` e aceita âncora na rota (`#evidencias`), `PAINEL_ROLAR` (pixels ou `fim`) e `PAINEL_STORAGE` (JSON do localStorage); não use o Chromium direto pela linha de comando.
[project/testing-06]: Evidência de tela que depende de interação ou animação (clique, arraste, zoom, reprodução, transição) é um vídeo `.webm` gravado com o Playwright, que mostra o comportamento do início ao fim, em vez de um print do estado final; o print fica para o que se confere parado (layout, largura, cores).
