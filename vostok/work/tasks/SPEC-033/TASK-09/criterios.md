### Concluída quando

- [ ] A lista de specs tem o botão "Nova spec", que pede o assunto e grava o pedido `criar-spec` com ele; sem assunto, o envio fica desligado (RF-05, CT-05).

  > Como comprovar: no print `nova-spec.png`, o botão no cabeçalho; em `botoes-spec-testes.txt`, `grava o pedido de nova spec com o assunto` e `mostra a recusa do pedido de nova spec` passam.

- [ ] A spec em rascunho tem o botão "Refinar", que grava o pedido `refinar-spec`, e as specs em outros status não têm (RF-05, CT-05).

  > Como comprovar: em `botoes-spec-testes.txt`, `pede o refinamento da spec em rascunho`, `não oferece refinar fora do rascunho` e `oferece refinar para a spec em rascunho` passam.

- [ ] A spec planejada tem o botão "Executar onda", que grava o pedido `executar-onda` com a menor onda que tem tarefa aprovada e pronta, e some quando nenhuma tarefa está pronta (RF-05, CT-06).

  > Como comprovar: no print `executar-onda.png`, o botão no cabeçalho da SPEC-033; em `botoes-spec-testes.txt`, `pede a execução da próxima onda`, `oferece executar a próxima onda executável` e `não oferece executar onda sem tarefa pronta` passam.

- [ ] Os botões novos e o campo do assunto são achados por papel e rótulo, e funcionam pelo teclado como os diálogos que já existem (CT-18, RNF-04).

  > Como comprovar: em `botoes-spec-testes.txt`, os testes usam `getByRole('button', { name })` e `getByRole('textbox', { name: 'Assunto da spec' })` e passam.
