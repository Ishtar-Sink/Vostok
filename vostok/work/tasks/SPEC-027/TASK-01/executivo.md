<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

Levanta, só pela leitura do projeto, o que o usuário faz hoje pelo chat e como cada ação seria feita pelo painel, como a aprovação pelo painel pode valer sem enfraquecer as regras e o que muda para tirar o servidor da premissa do produto.
Perfil julgamento: exige decidir entre alternativas de aprovação. Risco: recomendar uma aprovação que o agente consiga forjar.
Roda em paralelo com a medição do binário e alimenta os testes de aprovação (TASK-04) e o estudo final (TASK-05).

## O que foi feito

- Foram listadas as 35 ações que o usuário dispara hoje pelo chat. Destas, 14 podem virar ação direta no painel, 17 precisam que o agente escreva ou decida algo, e 4 só fazem sentido no chat ou no terminal.
- Para aprovar pelo painel sem deixar o agente aprovar no lugar do usuário, a recomendação é que o clique grave a aprovação com a origem "painel" e que um registro de aval, que só recebe acréscimos, seja conferido antes de concluir tarefas, registrar alterações no histórico do projeto e aprovar. A alternativa de deixar o agente processar um pedido de aprovação foi descartada.
- Continua aberto um risco: um agente que rode com o mesmo usuário do computador ainda pode forjar esse aval. Esse ponto será tratado na especificação da trava de aval.
- Nenhuma regra, memória ou pasta foi alterada. O levantamento só indica o que deve mudar para tirar a premissa de servidor do produto, e essa mudança será feita em outra especificação.
