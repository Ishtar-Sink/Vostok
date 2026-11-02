<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Troca as páginas provisórias de `dashboard` e `inbox` pelas do spike, importando as pendências e as ações de `shared/`. Cobre RF-03, RF-06, RF-11 e CT-03.

**Decisões.** As telas são as aprovadas pelo usuário no protótipo, sem mudança de desenho. Os prints usam o binário do spike pela variável `PAINEL_BIN`, porque a API nova roda na mesma onda; a TASK-11 refaz os prints com o binário novo.

**Riscos.** O contrato do JSON mudar na TASK-04: a TASK-04 mantém os nomes do spike.

**Arquivos em comum na onda.** Nenhum: cada tarefa de telas muda só as próprias pastas de feature, e as ações compartilhadas já estão em `shared/` (TASK-03).
