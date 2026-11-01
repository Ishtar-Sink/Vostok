<!-- Conteúdo dos documentos de #{SPEC_ID}, escrito pelo agente. O `vostok finalize build` transforma este arquivo em HTML. -->
<!-- Markdown aceito: parágrafos, listas com "- " ou "1. ", tabelas, blocos de código, `código` e **negrito**. Comentários como estes não aparecem nos documentos. -->
<!-- Prosa por padrão: cada seção abre pela conclusão, um parágrafo por ideia, de 2 a 5 frases. Lista só para passos em ordem ou itens curtos e paralelos (até 7, de no máximo duas linhas); tabela para comparar, para números e para mapas. -->
<!-- Cada título "## " precisa de texto, e todas as variáveis de template precisam ser substituídas. "Não se aplica." vale onde o comentário da seção permite. -->

# Executivo

<!-- Para quem decide ou acompanha sem ler código: lido em 3 a 5 minutos, de 400 a 800 palavras. Linguagem de negócio, sem caminhos, comandos nem nomes de funções (nada entre crases). -->

## Em resumo

<!-- A conclusão do documento inteiro: o que foi entregue e o ganho principal. Prosa, um parágrafo de 2 a 4 frases. Aparece na faixa do topo. -->
#{EXEC_SUMMARY}

## Em números

<!-- Até 3 números de destaque, um por linha, no formato "- **<valor>** <rótulo curto>" (ex.: "- **10** cenários iguais nos dois formatos"). Aparecem ao lado do resumo; cada número precisa estar explicado no texto. Sem número comprovável, escreva "Não se aplica.". -->
#{EXEC_NUMBERS}

## Objetivo

<!-- O problema ou a necessidade antes da entrega, e quem era afetado. Prosa, 1 ou 2 parágrafos. -->
#{EXEC_OBJECTIVE}

## O que foi entregue

<!-- Cada entrega contada pelo que passou a ser possível, não pelo que foi programado: um parágrafo de 2 a 4 frases por entrega, começando pelo nome dela em **negrito**. -->
#{EXEC_DELIVERED}

## Valor e impacto

<!-- O que melhora para quem usa, com números quando houver, sempre com a frase que os explica. Prosa, 1 ou 2 parágrafos, e uma tabela "| Situação | Antes | Depois |" de até 6 linhas quando houver comparação. -->
#{EXEC_VALUE}

## Ganhos secundários

<!-- Efeitos úteis que não eram o objetivo: defeitos achados, base para próximas entregas, testes que passam a proteger a área. Prosa, 1 parágrafo; lista curta (até 4) só se forem itens independentes. -->
#{EXEC_SECONDARY}

## Riscos e limites

<!-- O que ficou de fora e o que ainda pode dar errado, em linguagem de negócio. Prosa, 1 parágrafo. -->
#{EXEC_RISKS}

## Próximos passos

<!-- O que vem a seguir e as decisões que dependem de pessoas: lista numerada de até 4 itens ou um parágrafo. Sem próximos passos, escreva "Não se aplica.". -->
#{EXEC_NEXT}

# Técnico

<!-- Para quem vai manter, revisar ou dar sustentação ao que foi entregue, inclusive sem ter acompanhado a spec: serve de transferência de conhecimento. De 1.000 a 2.000 palavras. Explicação primeiro (contexto, como funciona, decisões), referência depois (operação, mapa do código). -->

## Em resumo

<!-- O que foi construído e como as partes se conectam, para quem só vai ler isto. Prosa, 1 parágrafo de 3 a 5 frases. Aparece na faixa do topo. -->
#{TECH_SUMMARY}

## Em números

<!-- Até 3 números técnicos de destaque, no formato "- **<valor>** <rótulo curto>", ou "Não se aplica.". -->
#{TECH_NUMBERS}

## Contexto

<!-- Como era antes e qual limitação técnica motivou a mudança. Prosa, 1 ou 2 parágrafos. -->
#{TECH_CONTEXT}

## Como funciona

<!-- O desenho da solução seguindo o caminho de uma operação real, do comando ao arquivo gravado. Prosa, de 2 a 5 parágrafos, com bloco de código quando ajudar. -->
#{TECH_HOW}

## Decisões

<!-- Uma decisão por parágrafo, com a decisão em **negrito** no início: o contexto, o que foi decidido, a alternativa descartada e a consequência. Tabela só para comparar opções. -->
#{TECH_DECISIONS}

## O que foi feito

<!-- As mudanças por tarefa ou por tema, um parágrafo por tema, ligando cada uma ao requisito que atende (RF, RN) e ao commit. -->
#{TECH_DONE}

## Operação e diagnóstico

<!-- Como usar o que foi entregue, em passos numerados (até 8), e uma tabela "| Sintoma | Causa provável | O que fazer |" (até 8 linhas) com as falhas conhecidas, como diagnosticar e como desfazer. -->
#{TECH_OPERATION}

## Mapa do código

<!-- Tabela "| Caminho | Papel | Testes |", uma linha por componente criado ou alterado, com os testes que o protegem. -->
#{TECH_CODE_MAP}

## Limites e dívidas

<!-- O que não foi feito de propósito, o que ficou frágil e o que pede atenção numa próxima mudança. Prosa, 1 a 3 parágrafos; lista (até 6) só se forem itens independentes. -->
#{TECH_LIMITS}

## Referências

<!-- A spec, os commits, as entradas de memória e os documentos relacionados, com o motivo de cada um. Lista. -->
#{TECH_REFERENCES}

# Evidências

<!-- O que prova que cada requisito foi cumprido. -->

## Em resumo

<!-- Quantas evidências, de quantas tarefas, e se todas passaram. Prosa, 1 ou 2 frases. Aparece na faixa do topo. -->
#{EVIDENCE_SUMMARY}

## Mapa de cobertura

<!-- Tabela "| ID | O que foi verificado | Evidência |", uma linha por requisito funcional (RF) e não funcional (RNF) da spec, citando a evidência ("TASK-NN / arquivo") que o prova. -->
#{EVIDENCE_COVERAGE}

<!-- Um bloco por evidência, criado pelo `vostok finalize create`. Preencha "Prova" (o requisito que a evidência sustenta, em linguagem direta), "Como chegar" (o comando ou os passos que a geram) e "Como ler" (o que observar nela). -->
