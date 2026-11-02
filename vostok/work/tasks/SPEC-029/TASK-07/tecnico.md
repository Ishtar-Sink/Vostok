<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Troca as páginas provisórias de `dashboard` e `inbox` pelas do spike, importando as pendências e as ações de `shared/`. Cobre RF-03, RF-06, RF-11 e CT-03.

**Decisões.** As telas são as aprovadas pelo usuário no protótipo, sem mudança de desenho. Os prints usam o binário do spike pela variável `PAINEL_BIN`, porque a API nova roda na mesma onda; a TASK-11 refaz os prints com o binário novo.

**Riscos.** O contrato do JSON mudar na TASK-04: a TASK-04 mantém os nomes do spike.

**Arquivos em comum na onda.** Nenhum: cada tarefa de telas muda só as próprias pastas de feature, e as ações compartilhadas já estão em `shared/` (TASK-03).


## O que foi feito

**O que mudou:**
- **Painel:** `features/dashboard/dashboard-page.tsx` e o CSS dele.
- **Pendências:** `features/inbox/inbox-page.tsx` e o CSS dele.
- **Origem do código:** o spike, com as ações vindas de `shared/action-bar` e `shared/attention`.
- **Testes:** 11, em `dashboard-page.test.tsx` e `inbox-page.test.tsx`.

**Ponto de atenção:**
- A lista de pendências existe duas vezes, uma em cada página, cada uma com o próprio CSS, porque a página do painel não importa `features/inbox`.
- Para ter uma só, ela teria de ir para `shared/` numa mudança futura.

**Verificação:**
- `tsc`, vitest e build passaram.
- A regressão não encontrou diferença.
- O teste gravou `task-07-testes.txt`, e os prints `painel.png` e `pendencias.png` foram conferidos pelo usuário.
