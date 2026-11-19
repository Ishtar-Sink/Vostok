# Estudo da revisão com nível de confiança

Data do estudo: 2026-11-19. Versão: `vostok` 0.9.1.

## Escopo e método

O estudo responde quatro perguntas:

1. O que a revisão sem contexto acha hoje, rodada a rodada, e o que ainda escapa para a execução.
2. Quais escapes uma revisão por leitura não acha, por natureza.
3. Como medir a confiança de que não restam problemas, sem depender de o agente dizer que está confiante.
4. Que mudanças tornam a revisão mais eficiente (menos rodadas e tokens por achado) e mais assertiva (menos achados errados ou que desfazem outro).

A base:

- **Este repositório:** os históricos de ajustes (autor `revisor`), os `registro.md`, os `conferencias.md` e as regressões aceitas das SPEC-060 a SPEC-076, em `vostok/work/.archive/` e `vostok/work/tasks/`.
- **Outros sete projetos desta máquina que usam o harness:**
  - um projeto pessoal com revisão em duas specs e sem revisão em outras duas, que é o único caso com comparação dentro do mesmo projeto;
  - seis projetos sem revisão (synestia, translunar, atria e vega, da organização Ishtar Sink, mais um projeto pessoal e um de trabalho). Eles servem de grupo de comparação.
  - Dos projetos de fora da organização, o estudo não cita nomes nem conteúdo.
- **Código e textos do mecanismo:** o código de `apps/cli/src/commands/review.rs`, a persona `vostok/personas/revisor.md` e o passo de revisão das skills.
- **Literatura:** inspeção de software (captura e recaptura, leitura por perspectiva) e calibração de confiança de LLMs.

Todas as contagens de achados são aproximadas: cada linha `revisor` foi dividida pelos itens da descrição. As contagens de etapas (ajuste, falha de teste, regressão aceita) são exatas.

## Como a revisão funciona hoje

- O `vostok review brief <fluxo> <SPEC>` imprime o roteiro. Ele traz:
  - o modelo do perfil `julgamento` e a persona `revisor`;
  - a lista fixa do que procurar e os arquivos;
  - a tabela de resposta (`Local | Problema | Correção | Muda o escopo`).
- Um único subagente lê e devolve a tabela. O Coordenador:
  - descarta o que é estilo;
  - aplica o que não muda escopo;
  - leva ao usuário o que muda.
- Com correção aplicada, há nova rodada, até o limite de `review_rounds` (SPEC-076). O padrão agora é 1 (commit `a51ab88`).
- O achado não tem severidade, categoria nem prova. O fim da revisão depende de duas coisas: uma rodada sem achado, ou o limite de rodadas. Nada mede quanto ainda falta achar.

## O que os dados mostram

### A revisão funciona e o efeito é grande

| Grupo | Tarefas iniciadas | Ajustes na execução | Testes falhos | Regressões aceitas |
|---|---|---|---|---|
| Projeto pessoal, 2 specs sem revisão | 28 | 40 | 4 | 2 |
| O mesmo projeto, 2 specs com revisão | 28 | 6 | 0 | 1 |
| ai-hub, 3 specs sem revisor (061, 062, 065) | – | 5 escapes | 0 | 0 |
| ai-hub, 6 specs com revisor (068, 072 a 076) | – | 1 escape e 2 redetalhamentos entre ondas | 0 | 2 (mesma causa) |
| 6 projetos sem revisão (outubro e novembro) | ≈ 310 | ≈ 270 | ≈ 45 | ≈ 100 |

Nos projetos da organização, 104 de 191 tarefas (54%) tiveram pelo menos um evento de retrabalho depois do início. No projeto pessoal, os ajustes na execução caíram de 40 para 6 com a revisão.

Uma ressalva: parte da queda vem de versões mais novas do harness e de regras dos projetos, e não só do revisor. Um projeto sem revisão, mas com versão recente, também chegou a zero regressões. Isso pesa sobretudo em "baseline que mede a entrega", a maior causa de regressão aceita em outubro (≈ 70 casos).

### Rodadas: a segunda rodada ainda acha muito

| Onde | Achados na 1ª rodada | Achados da 2ª em diante | Parte da 2ª em diante |
|---|---|---|---|
| ai-hub, spec | ≈ 40 | ≈ 23 | ≈ 37% |
| ai-hub, tarefas | ≈ 45 | ≈ 9 | ≈ 18% |
| Projeto pessoal, spec (SPEC-003, 5 rodadas) | 5 | 8, 6, 5, 3 | ≈ 80% |
| Projeto pessoal, tarefas | ≈ 75 | ≈ 113 | ≈ 60% |

Três leituras:

- **A curva cai, mas devagar.** Na spec de 23 tarefas do projeto pessoal, as rodadas 2 a 5 acharam 8, 6, 5 e 3 itens novos. Uma rodada fixa por padrão (SPEC-076) deixa a maior parte desses itens para a execução nas specs grandes.
- **Parte do que as rodadas seguintes acham é retrabalho da própria revisão.** Exemplos:
  - correção que apagou passos e foi restaurada;
  - correção da correção (tamanho de comentário);
  - uma rodada que desfez a anterior (075 T05, com `vostok_at` posto e tirado);
  - propagação de nomes depois que uma tarefa mudou um contrato.

  Isso é falta de assertividade: o achado não é conferido antes de ser aplicado.
- **Spec e tarefas pegam coisas diferentes.** Na spec, quase metade dos achados é ambiguidade de contrato. Nas tarefas, predominam a mecânica da Verificação e a consistência entre tarefas.

### O que escapa, por categoria

| Categoria | O revisor acha? | Peso nos escapes |
|---|---|---|
| Baseline que mede a entrega ou tem saída instável (ordem de testes paralelos, tempo, número de linha) | por leitura, às vezes. Rodando duas vezes, sempre | alto em outubro, ≈ 2 casos com revisor |
| Comando da Verificação ou da evidência que não roda ou não prova (`grep -c` sob `pipefail`, `--` que desliga o filtro, `wc` com espaços no macOS, variável que não passa entre linhas) | às vezes. Rodando, sempre | alto |
| Texto de `Falha` ou `Trecho` que aparece ou falta na saída normal | por leitura, raramente. Rodando, sempre | médio, ainda escapa com revisão |
| Arquivo faltando em Mudar, testes, golden ou suítes afetadas fora da Verificação | sim, com busca pelos consumidores | médio, ainda escapa (075 T03, três suítes) |
| Inconsistência entre tarefas (contrato suposto de tarefa anterior, conflito na onda) | sim, lendo todas juntas | alto sem revisão, quase zero com ela |
| Limite de tamanho de arquivo, compilação | talvez | médio |
| Defeito visual visto só no print | não | ≈ 12 recusas em 7 tarefas |
| Ambiente e ferramenta (simulador, arquitetura, rede) | não | maior categoria do projeto de trabalho |
| Requisito novo ou decisão do usuário | não: não é escape | ≈ 30 casos |

O principal achado: **as três primeiras categorias se descobrem rodando o comando, não lendo**. Hoje o revisor só lê. A Verificação e o baseline são executados pela primeira vez no `start`, já depois da aprovação, e é aí que esses defeitos aparecem.

Os escapes que se repetem custam muito. Uma única causa afetou 15 a 17 tarefas da mesma spec num dos projetos: o baseline listava pastas que a própria tarefa criava, e o comando de evidência levava um `--` que fazia o runner ignorar o filtro. Um achado antes da execução teria evitado dezenas de eventos.

## Confiança: o que não funciona e o que funciona

### Pedir ao agente que diga quão confiante está não funciona

Os estudos de calibração mostram LLMs muito confiantes demais quando declaram a própria confiança. Os métodos por consistência entre várias amostras calibram melhor que a autodeclaração. Num estudo com código, os modelos tiveram excesso de confiança sistemático, e só checar executando melhorou a calibração. Uma linha "confiança: 95%" na tabela do revisor seria um número sem lastro. Pior: viraria o critério de parada.

### Estimar o que falta pela sobreposição de revisores independentes

A inspeção de software tem uma técnica para isso: **captura e recaptura**. Dois revisores independentes leem o mesmo material:

- `n1` é o número de achados do primeiro;
- `n2` é o número de achados do segundo;
- `m` é o número de achados em comum.

O estimador de Chapman dá o total esperado:

```
N ≈ (n1 + 1)(n2 + 1) / (m + 1) − 1
restante ≈ N − (n1 + n2 − m)
```

Com sobreposição alta, quase nada falta. Com sobreposição baixa, cada revisor está vendo uma parte diferente de um conjunto maior, e vale outra rodada. Exemplo:

- `n1 = 4`, `n2 = 5` e `m = 4`: o restante estimado é 0.
- `n1 = 4`, `n2 = 5` e `m = 1`: o total estimado é 14, com 6 ainda não achados.

Limites conhecidos:

- Com dois revisores e poucos achados, a estimativa é instável.
- A técnica supõe revisores independentes. Revisores com perspectivas diferentes (leitura por perspectiva) acham mais, mas se sobrepõem menos, e isso infla a estimativa. Por isso, para estimar, os dois revisores usam o mesmo roteiro e o mesmo perfil. A diversidade vem de outro modelo ou de outra ordem de leitura, não de outro foco.
- O número serve como sinal de parada, não como garantia. Ele precisa ser calibrado com os escapes medidos depois (abaixo).

Os relatos de revisão de código com LLM vão na mesma direção:

- várias passadas independentes aumentam o recall;
- descartar o achado de uma passada só e conferir cada achado com uma prova no código aumenta a precisão, com pequena perda de recall.

### Confiança operacional: uma regra de parada medida

A proposta é trocar "o agente se diz confiante" por uma regra que a CLI calcula e grava. A revisão de um fluxo termina quando:

1. o **ensaio** (abaixo) passou, sem defeito mecânico aberto;
2. a última rodada não teve achado novo de severidade `alta`;
3. o **restante estimado** pela sobreposição dos dois revisores ficou abaixo de um limite (proposta inicial: `< 1`);
4. ou o limite de rodadas de `review_rounds` foi atingido. Nesse caso, a resposta diz que a confiança não foi alcançada e mostra o restante estimado.

O nível de confiança vira uma configuração, por exemplo `review_confidence.<fluxo>: 1` (restante máximo), em vez de um número fixo de rodadas. `review_rounds` continua como teto de custo.

## Propostas

### 1. Ensaio: rodar antes de aprovar o que hoje só se lê (maior ganho)

Antes da revisão das tarefas, um comando novo, como `vostok task rehearse <SPEC>`, roda na árvore atual, para cada tarefa:

- **Baseline:** cada comando de "Baseline" duas vezes. Saída diferente entre as duas vezes é baseline instável. Comando que cita um caminho `criar` ou uma pasta nova da tarefa é baseline que mede a entrega.
- **Evidências:** os comandos de evidência e os comandos rotulados da Verificação que já rodam antes da mudança. O ensaio confere:
  - código de saída inesperado por construção (`grep` sem resultado sob `pipefail`, `ls` sem arquivo);
  - o texto de `Falha` que já aparece na saída atual;
  - o filtro de teste que pega testes que não são da tarefa.
- **Mudar:** caminho `criar` que já existe, caminho `alterar` que não existe e arquivo em comum entre tarefas da mesma onda.

O ensaio grava os achados no mesmo formato da tabela do revisor, com a prova (o comando e a saída). Ele resolve as três categorias que mais escapam com o custo de uma execução, sem modelo. Parte disso já existe espalhada (`depends_errors`, a validação do plano). O ensaio junta tudo antes da aprovação.

### 2. Achado com prova, severidade e categoria (assertividade)

A tabela do revisor ganha três colunas:

- **Prova:** o arquivo e o trecho, ou o comando, que mostram o problema.
- **Severidade:** `alta` (quebra a execução ou a Verificação), `média` (retrabalho localizado) ou `baixa`.
- **Categoria:** uma da lista fechada do estudo.

O Coordenador confere a prova antes de aplicar. A CLI, num `vostok review check`, confere se o arquivo e o trecho citados existem. Achado sem prova verificável é descartado e contado. Isso ataca as correções que desfazem outras e as correções da correção.

### 3. Dois revisores em paralelo e a estimativa do restante (eficiência e confiança)

Cada rodada dispara dois revisores com o mesmo roteiro, por exemplo um com o modelo do perfil e outro com o apelido de reserva, ou o mesmo modelo com outra ordem dos arquivos. Depois:

- `vostok review merge` junta as duas tabelas;
- um passo de casamento, com o mesmo local e problema equivalente, conta `m`;
- o comando imprime o restante estimado e decide se há nova rodada.

O custo por rodada dobra, mas a rodada passa a render mais: dois revisores em paralelo acham mais que dois em sequência, sem esperar a correção. Além disso, a parada passa a ter base. Nas specs pequenas, a sobreposição alta encerra na primeira rodada. Nas grandes, onde a 2ª à 5ª rodada ainda acham 8, 6, 5 e 3 itens, a revisão continua enquanto vale a pena.

A rodada seguinte revisa só as tarefas corrigidas, como hoje, e o roteiro recebe a lista dos achados já aplicados. Assim o revisor não reabre o que foi decidido e não desfaz a correção anterior.

### 4. Roteiro guiado pelos escapes

A lista "Procure:" de `TASK_LOOKUP` é fixa e genérica. Ela passaria a ser montada das categorias que mais escaparam no projeto, com a ordem e os exemplos tirados do registro (proposta 5). Duas fontes:

- as categorias de leitura deste estudo: busca pelos consumidores de um campo ou estado novo, golden e suítes fora da Verificação, contrato suposto de tarefa anterior;
- o que o ensaio não cobre.

As categorias que o ensaio já cobre saem da lista do revisor, e o roteiro fica menor.

### 5. Medir para calibrar: fechar o ciclo

Hoje não dá para medir a confiança porque faltam dados:

- o histórico do plano some na aprovação;
- as recusas de evidência não ficam no `conferencias.md` (as 92 linhas lidas são todas `conferida`);
- o ajuste na execução não diz se foi um escape e de que categoria.

Para fechar o ciclo:

- O `vostok task adjust record` em tarefa iniciada pede `--escape <categoria>` ou `--decisao` (requisito novo, que não conta como escape).
- O `review brief` e o `review merge` gravam por spec as rodadas, os achados por severidade e categoria e o restante estimado. Por exemplo, em `vostok/work/specs/<SPEC>/revisao.yml`, arquivado com a spec.
- O `finalize-spec` junta o restante estimado na aprovação com os escapes reais na execução.

Com algumas specs medidas, o limite da regra de parada se ajusta ao dado: se o restante estimado `< 1` ainda deixa escapar 2 itens por spec, o limite desce. Isso é a calibração. Sem ela, qualquer nível de confiança é palpite.

### 6. Rodadas pelo tamanho e pelo risco, não fixas

O retrabalho se concentra nas specs grandes e com várias ondas. Duas specs de 26 e 36 tarefas tiveram escape em 20 e 35 delas. As specs pequenas ou de continuação ficaram perto de zero. Enquanto a regra de parada da proposta 3 não existe, o padrão de rodadas pode depender do plano. Por exemplo:

- 1 rodada até 4 tarefas;
- 2 rodadas até 10 tarefas;
- 3 rodadas acima disso ou com o perfil `julgamento`.

O usuário segue podendo fixar o número em `review_rounds`.

## Ordem sugerida

| Ordem | Proposta | Ganho | Custo |
|---|---|---|---|
| 1 | Ensaio antes da aprovação | Pega as categorias que mais escapam, sem modelo | Comando novo na CLI, reaproveitando o `baseline` |
| 2 | Medição (`--escape`, `revisao.yml`, recusas no `conferencias.md`) | Base para tudo que vem depois | Pequeno |
| 3 | Achado com prova, severidade e categoria, com `review check` | Assertividade, fim das correções que se desfazem | Roteiro, persona e um comando |
| 4 | Dois revisores e a estimativa do restante | Parada com base, nível de confiança configurável | Dobra os tokens por rodada; casamento de achados |
| 5 | Rodadas pelo tamanho do plano | Paliativo até a 4 | Pequeno |
| 6 | Roteiro guiado pelos escapes | Roteiro menor e mais certeiro | Depende da 2 |

## Implicação imediata para a SPEC-076

A SPEC-076 fixou 1 rodada por padrão. Os dados não sustentam esse padrão nas specs grandes:

- no ai-hub, cerca de 27% dos achados vieram da 2ª rodada;
- no projeto pessoal, a maior parte dos achados de tarefa veio da 2ª rodada em diante.

Há duas saídas:

- manter 1 por padrão e adotar logo a proposta 6;
- voltar a 2 por padrão até a regra de parada da proposta 3 existir.

## Lacunas

- Não há dado de revisão de plano, porque o histórico some na aprovação.
- As recusas de evidência foram contadas pelo texto dos ajustes, não pelo `conferencias.md`.
- Várias falhas de teste e evidências com erro não registram a causa.
- A comparação "com e sem revisão" mistura o efeito do revisor com o das versões novas do harness. A única comparação dentro do mesmo projeto tem 4 specs.
- Os números da literatura sobre revisão com LLM vêm de estudos pequenos ou de relatos de empresas.

## Referências

- [Comparison and integration of capture-recapture models (Briand, El Emam, Freimut)](https://ehealthinformation.ca/web/default/files/wp-files/isern-98-11.pdf)
- [Estimating the number of undetected software errors via the correlated capture-recapture model](https://repository.lsu.edu/infosys_pubs/66)
- [Padberg: estimating defect content after inspections](https://ps.ipd.kit.edu/814.php)
- [Perspective-based reading, University of Maryland](https://www.cs.umd.edu/projects/SoftEng/ESEG/topics/rtfault2.html)
- [Empirical investigation of perspective-based reading: a replicated experiment](https://core.ac.uk/works/31885195)
- [Can LLMs express their uncertainty? (Xiong et al.)](https://ar5iv.labs.arxiv.org/html/2306.13063)
- [An empirical study of security calibration in LLMs for code](https://arxiv.org/pdf/2606.31159)
- [Towards practical defect-focused automated code review](https://arxiv.org/html/2505.17928v2)
- [Multi-model AI code review: convergence loops](https://zylos.ai/research/2026-03-01-multi-model-ai-code-review-convergence)
- [Reducing false positives in AI code review](https://reviewate-website.pages.dev/blog/reduce-false-positives-ai-code-review)
- [LLM code reviews, case study (Zenodo)](https://zenodo.org/records/20519585)
