# TASK-04: Implementar a API de leitura e a marca

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-029
- **Status:** done
- **Onda:** 2
- **Dependências:** TASK-02

## Arquivos

- [Execução](execucao.md): o que ler, mudar e os passos.
- [Verificação](verificacao.md): baseline, comandos e evidências.
- [Critérios](criterios.md): quando a tarefa está concluída.
- [Resumo executivo](executivo.md)
- [Detalhes técnicos](tecnico.md)
- [Registro](registro.md): etapas da execução e histórico de ajustes.

## Card

- **Título:** Implementar a API de leitura e a marca
- **Tipo:** tarefa
- **Descrição:** As rotas de leitura (`visao`, `specs`, `planos`, `tarefas`, `documentos`, `configuracoes`, `marca`, `fonte`, `arquivo`) leem os arquivos de `vostok/` a cada requisição.
- **Critérios de aceite:**
  - [x] A API de leitura devolve specs, planos, tarefas, documentos, configurações e marca lidos dos arquivos a cada requisição (RF-02, RF-04, CT-03).
  - [x] Caminhos fora de `vostok/` são recusados, e uma pasta sem marca usa os tokens padrão (CB-05).
- **Rótulos:** SPEC-029, padrao
- **Estimativa:** padrao
- **Bloqueada por:** TASK-02
