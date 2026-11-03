# TASK-01: Configurar pastas de comunicação

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-033
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

- **Título:** Configurar pastas de comunicação
- **Tipo:** tarefa
- **Descrição:** Caminhos de pedidos, perguntas e avais, subpastas por estado e arquivo de avais sem spec em `config/main.yml`, lidos e validados pela CLI.
- **Critérios de aceite:**
  - [ ] Sem as chaves novas no `config/main.yml`, a CLI usa `work/pedidos`, `work/perguntas`, `work/avais` e as subpastas em português (RF-16).
  - [ ] Com os caminhos e as subpastas trocados para inglês na configuração, a CLI devolve os nomes configurados (RF-16, CT-19).
  - [ ] Dois estados com a mesma pasta, ou um nome vazio, com `/` ou `..`, são recusados com a mensagem que cita os estados ou a chave (RF-16, CT-20).
  - [ ] `request_dispatch` vem vazio por padrão e só aceita `codex` ou `claude` (RF-16, base do RF-13).
  - [ ] A leitura da configuração em três níveis não muda as leituras de dois níveis nem a atualização do harness (RF-16).
- **Rótulos:** SPEC-033, padrao
- **Estimativa:** padrao
- **Bloqueada por:** -
