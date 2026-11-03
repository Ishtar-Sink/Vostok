<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

O painel passa a saber quando o plano de tarefas de uma spec ficou desatualizado, porque a spec foi ajustada e aprovada de novo depois dele. Hoje ele só sabe se o plano existe e por isso oferece aprovar um plano que não cobre o que mudou. A informação vem da mesma regra que a linha de comando já usa, para o painel e a CLI nunca discordarem.

A tarefa é de perfil padrão, com risco baixo: só acrescenta uma informação, sem mudar telas nem o que já era mostrado. Ela desbloqueia a TASK-02, que usa essa informação para mostrar o aviso e trocar "Aprovar plano" por "Refazer o plano".
