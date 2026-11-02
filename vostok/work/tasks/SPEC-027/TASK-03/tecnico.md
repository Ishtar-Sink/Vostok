<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Estende o spike da TASK-02 em `/home/victor/projetos/ai-hub-painel`: API de leitura em JSON no `vostok painel`, eventos por Server-Sent Events e as telas em React com os componentes copiados do Atria. Os `.md` são renderizados no navegador. O resultado vai para `docs/estudos/painel-local/prototipo.md` e para os prints. Cobre PQ-01, PQ-09, PQ-10 e PQ-12; sustenta CA-05 e CA-07.

**Decisões.** A marca vem de `config/documents.yml` (inferência F, decisão 6 da spec), mapeada sobre os tokens de `@atria/theme`. A atualização ao vivo parte da consulta periódica (sem dependência) e é comparada com a crate `notify`, em tamanho e atraso. O plano de exemplo é a cópia do plano da SPEC-027 guardada em `plano-exemplo.md`, porque nenhuma spec da pasta tem plano aberto. `GET /api/arquivo` só serve caminhos dentro de `vostok/`.

**Riscos.** Componentes do Atria com dependências do app dele (roteador, consultas): copiar só os de apresentação. Cores da marca sem contraste: aplicar o mesmo ajuste de 4,5:1 de `documents.rs`.

**Arquivos em comum na onda.** Nenhum: é a única tarefa da onda 2.
