# Respostas

Define o formato das respostas do agente ao usuário.

[agent/responses-01]: Toda resposta ao usuário segue um template de `templates/responses/`, escolhido pela `description` do template.
[agent/responses-02]: Não crie formatos de resposta. Se nenhum template específico se aplica, use `general`.
[agent/responses-03]: Siga o `usage` do template. Partes opcionais que não se aplicam são omitidas inteiras, inclusive o título da seção. Nenhuma variável `#{...}` fica na resposta.
[agent/responses-04]: Uma skill pode acrescentar conteúdo próprio a um template, como uma tabela extra, mas não muda a estrutura dele.
[agent/responses-05]: Cada tipo de resposta tem estrutura fixa: as seções `##` do template, na ordem e com os títulos dele. Não reordene, não renomeie nem acrescente seções `##`, salvo o conteúdo próprio de uma skill ([agent/responses-04]).
[agent/responses-06]: Use tabela para itens com três ou mais atributos comparáveis (opções, tarefas, ondas, campos); lista para passos e itens paralelos curtos (arquivos alterados, comandos e resultados, pendências, próximos passos), numerada quando a ordem importa; prosa para raciocínio e decisões, no mínimo necessário para o leitor decidir.
[agent/responses-07]: Toda resposta sobre tarefas (aprovar o plano, aprovar, executar, ajustar e finalizar tarefa), inclusive a de bloqueio, traz a seção `## Status das tarefas` com a saída do `vostok task status <SPEC_ID>`, colada sem alteração.
