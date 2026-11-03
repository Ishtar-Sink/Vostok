### Concluída quando

- [ ] Com o plano desatualizado, a spec não oferece "Aprovar plano" e a spec aprovada oferece "Refazer o plano", que grava o pedido `planejar-spec` (RF-02, RF-03, CT-02).

  > Como comprovar: em `acoes-plano-tarefa-testes.txt`, `troca aprovar por refazer com o plano desatualizado` e `pede refazer o plano desatualizado` passam; no print `plano-desatualizado.png`, a barra tem "Refazer o plano" e não tem "Aprovar plano".

- [ ] Pendências não lista "Aprovar plano" para um plano desatualizado e continua listando para o plano em dia (RF-02, CT-03).

  > Como comprovar: em `acoes-plano-tarefa-testes.txt`, `não lista aprovar plano com o plano desatualizado` passa.

- [ ] A página da spec mostra o aviso "Plano desatualizado", com o porquê (a spec mudou depois do plano), no cabeçalho, no documento e na visão do plano, em tom de pendência e não de erro (RF-02, CT-04).

  > Como comprovar: em `acoes-plano-tarefa-testes.txt`, `mostra o aviso de plano desatualizado na spec` e `mostra o aviso de plano desatualizado na visão do plano` passam; no print `plano-desatualizado.png`, o selo amarelo no cabeçalho e a linha "Plano: Desatualizado: a spec mudou depois do plano." na coluna lateral.

- [ ] A spec de volta a refinada com um plano antigo mostra só "Aprovar spec" e "Pedir ajuste", sem botão de plano (RF-02, CT-05, CB-01).

  > Como comprovar: em `acoes-plano-tarefa-testes.txt`, `não oferece ação de plano na spec refinada com plano desatualizado` passa.

- [ ] Com o plano em dia, a spec oferece "Aprovar plano" e, ao lado, "Ajustar plano", cujo envio fica desligado até haver texto e grava o pedido `ajustar-plano` com ele (RF-04, CT-06).

  > Como comprovar: em `acoes-plano-tarefa-testes.txt`, `oferece ajustar e aprovar com o plano em dia` e `pede o ajuste do plano em dia com texto` passam.

- [ ] A tarefa em validação tem o botão de finalizar ("Pedir finalização"), que grava o pedido `finalizar-tarefa`, mesmo com print ainda não conferido (RF-05, CT-07, CB-03).

  > Como comprovar: em `acoes-plano-tarefa-testes.txt`, `oferece finalizar na tarefa em validação` e `pede a finalização da tarefa em validação` passam; no print `tarefa-em-validacao.png`, o selo "validando" e o botão "Pedir finalização".

- [ ] A ajuda dos diálogos de aprovar spec e tarefa e a mensagem de sucesso da aprovação citam o log de avais da spec, e nenhum texto do painel cita `avais.log` (RF-07, CT-09).

  > Como comprovar: em `acoes-plano-tarefa-testes.txt`, `cita o log de avais da spec na ajuda das aprovações` e `cita o log de avais da spec ao aprovar` passam e o `grep` por `avais.log` em `apps/painel/src` não encontra nada.

- [ ] Os botões novos e o campo do ajuste são achados por papel e rótulo, e os testes que já existiam e a checagem de tipos continuam passando (RF-02, RF-04, RF-05).

  > Como comprovar: em `acoes-plano-tarefa-testes.txt`, os testes usam `getByRole('button', { name })` e `getByRole('textbox', { name: 'O que ajustar' })`, nenhum teste aparece com × e o `tsc` termina sem erros.
