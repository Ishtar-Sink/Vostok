<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Tarefa documental: escreve `docs/estudos/painel-local.md` a partir de `levantamento.md`, `medicoes.md`, `prototipo.md` e `testes.md`, copia os prints da TASK-03 para `docs/estudos/painel-local/imagens/` e cria o `README.md` do índice. Cobre de PQ-01 a PQ-12; sustenta CA-01, CA-02, CA-03, CA-04, CA-11, CA-12 e CA-13.

**Decisões.** Uma subseção por pergunta, para a rastreabilidade com a spec; os números e as tabelas ficam nos documentos de apoio e são resumidos no estudo, no formato de `tarefa-como-pasta.md`. As imagens são copiadas porque a pasta de evidências é arquivada com a spec e o estudo precisa delas na `main`.

**Riscos.** Copiar algo do spike para a `main` por engano: a verificação confere `apps` e `packages`.

**Arquivos em comum na onda.** Nenhum: é a única tarefa da onda 4.

## O que foi feito

**Arquivos criados:**
- `docs/estudos/painel-local.md`, com as seções Escopo e método, Conclusão, Respostas, Medições, Protótipo, Arquitetura recomendada, Specs propostas e Remoção do servidor. Respostas tem 12 subseções `### PQ-NN`, cada uma com a conclusão e o link para o documento de apoio.
- `docs/estudos/painel-local/README.md`, com o índice dos 4 documentos de apoio e das 5 imagens e a nota de que o código do protótipo fica só no branch `spike/painel-local`.
- `docs/estudos/painel-local/imagens/`, com a cópia dos 5 `.png` da TASK-03.

**Decisões do estudo:**
- **Limite do binário:** revisto de 5,3 MB e 2,0 MB para 6,0 MB e 2,3 MB.
  - O protótipo completo mede 5 483 984 bytes e 2 068 511 compactados. O limite anterior foi calculado com o app mínimo da TASK-02, com bundle de 30 KB.
  - O novo limite deixa 516 KB de folga e recusa as variantes com `axum` (cerca de 537 KB a mais) e com React (cerca de 200 KB a mais).
- **Bundle:** gerado na CI da release, com a feature `painel`. Sem `dist/`, o `vostok` compila sem o painel.
  - Alternativa descartada: versionar o `dist/`, por causa dos arquivos gerados no git e do risco de um bundle velho no binário.
- **Arquitetura recomendada:** `tiny_http`, Preact com compat em `apps/painel` e WebSocket com consulta periódica.
  - Aprovação no mesmo processo, com `approved_by` e `avais.log`.
  - Pedidos em `work/pedidos/` e perguntas em `work/perguntas/`.
  - Cada parte da tabela traz a alternativa descartada e o motivo.
- **Ordem das specs propostas:**
  1. remover a premissa de servidor;
  2. trava de aval;
  3. painel de leitura;
  4. ações e edição;
  5. pedidos e loop.

**Pontos de atenção:**
- A troca de React por Preact exige mudar a regra `stack-02`, o que fica para a primeira spec proposta.
- `medicoes.md` chama de "Commit da worktree" o commit de base do spike (`cb57802`). O arquivo não estava em Mudar, por isso não foi corrigido.
- A evidência `sem-spike.txt` tem a mesma saída do baseline (nenhuma linha em `apps` e `packages`, contagem `0`). É o esperado, porque ela comprova que nada do spike entrou.

**Verificação:**
- **Testes:** passaram. O `grep` mostrou `12` subseções e as 8 seções, sem nenhuma linha de `git status` em `apps` e `packages`.
- **Regressão:** sem regressão. O `ls docs/estudos` só ganhou a linha `painel-local.md`.
- **Evidências:** `estudo.diff` e `sem-spike.txt`, com resultado `passed`.
