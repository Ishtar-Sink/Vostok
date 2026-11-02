# TASK-02: Comando detect

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-026
- **Status:** approved
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

- **Título:** Comando detect
- **Tipo:** historia
- **Descrição:** O `vostok documents detect` lista até 10 cores candidatas do repositório, ignorando pastas geradas e segredos.
- **Critérios de aceite:**
  - [ ] O `detect` lista as cores do repositório com arquivo, linha, nome e valor, primeiro as de marca (RF-02, CT-01).
  - [ ] Pastas geradas e arquivos de segredo não são lidos (RN-05, CT-02).
  - [ ] No máximo 10 cores, com a contagem das que ficaram de fora (CB-07).
  - [ ] Sem cores, a saída diz isso e termina em 0 (RF-03).
- **Rótulos:** SPEC-026, padrao
- **Estimativa:** padrao
- **Bloqueada por:** TASK-01
