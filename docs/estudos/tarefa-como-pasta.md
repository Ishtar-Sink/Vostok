# Estudo da tarefa como pasta

Data do estudo: 2026-11-01. Versão: `vostok` 0.2.6. Subsídio da SPEC-023.

## Escopo e método

O estudo responde três perguntas:

1. Quais são os prós e os contras de transformar cada tarefa numa pasta com arquivos dedicados.
2. Onde estão hoje o consumo de tokens e o tempo de execução ligados à tarefa.
3. Como a nova estrutura pode reduzir esse consumo e funcionar com modelos mais simples.

A base:

- as 111 tarefas arquivadas em `vostok/work/.archive/` (SPEC-002 a SPEC-022), medidas por seção;
- o template `packages/harness/templates/task/task.md`;
- o código que lê e grava a tarefa (`apps/cli/src/commands/task.rs`, `plan.rs` e `finalize.rs`);
- o fluxo das skills `approve-plan`, `execute-task`, `finish-task` e `finalize-spec`.

As contas de tokens usam cerca de 4 caracteres por token. É uma estimativa, não uma medição do `/usage`.

## Como é hoje

- **Arquivo:** um arquivo `TASK-NN.md` por tarefa, com front-matter (estado), `## Execução` (Ler, Mudar, Passos, Baseline, Verificação, Evidências, Concluída quando), `## Descrição` (Resumo executivo, Detalhes técnicos, Card), `## Registro de execução` e `## Histórico de ajustes`.
- **Evidências:** ficam em `tasks/<SPEC>/evidence/<TASK>/`, com `baseline/`, `validation/` e os arquivos de evidência. É uma pasta irmã das tarefas, não a pasta da tarefa.
- **Quem lê o quê:**

| Quem | Lê | Como |
|---|---|---|
| Agente que executa | front-matter + `## Execução` + bloco "Modelo" | `vostok task load` (já recorta a seção) |
| Subagente ou sessão que executa | regras, persona, roteiro, a mesma `## Execução` e o registro | `vostok task brief` |
| Finalização da tarefa | Verificação, Baseline, Evidências, Concluída quando | o próprio `vostok`, sem o agente ler o arquivo |
| Finalização da spec | a tarefa inteira e as evidências | `vostok finalize sources` e `build` |
| Pessoas | o arquivo inteiro, ou o HTML no fim da spec | leitura direta |

## Medidas

**Tamanho das 111 tarefas arquivadas**

| Medida | Caracteres | Tokens (estimativa) |
|---|---|---|
| Mediana | 6.475 | ~1.600 |
| Média | 8.500 | ~2.100 |
| Maior | 28.254 | ~7.000 |

**Peso médio por seção**

| Seção | Caracteres | Parte da tarefa | Quem usa |
|---|---|---|---|
| Passos | 2.592 | 30% | agente |
| Concluída quando | 767 | 9% | agente e pessoas |
| Registro de execução | 678 | 8% | `vostok` e pessoas |
| Ler | 606 | 7% | agente |
| Detalhes técnicos | 606 | 7% | pessoas |
| Mudar | 601 | 7% | agente e `vostok` |
| Front-matter | 585 | 7% | `vostok` |
| Evidências | 547 | 6% | agente e `vostok` |
| Card | 482 | 6% | pessoas |
| Verificação | 420 | 5% | agente e `vostok` |
| Resumo executivo | 312 | 4% | pessoas |
| Baseline | 151 | 2% | agente e `vostok` |
| Histórico de ajustes | 121 | 1% | `vostok` e pessoas |

- A parte do agente (`## Execução`) tem cerca de 5.700 caracteres, ou 67% da tarefa. A parte das pessoas (`## Descrição`) tem cerca de 1.400 caracteres (16%). Estado e histórico somam 17%.
- **O agente que executa já não lê a parte das pessoas.** O `vostok task load` recorta só a `## Execução`. O `brief` acrescenta as regras e a persona (cerca de 4.300 caracteres) e o registro.

**Onde o custo realmente está**

| Momento | O que custa | Ordem de grandeza |
|---|---|---|
| Detalhamento (`approve-plan`) | **escrever** todas as tarefas da spec, na sessão principal | SPEC-020: 54.700 caracteres (~14 mil tokens de saída) em 7 tarefas; SPEC-022: 28.100 em 5 |
| Execução | ler a `## Execução` mais o `brief` | ~2.500 tokens por tarefa, uma vez |
| Finalização da tarefa | quase nada no agente: o `vostok` roda e compara | baixo |
| Finalização da spec | ler as fontes e escrever o conteúdo | médio, uma vez por spec |

- Tokens de saída custam mais e demoram mais que os de entrada. O maior custo ligado à tarefa é **escrever** o detalhamento, não lê-lo.
- **Separar a tarefa em arquivos, sozinho, quase não muda o custo.** O texto é o mesmo, e a leitura do agente já é recortada.
- **Pedir executivo e técnico "mais detalhados" aumenta o custo**, se eles forem escritos pelo mesmo agente no mesmo momento do detalhamento. Esse é o principal conflito do pedido com a eficiência.

## Prós

1. **Leitura humana:** cada arquivo tem um assunto e um leitor. O executivo e o técnico podem crescer sem atrapalhar o agente, e uma pessoa abre só o que interessa.
2. **Tudo da tarefa num lugar:** evidências, baseline, validação e documentos na pasta da tarefa. Arquivar, copiar ou revisar uma tarefa fica direto, sem caçar a pasta de evidências solta.
3. **Carregamento sob medida:** o `vostok` passa a montar o contexto juntando arquivos, em vez de recortar seções de um markdown longo. Isso fica mais simples e mais difícil de quebrar com um título fora do lugar.
4. **Ajustes menores:** um ajuste nos passos regrava um arquivo pequeno, e o diff mostra exatamente o que mudou.
5. **Arquivos menores para modelos simples:** um modelo pequeno segue melhor um arquivo curto, com um só propósito, do que um documento de 8 mil caracteres com partes que ele precisa ignorar.
6. **Estado fora do texto:** o estado (status, resultados, tentativas, hashes) pode sair do front-matter para um arquivo de dados que só o `vostok` lê e grava. Hoje são 36 chamadas de `frontmatter_set` no markdown que as pessoas leem.
7. **Escrita em paralelo:** com arquivos separados, partes diferentes da tarefa podem ser escritas por agentes ou modelos diferentes, sem conflito no mesmo arquivo.

## Contras

1. **Mudança grande no código:** o formato atual está espalhado por `task.rs` (4.860 linhas), `plan.rs` e `finalize.rs`. São 40 chamadas de `subsection`, 23 de `markdown_section`, 16 de `evidence_dir`, 14 de `markdown_files` (que lista tarefas como arquivos `.md`) e 12 de `content_hash`, além dos 126 casos golden do grupo `task`.
2. **Compatibilidade:** a v0.1.x já é usada num projeto real (`~/rd/app`), então tarefas no formato antigo precisam ser lidas ou migradas. São dois formatos no código por um tempo, ou um comando de migração.
3. **Hash e aprovação:** hoje o hash é de um arquivo. Com uma pasta, é preciso definir quais arquivos entram na aprovação. Os de execução sim. Os de pessoas talvez não, para que melhorar o texto não exija aprovar de novo.
4. **Edição manual sem registro:** o `adjust record-manual` detecta mudança num arquivo. Numa pasta, ele precisa conferir vários arquivos e dizer qual mudou.
5. **Mais arquivos para navegar e commitar:** uma spec de 7 tarefas passa de 7 arquivos para algo como 35 a 50. Sem um índice, a pessoa não sabe por onde começar.
6. **Risco de divergência:** a mesma informação em dois arquivos (por exemplo, o critério no arquivo do agente e no card) pode ficar diferente. Hoje ela fica junta e diverge menos.
7. **Evidências grandes junto do texto:** as evidências da SPEC-022 somam 396 KB. Dentro da pasta da tarefa, ler ou listar a pasta fica mais pesado se nada separar o que é texto do que é artefato.

## Como otimizar a eficiência

As medidas abaixo valem com ou sem a pasta; com a pasta, ficam mais fáceis de fazer.

| ID | Medida | Ganho | Custo |
|---|---|---|---|
| E1 | **Derivar no `vostok` o que é repetido:** card (título, tipo, rótulos, estimativa, bloqueios e critérios vêm do plano e do "Concluída quando"), índice da pasta e cabeçalhos | Cerca de 6% da tarefa deixa de ser escrita pelo agente; nada diverge | Baixo |
| E2 | **Escrever os documentos para pessoas depois, e não no detalhamento:** o executivo e o técnico detalhados de cada tarefa são escritos na finalização da tarefa, quando o resultado é conhecido, a partir dos passos, do diff e das evidências | O detalhamento fica menor; os documentos descrevem o que foi feito, não o que se planejou | Médio |
| E3 | **Modelo simples para o texto de pessoas:** o executivo e o técnico são escritos por um subagente com o perfil `simples`, a partir de um resumo montado pelo `vostok` | Saída longa num modelo barato; a sessão principal não gasta saída com prosa | Médio; depende de um briefing bom |
| E4 | **Estado num arquivo de dados** (`estado.yml`), lido e gravado só pelo `vostok`, e registro e histórico gerados como tabela só para leitura | O agente nunca lê nem reescreve estado; menos risco de editar o que não deve | Médio |
| E5 | **Briefing por papel:** quem executa recebe Ler, Mudar, Passos e Concluída quando; quem finaliza não lê a tarefa (o `vostok` já faz); quem ajusta recebe só o arquivo ajustado | Leitura mínima em cada etapa | Baixo com a pasta |
| E6 | **Passos em formato fixo para modelos simples:** cada passo com arquivo, ação e conferência, numa lista curta; detalhes de desenho no técnico, fora do arquivo de passos | Modelos menores seguem melhor; menos releitura | Baixo, na skill e no template |
| E7 | **Hash só dos arquivos de execução:** melhorar o texto de pessoas não pede nova aprovação nem invalida a execução | Menos idas e voltas | Baixo |
| E8 | **Artefatos separados do texto na pasta:** subpasta `evidencias/` com baseline, validação e arquivos, e os arquivos de texto na raiz da pasta | Listar e ler a tarefa continua leve | Baixo |

**Velocidade**

- O tempo da execução vem quase todo dos comandos (testes, baseline), não da leitura da tarefa.
- O que acelera de fato:
  - menos saída no detalhamento (E1, E2);
  - menos idas e voltas na aprovação (E7);
  - menos releitura por modelos menores (E5, E6).

**Modelos mais simples**

- Funcionam melhor com instruções curtas, um propósito por arquivo, formato fixo e sem decisões em aberto.
- A pasta permite dar ao modelo simples só o arquivo de passos e a lista de critérios, com o roteiro do `brief`.
- O que um modelo simples não deve fazer é decidir desenho ou escrever a estratégia de verificação. Isso continua no detalhamento, com o modelo mais forte.

## Alternativas

| ID | Alternativa | O que muda | Prós | Contras |
|---|---|---|---|---|
| P1 | **Só mover as evidências** para `TASK-NN/evidencias/`, mantendo `TASK-NN/TASK-NN.md` | Tudo da tarefa num lugar | Mudança pequena; resolve a pasta solta | Não melhora a leitura nem o carregamento |
| P2 | **Separar só pessoas de agente:** `tarefa.md` (execução, para o agente) e `descricao.md` (executivo, técnico e card) | Leitura humana à parte | Mudança média; a parte do agente fica igual | O texto de pessoas continua escrito no detalhamento |
| P3 | **Pasta completa:** um arquivo por assunto (passos, verificação e evidências, critérios, executivo, técnico), estado em dados, evidências na pasta e índice gerado | Tudo do pedido | Maior ganho de leitura e de carregamento | Maior mudança e migração |
| P4 | **P3 com E1, E2, E3, E4 e E7** | Pasta completa, com o texto de pessoas escrito na finalização da tarefa por um modelo simples, e o card e o índice gerados | Atende ao pedido e **reduz** a saída do detalhamento em vez de aumentá-la | O mesmo custo de P3, mais o fluxo novo da finalização da tarefa |

Uma possível pasta para P3/P4 (nomes a decidir no refinamento):

```text
tasks/SPEC-023/TASK-01/
  README.md            índice gerado: título, status, onda e links
  estado.yml           estado, resultados, hashes (só o vostok)
  execucao.md          Ler, Mudar, Passos (agente)
  verificacao.md       Baseline, Verificação, Evidências (agente e vostok)
  criterios.md         Concluída quando (agente e pessoas)
  executivo.md         para pessoas, escrito na finalização da tarefa
  tecnico.md           para pessoas, escrito na finalização da tarefa
  registro.md          registro e histórico de ajustes (gerado)
  evidencias/          baseline/, validation/ e os arquivos de evidência
```

## Recomendação

- **P4.** A pasta entra (P3), e o detalhamento das pessoas muda de lugar e de modelo: ele passa para a finalização da tarefa (E2), com um modelo simples (E3).
  - Assim o pedido de executivo e técnico mais detalhados não aumenta o custo do detalhamento.
  - Os documentos passam a descrever o que foi feito de verdade.
- E1, E4, E5, E7 e E8 entram junto: são baratas com a pasta.
- E6 entra na `approve-plan` e no template.
- **Compatibilidade:** o `vostok` lê os dois formatos e oferece `vostok task migrate`, que converte as tarefas de uma spec ainda não finalizada. As specs arquivadas ficam como estão.
- Por ser grande, a spec deve ser planejada em ondas que não deixam o harness quebrado no meio:
  1. leitura dos dois formatos;
  2. escrita no formato novo;
  3. migração;
  4. skills e documentos.

## Perguntas para o refinamento

1. Qual alternativa: P1, P2, P3 ou P4?
2. O executivo e o técnico detalhados de cada tarefa são escritos no detalhamento, como hoje, ou na finalização da tarefa (E2)? Por um modelo simples (E3)?
3. O estado sai do markdown para um arquivo de dados (E4)?
4. Os arquivos de pessoas entram no hash da aprovação (E7)?
5. Compatibilidade: ler os dois formatos e migrar só as specs em andamento, ou migrar tudo?
6. Os documentos da finalização da spec (executivo, técnico e evidências) passam a juntar os documentos de cada tarefa?

## Fontes

- `vostok/work/.archive/*/tasks/TASK-*.md`: tamanhos por seção (111 tarefas).
- `apps/cli/src/commands/task.rs` (`load_text`, `brief`, `evidence_dir`, `find_task`, `content_hash`).
- `apps/cli/src/commands/plan.rs` e `apps/cli/src/commands/finalize.rs`.
- `packages/harness/templates/task/task.md` e `packages/harness/config/tasks.yml`.
- `docs/estudos/consumo-de-tokens.md`: medição de consumo da SPEC-019.
