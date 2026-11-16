# Escrita para pessoas

Define como o agente escreve os textos que pessoas leem: specs, resumos, critérios, respostas no chat, evidências e documentos finais. Os termos próprios do harness estão em `context/glossario.md`.

[agent/writing-01]: A regra vale para o texto lido por pessoas. `execucao.md` e `verificacao.md` (salvo os campos das evidências) e o que só o agente lê seguem densos.
[agent/writing-02]: Escreva para um leitor só por texto e abra pelo resultado: a primeira frase diz o que mudou ou o que se decide, e o detalhe vem depois.
[agent/writing-03]: Frases de até cerca de 25 palavras, na voz ativa. Divida a frase em duas antes de empilhar condições.
[agent/writing-04]: IDs (RF, CT, CB, RN, CA, DEC) ficam em colunas, em metadados e no parêntese final da linha, nunca no meio da frase. O texto diz em palavras o que o requisito exige.
[agent/writing-05]: Dê o nome por extenso à sigla na primeira menção. Termo interno do harness só entra com a explicação do glossário ou trocado por uma palavra comum.
[agent/writing-06]: Use estrutura só onde ela ajuda: lista para passos e itens paralelos curtos, tabela para três ou mais atributos comparáveis, prosa para raciocínio e decisões ([agent/responses-06]).
[agent/writing-07]: Quando o leitor precisar reproduzir algo, escreva o comando exato, pronto para copiar, e o resultado esperado. "Rode os testes" não basta.
[agent/writing-08]: Antes de entregar, confira as três perguntas: acho o que mudou? Entendo sem abrir o código? Consigo reproduzir? Com um "não", reescreva.

## Exemplos

| Antes | Depois |
|---|---|
| A janela abre por Mostra (RF-13, CT-11, CB-01) com blocos recolhidos conforme CB-02. | A janela abre pelo que a evidência mostra, depois onde olhar e o que seria falha; o resto fica recolhido (RF-13). |
| Perfil de julgamento: risco alto, decisão do Coordenador na onda 2. | Risco alto: um valor errado gravado pararia os comandos das tarefas. |
| Rodar os testes de recusa em `apps/cli/tests/settings.rs`. | Rodar `cargo test --test settings -- validacao_` e ver `test result: ok`. |
