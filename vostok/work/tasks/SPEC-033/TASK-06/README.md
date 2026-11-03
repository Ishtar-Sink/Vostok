# TASK-06: Histórico de status nas atividades

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-033
- **Status:** done
- **Onda:** 3
- **Dependências:** TASK-02

## Arquivos

- [Execução](execucao.md): o que ler, mudar e os passos.
- [Verificação](verificacao.md): baseline, comandos e evidências.
- [Critérios](criterios.md): quando a tarefa está concluída.
- [Resumo executivo](executivo.md)
- [Detalhes técnicos](tecnico.md)
- [Registro](registro.md): etapas da execução e histórico de ajustes.

## Card

- **Título:** Histórico de status nas atividades
- **Tipo:** tarefa
- **Descrição:** Linha de histórico nas mudanças de status da spec e as transições dos pedidos na `GET /api/atividades`.
- **Critérios de aceite:**
  - [x] Cada mudança de status da spec (approved, planned, implemented, archived) ganha uma linha de histórico, gravada pelo comando que já faz a mudança (RF-12, CT-13).
  - [x] A linha de histórico não muda o hash do conteúdo da spec nem quebra as fixtures de equivalência (RF-12).
  - [x] A `GET /api/atividades` mostra o histórico de status das specs e cada transição dos pedidos, também depois de arquivados (RF-12, CT-13).
  - [x] O código passa no clippy e no rustfmt (RF-12).
- **Rótulos:** SPEC-033, padrao
- **Estimativa:** padrao
- **Bloqueada por:** TASK-02
