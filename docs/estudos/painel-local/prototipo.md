# Protótipo do painel local

- Data: 2026-11-02
- Commit da worktree do spike: `cb57802` (branch `spike/painel-local`, em `/home/victor/projetos/ai-hub-painel`), com as mudanças do protótipo ainda não commitadas
- Variante: `tiny_http` + Preact com compat (`--features painel-tiny`, bundle com `PAINEL_PREACT=1`), a recomendada em [medicoes.md](medicoes.md)
- Persona: ui-ux-designer

Este documento registra o protótipo da TASK-03 da SPEC-027. O protótipo é descartável: o código fica só na worktree e não entra no repositório principal.

## Como reproduzir

```bash
cd /home/victor/projetos/ai-hub-painel/apps/painel && PAINEL_PREACT=1 npm run build
cargo build --release --manifest-path /home/victor/projetos/ai-hub-painel/apps/cli/Cargo.toml --features painel-tiny
B=/home/victor/projetos/ai-hub-painel/apps/cli/target/release/vostok
(cd /home/victor/projetos/ai-hub && "$B" painel)                 # porta livre escolhida pelo sistema; abre o navegador
"$B" painel --porta 4703 --pasta <outra pasta> --sem-navegador   # outra pasta, ao mesmo tempo
```

`vostok painel [--porta N] [--sem-navegador] [--pasta DIR]` sobe as pastas a partir de `DIR` (padrão: a atual) até achar `vostok/config/main.yml`, escuta só em `127.0.0.1` e embute o `dist/` do frontend com `include_dir`.

Para validar todas as etapas, a massa de dados fictícia (projeto "Órbita", 11 specs em todos os status, tarefas em todos os status, plano aberto, perguntas e specs arquivadas) é gerada por um script Python fora do repositório e aberta com `--pasta`.

## Telas

| Rota | Tela | O que mostra |
|---|---|---|
| `/` | Painel | Indicadores, "Aguardando você", specs em andamento com barra por status e ondas, atividade recente |
| `/pendencias` | Pendências | Tudo que espera o usuário: aprovações, finalizações, bloqueios e perguntas do agente |
| `/specs` | Specs | Tabela das specs abertas, arquivadas ou todas, com tipo, progresso e status |
| `/specs/<ID>` | Spec | Documento com índice à esquerda, metadados e tarefas à direita, ações no topo |
| `/specs/<ID>/quadro` | Quadro da spec | Kanban por status (para aprovar, aprovadas, em execução, bloqueadas, concluídas), com onda e dependências |
| `/specs/<ID>/plano` | Plano | O plano aberto, com o botão de aprovar (`/planos/<ID>` leva para cá) |
| `/tarefas` | Tarefas | Quadro de todas as tarefas abertas, com filtro por spec |
| `/tarefas/<SPEC>/<TASK>` | Tarefa | Executivo, técnico, critérios e evidências primeiro; a parte do agente (execução, verificação, registro) abaixo |
| `/documentos` | Documentos | Executivo, técnico e evidências das specs arquivadas |
| `/configuracoes` | Configurações | As opções de `vostok/config/` com nomes da interface e controles por tipo |

Recursos transversais:

- Menu lateral com as specs em andamento e um botão de voltar nas telas internas.
- Popover das referências (RF, RN, CB, CT, CA, PQ e tarefas): prévia ao passar o mouse e fixo com um clique, sem sair da página.
- Ações por status: aprovar spec, plano ou tarefa; pedir planejamento, execução, finalização, ajuste ou desbloqueio; responder pergunta.
  - No protótipo, o diálogo abre e mostra o que acontece, mas a gravação fica para a TASK-04.
- Padrão de tabela (`DataTable`), que vira cartões abaixo de 720 px. O Ishtar Sink ainda não tinha um.
- Estados de carregamento, vazio e erro; tema claro e escuro; foco visível.

Perguntas do agente, no formato proposto: um arquivo por pergunta em `vostok/work/perguntas/P-NNN.md`.
- Front-matter: `id`, `spec`, `task`, `status` (`aberta` ou `respondida`), `created_at` e `author`.
- Corpo: as seções `## Pergunta` e `## Opções`.
- A resposta entra numa seção `## Resposta`.

Configurações:
- **Nomes e seções:** cada opção tem um nome da interface, e as opções ficam agrupadas em seções por arquivo.
- **Controles por tipo de valor:**
  - seleção, abas e caixas de marcar;
  - etiquetas, lista ordenada e número com unidade;
  - formato do ID com prévia, seletor de cor do Atria e amostra de fonte.
- **Só leitura:** as cores derivadas dos temas.
- **Avançado:** o que não tem mapeamento fica num cartão "Avançado", com o nome derivado da chave.

## Marca e componentes

Os componentes vêm copiados do Atria (`packages/ui`), sem os testes:
- **Estrutura e feedback:** botão, cartão, cabeçalho de página, etiqueta, abas, barra de progresso, diálogo, confirmação e aviso;
- **Estados:** vazio, carregamento e erro;
- **Formulário:** campo de texto, de seleção, de cor, de etiquetas, campo segmentado e caixa de marcar.

O frontend usa os tokens do `@atria/theme` (`tokens.css`).

A marca da pasta vem de `GET /api/marca`, que lê `vostok/config/documents.yml`:
- o nome e o ícone do grupo `brand`, mostrados no topo do menu;
- os grupos `light` e `dark`, aplicados como variáveis CSS.

O `vostok documents brand` já grava esses grupos com os mesmos nomes de token do Atria. Por isso o mapeamento é direto, nome por nome:

| Token do Atria | Uso no painel |
|---|---|
| `bg`, `bg_elevated` | Fundo da página e do menu |
| `surface`, `surface_muted`, `surface_hover` | Cartões, colunas do quadro, item com o mouse em cima |
| `border`, `border_strong` | Linhas e bordas de campos |
| `primary`, `primary_hover`, `primary_soft`, `primary_on` | Botões principais, links, item ativo, texto sobre a cor principal |
| `accent`, `accent_soft` | Perguntas do agente |
| `text`, `text_muted`, `text_faint` | Textos |
| `success`, `warning`, `danger`, `info` e os `*_soft` | Status das tarefas e das validações |

Os tokens antigos `color_*` e as sombras não são usados. Sem marca, valem os tokens padrão do `documents.yml`.

Conferido em três pastas:
- **ai-hub:** roxo do harness;
- **Atria:** a marca do Atria;
- **pasta de exemplo:** verde-azulado `#0f766e`, criada com o `vostok documents brand`.

## Atualização ao vivo

A primeira versão usou Server-Sent Events, como pedia o passo 3, e não funcionou com o `tiny_http`:
- a resposta não era enviada até o fim da conexão;
- o navegador ficava esperando a página terminar de carregar.

O protótipo passou a usar WebSocket em `GET /api/ao-vivo`, só do servidor para o navegador:
- o handshake é feito com `sha1`;
- a conexão envia `conectado` e depois `mudou` sempre que a versão de `vostok/work/` muda;
- a versão é a maior data de modificação da pasta;
- o servidor confere a versão a cada 500 ms;
- ao receber `mudou`, cada tela busca os dados de novo, sem piscar;
- se a conexão cai, o navegador tenta de novo a cada 2 s, e o topo mostra "Conectando", "Ao vivo" ou "Reconectando".

Comparação entre a consulta periódica e a crate `notify` 8, na mesma variante. A pasta `work/` tem 1 358 arquivos no ai-hub e 1 553 no Atria; os números vêm de uma aba aberta.

| Forma | Binário | tar.gz | Memória (RSS) | Threads | CPU parado | CPU com 1 aba | Atraso do aviso |
|---|---:|---:|---:|---:|---:|---:|---:|
| Consulta periódica (ai-hub) | 5 483 984 | 2 068 511 | 5,1 MB | 6 + 1 por aba | 0% | 1,63% | 321 a 325 ms (média 324) |
| Consulta periódica (Atria) | 5 483 984 | 2 068 511 | 5,0 MB | 6 + 1 por aba | 0% | 1,80% | 319 a 501 ms (média 355) |
| `notify` (ai-hub) | 5 729 000 | 2 149 561 | 5,7 MB | 6 + 2 por aba | 0% | 0,00% | 299 a 326 ms (média 304) |
| `notify` (Atria) | 5 729 000 | 2 149 561 | 5,6 MB | 6 + 2 por aba | 0% | 0,03% | 299 a 304 ms (média 300) |

Leitura dos números:
- **Tamanho:** o `notify` acrescenta 245 016 bytes ao binário e 81 050 ao tar.gz.
- **Processador:** o `notify` tira o uso de 1,6% a 1,8% com uma aba aberta, que é o custo de varrer `work/` a cada 500 ms.
- **Atraso:** o `notify` quase não ganha, porque o laço de 500 ms ainda limita o aviso.
  - Para o `notify` render de verdade, o aviso deveria sair no próprio evento.

Recomendação para a TASK-05:
- **Agora:** consulta periódica, sem dependência e com custo baixo.
- **Depois:** o `notify` com aviso no próprio evento, se pastas grandes tornarem a varredura cara.

Script da medição: `consumo.py`, guardado fora do repositório.

## Tamanho do binário

O protótipo completo, em `--release` com `painel-tiny`, tem estes tamanhos:

| Parte | Tamanho |
|---|---:|
| Binário com o painel | 5 483 984 bytes |
| Binário compactado (tar.gz) | 2 068 511 bytes |
| Código do frontend (bundle JS) | 158 640 bytes |
| Bundle JS com gzip | 50 869 bytes |

Comparado com a `base` da TASK-02, o painel acrescenta 897 128 bytes (+19,6%) ao binário e 286 268 bytes ao tar.gz. O que pesa mais:
- o `tiny_http`;
- o frontend, que subiu de 118 KB para 158 KB com os campos de formulário do Atria (seletor de cor, campo de seleção e de etiquetas).

O total já passa o limite de 5,3 MB proposto em [medicoes.md](medicoes.md). A TASK-05 revisa o limite com o painel completo.

## Dois painéis ao mesmo tempo

Painéis abertos ao mesmo tempo com o mesmo binário, cada um com os dados e a marca da sua pasta:

| Porta | Pasta |
|---|---|
| 4700 | ai-hub |
| 4701 | Atria |
| 4702 | cópia de exemplo com plano aberto |
| 4703 | massa de dados fictícia |

`ss -ltnp | grep vostok` mostra só `127.0.0.1`:

```text
LISTEN 0 128 127.0.0.1:4700 0.0.0.0:* users:(("vostok",...))
LISTEN 0 128 127.0.0.1:4701 0.0.0.0:* users:(("vostok",...))
LISTEN 0 128 127.0.0.1:4702 0.0.0.0:* users:(("vostok",...))
LISTEN 0 128 127.0.0.1:4703 0.0.0.0:* users:(("vostok",...))
```

Sem `--porta`, o sistema escolhe uma porta livre para cada painel, e o painel imprime a URL.

## Retorno do usuário

O usuário usou o protótipo em rodadas e pediu os ajustes abaixo, todos aplicados antes da aprovação:

1. Menu lateral como o do Atria e atualização em tempo real (WebSocket ou similar), com a medição de consumo no PC do usuário.
2. Um painel de verdade:
   - menus para specs e documentos;
   - mais de uma spec em execução ao mesmo tempo;
   - um padrão de tabela, que o Ishtar Sink ainda não tem;
   - índice à esquerda na spec;
   - a tarefa com metadados à direita, o conteúdo humano primeiro (executivo e técnico) e a parte do agente abaixo.
3. Ações: aprovar tarefas e responder perguntas do agente (para o loop), e um quadro kanban por spec, porque as tarefas nem sempre rodam em sequência.
4. Quadro acessível de dentro da spec, botão de voltar e popover das referências internas.
5. Configurações do harness da pasta e botões de ação dentro da spec e da tarefa.
6. Configurações com nomes da interface no lugar das chaves do YAML e controles próprios (seleção, caixa de marcar) em vez de texto livre, mais uma massa de dados fictícia para validar todas as etapas.
7. Revisão dos espaçamentos onde havia texto colado.

Retorno final: "aprovado".
