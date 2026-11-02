<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Troca as páginas provisórias de `specs`, `spec` e `board` pelas do spike. O quadro (`board.tsx`) fica na própria feature `board`, usado pela spec e pela lista de tarefas, que são desta mesma tarefa. Cobre RF-03, RF-09, CT-03 e CB-07.

**Decisões.** O plano é uma aba da spec, como no protótipo aprovado; uma tela de plano separada foi descartada. O botão de editar só aparece para caminhos sob `vostok/work/` que não estão arquivados (CB-07).

**Riscos.** Contrato do JSON: os nomes seguem o spike (TASK-04).

**Arquivos em comum na onda.** Nenhum: cada tarefa de telas muda só as próprias pastas de feature, e as ações compartilhadas já estão em `shared/` (TASK-03).


## O que foi feito

**O que mudou:**
- `features/specs/specs-page.tsx`, `features/spec/spec-page.tsx`, `features/board/board.tsx` e `features/board/tasks-page.tsx`, com os CSS de cada um.
- **Testes:** 13, que falharam antes do código.

**Decisões:**
- **Quadro compartilhado:** o quadro (`board.tsx`) é usado pela spec, que o importa de `../board/board`. As duas features são desta mesma tarefa, como a própria tarefa previa. Por isso, o critério "nenhuma tela importa outra" foi lido como "nenhuma tela importa feature de outra tarefa": essa é a única linha que o grep lista.
- **CSS:** `.filters` foi duplicado em `board.module.css`, para `tasks-page` não depender de `features/specs`.

**Ponto de atenção:**
- A aba "Plano" só aparece quando a spec tem plano aberto.
- No print do quadro, as colunas da direita ficam fora porque o quadro rola na horizontal.

**Verificação:**
- `tsc`, vitest e build passaram.
- A regressão não encontrou diferença.
- O teste gravou `task-08-testes.txt`, e os prints `specs.png`, `spec.png` e `quadro.png` foram conferidos pelo usuário.
