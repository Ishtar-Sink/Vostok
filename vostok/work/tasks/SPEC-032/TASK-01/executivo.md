<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

Estudou como operar o projeto inteiramente pelo painel, sem o chat. A conclusão é que o caminho preferido, o painel gravar pedidos que o agente atende em loop, funciona. O que falta não é desempenho, e sim conversa: hoje, quando o agente precisa perguntar algo, o pedido é recusado. Por isso, criar e refinar specs pelo painel ainda não funciona, e esse é o primeiro ponto da segunda parte da versão 0.4.0 (SPEC-033).

O estudo também resolveu o acúmulo de arquivos entre o painel e o agente:
- pedidos e perguntas ficam em pastas por estado;
- pedidos, perguntas e aprovações de uma spec são arquivados junto com ela;
- um comando de limpeza, também no painel, apaga os já arquivados, sempre com a lista e a sua confirmação.

Assim o agente não gasta contexto lendo o que já terminou.

As medições mostraram que todas as fontes da nova tela de atividades leem em menos de 50 ms, então as tarefas seguintes desta spec seguem sem mudança. O estudo está em `docs/estudos/painel-ponta-a-ponta.md`.
