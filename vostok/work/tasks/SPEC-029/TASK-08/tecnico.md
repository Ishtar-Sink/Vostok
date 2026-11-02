<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Troca as páginas provisórias de `specs`, `spec` e `board` pelas do spike. O quadro (`board.tsx`) fica na própria feature `board`, usado pela spec e pela lista de tarefas, que são desta mesma tarefa. Cobre RF-03, RF-09, CT-03 e CB-07.

**Decisões.** O plano é uma aba da spec, como no protótipo aprovado; uma tela de plano separada foi descartada. O botão de editar só aparece para caminhos sob `vostok/work/` que não estão arquivados (CB-07).

**Riscos.** Contrato do JSON: os nomes seguem o spike (TASK-04).

**Arquivos em comum na onda.** Nenhum: cada tarefa de telas muda só as próprias pastas de feature, e as ações compartilhadas já estão em `shared/` (TASK-03).
