---
id: SPEC-023
title: "Tarefa como pasta com arquivos dedicados"
status: planned
author: "Victor Batalha"
created_at: 2026-11-01T11:34:34Z
updated_at: 2026-11-01T11:58:10Z
tags: []
type: improvement
source: "draft/SPEC-023-tarefa-como-pasta-com-arquivos-dedicados.md"
refined_at: 2026-11-01T11:46:21Z
adjustments: 1
adjusted_at: 2026-11-01T11:52:57Z
content_hash: e9ee9d33f2a1f725f539a571a767b8c38a6c0a7f720f82a19dff594365cfa4a5
approved_at: 2026-11-01T11:53:02Z
approved_hash: e9ee9d33f2a1f725f539a571a767b8c38a6c0a7f720f82a19dff594365cfa4a5
---

# Tarefa como pasta com arquivos dedicados - Especificação Refinada

Cada tarefa passa a ser uma pasta, com arquivos dedicados ao agente (execução, verificação, critérios), arquivos para pessoas (executivo e técnico detalhados, escritos na finalização da tarefa por um modelo simples), estado em dados e as evidências numa subpasta da própria tarefa. Estudo de apoio: `docs/estudos/tarefa-como-pasta.md`.

## Objetivo

O arquivo de cada tarefa ficou grande (mediana de 6.475 caracteres, maior de 28.254) e mistura o que o agente executa, o que as pessoas leem e o estado. O objetivo é separar esse conteúdo numa pasta por tarefa, com leitura humana mais detalhada, carregamento sob medida para o agente e as evidências junto da tarefa, sem aumentar o custo do detalhamento.

## Estado atual

- Cada tarefa é um arquivo `vostok/work/tasks/<SPEC>/TASK-NN.md`, criado pelo `vostok plan approve create-tasks` a partir de `packages/harness/templates/task/task.md`, com front-matter (estado), `## Execução` (Ler, Mudar, Passos, Baseline, Verificação, Evidências, Concluída quando), `## Descrição` (Resumo executivo, Detalhes técnicos, Card), `## Registro de execução` e `## Histórico de ajustes`.
- As evidências ficam em `vostok/work/tasks/<SPEC>/evidence/<TASK>/` (`evidence_dir` em `config/tasks.yml`), com `baseline/`, `validation/` e os arquivos de evidência.
- O `vostok task load` e o `vostok task brief` recortam a seção `## Execução`; os demais comandos leem subseções pelo título (`subsection`, `markdown_section`) e gravam o estado com `frontmatter_set` no próprio markdown.
- O texto para pessoas (resumo executivo, detalhes técnicos e card) é escrito no detalhamento (`approve-plan`), antes da execução, junto com tudo o mais: a SPEC-020 gerou 54.700 caracteres em 7 tarefas.
- O hash de aprovação (`approved_hash`) e o de edição manual (`content_hash`) são de um único arquivo.
- A finalização da spec (`vostok finalize sources`, `build` e `archive`) lê o arquivo da tarefa e a pasta `evidence/` e arquiva os dois em `vostok/work/.archive/<SPEC>/tasks/`.

## Escopo

### Dentro do escopo

- Tarefa como pasta `vostok/work/tasks/<SPEC>/TASK-NN/`, com os arquivos do contrato.
- Estado da tarefa em `estado.yml`, lido e gravado só pelo `vostok`.
- Evidências, baseline e validação na subpasta `evidencias/` da tarefa.
- `README.md` da tarefa gerado pelo `vostok`, com o card.
- Executivo e técnico de cada tarefa escritos na finalização da tarefa, por um subagente de perfil `simples`, a partir do briefing `vostok task describe`.
- Hash de aprovação e de edição manual só dos arquivos de execução.
- `vostok task load` e `vostok task brief` montados a partir dos arquivos, com `--part` para carregar um arquivo.
- Leitura dos dois formatos e `vostok task migrate` para as specs não arquivadas.
- Finalização da spec lendo e arquivando as pastas, com os documentos de cada tarefa entre as fontes.
- Passos em formato fixo na orientação da `approve-plan` e no template.
- Skills `approve-plan`, `adjust-task`, `approve-task`, `execute-task`, `finish-task` e `finalize-spec` atualizadas.

### Fora do escopo

- Migrar as specs arquivadas em `vostok/work/.archive/`.
- Mudar o formato da spec, do plano ou da memória.
- Validar o formato fixo dos passos por comando (fica como orientação).
- Mudar o conteúdo das seções do agente (Ler, Mudar, Passos, Baseline, Verificação, Evidências e Concluída quando).
- Painel web, servidor e MCP.

## Requisitos funcionais

| ID | Requisito |
|---|---|
| RF-01 | O `vostok plan approve create-tasks` deve criar cada tarefa como a pasta `TASK-NN/`, com `execucao.md`, `verificacao.md`, `criterios.md`, `executivo.md`, `tecnico.md`, `estado.yml` e `registro.md`, a partir dos templates de `templates/task/`. |
| RF-02 | O estado da tarefa (os campos do front-matter de hoje) deve ficar em `estado.yml`, gravado só pelo `vostok`; os arquivos markdown não têm front-matter de estado. |
| RF-03 | O registro de execução e o histórico de ajustes devem ficar em `registro.md`, gravado só pelo `vostok`. |
| RF-04 | O `vostok` deve gerar o `README.md` da tarefa (título, status, onda, dependências, links para os arquivos e o card) a cada mudança de estado; o card é montado do plano e do `criterios.md`, sem texto escrito pelo agente. |
| RF-05 | As evidências, o baseline e a validação devem ficar em `TASK-NN/evidencias/` (chave `task_evidence_dir`), e nenhuma pasta `evidence/` solta é criada para tarefas no formato novo. |
| RF-06 | O hash de aprovação e o de edição manual devem cobrir só `execucao.md`, `verificacao.md` e `criterios.md`; mudar `executivo.md` ou `tecnico.md` não pede nova aprovação. |
| RF-07 | O `vostok task adjust record-manual` deve dizer qual arquivo da pasta mudou. |
| RF-08 | O `vostok task load` e o `vostok task brief` devem montar o texto do agente a partir de `estado.yml`, `execucao.md`, `verificacao.md` e `criterios.md`, sem os arquivos de pessoas; `--part <arquivo>` imprime só um arquivo. |
| RF-09 | O `vostok task describe <SPEC> <TASK>` deve imprimir o briefing para escrever o executivo e o técnico: o resumo da tarefa, os passos, os caminhos mudados com o diff resumido, as evidências e o registro. |
| RF-10 | O `vostok task finish` deve recusar a tarefa enquanto `executivo.md` ou `tecnico.md` tiverem variáveis do template. |
| RF-11 | O `vostok` deve ler tarefas no formato antigo (`TASK-NN.md` com `evidence/TASK-NN/`) com o mesmo comportamento de hoje. |
| RF-12 | O `vostok task migrate <SPEC>` deve converter todas as tarefas de uma spec não arquivada para o formato novo, movendo as evidências para `evidencias/`, preservando estado, registro, histórico e hashes válidos, e reaproveitando o resumo executivo e os detalhes técnicos existentes. |
| RF-13 | O `vostok finalize sources`, `build` e `archive` devem ler e arquivar as pastas das tarefas, e o `sources` deve listar o `executivo.md` e o `tecnico.md` de cada tarefa. |
| RF-14 | O `tecnico.html` da spec deve incluir o técnico de cada tarefa. |
| RF-15 | A `approve-plan` deve detalhar só os arquivos do agente e orientar os passos em formato fixo (caminho, ação e conferência por passo). |
| RF-16 | A `finish-task` deve, depois das evidências e antes do `vostok task finish`, disparar um subagente de perfil `simples` com o `vostok task describe` para escrever o `executivo.md` e o `tecnico.md`. |
| RF-17 | As skills `adjust-task`, `approve-task`, `execute-task` e `finalize-spec` devem citar os arquivos novos no lugar das seções do arquivo único. |

## Requisitos não funcionais

| ID | Requisito | Como verificar |
|---|---|---|
| RNF-01 | Compatibilidade: projetos com tarefas no formato antigo continuam funcionando sem migrar, inclusive os casos golden atuais. | Os 126 casos golden do grupo `task` passam sem atualização de saída. |
| RNF-02 | Eficiência: o detalhamento de uma tarefa no formato novo escreve menos texto que no formato antigo, porque o card, o executivo e o técnico saem do detalhamento. | Comparar o tamanho dos arquivos escritos pela `approve-plan` numa spec de teste com o template antigo. |
| RNF-04 | Comportamento igual nos dois formatos: os cenários de referência, gravados no formato atual antes de qualquer mudança, dão o mesmo resultado no formato de pasta e numa spec migrada (saída, código de saída, estado, registro e lista de evidências), sem contar os caminhos que mudam de formato. Os cenários cobrem o fluxo feliz e os de falha. | `cargo test --test scenarios`, com as fixtures de `apps/cli/tests/fixtures/scenarios/` (CT-16, CT-17). |
| RNF-03 | Sem dependência nova no `apps/cli` além das já usadas (o YAML usa o leitor de configuração existente). | `apps/cli/Cargo.toml` sem crate nova. |

## Regras de negócio

| ID | Regra |
|---|---|
| RN-01 | Uma spec tem todas as tarefas num só formato; o `vostok` recusa uma spec com os dois. |
| RN-02 | `estado.yml`, `registro.md` e `README.md` são gravados só pelo `vostok`; editar à mão é tratado como edição manual. |
| RN-03 | O executivo e o técnico de cada tarefa descrevem o que foi feito, e são escritos depois das evidências. |
| RN-04 | Specs arquivadas não são migradas. |

## Fluxos

### Fluxo principal

1. O `vostok plan approve create-tasks` cria as pastas das tarefas com os templates (RF-01).
2. A `approve-plan` preenche `execucao.md`, `verificacao.md` e `criterios.md`; o `vostok` gera o `README.md` com o card (RF-04, RF-15).
3. A aprovação grava o hash dos arquivos de execução em `estado.yml` (RF-06).
4. O `vostok task start` coleta o baseline em `evidencias/baseline/` (RF-05).
5. A execução lê a tarefa pelo `vostok task brief` (RF-08).
6. A finalização roda testes, regressão e evidências, com as saídas em `evidencias/` (RF-05).
7. A `finish-task` dispara o subagente `simples`, que lê o `vostok task describe` e escreve `executivo.md` e `tecnico.md` (RF-09, RF-16).
8. O `vostok task finish` confere os documentos e conclui (RF-10).
9. Na finalização da spec, o `vostok finalize` lê as pastas, inclui o técnico de cada tarefa no `tecnico.html` e arquiva as pastas (RF-13, RF-14).

### Fluxos alternativos

- **FA-01 (passo 1), spec planejada no formato antigo:** o `vostok` segue no formato antigo (RF-11); o usuário pode rodar `vostok task migrate <SPEC>` (RF-12).
- **FA-02 (passo 7), subagente indisponível:** o Coordenador escreve os dois documentos na própria sessão a partir do mesmo `vostok task describe`.
- **FA-03 (passo 2), ajuste de um arquivo:** a `adjust-task` lê só o arquivo ajustado (`--part`) e o `record-manual` cita o arquivo que mudou (RF-07, RF-08).

## Casos de borda

| ID | Situação | Comportamento esperado | Requisitos |
|---|---|---|---|
| CB-01 | Spec com tarefas nos dois formatos | O comando recusa e indica `vostok task migrate` | RF-12 |
| CB-02 | `vostok task migrate` numa spec já no formato novo | Nada muda; o comando informa | RF-12 |
| CB-03 | `vostok task migrate` numa tarefa em execução | Migra, mantendo status, baseline e resultados | RF-12 |
| CB-04 | Tarefa antiga com resumo executivo e detalhes técnicos preenchidos | `executivo.md` e `tecnico.md` recebem esse texto e o `finish` não pede para reescrever | RF-10, RF-12 |
| CB-05 | Edição à mão de `executivo.md` numa tarefa aprovada | Não pede nova aprovação | RF-06 |
| CB-06 | Edição à mão de `estado.yml` | Tratada como edição manual, citando o arquivo | RF-07 |
| CB-07 | Arquivo de execução ausente na pasta | O comando recusa citando o arquivo | RF-08 |
| CB-08 | `vostok task finish` com `tecnico.md` ainda no template | Recusa citando o arquivo | RF-10 |

## Contratos e interfaces

Pasta da tarefa:

```text
vostok/work/tasks/SPEC-023/TASK-01/
  README.md        gerado: título, status, onda, dependências, links e card
  estado.yml       estado (só o vostok)
  execucao.md      ### Ler, ### Mudar, ### Passos
  verificacao.md   ### Baseline, ### Verificação, ### Evidências
  criterios.md     ### Concluída quando
  executivo.md     para pessoas, escrito na finalização da tarefa
  tecnico.md       para pessoas, escrito na finalização da tarefa
  registro.md      ## Registro de execução e ## Histórico de ajustes (só o vostok)
  evidencias/      baseline/, validation/ e os arquivos de evidência
```

`estado.yml` (os campos do front-matter de hoje):

```yaml
id: TASK-01
spec: SPEC-023
title: "Leitura dos dois formatos"
effort: julgamento
depends_on: []
covers: [RF-11]
type: tarefa
labels: [SPEC-023, julgamento]
status: pending
content_hash: null
approved_hash: null
approved_at: null
adjustments: 0
adjusted_at: null
attempts: 0
tests_result: null
regression_result: null
evidence_result: null
model: null
```

Configuração (`config/tasks.yml`):

```yaml
task_files:
  state: estado.yml
  readme: README.md
  execution: execucao.md
  verification: verificacao.md
  criteria: criterios.md
  executive: executivo.md
  technical: tecnico.md
  record: registro.md
task_evidence_dir: evidencias
```

Comandos:

```bash
vostok task load <SPEC> <TASK> [--part execucao|verificacao|criterios|executivo|tecnico]
vostok task describe <SPEC> <TASK>
vostok task migrate <SPEC>
```

Saída do `vostok task migrate`:

```text
Tarefas de SPEC-022 migradas para pastas:
- TASK-01: evidências movidas para TASK-01/evidencias
- TASK-02: evidências movidas para TASK-02/evidencias
```

Passo em formato fixo (orientação da `approve-plan`):

```markdown
1. `apps/cli/src/commands/task.rs` (alterar): em `fn start`, marque `in_progress` antes do baseline. Confira: `cargo test --test task -- start_progress_`.
```

## Componentes afetados

| Caminho | Ação | Motivo |
|---|---|---|
| `apps/cli/src/commands/task.rs` | alterar | Leitura dos dois formatos, estado em `estado.yml`, registro, `README.md`, `load --part`, `describe`, `migrate`, hashes e evidências na pasta |
| `apps/cli/src/commands/plan.rs` | alterar | `create-tasks` em pastas e validação do plano com os arquivos da tarefa |
| `apps/cli/src/commands/finalize.rs` | alterar | Leitura e arquivamento das pastas e técnico de cada tarefa no `tecnico.html` |
| `apps/cli/tests/task.rs` | alterar | Testes do formato novo, da compatibilidade e da migração |
| `apps/cli/tests/plan.rs` | alterar | Testes do `create-tasks` em pastas |
| `apps/cli/tests/finalize.rs` | alterar | Testes da finalização com pastas |
| `apps/cli/tests/scenarios.rs` | criar | Cenários de referência, felizes e de falha, rodados nos dois formatos |
| `apps/cli/tests/fixtures/scenarios` | criar | Transcrições normalizadas gravadas no formato atual, antes da mudança |
| `packages/harness/config/tasks.yml` | alterar | Chaves `task_files` e `task_evidence_dir` |
| `packages/harness/templates/task/execucao.md` | criar | Template de Ler, Mudar e Passos |
| `packages/harness/templates/task/verificacao.md` | criar | Template de Baseline, Verificação e Evidências |
| `packages/harness/templates/task/criterios.md` | criar | Template de Concluída quando |
| `packages/harness/templates/task/executivo.md` | criar | Template do executivo da tarefa |
| `packages/harness/templates/task/tecnico.md` | criar | Template do técnico da tarefa |
| `packages/harness/templates/task/readme.md` | criar | Template do `README.md` gerado |
| `packages/harness/skills/default/approve-plan/SKILL.md` | alterar | Detalhar só os arquivos do agente; passos em formato fixo |
| `packages/harness/skills/default/finish-task/SKILL.md` | alterar | Subagente `simples` para executivo e técnico |
| `packages/harness/skills/default/adjust-task/SKILL.md` | alterar | Ajuste por arquivo |
| `packages/harness/skills/default/approve-task/SKILL.md` | alterar | Arquivos novos na aprovação |
| `packages/harness/skills/default/execute-task/SKILL.md` | alterar | Arquivos novos na execução |
| `packages/harness/skills/default/finalize-spec/SKILL.md` | alterar | Documentos de cada tarefa entre as fontes |

## Premissas e dependências

- A v0.2.6 está publicada e é a base da mudança.
- O perfil `simples` existe em `config/models.yml` e aponta para um modelo mais barato.
- O leitor de configuração de `apps/cli/src/support.rs` (`config_get`, `config_list`) basta para o `estado.yml`.
- `vostok/work/.archive/` está fora do git e guarda as specs finalizadas como estão.

## Riscos

| Risco | Impacto | Mitigação |
|---|---|---|
| Mudança espalhada no `task.rs` (40 leituras por subseção, 36 gravações de estado) quebrar algo sem teste | alto | Uma camada única de leitura da tarefa com os dois formatos, entregue primeiro, com os goldens como rede |
| Dois formatos no código por muito tempo | médio | `vostok task migrate` e aviso nos comandos para specs no formato antigo |
| Texto do modelo simples fraco ou errado | médio | Briefing do `describe` com os fatos (diff, evidências, registro) e revisão do Coordenador antes do `finish` |
| Mais arquivos para navegar | baixo | `README.md` gerado como ponto de entrada |
| Divergência entre `criterios.md` e o card | baixo | Card gerado a partir do `criterios.md` (RF-04) |

## Casos de teste

| ID | Cenário | Passos | Resultado esperado | Requisitos |
|---|---|---|---|---|
| CT-01 | Criar tarefas em pastas | `vostok plan approve create-tasks` num plano | Pastas com os 7 arquivos e o `README.md` | RF-01, RF-04 |
| CT-02 | Estado fora do markdown | Aprovar e iniciar uma tarefa | `estado.yml` com status e hashes; markdown sem front-matter de estado | RF-02 |
| CT-03 | Registro gravado | Iniciar e finalizar | Linhas em `registro.md` | RF-03 |
| CT-04 | Card gerado | Mudar `criterios.md` e o status | `README.md` refeito com os critérios e o status | RF-04 |
| CT-05 | Evidências na pasta | `start`, `tests`, `evidence` | Arquivos em `TASK-01/evidencias/`, sem `evidence/` solta | RF-05 |
| CT-06 | Hash só da execução | Editar `executivo.md` de tarefa aprovada | `task start` não pede nova aprovação | RF-06 |
| CT-07 | Edição manual citada | Editar `passos` em `execucao.md` | `record-manual` cita `execucao.md` | RF-07 |
| CT-08 | Carregamento do agente | `vostok task load` e `--part tecnico` | Só os arquivos do agente; com `--part`, só o arquivo | RF-08 |
| CT-09 | Briefing dos documentos | `vostok task describe` numa tarefa validada | Passos, caminhos com diff, evidências e registro | RF-09 |
| CT-10 | Documentos obrigatórios | `vostok task finish` com `tecnico.md` no template | Recusa citando o arquivo | RF-10 |
| CT-11 | Formato antigo | Fluxo inteiro numa spec com `TASK-NN.md` | Comportamento atual; goldens passam | RF-11 |
| CT-12 | Migração | `vostok task migrate` numa spec com uma tarefa concluída e uma em execução | Pastas criadas, evidências movidas, estado e registro preservados, aprovação válida | RF-12 |
| CT-13 | Finalização com pastas | `finalize sources`, `build` e `archive` | Fontes com executivo e técnico de cada tarefa; pastas arquivadas | RF-13 |
| CT-14 | Técnico das tarefas no documento | `finalize build` | `tecnico.html` com o técnico de cada tarefa | RF-14 |
| CT-16 | Cenários de referência (antes) | Rodar no formato atual, antes de mudar o código, os cenários: fluxo completo até a finalização da spec; teste que falha até bloquear e a retomada; regressão que bloqueia, aceite com `--accept-regression` e acréscimo que passa; evidência que falha, `Requer:` ausente e tempo-limite; `start` recusado por dependência pendente e por tarefa mudada depois da aprovação; `finish` recusado por critério sem marcar e por resultado pendente; edição manual sem registro e ajuste em execução; `tests` sem baseline | Transcrição normalizada de cada cenário gravada em `apps/cli/tests/fixtures/scenarios/` | RNF-04 |
| CT-17 | Equivalência (depois) | Rodar os mesmos cenários no formato de pasta e numa spec migrada | Transcrição igual à fixture de cada cenário | RNF-04, RF-11, RF-12 |
| CT-15 | Orientação | Ler as seis skills | Arquivos novos, passos em formato fixo e subagente `simples` na `finish-task` | RF-15, RF-16, RF-17 |

## Critérios de aceite

- [ ] CA-01: Tarefas novas são pastas com os arquivos do contrato, estado em `estado.yml` e `README.md` gerado (RF-01, RF-02, RF-03, RF-04).
- [ ] CA-02: Evidências, baseline e validação ficam em `evidencias/` da tarefa (RF-05).
- [ ] CA-03: Só os arquivos de execução entram no hash, e a edição manual cita o arquivo (RF-06, RF-07).
- [ ] CA-04: O agente carrega só os arquivos dele, e `--part` carrega um arquivo (RF-08).
- [ ] CA-05: O executivo e o técnico são escritos na finalização da tarefa a partir do `describe`, e o `finish` exige os dois (RF-09, RF-10, RF-16).
- [ ] CA-06: Tarefas no formato antigo funcionam como hoje e migram com `vostok task migrate` (RF-11, RF-12).
- [ ] CA-07: A finalização da spec lê e arquiva as pastas, com o técnico de cada tarefa no `tecnico.html` (RF-13, RF-14).
- [ ] CA-09: Os cenários de referência, felizes e de falha, dão o mesmo resultado no formato atual, no formato de pasta e numa spec migrada (RNF-04, RF-11, RF-12).
- [ ] CA-08: As skills descrevem os arquivos novos e o formato fixo dos passos (RF-15, RF-17).

## Referências

| Tipo | Referência | Uso |
|---|---|---|
| documentação | `docs/estudos/tarefa-como-pasta.md` | Medidas, prós e contras, alternativas e recomendação P4 |
| arquivo | `packages/harness/templates/task/task.md` | Seções do formato atual |
| arquivo | `packages/harness/config/tasks.yml` | `evidence_dir`, nomes das seções e subseções |
| arquivo | `apps/cli/src/commands/task.rs` | `load_text`, `brief`, `find_task`, `evidence_dir`, `content_hash`, `frontmatter_set` |
| arquivo | `apps/cli/src/commands/plan.rs` | `create-tasks` e `task_profile` |
| arquivo | `apps/cli/src/commands/finalize.rs` | `task_evidence_dir`, `sources`, `archive` |
| arquivo | `apps/cli/tests/fixtures/golden/task` | 126 casos que fixam o formato atual |
| documentação | `docs/estudos/consumo-de-tokens.md` | Custo do detalhamento e dos subagentes |
| memória | ISS-001 | Bloqueio por regressão com diferença aceita, relacionado ao fluxo da finalização |

## O que mudou

| # | Assunto | Estado no Original | Estado Refinado |
|---|---|---|---|
| 1 | Estrutura | Tarefa em pasta, quebrada em arquivos | Sete arquivos com nomes e conteúdo definidos, mais o `README.md` gerado |
| 2 | Executivo e técnico | Mais detalhados | Escritos na finalização da tarefa por um subagente `simples` |
| 3 | Arquivos do agente | Passos e critérios | `execucao.md`, `verificacao.md` e `criterios.md`, com passos em formato fixo |
| 4 | Evidências | Dentro da pasta da tarefa | Subpasta `evidencias/` |
| 5 | Eficiência | Melhor carregamento | Card e índice gerados, estado em dados, hash só da execução e `load --part` |
| 6 | Compatibilidade | Não tratada | Leitura dos dois formatos e `vostok task migrate` |
| 7 | Garantia de comportamento | Não tratada | Cenários de referência, felizes e de falha, gravados antes e comparados depois nos dois formatos |

## Registro de decisões

| # | Decisão | Alternativa descartada | Motivo |
|---|---|---|---|
| 1 | Pasta completa com texto de pessoas na finalização da tarefa (P4) | P1, P2 e P3 | resposta do usuário ("vou seguir com suas recomendações") |
| 2 | Executivo e técnico escritos depois das evidências, por um subagente `simples` | Escrever no detalhamento | resposta do usuário (E2, E3) |
| 3 | Estado em `estado.yml` | Front-matter no markdown | resposta do usuário (E4) |
| 4 | Arquivos de pessoas fora do hash de aprovação | Hash da pasta inteira | resposta do usuário (E7) |
| 5 | Evidências na subpasta `evidencias/` | Evidências na raiz da pasta | resposta do usuário (E8) |
| 6 | Ler os dois formatos e migrar só specs não arquivadas | Migrar tudo | resposta do usuário (recomendação do estudo) |
| 7 | `tecnico.html` da spec inclui o técnico de cada tarefa | Só as fontes do agente | inferência A |
| 8 | Nomes dos arquivos em português, configuráveis em `task_files` | Nomes fixos no código | inferência B |
| 9 | Os títulos `###` das subseções continuam dentro de cada arquivo | Arquivos sem títulos | inferência C |
| 10 | `registro.md` separado do `estado.yml` | Registro dentro do YAML | inferência D |
| 11 | Uma spec num só formato; migração da spec inteira | Migrar por tarefa | inferência E |
| 12 | Formato fixo dos passos só como orientação | Validação por comando | inferência F |
| 13 | `vostok task describe` como briefing do subagente | O subagente ler a pasta inteira | inferência G |
| 14 | Teste dedicado com antes e depois, incluindo cenários de falha | Confiar só nos casos golden e nos testes por tarefa | pedido do usuário no planejamento |

## Pendências

| # | Pendência | Opções em aberto |
|---|---|---|
| - | Nenhuma | - |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
| 1 | 2026-11-01 | agente | nova RNF-04 de comportamento igual nos dois formatos; novos CT-16 (cenários de referência antes) e CT-17 (equivalência depois), com cenários de falha; novo CA-09; componentes tests/scenarios.rs e fixtures/scenarios |
