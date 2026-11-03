<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

Os avais que você dá pelo painel agora ficam em um arquivo por spec, em vez de um único arquivo que só crescia. Um comando novo mostra os avais de uma spec ou de uma tarefa, do mais novo para o mais antigo, para que ninguém precise abrir esses arquivos para consultar uma aprovação.

Na primeira leitura ou gravação com a versão nova, o arquivo antigo foi dividido por spec, uma única vez, sem alterar nenhuma linha. Esse era um ponto de risco alto: a trava de aval depende desses registros para aceitar ou recusar uma conclusão. Por isso a divisão roda antes de qualquer leitura ou gravação, e a trava nunca vê um estado pela metade. Com isso, o arquivamento dos avais junto com a spec (TASK-05) e as novas ações do painel (TASK-08) ficam desbloqueados.
