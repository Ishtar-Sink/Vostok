<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

Agora o painel tem botões para começar e avançar uma spec sem abrir o chat. Na lista de specs, o botão "Nova spec" pede o assunto que você escreve, e o agente cria o rascunho. Na spec em rascunho, "Refinar" manda o agente detalhá-la. Na spec planejada, "Executar onda" pede a próxima leva de tarefas prontas, e a tela mostra quais são antes de você confirmar.

Cada botão grava um pedido que o agente atende, como já acontecia com as outras ações do painel. A entrega foi conferida por testes automatizados, pela verificação de regressão e pelas evidências. O print da tela "Nova spec" foi conferido e aceito pelo usuário. O botão "Executar onda" não tem print: na finalização, a SPEC-033 não tinha nenhuma tarefa aprovada e pronta, então o botão corretamente não aparecia. Por isso, o comportamento dele foi comprovado só pelos testes, com aprovação do usuário.
