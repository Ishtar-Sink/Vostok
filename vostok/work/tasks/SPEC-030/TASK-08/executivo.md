<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

Dá ao harness a instrução padrão para um chat atender sozinho os pedidos feitos pelo painel, um por vez, pela skill certa de cada ação.
Perfil padrao. Risco: o loop decidir algo que é do usuário; a skill manda recusar com o motivo.
Depende do comando de pedidos (TASK-04); desbloqueia o processo do Codex (TASK-09).

**O que foi entregue.** O harness tem uma skill que atende sozinha os pedidos feitos pelo painel. Basta chamá-la com um intervalo, como `/process-requests 1m`, que ela mesma arma o loop. No teste real, ela pegou um pedido do painel e o recusou como o texto pedia, sem mexer em nada.
