---
id: SPEC-024
title: "Executivo e técnico preservados e escritos desde o planejamento"
status: implemented
author: "Victor Batalha"
created_at: 2026-11-01T21:16:41Z
updated_at: 2026-11-01T21:38:26Z
tags: []
type: fix
source: "draft/SPEC-024-executivo-e-tecnico-preservados-e-escritos-desde-o-planejamento.md"
refined_at: 2026-11-01T21:19:11Z
adjustments: 0
adjusted_at: null
content_hash: 631d0c6b7b3ac847cbce7f801055f7027f580221aeabcd1394c08ff7a96e44a3
approved_at: 2026-11-01T21:20:30Z
approved_hash: 631d0c6b7b3ac847cbce7f801055f7027f580221aeabcd1394c08ff7a96e44a3
---

# Executivo e técnico preservados e escritos desde o planejamento - Especificação Refinada

Corrige o defeito da issue #9 (https://github.com/Ishtar-Sink/Vostok/issues/9): na tarefa em pasta, gravar o registro move o conteúdo de `executivo.md` e `tecnico.md` para o `registro.md`. Junto, o executivo e o técnico passam a ser escritos no planejamento e enriquecidos na conclusão, com o `finish` exigindo o enriquecimento.

## Objetivo

Garantir que nenhum comando do `vostok` altere o executivo ou o técnico de uma tarefa em pasta, e que esses documentos existam desde o planejamento, para o usuário entender a tarefa antes de aprová-la, e sejam enriquecidos com o que foi feito antes da conclusão.

## Defeito

- **Comportamento atual:** Na 0.2.7, numa tarefa em pasta, o `vostok task finish` (e todo comando que grava o registro) move para o início de `registro.md` o conteúdo de `executivo.md` e `tecnico.md` a partir do primeiro `## `; nos dois arquivos sobra só o que vinha antes, e uma linha `# ` some. Aconteceu em 10 de 11 tarefas de uma spec do autor.
- **Comportamento esperado:** O comando só acrescenta a linha à tabela do `registro.md` (ou muda o `estado.yml`); `executivo.md` e `tecnico.md` ficam byte a byte como foram escritos, com ou sem títulos.
- **Como reproduzir:** Numa tarefa em pasta em execução, com testes, regressão e evidências `passed` e critérios marcados, escreva `executivo.md` com `## O que mudou` e um parágrafo e `tecnico.md` com `## O que foi feito` e um parágrafo; rode `vostok task finish SPEC-001 TASK-01`; os dois arquivos ficam só com o comentário do template e o `registro.md` começa com as duas seções.
- **Causa raiz:** `TaskDoc::text` monta o texto virtual com o executivo e o técnico sob `### Resumo executivo` e `### Detalhes técnicos`, e `TaskDoc::write_text` o divide de volta com `split_task_text` (`apps/cli/src/commands/task.rs`), em que todo `## ` diferente de "Execução" e "Descrição" passa a ser o registro e uma linha `# ` encerra a parte atual sem destino. Os testes não pegaram porque os documentos de teste têm uma linha, sem títulos.

## Escopo

### Dentro do escopo

- Gravação da tarefa em pasta sem tocar no executivo nem no técnico.
- Executivo e técnico escritos no detalhamento das tarefas (skill `approve-plan`) e exigidos na aprovação.
- Hash dos dois documentos gravado no `start` e exigência de enriquecimento no `finish`.
- Skills `approve-plan`, `finish-task`, `approve-task` e `adjust-task` e templates `executivo.md` e `tecnico.md` alinhados ao ciclo novo.
- Testes com documentos estruturados (títulos, blocos de código) e os cenários de referência sem mudança.

### Fora do escopo

- Reparo automático das tarefas já afetadas: a resposta da issue explica a recuperação manual.
- Trava de enriquecimento no formato de arquivo único (resposta 2): ele mantém o comportamento atual.
- Mudanças no `vostok task migrate`, além do que a correção da gravação já cobre.
- Mudanças na finalização da spec (`vostok finalize`).

## Requisitos funcionais

| ID | Requisito |
|---|---|
| RF-01 | O `TaskDoc` deve gravar, numa tarefa em pasta, só os arquivos que o comando altera (`registro.md`, `estado.yml` e os de execução), sem regravar `executivo.md` nem `tecnico.md`. |
| RF-02 | O sistema deve preservar byte a byte `executivo.md` e `tecnico.md` em `start`, `baseline`, `tests`, `regression`, `evidence`, `resume`, `finish`, `persist` e nos comandos de ajuste e aprovação. |
| RF-03 | A validação da tarefa em pasta (`approve validate`, `approve approve`, `adjust validate`) deve recusar `executivo.md` ou `tecnico.md` com variável de template sem preencher. |
| RF-04 | A skill `approve-plan` deve escrever, no detalhamento, o executivo e o técnico de cada tarefa com o que ela vai fazer. |
| RF-05 | O `vostok task start` deve gravar no `estado.yml` o hash de `executivo.md` e o de `tecnico.md`. |
| RF-06 | O `vostok task finish` deve recusar a tarefa em pasta quando `executivo.md` ou `tecnico.md` for igual ao hash gravado no `start`, com a mensagem `o documento <arquivo> de <TASK> não foi enriquecido desde o início; enriqueça-o a partir de vostok task describe`. |
| RF-07 | Sem o hash do `start`, o `vostok task finish` deve recusar só documento com variável de template, como na 0.2.7. |
| RF-08 | A skill `finish-task` deve enriquecer os documentos existentes com o que foi feito, a partir do `vostok task describe`, em vez de escrevê-los do zero. |
| RF-09 | Os templates `executivo.md` e `tecnico.md` devem dizer que o texto é escrito no planejamento e enriquecido na conclusão. |

## Casos de borda

| ID | Situação | Comportamento esperado | Requisitos |
|---|---|---|---|
| CB-01 | Executivo com título `# `, `## ` e `### ` | Arquivo intacto depois de qualquer comando | RF-01, RF-02 |
| CB-02 | Técnico com bloco de código que contém `## ` | Arquivo intacto | RF-01, RF-02 |
| CB-03 | Documento só em prosa | Arquivo intacto, como hoje | RF-02 |
| CB-04 | Tarefa iniciada na 0.2.7, sem hash do `start` | `finish` só confere o template | RF-07 |
| CB-05 | Só um dos dois documentos enriquecido | `finish` recusa citando o que não mudou | RF-06 |
| CB-06 | Tarefa no formato de arquivo único | Sem trava de enriquecimento; comportamento atual | RF-06 |
| CB-07 | Ajuste que edita o executivo durante a execução | Não pede nova aprovação; conta como enriquecimento se o hash mudou | RF-06 |
| CB-08 | `resume` ou ajuste registrado depois de enriquecidos os documentos | Documentos intactos e hash do `start` mantido | RF-02, RF-05 |

## Componentes afetados

| Caminho | Ação | Motivo |
|---|---|---|
| `apps/cli/src/commands/task.rs` | alterar | `TaskDoc::write_text`, validação da pasta, hash no `start` e exigência no `finish` |
| `apps/cli/tests/task.rs` | alterar | Testes com documentos estruturados e da trava de enriquecimento |
| `apps/cli/tests/scenarios.rs` | alterar | Executivo e técnico com títulos nas variantes de pasta, sem mudar as fixtures |
| `packages/harness/templates/task/executivo.md` | alterar | Comentário do ciclo novo |
| `packages/harness/templates/task/tecnico.md` | alterar | Comentário do ciclo novo |
| `packages/harness/skills/default/approve-plan/SKILL.md` | alterar | Escrever o executivo e o técnico no detalhamento |
| `packages/harness/skills/default/finish-task/SKILL.md` | alterar | Enriquecer em vez de escrever do zero |
| `packages/harness/skills/default/approve-task/SKILL.md` | alterar | Recusa com documento no template |
| `packages/harness/skills/default/adjust-task/SKILL.md` | alterar | Ajuste do executivo ou do técnico durante a execução |
| `packages/harness/personas/arquiteto.md` | alterar | Escreve o `tecnico.md` no detalhamento |

## Premissas e dependências

- A versão 0.2.7 (SPEC-023) está publicada, com `TaskDoc`, `TaskLayout` e os cenários de referência em três variantes.
- `executivo.md` e `tecnico.md` continuam fora do hash de aprovação.
- O hash do `start` usa a mesma função de hash dos arquivos de execução (`sha256_hex`).

## Riscos

| Risco | Impacto | Mitigação |
|---|---|---|
| Mudança na gravação quebrar a ida e volta do formato de pasta | alto | Cenários de referência nas três variantes e testes com documentos estruturados |
| Tarefas em andamento travarem no `finish` depois da atualização | médio | Sem hash do `start`, só a checagem de template (RF-07) |
| Enriquecimento cosmético só para passar na trava | baixo | A skill `finish-task` orienta o conteúdo a partir do `describe` |

## Casos de teste

| ID | Cenário | Passos | Resultado esperado | Requisitos |
|---|---|---|---|---|
| CT-01 | Finish com documentos estruturados | Escrever `## O que mudou` e `## O que foi feito`; tests, regression, evidence, finish | Os dois arquivos idênticos; `registro.md` só ganha linhas na tabela | RF-01, RF-02 |
| CT-02 | Títulos e código | Executivo com `#`, `##`, `###`; técnico com bloco de código contendo `## `; rodar start, tests, resume e um ajuste | Arquivos idênticos | RF-01, RF-02 |
| CT-03 | Aprovação com template | `approve approve` com `tecnico.md` no template | Recusa citando o arquivo | RF-03 |
| CT-04 | Hash no start | `start` numa tarefa em pasta | `estado.yml` com os dois hashes | RF-05 |
| CT-05 | Finish sem enriquecer | Finish com os documentos iguais aos do start | Recusa com a mensagem do RF-06; depois de enriquecidos, conclui | RF-06 |
| CT-06 | Tarefa sem hash do start | Remover os hashes do estado e rodar o finish | Conclui com documentos preenchidos | RF-07 |
| CT-07 | Skills e templates | Ler `approve-plan`, `finish-task`, `approve-task`, `adjust-task` e os templates | Descrevem escrita no planejamento e enriquecimento na conclusão | RF-04, RF-08, RF-09 |
| CT-08 | Equivalência | Cenários de referência com documentos estruturados nas variantes de pasta | Mesma transcrição das fixtures | RF-02 |

## Critérios de aceite

- [ ] CA-01: O `vostok task finish` só acrescenta a linha de conclusão ao `registro.md`, e `executivo.md` e `tecnico.md` ficam como foram escritos, com ou sem títulos (RF-01, RF-02).
- [ ] CA-02: Nenhum comando que grava o registro ou o estado altera o executivo ou o técnico (RF-02).
- [ ] CA-03: Tarefa em pasta com executivo ou técnico no template não é aprovada (RF-03).
- [ ] CA-04: As skills orientam escrever os documentos no planejamento e enriquecê-los na conclusão (RF-04, RF-08, RF-09).
- [ ] CA-05: O `finish` recusa documentos não enriquecidos desde o `start` e aceita tarefas sem esse hash só com a checagem de template (RF-05, RF-06, RF-07).
- [ ] CA-06: Os cenários de referência continuam iguais nas três variantes.

## Referências

| Tipo | Referência | Uso |
|---|---|---|
| documentação | issue #9 do Ishtar-Sink/Vostok | Relato, reprodução e causa provável |
| arquivo | `apps/cli/src/commands/task.rs` (`TaskDoc::text`, `write_text`, `split_task_text`, `task_errors`, `finish`) | Causa raiz e pontos de mudança |
| arquivo | `apps/cli/tests/scenarios.rs` | Variantes de formato e por que os documentos de uma linha não pegaram o defeito |
| arquivo | `packages/harness/skills/default/approve-plan/SKILL.md` | Hoje diz que executivo e técnico são escritos na finalização |
| arquivo | `packages/harness/skills/default/finish-task/SKILL.md` | Hoje escreve os documentos do zero |
| arquivo | `packages/harness/personas/arquiteto.md` | Papel de escrever os detalhes técnicos |
| arquivo | `packages/harness/templates/task/executivo.md`, `tecnico.md` | Comentário "Escrito na finalização da tarefa" |
| memória | busca `executivo técnico` | Nenhuma entrada |

## O que mudou

| # | Assunto | Estado no Original | Estado Refinado |
|---|---|---|---|
| 1 | Causa | Provável, no finish | Confirmada na camada `TaskDoc`, valendo para todo comando que grava |
| 2 | Sem hash do start | Em aberto | Só checa o template |
| 3 | Arquivo único | Em aberto | Sem trava de enriquecimento |
| 4 | Aprovação | Não citada | Recusa documento no template |
| 5 | Componentes | Por tema | Caminhos de código, testes, skills, templates e persona |

## Registro de decisões

| # | Decisão | Alternativa descartada | Motivo |
|---|---|---|---|
| 1 | Sem hash do start, o finish só confere o template | Comparar com a aprovação | resposta 1A |
| 2 | Trava só na tarefa em pasta | Valer também no arquivo único | resposta 2A |
| 3 | Aprovação recusa documento no template | Aprovar com aviso | resposta 3A |
| 4 | Trava pelo hash gravado no start | Só orientação da skill | resposta do usuário na conversa |
| 5 | Tipo fix com o adendo na mesma spec | Duas specs | inferência A |
| 6 | Gravação só dos arquivos alterados | Corrigir o `split_task_text` para títulos | inferência B |
| 7 | Arquiteto escreve o técnico no detalhamento | Planejador escreve os dois | inferência C |
| 8 | Validação da pasta volta a checar os documentos | Só o approve checa | inferência D |
| 9 | Os dois documentos precisam mudar | Basta um | inferência E |
| 10 | Sem reparo automático | Comando de reparo | inferência F |

## Pendências

| # | Pendência | Opções em aberto |
|---|---|---|
| - | Nenhuma | - |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
