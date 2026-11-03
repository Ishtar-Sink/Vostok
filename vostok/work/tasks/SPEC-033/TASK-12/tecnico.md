<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.**
- Largura: com a sidebar de 252 px e o padding de 24 px da página, o quadro tem cerca de 1140 px a 1440 px de tela. Seis colunas de 220 px mais cinco espaços de 12 px pedem 1380 px, por isso a sexta fica atrás da rolagem. Com `minmax(160px, 1fr)`, as seis pedem 1020 px e se esticam até ~180 px cada; abaixo disso, a rolagem lateral continua como hoje.
- Título com barras: o `estado.yml` grava o título com `yaml_escape` (`\"` e `\\`), e o painel lê com `config_get` (quadro, `task_summary`) e com o `yaml_json` (página da tarefa), que tiram as aspas sem desfazer o escape. A correção é na CLI, em `read.rs`: os valores entre aspas duplas passam pela mesma função que o `frontmatter_get` já usa, agora pública como `yaml_unescape`.
- Cor de "Em validação": a bolinha deixa de ser um `::before` por coluna no CSS e passa a ser um `span` com `data-tone`, renderizado pelo `board.tsx` para as colunas do fluxo. Isso permite testar a cor e deixa um lugar só para os tons.

**Decisões.**
- Corrigir o escape na leitura do painel (`read.rs`) e não no `config_get`: descartado mudar o `config_get`, usado em mais de 50 pontos da CLI para ler configuração, onde uma barra dentro de aspas pode ser literal. A leitura do `estado.yml` do painel é o único ponto com o defeito visível.
- Corrigir na CLI e não no frontend: descartado desfazer o escape no `board.tsx`, porque o mesmo texto chega também à página da tarefa, à lista de tarefas da spec e às pendências; a API passa a entregar o texto certo para todos.
- Tom `accent` (verde-azulado) para "Em validação": descartado repetir o `info` de "Em execução", porque as duas colunas ficam lado a lado e a cor igual não as distingue; o selo "validando" da tarefa continua `info` (`status.ts`), fora do escopo.
- Colunas de 160 px no mínimo, em vez de quebrar o quadro em duas linhas: descartada a quebra, que separa o fluxo e confunde a ordem das etapas.

**Riscos.** Cartões mais estreitos quebram mais os títulos longos; o `flex-wrap` no topo do cartão evita que o id com a spec vaze na página de tarefas. A mudança em `read.rs` troca a leitura do `task_summary` de `config_get` por uma leitura única do arquivo, com o mesmo resultado para valores sem aspas.

**Arquivos fora dos componentes do plano.** `apps/cli/src/painel/read.rs` e `apps/cli/src/support.rs`, porque a causa do título com barras está na leitura da CLI. O `read.rs` também é alterado pela TASK-02 e pela TASK-06, em ondas posteriores (2 e 3); na onda 1 não há arquivo em comum com a TASK-01.

**Referências da spec.** RF-14, CT-15, CA-07, DT-017.
