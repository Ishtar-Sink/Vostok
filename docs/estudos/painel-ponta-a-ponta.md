# Estudo da operação de ponta a ponta pelo painel

Data do estudo: 2026-11-03. Versão: `vostok` 0.3.0. Primeira tarefa da SPEC-032; subsídio da SPEC-033.

## Pergunta

O que falta para operar um projeto com o Vostok inteiramente pelo painel, sem interagir com o chat do agente, e como fazer isso sem expandir muito os mecanismos que já existem?

## Resposta curta

Dá para chegar lá com o caminho que o usuário prefere: o pedido em loop. O bloqueio principal não é técnico de desempenho, e sim de conversa: hoje, quando uma skill precisa perguntar algo ao usuário, a `process-requests` recusa o pedido. Por isso, criar e refinar specs pelo painel não funciona, e é a lacuna que a SPEC-033 precisa fechar primeiro.

As medições mostram que ler arquivos não é um problema de tempo. Listar 10.000 pedidos leva 59 ms, e ler 7.000 linhas de registro, 7 ms. O problema é de volume para o agente e para o usuário: a listagem sem filtro de 10.000 pedidos imprime 604 KB, o que custa caro no contexto do agente, e uma pasta com milhares de arquivos confunde quem a abre. Separar pedidos e perguntas em pastas por estado resolve as duas coisas com uma mudança pequena.

Todas as fontes de atividade cabem com folga no limite de 500 ms da `GET /api/atividades`, então a TASK-05 da SPEC-032 pode seguir sem ajuste.

## Como foi feito

- Leitura do código do painel (`apps/cli/src/painel/`), dos pedidos (`apps/cli/src/commands/pedidos.rs`), da trava (`apps/cli/src/aval.rs`), da skill `process-requests` e do catálogo de skills.
- Medições num projeto descartável criado com `vostok init` no scratchpad, nunca em `vostok/work/` do ai-hub:
  - `vostok pedidos listar` com 10, 100, 1.000 e 10.000 pedidos, mediana de 5 execuções;
  - `GET /api/visao` do `vostok painel` com 70, 700 e 7.000 linhas de registro de execução, mediana de 5 requisições;
  - leitura de um `avais.log` com 1.000, 10.000 e 100.000 linhas, média de 5 leituras (script em Python, como estimativa do custo de leitura do arquivo).
- Volume real do ai-hub, o projeto com mais histórico: 30 specs arquivadas, 167 tarefas arquivadas e cerca de 1.190 linhas de registro de execução.
- Nem o `claude` nem o `codex` estão no PATH do ambiente do estudo, então o chat dentro do painel foi avaliado pelo desenho e pela documentação das ferramentas, sem execução.

## Técnicas para operar sem o chat

| Técnica | Custo de implementação | Depende de o agente estar rodando | Tempo até a resposta | O que muda no `vostok` e nas skills |
|---|---|---|---|---|
| Pedido em loop (hoje) | Baixo: já existe | Sim: um chat com `/process-requests 1m` ou o `vostok pedidos vigiar` | Até o intervalo do loop (1 min no Claude Code, 15 s no `vigiar`) mais a execução | Perguntas do agente pelo painel; mais ações de pedido |
| Disparo sob demanda | Médio | Não: o `vostok painel` dispara `codex exec` ou `claude -p` quando grava o pedido | Só a execução | Um disparador no painel, com o mesmo código do `vigiar`; as mesmas perguntas |
| Chat dentro do painel | Alto | Não, mas o painel mantém um processo do agente por conversa | Imediato, com resposta em fluxo | Um canal de mensagens nos dois sentidos, sessão do agente por conversa, permissões e custo por mensagem |

**Pedido em loop.** É o caminho preferido e o mais barato: as 11 ações de pedido já existem, a trava de aval já protege as ações sensíveis e a `process-requests` já encadeia a skill certa. O custo é que alguém precisa deixar o loop rodando. No Claude Code, cada volta do `/loop` é uma rodada do modelo com todo o contexto da conversa, então um loop ocioso ainda gasta tokens; o `vigiar` do Codex não tem esse custo ocioso, porque só chama o agente quando há pedido.

**Disparo sob demanda.** É uma variação do loop que elimina a espera e o custo ocioso: o painel chama o agente em modo não interativo assim que grava o pedido, como o `vigiar` já faz com o `codex exec --full-auto`. Ela reaproveita o `process_with_codex` de `pedidos.rs` e a mesma skill. O risco é o painel passar a disparar um agente com permissão de escrita por conta própria; isso precisa de uma chave de configuração desligada por padrão, como a `approval_lock`.

**Chat dentro do painel.** É viável no desenho, mas é o mecanismo mais caro. O WebSocket do painel hoje é escrito à mão e só manda mensagens curtas do servidor para o navegador (SPEC-029), então seria preciso um canal nos dois sentidos, guardar a sessão do agente entre mensagens e mostrar a resposta em fluxo. Também traz para o painel decisões que hoje ficam na ferramenta do usuário: autenticação, permissões e custo por mensagem. Não foi possível medir, porque nenhuma das duas CLIs está instalada no ambiente do estudo. A recomendação é não fazer na v0.4.0.

**Recomendação.** Pedido em loop, como base, e o disparo sob demanda como opção da SPEC-033, desligado por padrão. O chat dentro do painel fica para depois, se o loop não bastar.

## Processos a operar pelo painel

Sem as skills do próprio harness (report-harness-bug, suggest-harness-improvement, suggest-harness-feature, validate-harness-bug, create-skill) e as de configuração inicial (init-harness, adjust-rules, customize-documents), sobram 14 skills.

| Skill | O que o painel faz hoje | O que falta |
|---|---|---|
| create-spec | Nada | Pedido `criar-spec` com o texto do assunto |
| refine-spec | Nada | Pedido `refinar-spec` e as perguntas do agente pelo painel |
| adjust-spec | Pedido `ajustar-spec` | Perguntas do agente pelo painel |
| approve-spec | Aprova direto | Nada |
| plan-tasks | Pedido `planejar-spec` | Nada |
| adjust-plan | Pedido `ajustar-plano` | Perguntas do agente pelo painel |
| approve-plan | Pedido `aprovar-plano`, com aval | Nada |
| approve-task | Aprova direto | Nada |
| adjust-task | Pedido `ajustar-tarefa` | Perguntas do agente pelo painel |
| execute-task | Pedido `executar-tarefa` | Execução de uma onda inteira por pedido |
| finish-task | Pedidos `finalizar-tarefa` e `desbloquear-tarefa` | Conferir prints pelo painel (hoje a `--reviewed` exige o usuário no chat) |
| finalize-spec | Pedido `finalizar-spec`, com aval | Nada |
| approve-commit | Pedido `fazer-commit`, com aval | Liberar ou desfazer os commits apontados pelo `vostok commit check` |
| process-requests | É o próprio loop | Abrir pergunta em vez de recusar quando a skill pergunta |

Das 14, cinco esbarram no mesmo ponto: a skill pergunta e o pedido é recusado. Duas têm lacunas próprias: a conferência de prints na `finish-task` e a liberação de commits na `approve-commit`.

## Status das entidades

| Entidade | Onde o status fica hoje | Mudanças que não são gravadas | Fluxo existente onde gravar |
|---|---|---|---|
| Spec | `status` e datas (`refined_at`, `approved_at`) no front-matter; ajustes no "Histórico de ajustes" | A passagem para `approved`, `planned`, `implemented` e `archived` não tem linha própria, só a data no front-matter | Os comandos que já mudam o status (`spec approve`, `plan approve mark-planned`, `task finish`, `finalize archive`) acrescentam uma linha ao histórico da spec |
| Plano | Existência do arquivo e "Histórico de ajustes" | Criação e aprovação: o arquivo some quando o plano é aprovado | `plan create` e `plan approve mark-planned` gravam a linha no histórico da spec |
| Tarefa | `status` no `estado.yml` e o "Registro de execução" | A conferência do `vostok task check` (a SPEC-032 resolve, RF-03) | Já coberto |
| Pedido | `estado` e `atualizado_em` no front-matter | As transições: só o último estado fica | `vostok pedidos iniciar/concluir/recusar` acrescentam uma linha a uma seção "Histórico" do próprio pedido |
| Pergunta | `status` no front-matter (o painel lê `aberta`) | Tudo: nada grava perguntas hoje | O comando novo de perguntas (abaixo) |

Nenhuma dessas mudanças exige mecanismo novo: são linhas a mais em arquivos que os mesmos comandos já gravam.

## Perguntas do agente

Hoje o painel lê `work/perguntas/*.md` e tem a ação `responder-pergunta`, mas nenhum comando nem skill grava uma pergunta. Pior: a `process-requests` manda recusar o pedido quando a skill para com uma pergunta. É por isso que refinar uma spec pelo painel, que sempre pergunta, não funciona.

Proposta, sem estado novo de pedido:

1. Um subcomando `vostok perguntas abrir <PEDIDO>`, com o texto pela entrada padrão (as perguntas e as opções, no formato do template `question`), grava `work/perguntas/P-NNN.md` com `status: aberta`, o pedido de origem, a spec e a tarefa.
2. A `process-requests` deixa de recusar quando a skill pergunta: abre a pergunta e conclui o pedido com o resultado "aguardando a pergunta P-NNN".
3. O usuário responde no painel; a ação `responder-pergunta` grava a resposta na pergunta, marca `respondida` e cria o pedido de resposta.
4. A `process-requests` atende esse pedido rodando de novo a skill do pedido de origem, com o texto original e as respostas.

As skills de conversa (create-spec, refine-spec, adjust-spec, adjust-plan, adjust-task) já juntam as perguntas numa rodada só; a mudança nelas é só o destino da rodada quando rodam por um pedido. O limite de 2 rodadas da refine-spec continua valendo, contando as perguntas abertas.

## Arquivos de comunicação

| Pedidos na pasta | Arquivos lidos por `pedidos listar --estado pendente` | Tempo (mediana) | Saída de `pedidos listar` sem filtro |
|---|---|---|---|
| 10 | 10 | 1,6 ms | menos de 1 KB |
| 100 | 100 | 2,7 ms | cerca de 6 KB |
| 1.000 | 1.000 | 7,3 ms | cerca de 60 KB |
| 10.000 | 10.000 | 58,9 ms | 604 KB |

O tempo não é problema. O problema é o volume:
- **Para o agente:** a `process-requests` usa o filtro de pendentes, mas qualquer listagem sem filtro, ou um `ls` da pasta, coloca no contexto uma lista que cresce sem limite. Com 10.000 pedidos, são 604 KB numa só saída.
- **Para o usuário:** uma pasta com milhares de `PED-NNN.md` misturados não mostra o que está pendente.

| `avais.log` | Tamanho | Leitura completa |
|---|---|---|
| 1.000 linhas | 124 KB | 0,3 ms |
| 10.000 linhas | 1,2 MB | 2,9 ms |
| 100.000 linhas | 12,1 MB | 29,1 ms |

O tempo de leitura do `avais.log` não preocupa; o que preocupa é o consumo de contexto, se o agente abrir o arquivo. Hoje nenhuma skill, regra ou persona manda o agente ler o `avais.log`: só a CLI o lê (`apps/cli/src/aval.rs`), e quando a trava recusa, a mensagem traz só o ID, a ação e os dois hashes, não o arquivo. O consumo de contexto hoje é zero. O risco é o agente abrir o arquivo por conta própria para diagnosticar uma recusa: com 100.000 linhas, são 12,1 MB, o que estoura qualquer contexto.

O arquivo precisa continuar sendo só de acréscimo, porque é o que torna a forja visível (DT-014). Duas mudanças pequenas tiram o risco sem perder isso:
- **Um arquivo por spec** (`avais/SPEC-NNN.log`), cada um só de acréscimo e arquivado com a spec (detalhes na recomendação abaixo). Mesmo que o agente abra um arquivo, ele tem o tamanho de uma spec.
- **Um comando de consulta**, `vostok avais <ID>`, que imprime só os avais daquele alvo. As skills e a mensagem de recusa passam a citá-lo, e uma regra proíbe ler os arquivos de aval direto, como já proíbe gravá-los ([agent/boundaries-05]).

Opções comparadas para pedidos e perguntas:

| Opção | O agente lê | O usuário vê | Custo |
|---|---|---|---|
| Pasta única (hoje) | Todos os arquivos | Tudo misturado | Nenhum |
| Pastas por estado (`pendente/`, `em_andamento/`, `feito/`, `recusado/`) | Só `pendente/` | O que espera ação, separado | Mudar `pedidos.rs`, o painel e a `process-requests`; transição vira um `rename`, atômico |
| Índice num arquivo só | O índice | Um arquivo a mais para entender | Manter o índice coerente com os arquivos, com risco de divergir |

**Recomendação.** Pastas por estado, para pedidos e perguntas: a mudança de estado passa a ser mover o arquivo, o que também deixa o estado visível no próprio caminho.

**O ciclo de vida acompanha a spec.** Todo pedido e toda pergunta já guardam a `spec` no front-matter. Quando o `vostok finalize archive` arquiva a spec, ele leva junto os pedidos e as perguntas dela para `<archive>/<SPEC>/pedidos/` e `<archive>/<SPEC>/perguntas/`, como já leva as tarefas e as evidências. As pastas de trabalho ficam só com o que é das specs em andamento, sem regra de idade. A atividade continua lendo o arquivo morto, então o histórico não some.

Os avais seguem a mesma lógica: um arquivo só de acréscimo por spec (`avais/SPEC-NNN.log`), arquivado com ela. Mover o arquivo inteiro não reescreve nenhuma linha, então a forja continua visível (DT-014). Um aval sem spec vai para `avais/sem-spec.log`.

**Pedidos sem spec.** A spec é opcional no pedido: um `fazer-commit` geral, por exemplo, pode não citar nenhuma. Esses ficam na pasta de trabalho até o `prune`.

**Um comando de limpeza.** `vostok pedidos prune` apaga os pedidos e as perguntas já arquivados e os concluídos sem spec com mais de N dias (padrão 30). Ele é destrutivo e o arquivo morto fica fora do git, então:
- mostra o que vai apagar e só apaga com `--confirmar`;
- nunca apaga avais, que provam as aprovações;
- registra na saída quantos arquivos e de quais specs, para o usuário saber que essa parte da atividade some.

No painel, a limpeza fica nas Configurações, com um botão "Limpar pedidos arquivados". Ela não passa pelo agente: o painel chama a mesma função do `prune` diretamente, como já faz nas aprovações. Primeiro mostra a lista do que vai apagar e só apaga depois que você confirma ali mesmo.

Hoje as pastas `pedidos/` e `perguntas/` estão vazias no ai-hub e no app da RD; o `avais.log` existente é dividido por spec na primeira execução da versão nova, numa migração única.

## Fontes de atividade

| Fonte | Volume medido ou real | Custo de leitura | Inclui na `GET /api/atividades`? |
|---|---|---|---|
| Registro de execução das tarefas abertas | 7.000 linhas | 6,7 ms para o `GET /api/visao` inteiro | Sim |
| Registro das tarefas arquivadas | cerca de 1.190 linhas em 167 tarefas no ai-hub | Da mesma ordem: alguns milissegundos | Sim |
| Histórico de ajustes das specs e dos planos | 10 linhas nas 30 specs arquivadas do ai-hub | Desprezível | Sim |
| Pedidos | 10.000 arquivos em 59 ms | Baixo; cai com as pastas por estado | Sim, com a criação e o estado atual |
| `avais.log` | 100.000 linhas em 29 ms | Baixo | Sim |

Com o volume real do projeto mais antigo, todas as fontes juntas ficam abaixo de 50 ms, longe dos 500 ms da RNF-01 da SPEC-032. A ressalva é que os pedidos só guardam o último estado: a atividade mostra a criação e o estado atual, e as transições só aparecem depois da proposta de histórico do pedido, na SPEC-033.

## Proposta da SPEC-033

**Para a SPEC-032.** Nenhuma tarefa precisa de ajuste. A TASK-05 pode incluir as cinco fontes como planejado.

**Escopo recomendado da SPEC-033**, em ordem:

1. **Perguntas do agente pelo painel:** `vostok perguntas abrir`, a `process-requests` abrindo pergunta em vez de recusar e as skills de conversa usando esse destino quando rodam por um pedido.
2. **Criar e refinar spec pelo painel:** ações `criar-spec` e `refinar-spec`, com o botão "Nova spec" e "Refinar" no painel.
3. **Pastas por estado** para pedidos e perguntas, com a transição por `rename`; pedidos, perguntas e avais (um arquivo por spec) arquivados junto com a spec pelo `vostok finalize archive`; `vostok pedidos prune` para limpar os já arquivados, também no painel, com a lista e a confirmação na tela; e `vostok avais <ID>` para consultar avais sem o agente abrir os arquivos.
4. **Histórico de status** dos pedidos e das specs, nas linhas propostas em "Status das entidades", alimentando a atividade.
5. **Lacunas das skills existentes:** conferir prints pelo painel na `finish-task`, liberar ou desfazer commits apontados pela `approve-commit` e executar uma onda inteira por pedido.
6. **Disparo sob demanda**, opcional e desligado por padrão, reaproveitando o código do `vigiar`.
7. **Versão 0.4.0.**

Mudanças sugeridas no rascunho da SPEC-033:
- tirar o chat dentro do painel do escopo, deixando-o como alternativa registrada;
- acrescentar o disparo sob demanda como opção;
- acrescentar as lacunas do item 5, que o rascunho não cita;
- avaliar incluir o aviso das tarefas no formato antigo (DT-015), que também atrapalha operar só pelo painel.
