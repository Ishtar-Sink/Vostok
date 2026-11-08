# Interação

Define como o agente devolve ao usuário as perguntas e as opções de decisão.



[project/interaction-01]: No Claude Code, as perguntas e as opções relevantes que pedem decisão do usuário (rodadas de refinamento, conferência de evidências, avais, escolha entre caminhos) vão pelo prompt de pergunta da ferramenta (AskUserQuestion), com as mesmas opções e a recomendada em primeiro, em vez de só em texto. Os próximos passos também vão pelo prompt de pergunta, com o passo recomendado em primeiro.
