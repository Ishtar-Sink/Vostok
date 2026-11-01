---
name: finalize-spec
description: "Finaliza uma especificação (spec) implementada, com status implemented e todas as tarefas done, só a pedido do usuário: lê as fontes (spec, tarefas, evidências, memória, commits e código), sintetiza o conteúdo dos documentos executivo, técnico e de evidências, gera os três HTML, arquiva a spec com as tarefas e publica o executivo e o técnico em docs/specs/. Use quando o usuário pedir para finalizar, fechar, encerrar ou arquivar uma spec implementada, ou gerar os documentos finais de uma spec."
allowed-tools: Read Edit Glob Grep Bash(vostok *)
metadata:
  version: "1.2.0"
  author: "Victor Batalha"
  scripts: "vostok context load, vostok finalize sources, vostok finalize create, vostok finalize validate, vostok finalize build, vostok finalize archive, vostok commit check"
  templates: "templates/finalize/content.md templates/responses/question.md templates/responses/blocker.md templates/responses/completion.md"
---

# Finalizar spec

Fecha o ciclo de uma spec implementada: três documentos sintetizados em HTML, a spec arquivada com as tarefas e as evidências, e o executivo e o técnico publicados em `docs/specs/`.

## Termos

- **Spec finalizável:** spec com status `implemented` e todas as tarefas `done`.
- **Fontes:** o que o `vostok finalize sources` imprime (spec, registro e histórico de cada tarefa, evidências, caminhos alterados, memória e commits do período, e, nas tarefas em pasta, o `executivo.md` e o `tecnico.md` de cada uma) mais o código dos caminhos alterados. O executivo e o técnico das tarefas são a base da síntese; o `tecnico.html` já inclui o técnico de cada tarefa.
- **Conteúdo:** o arquivo `<paths.finalize>/<SPEC_ID>/content.md`, criado a partir de `templates/finalize/content.md` e escrito pelo agente. É a única entrada de texto dos documentos.
- **Documentos:** `executivo.html`, `tecnico.html` e `evidencias.html`, gerados pelo `vostok finalize build` a partir do conteúdo, da spec e das tarefas.
- **Arquivo:** a pasta `<paths.archive>/<SPEC_ID>/`, fora do git, para onde vão a spec, o conteúdo, as tarefas, as evidências e os documentos.

## Regras

[finalize-spec-01]: Só finalize a pedido do usuário, com a spec citada ou confirmada por ele (RN-01).
[finalize-spec-02]: Os documentos são sintetizados, não cópias: escreva cada seção do conteúdo a partir das fontes, sem colar trechos da spec ou das tarefas (RN-02).
[finalize-spec-03]: O executivo é para quem decide sem ler código: objetivo, entregas, valor e impacto, ganhos secundários, riscos e próximos passos, em linguagem de negócio, sem caminhos, comandos, nomes de funções nem IDs de requisito, de 400 a 800 palavras.
[finalize-spec-04]: O técnico é a transferência de conhecimento de quem vai manter: o quê, por quê e como, mais operação e diagnóstico, mapa do código e limites e dívidas, de 1.000 a 2.000 palavras, e "Referências" com o motivo de cada item.
[finalize-spec-05]: Não edite a spec, as tarefas nem as evidências. Só o conteúdo é escrito à mão; os documentos e o arquivo são gerados pelos comandos.
[finalize-spec-06]: Não invente números. Um número em "Em números" ou em "Valor e impacto" só entra se vier das fontes (contado ou medido); sem nenhum, escreva "Não se aplica." em "Em números".
[finalize-spec-07]: No máximo 2 rodadas de correção do `vostok finalize validate` ([agent/anti-loop-02]). Na terceira falha, **PARE** com o template `blocker`.
[finalize-spec-08]: Não faça commit nem stage ([agent/boundaries-01], [agent/boundaries-02]).
[finalize-spec-09]: A skill é executada no Coordenador, uma spec por vez, sem disparar subagente: ele já tem as fontes no contexto, e um subagente recomeçaria a leitura do zero.
[finalize-spec-10]: Escreva em prosa por padrão: cada seção abre pela conclusão, e cada parágrafo, de 2 a 5 frases, trata de uma ideia. Lista só para passos em ordem ou para itens curtos e paralelos (até 7, de no máximo duas linhas); tabela para comparar, para números e para os mapas. Nenhum número sem a frase que o explica; cada decisão com o contexto, a alternativa descartada e a consequência. Os tamanhos são faixas de referência, não limites.

## Checklist

Copie e marque o progresso:

```
- [ ] 1. Carregar o contexto
- [ ] 2. Identificar a spec
- [ ] 3. Ler as fontes
- [ ] 4. Criar e escrever o conteúdo
- [ ] 5. Validar o conteúdo
- [ ] 6. Gerar e conferir os documentos
- [ ] 7. Arquivar
- [ ] 8. Responder ao usuário
```

## Passos

### 1. Carregar o contexto

Execute:

```bash
vostok context load --skill finalize-spec
```

### 2. Identificar a spec

As specs ficam em `vostok/work/specs/refined/`. Use a spec citada pelo usuário. Sem citação, leia o front-matter das refinadas e liste as com status `implemented`:

- Uma só: use essa.
- Mais de uma: **PARE** e pergunte qual, com o template `question` (FA-02).
- Nenhuma: informe as specs e o status de cada uma, e encerre.

### 3. Ler as fontes

Execute:

```bash
vostok finalize sources <SPEC_ID>
```

Se o comando recusar (spec fora de `implemented` ou tarefa fora de `done`), **PARE** e responda com o template `blocker`, com a mensagem do comando (FA-01).

Leia a saída inteira, a spec, as evidências citadas e o código dos caminhos alterados. Anote, para a síntese: o problema e o resultado; as entregas e o valor de cada uma; o que mudou para quem usa; os números que as fontes comprovam; as decisões, os aprendizados (memória do período, ajustes, bloqueios e retomadas) e os riscos.

### 4. Criar e escrever o conteúdo

Execute:

```bash
vostok finalize create <SPEC_ID>
```

O comando imprime o caminho do conteúdo. Se ele já existir, o comando recusa: leia o conteúdo existente e continue a partir dele.

Substitua cada variável `#{...}` seguindo os comentários do arquivo, que trazem o propósito, o formato e o tamanho de cada seção, e as regras [finalize-spec-02] a [finalize-spec-06] e [finalize-spec-10]:

- `# Executivo`: Em resumo (a conclusão de tudo, na faixa do topo), Em números (até 3 itens `- **<valor>** <rótulo>`, ou "Não se aplica."), Objetivo, O que foi entregue (um parágrafo por entrega, com o nome em negrito), Valor e impacto (com a tabela antes e depois quando houver), Ganhos secundários, Riscos e limites e Próximos passos.
- `# Técnico`: Em resumo, Em números, Contexto, Como funciona (seguindo uma operação real), Decisões (uma por parágrafo), O que foi feito (por tema, com os requisitos e os commits), Operação e diagnóstico (passos numerados e a tabela de sintomas), Mapa do código (tabela de caminho, papel e testes), Limites e dívidas e Referências.
- `# Evidências`: Em resumo, Mapa de cobertura (uma linha por RF e RNF da spec, com a evidência que o prova) e, em cada bloco, "Prova" (o requisito que a evidência sustenta), "Como chegar" (o comando ou os passos que a geram) e "Como ler" (o que observar nela).

Use só o Markdown aceito, descrito no topo do arquivo. O `vostok finalize validate` confere o que é mecânico (seções, Em números, as tabelas do técnico, o mapa de cobertura, o executivo sem código e itens de lista longos demais); o resto destas regras é seu.

### 5. Validar o conteúdo

Execute:

```bash
vostok finalize validate <SPEC_ID>
```

Corrija no conteúdo cada problema listado e valide de novo, respeitando [finalize-spec-07]. Uma evidência ausente não se corrige no conteúdo: **PARE** e responda com o template `blocker`, citando a evidência e a tarefa.

### 6. Gerar e conferir os documentos

Execute:

```bash
vostok finalize build <SPEC_ID>
```

Confira os três documentos listados pelo comando:

- `executivo.html`: o resumo e os números na faixa do topo; o objetivo antes das entregas; cada número explicado; nenhum caminho, comando ou nome de função ([finalize-spec-03]).
- `tecnico.html`: contexto, como funciona e decisões antes da referência; operação e diagnóstico e mapa do código prontos para quem dá sustentação; requisitos, arquivos, diffs e commits.
- `evidencias.html`: o mapa de cobertura primeiro e uma peça por evidência, com "Prova", "Como chegar" e "Como ler".

Se algo estiver errado, corrija o conteúdo e repita os passos 5 e 6.

### 7. Arquivar

Execute:

```bash
vostok finalize archive <SPEC_ID>
```

Se o comando recusar porque um destino já existe (FA-04), **PARE** e responda com o template `blocker`, com a mensagem do comando. Não remova nem renomeie o destino sem a decisão do usuário.

### 8. Responder ao usuário

Execute `vostok commit check` e responda com o template `completion`, com os caminhos do arquivo e dos documentos publicados. Em "Próximos passos", sugira o commit dos documentos em `docs/specs/<SPEC_ID>/`.

## Formatos de resposta

Respostas ao usuário usam os templates de `templates/responses/`. Esta skill não tem formatos próprios.
