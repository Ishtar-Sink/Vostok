# TASK-03: Nome, ícone e coluna de IDs nos documentos

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-026
- **Status:** approved
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

- **Título:** Nome, ícone e coluna de IDs nos documentos
- **Tipo:** historia
- **Descrição:** O build mostra o nome e embute o ícone configurados no topo dos três documentos, mantém a identidade atual sem configuração e deixa os IDs da cobertura numa linha.
- **Critérios de aceite:**
  - [ ] O nome configurado aparece no topo dos três documentos, escapado, e o rodapé cita o Vostok (RF-06, RN-04, CB-08).
  - [ ] O ícone configurado vai embutido no topo, sem requisição externa (RF-07, RNF-02).
  - [ ] Ícone ausente, grande demais ou de formato não suportado faz o build falhar com o motivo (CB-02, CB-03, RNF-05).
  - [ ] Sem identidade configurada, os documentos ficam como hoje e os goldens não mudam (RF-08, RNF-04).
  - [ ] Os IDs do mapa de cobertura ficam numa linha só (RF-12).
  - [ ] Nenhuma cor fixa nova nos templates (RNF-01).
- **Rótulos:** SPEC-026, padrao
- **Estimativa:** padrao
- **Bloqueada por:** -
