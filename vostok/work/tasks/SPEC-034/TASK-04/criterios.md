### Concluída quando

- [ ] Com a feature `painel`, um `dist` mais antigo que o código do painel e sem `pnpm`, o build termina com erro e a mensagem diz para rodar `pnpm --dir apps/painel build` (RF-08, CT-10, CA-06).

  > Como comprovar: em `build-painel-dist.txt`, ver `ok CT-10 dist velho sem pnpm` e, logo abaixo, a mensagem do build com `pnpm --dir apps/painel build`.

- [ ] Com o `dist` em dia, o build termina sem erro e sem precisar do `pnpm` (RF-08, CT-11).

  > Como comprovar: em `build-painel-dist.txt`, ver `ok CT-11 dist em dia`.

- [ ] Com o `dist` velho e o `pnpm` disponível, o build reconstrói o `dist` sozinho e termina sem erro, sem parar o ciclo de editar o painel e rodar os testes (RF-08, inferência H).

  > Como comprovar: em `build-painel-dist.txt`, ver `ok Reconstrução com pnpm`.

- [ ] Sem a feature `painel` nada muda, e sem o `dist` o build segue com o aviso de hoje (CB-05).

  > Como comprovar: em `build-painel-dist.txt`, ver `ok CB-05 sem a feature` e `ok CB-05 sem dist`.

- [ ] A verificação não altera o repositório e o resto do `vostok` continua compilando e passando (RF-08).

  > Como comprovar: o `git status --short apps/painel` da verificação sai vazio; o clippy sai sem avisos e os testes `base` e `painel_server` passam, sem regressão contra o baseline.
