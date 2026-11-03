# TASK-02: Pedidos e perguntas em pastas por estado

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-033
- **Status:** done
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

- **Título:** Pedidos e perguntas em pastas por estado
- **Tipo:** tarefa
- **Descrição:** Transição por `rename`, histórico do pedido, leitura no painel e migração única do formato antigo.
- **Critérios de aceite:**
  - [x] Iniciar, concluir e recusar um pedido movem o arquivo para a subpasta do novo estado, e o estado no front-matter continua com os valores atuais (RF-08, CT-09).
  - [x] Cada transição deixa uma linha na seção "Histórico" do pedido, com a data, o estado de origem e o de destino (RF-08).
  - [x] Listar os pendentes lê só a subpasta de pendentes (RF-08, RNF-01).
  - [x] Pedidos e perguntas soltos do formato antigo, ou em pastas de nomes antigos, vão uma vez para as subpastas do seu estado, sem mudar o conteúdo (CT-17, CB-07).
  - [x] Com as pastas configuradas em inglês, o pedido vai para elas (RF-08, CT-19).
  - [x] O painel grava o pedido novo em `pendente/`, lê as atividades de todas as subpastas e as perguntas só das abertas, e o commit travado acha o pedido na subpasta (RF-08).
- **Rótulos:** SPEC-033, julgamento
- **Estimativa:** julgamento
- **Bloqueada por:** TASK-01
