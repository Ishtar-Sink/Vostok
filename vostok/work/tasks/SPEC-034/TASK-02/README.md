# TASK-02: Ações de plano e tarefa no painel

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-034
- **Status:** done
- **Onda:** 2
- **Dependências:** TASK-01

## Arquivos

- [Execução](execucao.md): o que ler, mudar e os passos.
- [Verificação](verificacao.md): baseline, comandos e evidências.
- [Critérios](criterios.md): quando a tarefa está concluída.
- [Resumo executivo](executivo.md)
- [Detalhes técnicos](tecnico.md)
- [Registro](registro.md): etapas da execução e histórico de ajustes.

## Card

- **Título:** Ações de plano e tarefa no painel
- **Tipo:** tarefa
- **Descrição:** Aviso de plano desatualizado, "Refazer o plano", "Ajustar plano", "Finalizar" em validação e textos do aval no painel.
- **Critérios de aceite:**
  - [x] Com o plano desatualizado, a spec não oferece "Aprovar plano" e a spec aprovada oferece "Refazer o plano", que grava o pedido `planejar-spec` (RF-02, RF-03, CT-02).
  - [x] Pendências não lista "Aprovar plano" para um plano desatualizado e continua listando para o plano em dia (RF-02, CT-03).
  - [x] A página da spec mostra o aviso "Plano desatualizado", com o porquê (a spec mudou depois do plano), no cabeçalho, no documento e na visão do plano, em tom de pendência e não de erro (RF-02, CT-04).
  - [x] A spec de volta a refinada com um plano antigo mostra só "Aprovar spec" e "Pedir ajuste", sem botão de plano (RF-02, CT-05, CB-01).
  - [x] Com o plano em dia, a spec oferece "Aprovar plano" e, ao lado, "Ajustar plano", cujo envio fica desligado até haver texto e grava o pedido `ajustar-plano` com ele (RF-04, CT-06).
  - [x] A tarefa em validação tem o botão de finalizar ("Pedir finalização"), que grava o pedido `finalizar-tarefa`, mesmo com print ainda não conferido (RF-05, CT-07, CB-03).
  - [x] A ajuda dos diálogos de aprovar spec e tarefa e a mensagem de sucesso da aprovação citam o log de avais da spec, e nenhum texto do painel cita `avais.log` (RF-07, CT-09).
  - [x] Os botões novos e o campo do ajuste são achados por papel e rótulo, e os testes que já existiam e a checagem de tipos continuam passando (RF-02, RF-04, RF-05).
- **Rótulos:** SPEC-034, padrao
- **Estimativa:** padrao
- **Bloqueada por:** TASK-01
