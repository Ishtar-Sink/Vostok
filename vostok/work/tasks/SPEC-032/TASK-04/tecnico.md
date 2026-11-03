<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** O painel passa a receber `check_result` no resumo de cada tarefa e a regra da pendência em `attentionItems` exige `passed`. `taskActions`, usada na tela da tarefa, fica como está (RF-04).

**Decisões.** Tarefas iniciadas antes desta versão, sem `check_result`, saem das pendências até a próxima conferência (CB-01); o usuário aceitou esse efeito no refinamento. Os arquivos `api.ts` e `read.rs` também são mudados pela TASK-05, de outra onda; as mudanças são em pontos diferentes.

**Riscos.** Esquecer o campo no `task_summary` faria a pendência nunca aparecer; o teste de `read.rs` e o de `attention` conferem.

**Referências da spec.** RF-04, RN-01, CT-06, CB-01.

**Arquivos em comum com a onda 3.** Nenhum entre TASK-04, TASK-06 e TASK-07.
