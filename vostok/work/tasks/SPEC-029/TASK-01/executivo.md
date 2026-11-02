<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

Prepara a CLI para distinguir quem aprovou: pelo chat ou pelo painel. É a base para o painel aprovar sem enfraquecer as regras.
Perfil padrao. Risco: quebrar os testes de equivalência com os scripts antigos.
Roda junto com o servidor e o frontend; desbloqueia as ações do painel (TASK-06).

## O que foi feito

- **Origem da aprovação:** toda aprovação feita pelo chat passa a ficar registrada como vinda do chat. Quando um ajuste derruba uma aprovação, esse registro também é apagado.
- **Base para o painel:** a CLI ganhou as funções que o painel vai usar para aprovar e registrar edições. Pela linha de comando, o agente não tem como se passar pelo painel.
- **Testes antigos:** os testes que comparam a CLI com os scripts antigos continuam passando.
