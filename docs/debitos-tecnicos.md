# Débitos técnicos

Problemas conhecidos que ficaram para depois: lacunas de comportamento, de cobertura de testes ou de clareza. Cada débito tem um ID fixo (`DT-NNN`), é registrado quando é encontrado e só sai da lista de abertos quando for resolvido ou descartado, com o motivo.

## Resumo

| ID | Título | Área | Prioridade | Status |
|---|---|---|---|---|
| DT-001 | Tarefa em execução não pode ser ajustada | harness: fluxo da tarefa | alta | resolvido |
| DT-002 | Retomada e regressão sem cenário com agente | harness: testes com agente | média | resolvido |
| DT-003 | Hash da tarefa diverge do hash aprovado após o início | harness: fluxo da tarefa | baixa | resolvido |
| DT-004 | Commits da finalização sem data final | harness: finalização da spec | baixa | resolvido |
| DT-005 | `--build` informa sucesso quando a geração falha | harness: finalização da spec | média | resolvido |
| DT-006 | Desfazer do `--archive` sem teste | harness: finalização da spec | média | resolvido |
| DT-007 | Lista de pendências duplicada no painel | painel: frontend | baixa | aberto |
| DT-008 | `base64` e `content_type` duplicados no servidor do painel | painel: servidor | baixa | aberto |
| DT-009 | Sem teste da aprovação de spec que dá certo pelo painel | painel: testes | média | aberto |
| DT-010 | Configurações sem "gravar por cima" e com o hash calculado no navegador | painel: configurações | média | aberto |
| DT-011 | Painel derruba a resposta com a saída padrão fechada | painel: servidor | média | aberto |
| DT-012 | Caminho sem bundle não roda na suíte | painel: build | baixa | aberto |
| DT-013 | `vostok pedidos vigiar` sem teste com o Codex real | harness: loop de pedidos | média | aberto |
| DT-014 | Forja de aval visível, mas não impedida | harness: trava de aval | baixa | aberto |
| DT-015 | Painel ignora tarefas no formato antigo sem avisar | painel: leitura das tarefas | média | aberto |
| DT-016 | Resumo do plano sai incompleto na resposta da plan-tasks | harness: respostas das skills | média | aberto |

Prioridade: **alta** bloqueia um fluxo real; **média** deixa um comportamento sem prova; **baixa** confunde, mas não quebra nada.

## DT-001: Tarefa em execução não pode ser ajustada

- **Área:** harness, skill `adjust-task`
- **Encontrado em:** 2026-10-02, SPEC-002 TASK-08, primeira execução do cenário `tk09`
- **Prioridade:** alta
- **Status:** resolvido

**Descrição.** A `adjust-task` aceita só tarefas `pending` ou `approved`. Quando a execução revela que a tarefa está errada (passos, verificação ou arquivos que não batem com a spec), não há caminho para corrigi-la: ela está `in_progress` e só sai desse estado para `validating`, `blocked` ou `done`.

**Impacto.** O agente fica sem saída válida. Ele precisa parar e perguntar, e o usuário também não tem um comando para devolver a tarefa a um estado ajustável. A alternativa seria editar o arquivo à mão, fora do fluxo.

**Proposta.** Definir como uma tarefa em execução volta a ser ajustável. Uma opção é a `adjust-task` aceitar `in_progress` e `blocked`, derrubar a aprovação, descartar o baseline e as evidências e registrar o ajuste. Outra é uma etapa explícita de "devolver para ajuste". Exige decisão do usuário antes de implementar.

**Ocorrências.** 2026-10-06, SPEC-003 TASK-05: o usuário mudou o foco e o visual do documento executivo com a tarefa já em validação. O ajuste foi registrado à mão no histórico de ajustes da tarefa, com os arquivos novos em "Mudar" e o critério novo em "Concluída quando".

**Referências.** `packages/harness/skills/default/adjust-task/`, `packages/harness/tests/agent-scenarios.md` (resultado do `tk09`).

**Resolução.** Resolvido na SPEC-004 pelas TASK-02 a TASK-05 (commits `123a608` e `5721c1b`): a `adjust-task` aceita tarefas `in_progress`, `validating` e `blocked`, mantém o baseline, zera tentativas e resultados, grava a linha `ajuste` no registro e não pede nova aprovação; tarefas `done` são recusadas. A conclusão ficou só na skill `finish-task`.

## DT-002: Retomada e regressão sem cenário com agente

- **Área:** harness, `tests/agent-scenarios.md`
- **Encontrado em:** 2026-10-02, SPEC-002 TASK-08
- **Prioridade:** média
- **Status:** resolvido

**Descrição.** O CA-05 da SPEC-002 pede que os cenários tristes com agente parem na falha de testes e na regressão e retomem com `--resume`. Os cenários com agente cobrem só a parada por falha de testes (`tk15`). A parada por regressão e a retomada de uma tarefa `blocked` com a decisão do usuário têm cobertura apenas na suíte (`tests/run-tests.sh`).

**Impacto.** Não há prova de que o agente reage bem à regressão, nem de que conduz a retomada como o fluxo manda: registrar a decisão e voltar a validar.

**Proposta.** Acrescentar à fixture um cenário com regressão (baseline que muda depois da tarefa) e um cenário de retomada, que parte de uma tarefa `blocked` e tem como mensagem do usuário uma das opções do `blocker`.

**Referências.** `packages/harness/tests/fixtures/agent-scenarios.sh`, `packages/harness/skills/default/finish-task/`.

**Resolução.** Resolvido na SPEC-004 pela TASK-06 (commit `dddd36d`): os cenários `tk21` (parada por regressão), `tk22` (retomada de uma tarefa `blocked` com a decisão do usuário) e `tk23` (ajuste em execução) foram executados com agente e passaram.

## DT-003: Hash da tarefa diverge do hash aprovado após o início

- **Área:** harness, skills `execute-task` e `finish-task`
- **Encontrado em:** 2026-10-02, SPEC-002 TASK-08, cenários `tk09` e `tk15`
- **Prioridade:** baixa
- **Status:** resolvido

**Descrição.** O `--start` compara o `approved_hash` com o conteúdo antes de iniciar. Depois, cada etapa grava uma linha no "Registro de execução" e atualiza o `content_hash`, que deixa de bater com o `approved_hash`. O comportamento está correto, mas dois agentes estranharam a diferença e a citaram como possível falha.

**Impacto.** Gera dúvida no agente e no usuário, e pode levar alguém a "consertar" algo que não está quebrado.

**Proposta.** Explicar a diferença na `SKILL.md` da `execute-task` (e no front-matter, se fizer sentido), ou excluir o "Registro de execução" do cálculo do hash.

**Referências.** `packages/harness/skills/default/execute-task/scripts/execute-task.sh`, `packages/harness/scripts/lib/task.sh` (`task_record_step`).

**Resolução.** Resolvido na SPEC-004 pela TASK-01 (commit `123a608`): o hash da tarefa ignora a seção "Registro de execução", e o `content_hash` continua igual ao `approved_hash` durante a execução. Uma edição à mão nessa seção deixou de ser detectada, risco aceito porque o registro é escrito só pelos scripts.

## DT-004: Commits da finalização sem data final

- **Área:** harness, `finalize-spec`
- **Encontrado em:** 2026-10-06, SPEC-003 TASK-12, finalização da SPEC-002
- **Prioridade:** baixa
- **Status:** resolvido

**Descrição.** As fontes da finalização listavam os commits desde o `approved_at` da spec, sem data final. O técnico da SPEC-002 trouxe commits da SPEC-003 que tocaram os mesmos caminhos.

**Impacto.** O documento técnico mostrava commits que não pertencem à spec, o que confunde quem o lê e exige revisão do agente na síntese.

**Proposta.** Limitar a lista ao `approved_at` mais antigo das specs aprovadas depois da finalizada, lidas de `refined/` e do arquivo. Sem outra spec aprovada depois, sem limite final.

**Resolução.** Resolvido na SPEC-004 pela TASK-07 (commit `123a608`): `finalize_next_approved_at` passa o limite ao `git log --until`.

**Referências.** `packages/harness/scripts/lib/finalize.sh` (`finalize_sources_commits`), `docs/specs/SPEC-002/tecnico.html`.

## DT-005: `--build` informa sucesso quando a geração falha

- **Área:** harness, `finalize-spec`
- **Encontrado em:** 2026-10-06, SPEC-003 TASK-12, finalização da SPEC-002
- **Prioridade:** média
- **Status:** resolvido

**Descrição.** O `finalize-spec.sh --build` imprimia a mensagem de sucesso mesmo quando um comando da geração falhava no meio. Na SPEC-002, o limite de tamanho de uma variável de ambiente fez o documento técnico sair quebrado, com mensagens de erro no terminal e a lista dos três documentos como se tudo estivesse certo.

**Impacto.** Um documento parcial ou quebrado podia ser conferido e publicado sem aviso.

**Proposta.** Gerar cada documento num arquivo temporário, terminar com erro se algum comando falhar ou escrever no stderr, e só então substituir o documento anterior.

**Resolução.** Resolvido na SPEC-004 pela TASK-08 (commit `123a608`): `finalize_generate` faz a geração num temporário e o `--build` para no primeiro documento com falha, mantendo os anteriores.

**Referências.** `packages/harness/scripts/lib/finalize.sh` (`finalize_generate`), `packages/harness/skills/default/finalize-spec/scripts/finalize-spec.sh`, commit `29f3fbd`.

## DT-006: Desfazer do `--archive` sem teste

- **Área:** harness, `finalize-spec`
- **Encontrado em:** 2026-10-06, SPEC-003 TASK-09
- **Prioridade:** média
- **Status:** resolvido

**Descrição.** Quando uma cópia do `--archive` falha, o `finalize_archive` remove o arquivo e os documentos publicados parciais e mantém a spec, as tarefas e o trabalho no lugar. Nenhum teste forçava essa falha, e o mutante que tirava o desfazer sobrevivia.

**Impacto.** Uma regressão no desfazer deixaria uma spec meio arquivada, sem que a suíte acusasse.

**Proposta.** Um teste que force a falha de uma cópia e confira que nada ficou pela metade, validado por mutação.

**Resolução.** Resolvido na SPEC-004 pela TASK-09 (commit `123a608`): o teste torna o destino `docs/specs` impossível de criar; sem a linha do desfazer, o caso novo falha.

**Referências.** `packages/harness/scripts/lib/finalize.sh` (`finalize_archive`), `packages/harness/tests/run-tests.sh`.

## DT-007: Lista de pendências duplicada no painel

- **Área:** painel, frontend
- **Encontrado em:** 2026-11-02, publicação da SPEC-029 (commit `9c1ff2e`)
- **Prioridade:** baixa
- **Status:** aberto

**Descrição.** A lista de pendências existe duas vezes: uma no painel e outra na tela de pendências.

**Impacto.** Uma mudança numa das listas pode não chegar à outra, e o mesmo comportamento fica mantido em dois lugares.

**Proposta.** Levar a lista para a pasta `shared/` do frontend e usá-la nas duas telas.

**Referências.** `apps/painel/src/features/`, `docs/specs/SPEC-029/tecnico.html` (seção "Limites e dívidas").

## DT-008: `base64` e `content_type` duplicados no servidor do painel

- **Área:** painel, servidor
- **Encontrado em:** 2026-11-02, publicação da SPEC-029 (commit `9c1ff2e`)
- **Prioridade:** baixa
- **Status:** aberto

**Descrição.** A codificação `base64` aparece em `read.rs` e em `live.rs`, e a detecção de `content_type` aparece em `read.rs` e em `server.rs`.

**Impacto.** Uma correção feita numa cópia pode não chegar à outra, e a mesma regra fica espalhada pelo servidor.

**Proposta.** Concentrar a codificação e a detecção de tipo de conteúdo num único módulo de apoio do painel e usá-lo nos três arquivos.

**Referências.** `apps/cli/src/painel/read.rs`, `apps/cli/src/painel/live.rs`, `apps/cli/src/painel/server.rs`.

## DT-009: Sem teste da aprovação de spec que dá certo pelo painel

- **Área:** painel, testes
- **Encontrado em:** 2026-11-02, publicação da SPEC-029 (commit `9c1ff2e`)
- **Prioridade:** média
- **Status:** aberto

**Descrição.** A aprovação de spec pelo painel tem teste só para o caso de recusa (422). O caminho que aprova com sucesso não tem teste.

**Impacto.** Uma regressão na aprovação bem-sucedida pelo painel passaria sem aviso na suíte.

**Proposta.** Acrescentar um teste que aprove uma spec válida pelo painel e confira o status e o `approved_by: painel`.

**Referências.** `apps/cli/src/painel/`, `docs/specs/SPEC-029/tecnico.html` (seção "Limites e dívidas").

## DT-010: Configurações sem "gravar por cima" e com o hash calculado no navegador

- **Área:** painel, configurações
- **Encontrado em:** 2026-11-02, publicação da SPEC-029 (commit `9c1ff2e`)
- **Prioridade:** média
- **Status:** aberto

**Descrição.** A tela de configurações não oferece "gravar por cima" num conflito (RN-06). Além disso, ela calcula o hash no navegador, porque `/api/configuracoes` não o devolve.

**Impacto.** Num conflito de edição, o usuário não tem como sobrescrever a versão atual de forma deliberada, e o cálculo no navegador pode divergir do que o servidor grava.

**Proposta.** Devolver o hash na resposta de `/api/configuracoes` e oferecer "gravar por cima" como ação explícita no conflito.

**Referências.** `apps/cli/src/painel/read.rs`, `apps/painel/src/features/settings/settings-page.tsx`, `docs/specs/SPEC-029/tecnico.html` (seção "Limites e dívidas").

## DT-011: Painel derruba a resposta com a saída padrão fechada

- **Área:** painel, servidor
- **Encontrado em:** 2026-11-02, publicação da SPEC-029 (commit `9c1ff2e`)
- **Prioridade:** média
- **Status:** aberto

**Descrição.** O `record_manual` imprime uma linha na saída padrão do painel a cada edição. Com a saída fechada, o `println!` derruba a resposta. Quem roda o painel em segundo plano precisa redirecionar a saída.

**Impacto.** Ao rodar o painel em segundo plano sem redirecionar a saída, uma edição falha no meio da resposta.

**Proposta.** Trocar o `println!` por uma gravação que ignore erro de saída, ou registrar a linha no log do painel.

**Referências.** `apps/cli/src/commands/spec.rs`, `apps/cli/src/painel/actions.rs`, `docs/specs/SPEC-029/tecnico.html` (seção "Limites e dívidas").

## DT-012: Caminho sem bundle não roda na suíte

- **Área:** painel, build
- **Encontrado em:** 2026-11-02, publicação da SPEC-029 (commit `9c1ff2e`)
- **Prioridade:** baixa
- **Status:** aberto

**Descrição.** O caminho sem o bundle do painel compila, mas não roda na suíte, porque o `dist/` existe no ambiente de teste.

**Impacto.** O caminho de compilação sem Node e sem bundle fica sem prova automatizada.

**Proposta.** Rodar a suíte com o `dist/` ausente, removendo-o temporariamente ou apontando o build para uma pasta vazia.

**Referências.** `apps/cli/build.rs`, `docs/specs/SPEC-029/tecnico.html` (seção "Limites e dívidas").

## DT-013: `vostok pedidos vigiar` sem teste com o Codex real

- **Área:** harness, loop de pedidos
- **Encontrado em:** 2026-11-02, publicação da SPEC-030 (commit `32e24b5`)
- **Prioridade:** média
- **Status:** aberto

**Descrição.** O `vostok pedidos vigiar` foi testado com um Codex simulado. O uso real depende de `codex exec --full-auto` aceitar a instrução e liberar os comandos do `vostok` no sandbox, e esse teste real ficou para depois.

**Impacto.** O loop de pedidos com o Codex real pode falhar no sandbox sem que a suíte acuse.

**Proposta.** Rodar `vostok pedidos vigiar --agente codex` com o Codex real num pedido de teste e registrar a evidência.

**Referências.** `docs/specs/SPEC-030/tecnico.html` (seção "Limites e dívidas"), `apps/cli/src/commands/pedidos.rs`.

## DT-014: Forja de aval visível, mas não impedida

- **Área:** harness, trava de aval
- **Encontrado em:** 2026-11-02, publicação da SPEC-030 (commit `32e24b5`)
- **Prioridade:** baixa
- **Status:** aberto

**Descrição.** A trava não impede a escrita direta no `avais.log`: um processo do mesmo usuário pode forjar uma linha. A trava só torna a forja visível na validação e presa ao hash.

**Impacto.** Um aval forjado passa pela trava se o hash estiver certo. Proteger o arquivo exigiria um segredo fora do alcance do agente, o que está fora do escopo atual.

**Proposta.** Decidir se vale um segredo fora do alcance do agente para assinar os avais. Exige decisão do usuário antes de implementar.

**Referências.** `docs/specs/SPEC-030/tecnico.html` (seção "Limites e dívidas").

## DT-015: Painel ignora tarefas no formato antigo sem avisar

- **Área:** painel, leitura das tarefas
- **Encontrado em:** 2026-11-03, uso do painel num projeto que começou na v0.1 (15 tarefas no formato de arquivo único)
- **Prioridade:** média
- **Status:** aberto

**Descrição.** O painel só lê tarefas no formato de pasta (`TASK-NN/estado.yml`). Uma spec com tarefas no formato antigo, um `TASK-NN.md` por tarefa, aparece sem tarefas no painel, sem nenhum aviso. A CLI continua aceitando o formato antigo, então o projeto funciona no chat e parece vazio no painel.

**Impacto.** Quem atualiza da v0.1 para a v0.3.0 abre o painel e não vê as tarefas, sem pista de que precisa migrar. A saída é rodar `vostok task migrate <SPEC_ID>`, que hoje só se descobre lendo a ajuda da CLI.

**Proposta.** Fazer o painel detectar specs com tarefas no formato antigo e mostrar um aviso com o comando `vostok task migrate <SPEC_ID>`. Avaliar também se o `vostok update` deve avisar sobre essas specs ao final.

**Referências.** `apps/cli/src/painel/read.rs` (fn `task_dirs`), `apps/cli/src/commands/task.rs` (fn `migrate`).

## DT-016: Resumo do plano sai incompleto na resposta da plan-tasks

- **Área:** harness, respostas das skills `plan-tasks` e `adjust-plan`
- **Encontrado em:** 2026-11-03, planejamento da SPEC-032
- **Prioridade:** média
- **Status:** aberto

**Descrição.** A `plan-tasks` pede que a resposta traga o resumo do plano: a visão geral, a tabela de tarefas com perfis, dependências, ondas e cobertura, e os arquivos em comum por onda. O agente montou esse resumo à mão e entregou só uma parte, sem a tabela de ondas e sem a coluna de cobertura, e o usuário precisou redirecionar o chat para ver o plano. Diferente do `vostok task status`, que imprime as tabelas prontas e é colado sem alteração, o resumo do plano depende de o agente copiar as tabelas certas do arquivo.

**Impacto.** O usuário não consegue revisar o plano pela resposta e precisa pedir as tabelas de novo, o que atrasa a aprovação e quebra a confiança na comunicação do agente.

**Proposta.** Criar um comando que imprima o resumo do plano pronto (tarefas com persona, perfil, dependências, onda e cobertura, e a tabela de ondas com os arquivos em comum), como o `vostok task status`, e fazer a `plan-tasks` e a `adjust-plan` colarem a saída sem alteração, na regra [agent/responses-07]. Avaliar também mudar o perfil de uma tarefa sem marcar fatos (hoje, subir o estudo para julgamento exigiu marcar inferência e risco alto).

**Referências.** `packages/harness/skills/default/plan-tasks/SKILL.md` (passo 7 e "Resumo do plano"), `packages/harness/skills/default/adjust-plan/SKILL.md`.

## Como registrar um débito

1. Use o próximo ID livre (`DT-NNN`) e acrescente uma linha ao resumo.
2. Crie a seção com área, origem (data e onde foi encontrado), prioridade, status, descrição, impacto, proposta e referências.
3. Ao resolver, mude o status para `resolvido` e cite o commit ou a tarefa; ao descartar, mude para `descartado` e diga o motivo. Não apague a seção.
