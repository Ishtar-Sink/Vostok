<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

Cria o comando que abre o painel numa pasta e o servidor local que o atende, com a proteção que impede o agente de aprovar pelo painel sem o usuário.
Perfil julgamento: define a interface pública e cobre o risco alto da aprovação. Risco: o servidor ficar acessível fora da máquina.
Roda junto com a CLI e o frontend; desbloqueia a leitura, o ao vivo e as ações (onda 2).

## O que foi feito

- **Comando novo:** `vostok painel` abre o painel da pasta do projeto, num endereço que só a própria máquina acessa, numa porta livre.
- **Fora de um projeto:** o comando para e explica o que procurou.
- **Ações protegidas:** toda ação exige o código de sessão que só a página aberta pelo usuário recebe. Assim, o agente não consegue aprovar pelo painel chamando o endereço por fora.
- **Sem Node:** o programa continua compilando sem Node. Nesse caso, o comando explica como gerar a parte visual.
