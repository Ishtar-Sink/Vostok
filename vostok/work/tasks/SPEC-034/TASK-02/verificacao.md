### Baseline

- `pnpm --dir apps/painel exec vitest run src/shared/attention.test.ts src/features/spec/ src/features/task/ --reporter=verbose 2>&1 | grep -E '✓|✗|×' | sed 's/ [0-9]*ms$//' | sort | grep -v 'desatualizado\|plano em dia\|em validação\|log de avais\|refazer'`
- `pnpm --dir apps/painel exec tsc --noEmit`

### Verificação

```bash
# evidência: acoes-plano-tarefa-testes.txt
pnpm --dir apps/painel exec vitest run src/shared/attention.test.ts src/features/spec/ src/features/task/ --reporter=verbose
pnpm --dir apps/painel exec tsc --noEmit
! grep -rn 'avais\.log' apps/painel/src
```

### Evidências

- `acoes-plano-tarefa-testes.txt` (automática, da verificação)
  - Prova: com o plano desatualizado não há "Aprovar plano" na spec nem em Pendências e a spec aprovada oferece "Refazer o plano" (`planejar-spec`); a spec refinada com plano antigo só tem as ações dela; o plano em dia tem "Aprovar plano" e "Ajustar plano" com texto obrigatório (`ajustar-plano`); a página mostra o aviso na spec e na visão do plano; a tarefa em `validating` grava `finalizar-tarefa`; a ajuda e a mensagem da aprovação citam o log da spec (RF-02, RF-03, RF-04, RF-05, RF-07, CT-02 a CT-07, CT-09, CB-01, CB-03).
  - Trecho: com ✓, `oferece finalizar na tarefa em validação`, `troca aprovar por refazer com o plano desatualizado`, `não oferece ação de plano na spec refinada com plano desatualizado`, `oferece ajustar e aprovar com o plano em dia`, `não lista aprovar plano com o plano desatualizado`, `cita o log de avais da spec na ajuda das aprovações`, `mostra o aviso de plano desatualizado na spec`, `mostra o aviso de plano desatualizado na visão do plano`, `pede refazer o plano desatualizado`, `pede o ajuste do plano em dia com texto`, `cita o log de avais da spec ao aprovar` e `pede a finalização da tarefa em validação`.
  - Falha: algum desses com ×, um teste antigo quebrado ou erro do `tsc`.
- `plano-desatualizado.png` (manual): print da página de uma spec com o plano desatualizado, tirado no projeto de demonstração com o binário do passo 9. Para chegar ao estado: refinar, aprovar e planejar uma spec; ajustá-la pelo painel e aprová-la de novo; abrir `/specs/<SPEC>` na visão "Documento", em 1280x900.
  - Prova: o aviso visível na tela real, com o porquê, e a troca do botão (RF-02, RF-03, CT-02, CT-04).
  - Trecho: no cabeçalho, ao lado do selo do status "aprovada", o selo amarelo "Plano desatualizado" e o botão primário "Refazer o plano", sem "Aprovar plano" nem "Ajustar plano"; na coluna lateral, a linha "Plano: Desatualizado: a spec mudou depois do plano."
  - Falha: "Aprovar plano" na barra, sem o selo, selo em vermelho (tom de erro) ou sem o texto do porquê.
- `tarefa-em-validacao.png` (manual): print da página de uma tarefa em validação, tirado no projeto de demonstração com o binário do passo 9. Para chegar ao estado: executar e finalizar pelo painel uma tarefa com evidência `(print)`, sem conferir o print; abrir `/tarefas/<SPEC>/<TASK>`, em 1280x900.
  - Prova: a tarefa em `validating` tem o botão de finalizar na tela real (RF-05, CT-07, CB-03).
  - Trecho: no cabeçalho, o selo "validando" e os botões "Pedir finalização" (primário) e "Pedir ajuste".
  - Falha: só "Pedir ajuste" no cabeçalho.
