<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

Testa no protótipo o que torna o painel útil no dia a dia: aprovar e editar pelo painel sem quebrar as regras do harness, não perder dados quando o painel e o agente mexem no mesmo arquivo e um chat em loop atender um pedido feito pelo painel.
Perfil julgamento. Risco: a aprovação pelo painel não passar nas validações ou o loop depender demais do Claude Code.
Depende do levantamento (TASK-01) e do protótipo (TASK-03) e desbloqueia o estudo final (TASK-05).

## O que foi feito

- O painel passou a agir, não só mostrar: aprova specs e tarefas, edita os arquivos de plano, spec e tarefa e grava pedidos para o agente.
- A aprovação feita no painel vale igual à do chat: passa pelas mesmas conferências do `vostok`, fica marcada como feita pelo painel e deixa um registro do aval. Uma chamada que não vem da página do painel é recusada.
- Editar pelo painel não quebra nada:
  - a edição é registrada como ajuste e validada pelo `vostok`;
  - se o agente mudar o mesmo arquivo ao mesmo tempo, o painel avisa e mostra os dois textos, sem perder nenhum;
  - se o agente mudar depois, a tela se atualiza sozinha em cerca de um terço de segundo.
- Um chat do Claude Code em loop processou, sem nenhuma mudança nas skills, um pedido feito pelo painel em 1 min 18 s, a maior parte esperando o próximo ciclo.
  - O primeiro pedido foi recusado com o motivo explicado, porque a tarefa tinha um problema que só o usuário pode decidir.
- O Codex não tem um loop dentro da sessão. O estudo descreve como um processo externo faria esse papel.
