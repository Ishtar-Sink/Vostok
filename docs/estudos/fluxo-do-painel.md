# Estudo de UX do fluxo do painel

Data do estudo: 2026-11-11. Versão: `vostok` 0.5.4. Primeira tarefa da SPEC-049 (RF-01); subsídio das tarefas do quadro e da navegação (TASK-08 e TASK-09) e dos tipos de visualização da SPEC-050.

## Escopo e método

O estudo responde a uma queixa do usuário: "está confuso hoje ter um menu de tarefas, e dentro da tarefa [spec] ter um quadro que mostra a mesma visão". Ele cobre o caminho entre as telas do painel local (início, specs, spec, menu Tarefas e tarefa) e o que o usuário faz hoje para conferir as evidências de uma onda.

O método foi a leitura do código em `apps/painel/src`: as rotas (`shared/router.ts`), o menu e as migalhas (`layout/sidebar.tsx`, `layout/topbar.tsx`), as telas (`features/dashboard`, `features/specs`, `features/spec`, `features/board`, `features/task`), o `BackLink` (`shared/back-link.tsx`) e as seções do quadro (`BOARD_SECTIONS`, em `shared/status.ts`). Não houve teste com usuários além do relato do próprio usuário; as conclusões vêm da estrutura das telas.

## Fluxo atual

O menu lateral tem quatro itens: Painel (`/`), Specs (`/specs`), Tarefas (`/tarefas`) e Documentos (`/documentos`). As rotas e o que cada tela faz:

| Tela | Rota | Componente | Mostra | Leva a |
|---|---|---|---|---|
| Início | `/` | `DashboardPage` | Indicadores (tarefas que faltam, specs abertas, em execução, concluídas), cartões das specs em andamento e a lista "Tarefas que faltam" | Os indicadores de tarefas levam ao menu Tarefas (`/tarefas`); o cartão da spec leva a `/specs/<id>` e a `/specs/<id>/quadro`; cada tarefa da lista leva a `/tarefas/<spec>/<tarefa>` |
| Specs | `/specs` | `SpecsPage` | Lista das specs | `/specs/<id>` |
| Spec, documento | `/specs/<id>` | `SpecPage` (visão `documento`) | Abas do texto da spec e, ao lado, os metadados e a lista de tarefas com o status | Botão "Ver o quadro" e link "Quadro" (`/specs/<id>/quadro`); cada tarefa (`/tarefas/<spec>/<tarefa>`) |
| Spec, quadro | `/specs/<id>/quadro` | `SpecPage` (visão `quadro`) com `Board` | Quadro das tarefas só desta spec, sem filtros | Cada cartão leva à tarefa |
| Spec, plano | `/specs/<id>/plano` | `SpecPage` (visão `plano`) | Plano de tarefas, enquanto existir | — |
| Tarefas | `/tarefas` | `TasksPage` com `Board` | Quadro das tarefas de todas as specs abertas, com filtro por spec e quatro seletores (agrupar, ordenar, onda, persona) | Cada cartão leva à tarefa |
| Tarefa | `/tarefas/<spec>/<tarefa>` | `TaskPage` | Cabeçalho, abas (resumo, evidências, execução), dependências e resultados | "Voltar" (`BackLink`, com `history.back()` e, sem histórico, `/specs/<spec>/quadro`); links de dependência para outras tarefas |

Observações do caminho:

- A migalha da tarefa é `Tarefas › <spec> › <tarefa>`. O primeiro nível leva a `/tarefas` e o segundo a `/specs/<spec>/quadro`, ou seja, ela mistura os dois quadros no mesmo caminho. O destino padrão do "Voltar" também é o quadro da spec, mesmo quando o usuário chegou pelo menu Tarefas.
- `navigate()` em `shared/router.ts` rola a página ao topo em toda navegação sem âncora, e o `BackLink` usa `history.back()` sem restaurar a rolagem. Voltar de uma tarefa abre o quadro no topo.
- O estado dos filtros do menu Tarefas fica no `localStorage` (`vostok.quadro.*`) para agrupamento, ordem, onda e persona, mas o filtro de spec é um `useState` local e some ao sair da tela.

Passos que o usuário dá hoje para conferir as evidências de uma onda (a spec tem três tarefas na onda, por exemplo):

1. Abre o menu Tarefas, ou a spec e o quadro dela (dois caminhos para a mesma lista).
2. Filtra pela spec (no menu Tarefas) e pela onda, e procura a raia "Prontas para finalizar", que fica no meio do quadro, depois de "Prontas para execução" e das raias de entrada.
3. Abre a primeira tarefa, vai à aba Evidências e confere.
4. Volta pelo "Voltar", perde a rolagem e o contexto, e recomeça do passo 2 para a segunda tarefa.
5. Alternativa: dentro da tarefa, usa os links de dependência (`Depende de`, `Fecha depois de`), que só levam à ligação lógica, não à vizinha da onda.

Cada tarefa custa uma ida e volta ao quadro, sem barra de navegação entre as tarefas da onda.

## Quadro repetido

O mesmo componente `Board` aparece em duas telas:

| Aspecto | Menu Tarefas (`TasksPage`) | Quadro da spec (`SpecPage`, visão `quadro`) |
|---|---|---|
| Fonte | `/api/visao`, specs abertas com tarefas | `/api/specs/<id>`, só a spec aberta |
| Escopo | Todas as specs, ou uma, pelo filtro "Todas / SPEC-xxx" | Uma spec |
| Controles | Filtro de spec, agrupar por (etapa, spec, onda), ordenar por, onda, persona | Nenhum |
| Cartão | Mostra `SPEC · TASK` quando o filtro é "Todas" | Só o ID da tarefa |
| Raias | As oito de `BOARD_SECTIONS`, na ordem do fluxo | As mesmas oito, na mesma ordem |
| Lembra a escolha | Sim, no `localStorage`, menos o filtro de spec | Não |

Com o filtro de spec escolhido, o menu Tarefas mostra a mesma coisa que o quadro da spec, com mais controles. Isso é a sobreposição que o usuário relata: duas portas para a mesma visão, com capacidades diferentes e sem pista de qual usar. O quadro da spec é a versão pobre do outro: quem precisa de onda ou persona dentro da spec tem de sair para o menu Tarefas e refazer o filtro de spec.

Problemas concretos:

1. Duas portas, duas migalhas. A tarefa lembra `Tarefas › spec`, e o início tem os dois caminhos (`/tarefas` e `/specs/<id>/quadro`), sem diferença visível entre eles.
2. Posição perdida ao voltar. O `navigate()` volta ao topo e o `BackLink` usa o histórico sem restaurar a rolagem (RF-20).
3. As raias ativas ficam no fim. A ordem do fluxo vai de "Para aprovar" até "Concluídas"; o que está em execução, pronto para finalizar, em validação ou bloqueado fica no meio ou no fim, e em specs grandes exige rolar para chegar ao que importa (RF-17, RF-18).
4. Sem resumo por raia. Não há como ver quantas tarefas há em cada raia nem ir direto a uma delas (RF-19).
5. Ida e volta entre tarefas. Não há anterior e próxima na onda; cada tarefa exige voltar ao quadro (RF-21, RF-22).
6. O filtro de spec do menu Tarefas não vai para o endereço nem é lembrado; compartilhar ou recarregar volta para "Todas".
7. O "Voltar" da tarefa tem destino padrão no quadro da spec, mesmo para quem veio do menu Tarefas.

## Alternativas

### A. Um único quadro, no menu Tarefas, com a spec como filtro no endereço

O menu Tarefas passa a ser o único lugar do quadro, com a spec escolhida no endereço (`/tarefas?spec=SPEC-049`). A página da spec perde a visão `quadro`: a aba some, e o botão "Ver o quadro" e o link "Quadro" passam a abrir `/tarefas?spec=<id>`. A rota antiga `/specs/<id>/quadro` redireciona para o endereço novo, como já acontece com `/planos/<id>`. Na spec fica só o resumo: a lista de tarefas da lateral, com progresso "N de M concluídas" e status.

- Prós: uma só visão e um só conjunto de controles; os filtros e a posição valem para qualquer escopo; o "Voltar" da tarefa tem um destino só; a visão por spec continua a um clique; o endereço com o filtro de spec pode ser compartilhado e recarregado.
- Contras: ao abrir "o quadro da spec", o usuário sai do contexto da spec (a migalha muda de Specs para Tarefas); o redirecionamento da rota antiga é código a manter.
- Impacto em TASK-08 (quadro com filtro, ordem e contagem): o filtro de status, a ordem invertida e a faixa de contagem são feitos uma vez, no `Board`, e valem para tudo; o filtro de spec passa a ser lido do endereço e a faixa de contagem respeita a spec escolhida. TASK-08 deixa de ter dois pontos de uso para testar e ganha o teste do filtro pelo endereço.
- Impacto em TASK-09 (navegação e posição): a barra da onda, a navegação de revisão e a restauração da rolagem têm um só quadro de origem; o "Voltar" restaura a rolagem de `/tarefas?spec=...`. O destino padrão do `BackLink` vira o endereço com o filtro.

### B. O quadro fica só na spec e o menu Tarefas vira a lista de todas as specs

O menu Tarefas mostra, por spec, o progresso e o que está ativo (um resumo, no estilo dos cartões do início), e o quadro completo só existe dentro da spec (`/specs/<id>/quadro`).

- Prós: o quadro tem sempre o contexto da spec; o menu Tarefas serve de visão geral do trabalho em aberto.
- Contras: perde-se a visão de todas as tarefas de todas as specs em um quadro, que é o uso do menu hoje (por exemplo, o que está bloqueado em qualquer spec); o resumo do menu repete o que o início já mostra (cartões "Em andamento" e "Tarefas que faltam"), trocando uma repetição por outra; as specs acumuladoras da 0.6.0 (SPEC-048, 049 e 050) são vistas em conjunto, e o filtro entre elas some.
- Impacto em TASK-08: o quadro continua em um só lugar, mas sem filtro de spec; os filtros de status e a faixa de contagem valem por spec. Impacto em TASK-09: a barra da onda e a revisão ficam dentro de uma spec, o que combina com a conferência de uma onda, mas a revisão entre specs diferentes exige voltar ao menu.

### C. Manter os dois e só tirar a diferença (mesmo quadro, mesmos controles)

O quadro da spec ganha os mesmos controles do menu Tarefas, com o filtro de spec travado.

- Prós: menor mudança de rota; nada some.
- Contras: não resolve a queixa, pois continuam duas portas para a mesma visão; dobra a superfície de teste e de manutenção dos controles; os problemas 1, 6 e 7 do quadro repetido ficam.
- Impacto em TASK-08 e TASK-09: o trabalho é o mesmo, mais o de manter os dois pontos de uso iguais. Descartada.

## Recomendação

Adotar a alternativa A: o quadro mora só no menu Tarefas, e a spec aponta para ele pelo filtro no endereço.

Por quê:

1. Resolve a queixa na raiz: deixa de haver duas telas com a mesma visão, e o usuário sabe onde estão as tarefas.
2. É a menor mudança que mantém o que funciona: o menu Tarefas já tem a base de filtros e é o único que lembra as escolhas; o quadro da spec só tem o subconjunto.
3. Faz o RF-17 a RF-22 valer num lugar só, em vez de implementá-los e testá-los duas vezes.
4. Serve de base para os tipos de visualização da SPEC-050: o seletor de visualização (lista e blocos) entra num quadro só, e a escolha lembrada vale para qualquer spec.
5. Mantém a spec como centro do documento: a página da spec fica com o texto, os metadados e o resumo das tarefas, com um link claro para o quadro filtrado.

Como fica o fluxo:

- Início: os indicadores e os cartões levam a `/tarefas` ou a `/tarefas?spec=<id>`.
- Spec: sem a aba Quadro; o botão e o link do resumo abrem `/tarefas?spec=<id>`; a rota `/specs/<id>/quadro` redireciona para o mesmo endereço.
- Menu Tarefas: o filtro de spec vai para o endereço e continua lembrado, junto dos filtros de status, a ordem das raias, a onda e a persona.
- Tarefa: a migalha vira `Tarefas › <spec> › <tarefa>`, com o segundo nível levando a `/tarefas?spec=<spec>`; o "Voltar" tem esse destino padrão e restaura a rolagem; a barra anterior, próxima e posição na onda leva às vizinhas.
- Conferência da onda: pela barra da tarefa, o usuário vai de uma tarefa à próxima com evidência a conferir, sem voltar ao quadro; o quadro só serve para escolher a onda e para voltar à posição certa.

## Impacto na spec

Esta seção diz, item a item, o que muda na SPEC-049 e na SPEC-050. O que muda de escopo entra por ajuste da spec (skill `adjust-spec`, RN-04 e FA-01), antes de TASK-08 e TASK-09 começarem. Este estudo não ajusta a spec.

| Item | Decisão | Detalhe |
|---|---|---|
| RF-17 (filtro de status com seleção múltipla, começando pelas raias ativas, lembrado no navegador) | Fica como está | Ele vale no quadro único. Um ajuste de detalhe: o filtro de status e o filtro de spec devem conviver no mesmo conjunto de controles, e a seleção lembrada é a do status; a spec vai no endereço. CB-07 continua valendo. |
| RF-18 (ordem invertida por padrão, com controle) | Fica como está | Sem mudança de escopo. O controle fica junto dos filtros do quadro único. |
| RF-19 (faixa fixa com a contagem por raia) | Muda por ajuste da spec | A contagem passa a respeitar a spec escolhida no endereço, e a faixa deve valer também com o agrupamento por spec e por onda (hoje o quadro tem três agrupamentos, e as raias da faixa são as da etapa). A redação deve dizer que a faixa conta o que está filtrado e leva à raia, ou ao grupo, com um clique. |
| RF-20 (restaurar a rolagem ao voltar) | Muda por ajuste da spec | O destino do "Voltar" passa a ser `/tarefas?spec=<id>`, e a rolagem tem de ser restaurada no botão do painel, no do navegador e no do mouse, com o filtro no endereço. Deve ser acrescentado o redirecionamento de `/specs/<id>/quadro`, para que links antigos e o histórico abram no quadro único. |
| RF-21 (barra anterior, próxima e posição na onda, com ← e →) | Fica como está | Sem mudança de escopo; CB-08 continua valendo. A barra vai na página da tarefa, que é a mesma nos dois fluxos. |
| RF-22 (navegação de revisão até a próxima tarefa da onda com evidência a conferir) | Fica como está | Sem mudança de escopo; CB-09 continua valendo. O estudo reforça o valor dele: é o que corta a ida e volta do passo 4 do fluxo atual. |

Itens novos que o estudo recomenda levar à spec, por ajuste:

1. Quadro único: remover a visão `quadro` da página da spec e apontar o botão "Ver o quadro" e o link "Quadro" para `/tarefas?spec=<id>`; a rota `/specs/<id>/quadro` redireciona. Isso entra em "Escopo", "Requisitos funcionais" (um requisito novo, por exemplo RF-23) e "Componentes afetados": `apps/painel/src/features/spec/spec-page.tsx`, `apps/painel/src/layout/topbar.tsx`, `apps/painel/src/shared/router.ts`, `apps/painel/src/features/dashboard/dashboard-page.tsx` e `apps/painel/src/features/board/tasks-page.tsx`.
2. Filtro de spec no endereço, lembrado pelo navegador junto dos demais, e migalha e destino padrão da tarefa apontando para ele.
3. O resumo da spec ganha o progresso por raia, ou ao menos o "N de M concluídas", que já existe na lateral; nenhum controle novo na spec.

Fica para os tipos de visualização da SPEC-050 (RF-11 e RF-12):

- O seletor de visualização (lista e blocos "Agora", "A seguir" e "Concluídas") e a escolha lembrada: ele entra no quadro único, depois dos filtros da TASK-08, e vale para qualquer spec.
- A visualização em blocos reaproveita o filtro de status e a faixa de contagem da SPEC-049 (RN-04 da SPEC-050); o ajuste de RF-19 acima já cobre que a faixa conte grupos, o que serve aos blocos.
- A visualização em quadro (colunas) continua prevista para o futuro, fora de escopo.
- A SPEC-050 deve ajustar o texto de RN-04 e de "Componentes afetados" (`board.tsx`, `tasks-page.tsx`) para citar o quadro único, sem mudar os requisitos.

## Riscos e pontos em aberto

- O redirecionamento de `/specs/<id>/quadro` precisa preservar o endereço no histórico, para o "Voltar" do navegador não prender o usuário num ciclo.
- A decisão final entre as alternativas A e B é do usuário; B só se justifica se a visão de todas as specs em um quadro não for usada.
- O nome do item do menu ("Tarefas") continua adequado na alternativa A; o título da tela ("Quadro de tarefas") deve passar a mostrar a spec escolhida.
