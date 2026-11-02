<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** A tarefa criou `apps/cli/tests/painel.rs`, com os testes de ponta a ponta, e alterou a release para gerar o bundle, compilar com a feature `painel` e conferir o tamanho. Cobre RF-01, RF-13, CT-01 e CT-15.

**O que mudou.**

- `apps/cli/tests/painel.rs`, com dois testes. O arquivo inteiro exige a feature `painel`.
  - `ct01_two_panels_run_together_each_with_its_own_brand` sobe dois painéis com `--porta 0`, um deles com outra cor primária em `documents.yml`. Confere que as portas são diferentes, que os dois escutam em `127.0.0.1` e que cada `/api/marca` traz a sua cor.
  - `approve_edit_request_and_live_update_end_to_end` só roda com `cfg(painel_bundle)`. O passo a passo:
    - lê o token da página;
    - abre o WebSocket e recebe o `conectado`;
    - aprova a tarefa e confere o `approved_by: painel` e a linha do `avais.log`;
    - edita os critérios pelo PUT com o hash da fonte e roda o `task adjust validate`;
    - grava o `PED-001.md`;
    - espera o `mudou` por até 5 s.
  - As requisições são HTTP/1.1 cruas, sem dependência nova.
- `.github/workflows/release.yml` ganhou quatro passos:
  - `pnpm/action-setup` (pnpm 11);
  - `actions/setup-node` (Node 24, com cache do `pnpm-lock.yaml`);
  - "Build panel" (`install --frozen-lockfile` e `build`);
  - "Check size", depois do Package, que falha com os dois números se o binário passar de 6 000 000 bytes ou o tar.gz de 2 300 000 bytes.

  O `cargo build` passou a usar `--features painel`. O passo de teste da release continua sem a feature.

**Decisões.** A conferência do tamanho fica na release, que é onde o binário distribuído é gerado; uma verificação só local foi descartada porque não protege as plataformas da matriz. Os prints finais usam o binário novo, substituindo os da onda 2, que usaram o spike.

**Riscos.** Passar do limite: o estudo prevê embutir o bundle com gzip (cerca de 108 KB a menos) como saída, numa tarefa nova se for preciso. O build de release leva minutos: a evidência do tamanho tem tempo-limite de 600 s.

**Arquivos em comum na onda.** Nenhum: é a única tarefa da onda 3.

**Verificação.** Os testes de `--test painel`, a suíte com e sem a feature e o clippy com `-D warnings` passaram. A evidência `integracao-testes.txt` guardou só o primeiro comando da Verificação; as três execuções estão em `validation/tests.out`, sem `FAILED`. A regressão apontou só a mudança esperada no número de passos da release.

**Na execução.** O registro do ajuste manual (`record_manual`) imprime na saída padrão do painel a cada edição gravada. O teste lê essa saída até o fim, porque com o pipe fechado o `println!` derruba a resposta da gravação. Na linha de comando a saída é o terminal, e o efeito é só uma linha de log por edição. O tamanho medido localmente foi de 5 569 408 bytes no binário (4 617 096 sem a feature) e de 2 101 858 bytes no tar.gz, dentro dos limites.
