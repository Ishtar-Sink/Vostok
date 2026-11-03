# TASK-03: Gravar o resultado do `vostok task check`

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-032
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

- **Título:** Gravar o resultado do `vostok task check`
- **Tipo:** tarefa
- **Descrição:** Grava `check_result` e `checked_at` no `estado.yml` e a linha `check` no registro, limpos no ajuste e no resume.
- **Critérios de aceite:**
  - [ ] O `vostok task check` grava `check_result` (`passed` ou `failed`), `checked_at` e a linha `check` no registro (RF-03, CT-03, CT-04).
  - [ ] O ajuste e o resume limpam `check_result` (RF-03, CT-05).
- **Rótulos:** SPEC-032, padrao
- **Estimativa:** padrao
- **Bloqueada por:** TASK-01
