<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Estende o spike da TASK-02 em `/home/victor/projetos/ai-hub-painel`: API de leitura em JSON no `vostok painel`, eventos por Server-Sent Events e as telas em React com os componentes copiados do Atria. Os `.md` são renderizados no navegador. O resultado vai para `docs/estudos/painel-local/prototipo.md` e para os prints. Cobre PQ-01, PQ-09, PQ-10 e PQ-12; sustenta CA-05 e CA-07.

**Decisões.** A marca vem de `config/documents.yml` (inferência F, decisão 6 da spec), mapeada sobre os tokens de `@atria/theme`. A atualização ao vivo parte da consulta periódica (sem dependência) e é comparada com a crate `notify`, em tamanho e atraso. O plano de exemplo é a cópia do plano da SPEC-027 guardada em `plano-exemplo.md`, porque nenhuma spec da pasta tem plano aberto. `GET /api/arquivo` só serve caminhos dentro de `vostok/`.

**Riscos.** Componentes do Atria com dependências do app dele (roteador, consultas): copiar só os de apresentação. Cores da marca sem contraste: aplicar o mesmo ajuste de 4,5:1 de `documents.rs`.

**Arquivos em comum na onda.** Nenhum: é a única tarefa da onda 2.

## O que foi feito

**Spike.** Tudo fica na worktree `/home/victor/projetos/ai-hub-painel` (branch `spike/painel-local`, base `cb57802`), sem commit. Variante `tiny_http` com Preact com compat (`--features painel-tiny`, bundle com `PAINEL_PREACT=1`).

**Servidor (`apps/cli/src/commands/painel.rs`).**
- Comando: `vostok painel [--porta N] [--sem-navegador] [--pasta DIR]`.
  - Sobe as pastas a partir de `DIR` até achar `vostok/config/main.yml`.
  - Escuta só em `127.0.0.1`; a porta 0 (padrão) deixa o sistema escolher.
- Rotas JSON:
  - `/api/visao`, com specs, plano, tarefas, arquivadas, atividade dos `registro.md` e perguntas abertas;
  - `/api/specs/<ID>` e `/api/tarefas/<SPEC>/<TASK>`, que leem também as arquivadas;
  - `/api/planos/<ID>`, `/api/marca`, `/api/documentos`, `/api/configuracoes` e `/api/versao`.
- Arquivos:
  - `/api/arquivo?caminho=` e `/arquivos/<caminho>` só servem caminhos dentro de `vostok/`, por segmentos seguros;
  - o segundo serve para os links relativos dos documentos funcionarem.
- Ao vivo:
  - `/api/ao-vivo` é um WebSocket só do servidor para o navegador, com handshake por `sha1`;
  - a conexão envia `conectado` e depois `mudou` quando a versão de `work/` muda;
  - a versão é a maior data de modificação da pasta, conferida a cada 500 ms;
  - cada requisição roda numa thread.

**Frontend (`apps/painel/src/`).**
- **Rotas:** `/`, `/pendencias`, `/specs`, `/specs/<ID>[/quadro|/plano]`, `/tarefas`, `/tarefas/<SPEC>/<TASK>`, `/documentos` e `/configuracoes`. `/planos/<ID>` é um atalho para a aba Plano.
- **Componentes do Atria, sem testes:**
  - botão, cartão, cabeçalho, etiqueta, abas;
  - estados vazio, carregamento e erro;
  - barra de progresso, diálogo, confirmação e aviso;
  - campos de texto, seleção, cor, etiquetas, segmentado e caixa de marcar.
- **Peças próprias:**
  - `DataTable`, o padrão de tabela que vira cartões abaixo de 720 px;
  - quadro kanban e popover de referências;
  - `ActionBar` com as ações por status;
  - tela de configurações com um mapa de nomes e controles por chave (`features/settings/schema.ts`) e o restante em "Avançado".
- **Marca:** os grupos `light` e `dark` de `documents.yml` já usam os nomes de token do Atria e são aplicados como variáveis CSS.

**Repositório principal.** `docs/estudos/painel-local/prototipo.md` (novo): reprodução, telas, mapeamento da marca, atualização ao vivo com números, tamanho do binário, dois painéis e o retorno do usuário.

**Decisões.**
- **SSE trocado por WebSocket:** com o `tiny_http`, os Server-Sent Events não eram enviados até o fim da conexão e travavam o carregamento da página.
- **Consulta periódica em vez de `notify`:** o `notify` acrescenta 245 016 bytes e zera o uso de processador com uma aba aberta (de 1,6% a 1,8% para 0%). O atraso quase não cai (média de 304 ms contra 324 ms no ai-hub), porque o laço de 500 ms continua.
- **Perguntas do agente:** ficam em `vostok/work/perguntas/P-NNN.md`, com front-matter `id`, `spec`, `task`, `status`, `created_at` e `author`, e seções `## Pergunta`, `## Opções` e `## Resposta`.
- **Ações sem gravação:** os diálogos mostram o que acontece, e a gravação com aval é da TASK-04.

**Resultados.**
- Binário: 5 483 984 bytes (tar.gz 2 068 511), ou +897 128 (+19,6%) sobre a base.
- Bundle JS: 158 640 bytes (gzip 50 869).
- RSS de cerca de 5 MB, com 6 threads e mais 1 por aba.
- Quatro painéis ao mesmo tempo (4700 a 4703), todos em `127.0.0.1` segundo o `ss`.

**Pontos de atenção.**
- O tamanho passa o limite de 5,3 MB da TASK-02; os campos de formulário do Atria levaram o bundle de 118 para 158 KB. A TASK-05 revê o limite.
- O Atria não tem identidade própria (o grupo `brand` está vazio), então o print dele usa o visual padrão. A troca de cor foi comprovada na pasta de exemplo e na massa fictícia.
- A atividade recente lista só as 20 linhas mais novas das specs abertas, e as perguntas respondidas não aparecem em lugar nenhum.
- O `vostok task evidence --reviewed <arquivo>` regera todos os prints a cada chamada. Como a visão geral mostra o registro, a conferência de um print invalidava a dos outros. Os cinco foram conferidos numa única chamada.
- A massa de dados fictícia vem de um script fora do repositório: `scratchpad/tools/massa.py`.

**Verificação.** Testes, regressão (1 comando igual ao baseline) e evidências passaram. As 6 evidências (5 prints conferidos pelo usuário e `prototipo.diff`) estão em `TASK-03/evidencias/`.
