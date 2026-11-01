# TASK-01: Documentos preservados e exigidos no código

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-024
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

- **Título:** Documentos preservados e exigidos no código
- **Tipo:** bug
- **Descrição:** A gravação da pasta deixa o executivo e o técnico intactos, a aprovação recusa os dois no template e o finish exige o enriquecimento desde o start.
- **Critérios de aceite:**
  - [x] O `finish` deixa o executivo e o técnico com títulos como foram escritos, e o registro só ganha a linha de conclusão (RF-01, CT-01).
  - [x] Nenhum outro comando muda os documentos, com títulos de qualquer nível ou código (RF-02, CT-02).
  - [x] A aprovação recusa documento no template (RF-03, CT-03).
  - [x] O `start` grava os hashes e o `finish` exige o enriquecimento, sem travar tarefas antigas (RF-05, RF-06, RF-07, CT-04, CT-05, CT-06).
  - [x] Os cenários de referência não mudaram (CT-08).
- **Rótulos:** SPEC-024, julgamento
- **Estimativa:** julgamento
- **Bloqueada por:** -
