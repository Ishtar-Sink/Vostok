<!-- Escrito na finalização da tarefa. -->

Corrige o defeito que apagava o resumo executivo e o detalhamento técnico de uma tarefa ao concluí-la e passa a exigir que esses documentos existam desde o planejamento e sejam completados com o que foi feito antes da conclusão. Quem acompanha as tarefas volta a ter os dois documentos sempre inteiros e atualizados. O risco está em mexer na forma como a tarefa é gravada, coberto pelos cenários de referência. Libera a próxima versão do harness.

## O que foi feito

Os dois documentos agora sobrevivem à conclusão da tarefa com o texto que foi escrito, em vez de serem apagados. A aprovação passou a recusar uma tarefa cujos documentos ainda estão só com o modelo em branco, e a conclusão recusa um documento que não foi completado desde o início. Durante o trabalho apareceu mais um caso: no formato de arquivo único, o título do card também cortava o detalhamento técnico, e isso foi corrigido junto.

A verificação completa passou sem falhas, e os cenários de referência não mostraram regressão. Nas tarefas iniciadas antes desta versão não há como comparar com o começo, então, para elas, a conferência se limita a ver se o documento deixou de ser o modelo em branco.
