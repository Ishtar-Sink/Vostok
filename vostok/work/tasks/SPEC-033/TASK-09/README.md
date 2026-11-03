# TASK-09: Botões de spec no painel

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-033
- **Status:** done
- **Onda:** 4
- **Dependências:** TASK-08

## Arquivos

- [Execução](execucao.md): o que ler, mudar e os passos.
- [Verificação](verificacao.md): baseline, comandos e evidências.
- [Critérios](criterios.md): quando a tarefa está concluída.
- [Resumo executivo](executivo.md)
- [Detalhes técnicos](tecnico.md)
- [Registro](registro.md): etapas da execução e histórico de ajustes.

## Card

- **Título:** Botões de spec no painel
- **Tipo:** historia
- **Descrição:** "Nova spec", "Refinar" e "Executar onda" nas páginas de specs e da spec.
- **Critérios de aceite:**
  - [x] A lista de specs tem o botão "Nova spec", que pede o assunto e grava o pedido `criar-spec` com ele; sem assunto, o envio fica desligado (RF-05, CT-05).
  - [x] A spec em rascunho tem o botão "Refinar", que grava o pedido `refinar-spec`, e as specs em outros status não têm (RF-05, CT-05).
  - [x] A spec planejada tem o botão "Executar onda", que grava o pedido `executar-onda` com a menor onda que tem tarefa aprovada e pronta, e some quando nenhuma tarefa está pronta (RF-05, CT-06).
  - [x] Os botões novos e o campo do assunto são achados por papel e rótulo, e funcionam pelo teclado como os diálogos que já existem (CT-18, RNF-04).
- **Rótulos:** SPEC-033, padrao
- **Estimativa:** padrao
- **Bloqueada por:** TASK-08
