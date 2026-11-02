# TASK-11: Integrar o painel e a release

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-029
- **Status:** done
- **Onda:** 3
- **Dependências:** TASK-04, TASK-05, TASK-06, TASK-07, TASK-08, TASK-09, TASK-10

## Arquivos

- [Execução](execucao.md): o que ler, mudar e os passos.
- [Verificação](verificacao.md): baseline, comandos e evidências.
- [Critérios](criterios.md): quando a tarefa está concluída.
- [Resumo executivo](executivo.md)
- [Detalhes técnicos](tecnico.md)
- [Registro](registro.md): etapas da execução e histórico de ajustes.

## Card

- **Título:** Integrar o painel e a release
- **Tipo:** tarefa
- **Descrição:** Testes de integração do painel em duas pastas, passo de Node e bundle na release e conferência do limite de 6,0 MB e 2,3 MB.
- **Critérios de aceite:**
  - [x] Dois painéis rodam ao mesmo tempo em portas diferentes, só em `127.0.0.1`, cada um com a sua marca, e o fluxo completo funciona (RF-01, CT-01, RNF-02).
  - [x] A release gera o bundle e confere o tamanho, e o binário cabe em 6,0 MB e 2,3 MB (RF-13, CT-15, RNF-01).
  - [x] A suíte da CLI passa com e sem a feature (RNF-04).
- **Rótulos:** SPEC-029, padrao
- **Estimativa:** padrao
- **Bloqueada por:** TASK-04, TASK-05, TASK-06, TASK-07, TASK-08, TASK-09, TASK-10
