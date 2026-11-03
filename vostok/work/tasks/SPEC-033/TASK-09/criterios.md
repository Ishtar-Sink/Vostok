### Concluída quando

- [x] A lista de specs tem o botão "Nova spec", que pede o assunto e grava o pedido `criar-spec` com ele; sem assunto, o envio fica desligado (RF-05, CT-05).

  > Como comprovar: no print `nova-spec.png`, o botão no cabeçalho; em `botoes-spec-testes.txt`, `grava o pedido de nova spec com o assunto` e `mostra a recusa do pedido de nova spec` passam.

- [x] A spec em rascunho tem o botão "Refinar", que grava o pedido `refinar-spec`, e as specs em outros status não têm (RF-05, CT-05).

  > Como comprovar: em `botoes-spec-testes.txt`, `pede o refinamento da spec em rascunho`, `não oferece refinar fora do rascunho` e `oferece refinar para a spec em rascunho` passam.

- [x] A spec planejada tem o botão "Executar onda", que grava o pedido `executar-onda` com a menor onda que tem tarefa aprovada e pronta, e some quando nenhuma tarefa está pronta (RF-05, CT-06).

  > Como comprovar: em `botoes-spec-testes.txt`, `pede a execução da próxima onda`, `oferece executar a próxima onda executável` e `não oferece executar onda sem tarefa pronta` passam (a SPEC-033 real não tem tarefa pronta na finalização, então o botão não aparece num print).

- [x] Os botões novos e o campo do assunto são achados por papel e rótulo, e funcionam pelo teclado como os diálogos que já existem (CT-18, RNF-04).

  > Como comprovar: em `botoes-spec-testes.txt`, os testes usam `getByRole('button', { name })` e `getByRole('textbox', { name: 'Assunto da spec' })` e passam.
