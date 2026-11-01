<!-- Escrito na finalização da tarefa. -->

Alinha as skills approve-plan, finish-task, approve-task e adjust-task, os templates executivo.md e tecnico.md e a persona do arquiteto ao ciclo da SPEC-024: escrita no detalhamento (RF-04), enriquecimento na conclusão (RF-08) e comentário dos templates (RF-09). O comportamento do vostok que as skills descrevem, a recusa na aprovação e no finish, é entregue pela TASK-01; esta tarefa só muda texto e pode rodar em paralelo, porque não divide arquivos com ela. A alternativa de deixar o planejador escrever os dois documentos foi descartada: o arquiteto já toma as decisões de desenho que vão no técnico (inferência C da spec). As evidências são diffs, como na TASK-09 da SPEC-023.

## O que foi feito

Foram alterados sete arquivos de texto: as skills approve-plan, finish-task, approve-task e adjust-task, a persona do arquiteto e os dois templates de documento, nos quais só o comentário da primeira linha mudou. No detalhamento, a approve-plan manda preencher também no executivo e no técnico com o que a tarefa vai fazer, e a aprovação recusa os que ainda estão no modelo. Na conclusão, a finish-task enriquece os documentos em vez de escrevê-los do zero; a adjust-task deixa claro que ajustá-los não pede nova aprovação e conta como enriquecimento. Uma frase da finish-task foi corrigida depois da execução, de "escreva" para "enriqueça". O trabalho foi feito por um subagente, e as evidências são sete diffs, um por arquivo alterado.

Ponto de atenção: as skills descrevem recusas na aprovação e na conclusão que só existem com a correção de código da TASK-01; as duas partes saem juntas na mesma versão.
