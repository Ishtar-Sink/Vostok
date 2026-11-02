# TASK-08: Criar a skill do loop de pedidos

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-030
- **Status:** done
- **Onda:** 2
- **Dependências:** TASK-04

## Arquivos

- [Execução](execucao.md): o que ler, mudar e os passos.
- [Verificação](verificacao.md): baseline, comandos e evidências.
- [Critérios](criterios.md): quando a tarefa está concluída.
- [Resumo executivo](executivo.md)
- [Detalhes técnicos](tecnico.md)
- [Registro](registro.md): etapas da execução e histórico de ajustes.

## Card

- **Título:** Criar a skill do loop de pedidos
- **Tipo:** tarefa
- **Descrição:** Skill `process-requests` com a instrução do loop, no catálogo e no mapa de contexto.
- **Critérios de aceite:**
  - [x] A skill `process-requests` traz a instrução do loop, com o mapa das 11 ações para as skills, e está no catálogo e no mapa de contexto (RF-14).
  - [x] `/process-requests <intervalo>` arma o loop sozinho e processa um pedido do painel até `feito` ou `recusado`; sem intervalo, faz uma rodada (CT-15, RN-04).
- **Rótulos:** SPEC-030, padrao
- **Estimativa:** padrao
- **Bloqueada por:** TASK-04
