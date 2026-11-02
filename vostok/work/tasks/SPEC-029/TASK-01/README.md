# TASK-01: Gravar a origem da aprovação na CLI

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-029
- **Status:** done
- **Onda:** 1
- **Dependências:** -

## Arquivos

- [Execução](execucao.md): o que ler, mudar e os passos.
- [Verificação](verificacao.md): baseline, comandos e evidências.
- [Critérios](criterios.md): quando a tarefa está concluída.
- [Resumo executivo](executivo.md)
- [Detalhes técnicos](tecnico.md)
- [Registro](registro.md): etapas da execução e histórico de ajustes.

## Card

- **Título:** Gravar a origem da aprovação na CLI
- **Tipo:** tarefa
- **Descrição:** As aprovações de spec e tarefa gravam `approved_by` (`chat` pela CLI), o `drop_approval` o limpa, e há funções de aprovação e de registro manual para o painel.
- **Critérios de aceite:**
  - [x] As aprovações pela CLI gravam `approved_by: chat`, e um ajuste que derruba a aprovação limpa o campo (RF-07, CT-06, CT-07).
  - [x] Os golden de tarefa e spec continuam passando, e as funções para o painel existem sem argumento de origem na linha de comando (RF-07, RN-01).
- **Rótulos:** SPEC-029, padrao
- **Estimativa:** padrao
- **Bloqueada por:** -
