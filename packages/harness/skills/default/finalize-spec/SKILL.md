---
name: finalize-spec
description: "Finaliza uma especificação (spec) implementada, com status implemented e todas as tarefas done, só a pedido do usuário: lê as fontes (spec, tarefas, evidências, memória, commits e código), sintetiza o conteúdo dos documentos executivo, técnico e de evidências, gera os três HTML, arquiva a spec com as tarefas e publica o executivo e o técnico em docs/specs/. Use quando o usuário pedir para finalizar, fechar, encerrar ou arquivar uma spec implementada, ou gerar os documentos finais de uma spec."
allowed-tools: Read Edit Glob Grep Bash(packages/harness/skills/default/finalize-spec/scripts/*) Bash(packages/harness/scripts/default/*)
metadata:
  version: "1.0.0"
  author: "Victor Batalha"
  requires: "config/specs.yml config/documents.yml rules/agent/anti-loop.md"
  scripts: "skills/default/finalize-spec/scripts/finalize-spec.sh scripts/default/git/check-commits.sh"
  templates: "templates/finalize/content.md templates/responses/question.md templates/responses/blocker.md templates/responses/completion.md"
---

# Finalizar spec

Fecha o ciclo de uma spec implementada: três documentos sintetizados em HTML, a spec arquivada com as tarefas e as evidências, e o executivo e o técnico publicados em `docs/specs/`.

## Termos

- **Spec finalizável:** spec com status `implemented` e todas as tarefas `done`.
- **Fontes:** o que o `--sources` imprime (spec, registro e histórico de cada tarefa, evidências, caminhos alterados, memória e commits do período) mais o código dos caminhos alterados.
- **Conteúdo:** o arquivo `<paths.finalize>/<SPEC_ID>/content.md`, criado a partir de `templates/finalize/content.md` e escrito pelo agente. É a única entrada de texto dos documentos.
- **Documentos:** `executivo.html`, `tecnico.html` e `evidencias.html`, gerados pelo `--build` a partir do conteúdo, da spec e das tarefas.
- **Arquivo:** a pasta `<paths.archive>/<SPEC_ID>/`, fora do git, para onde vão a spec, o conteúdo, as tarefas, as evidências e os documentos.

## Regras

[finalize-spec-01]: Só finalize a pedido do usuário, com a spec citada ou confirmada por ele (RN-01).
[finalize-spec-02]: Os documentos são sintetizados, não cópias: escreva cada seção do conteúdo a partir das fontes, sem colar trechos da spec ou das tarefas (RN-02).
[finalize-spec-03]: O executivo é para executivos: entregas e valor, em linguagem de negócio, sem caminhos, comandos, nomes de funções nem IDs de requisito.
[finalize-spec-04]: O técnico explica brevemente o que foi feito e como, e lista em "Referências" o que o leitor deve abrir para se aprofundar, com o motivo de cada item.
[finalize-spec-05]: Não edite a spec, as tarefas nem as evidências. Só o conteúdo é escrito à mão; os documentos e o arquivo são gerados pelos scripts.
[finalize-spec-06]: Não invente indicadores. Um KPI de valor só entra se vier das fontes (contado ou medido); sem nenhum, escreva "Não se aplica.".
[finalize-spec-07]: No máximo 2 rodadas de correção do `--validate` ([agent/anti-loop-02]). Na terceira falha, **PARE** com o template `blocker`.
[finalize-spec-08]: Não faça commit nem stage ([agent/boundaries-01], [agent/boundaries-02]).

## Checklist

Copie e marque o progresso:

```
- [ ] 1. Identificar a spec
- [ ] 2. Ler as fontes
- [ ] 3. Criar e escrever o conteúdo
- [ ] 4. Validar o conteúdo
- [ ] 5. Gerar e conferir os documentos
- [ ] 6. Arquivar
- [ ] 7. Responder ao usuário
```

## Passos

### 1. Identificar a spec

As specs ficam em `packages/harness/work/specs/refined/`. Use a spec citada pelo usuário. Sem citação, leia o front-matter das refinadas e liste as com status `implemented`:

- Uma só: use essa.
- Mais de uma: **PARE** e pergunte qual, com o template `question` (FA-02).
- Nenhuma: informe as specs e o status de cada uma, e encerre.

### 2. Ler as fontes

Execute:

```bash
packages/harness/skills/default/finalize-spec/scripts/finalize-spec.sh --sources <SPEC_ID>
```

Se o script recusar (spec fora de `implemented` ou tarefa fora de `done`), **PARE** e responda com o template `blocker`, com a mensagem do script (FA-01).

Leia a saída inteira, a spec, as evidências citadas e o código dos caminhos alterados. Anote, para a síntese: o problema e o resultado; as entregas e o valor de cada uma; o que mudou para quem usa; os números que as fontes comprovam; as decisões, os aprendizados (memória do período, ajustes, bloqueios e retomadas) e os riscos.

### 3. Criar e escrever o conteúdo

Execute:

```bash
packages/harness/skills/default/finalize-spec/scripts/finalize-spec.sh --create <SPEC_ID>
```

O script imprime o caminho do conteúdo. Se ele já existir, o script recusa: leia o conteúdo existente e continue a partir dele.

Substitua cada variável `#{...}` seguindo os comentários do arquivo e as regras [finalize-spec-02] a [finalize-spec-06]:

- `# Executivo`: Panorama, Indicadores de valor, Recorte, Entregas (um `### ` por entrega, com Impacto, O que mudou, Valor e Como perceber), Antes e depois, Próximos passos e Riscos e limitações.
- `# Técnico`: Visão geral, Arquitetura e componentes, O que foi feito, Decisões, Aprendizados, Pontos de atenção e Referências.
- `# Evidências`: em cada bloco, "Como chegar" (o comando ou os passos que geram a evidência) e "Como ler" (o que observar nela).

Use só o Markdown aceito, descrito no topo do arquivo.

### 4. Validar o conteúdo

Execute:

```bash
packages/harness/skills/default/finalize-spec/scripts/finalize-spec.sh --validate <SPEC_ID>
```

Corrija no conteúdo cada problema listado e valide de novo, respeitando [finalize-spec-07]. Uma evidência ausente não se corrige no conteúdo: **PARE** e responda com o template `blocker`, citando a evidência e a tarefa.

### 5. Gerar e conferir os documentos

Execute:

```bash
packages/harness/skills/default/finalize-spec/scripts/finalize-spec.sh --build <SPEC_ID>
```

Confira os três documentos listados pelo script:

- `executivo.html`: o Panorama abre o documento; cada entrega tem valor; nenhum caminho, comando ou nome de função ([finalize-spec-03]).
- `tecnico.html`: requisitos com as tarefas, arquivos, diffs, commits e referências.
- `evidencias.html`: uma peça por evidência, com "Como chegar" e "Como ler".

Se algo estiver errado, corrija o conteúdo e repita os passos 4 e 5.

### 6. Arquivar

Execute:

```bash
packages/harness/skills/default/finalize-spec/scripts/finalize-spec.sh --archive <SPEC_ID>
```

Se o script recusar porque um destino já existe (FA-04), **PARE** e responda com o template `blocker`, com a mensagem do script. Não remova nem renomeie o destino sem a decisão do usuário.

### 7. Responder ao usuário

Execute `packages/harness/scripts/default/git/check-commits.sh` e responda com o template `completion`, com os caminhos do arquivo e dos documentos publicados. Em "Próximos passos", sugira o commit dos documentos em `docs/specs/<SPEC_ID>/`.

## Formatos de resposta

Respostas ao usuário usam os templates de `templates/responses/`. Esta skill não tem formatos próprios.
