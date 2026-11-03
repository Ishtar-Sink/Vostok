# TASK-04: Build sem painel desatualizado

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-034
- **Status:** done
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

- **Título:** Build sem painel desatualizado
- **Tipo:** bug
- **Descrição:** O `build.rs` reconstrói o `dist` mais antigo que o código e recusa o build se não conseguir.
- **Critérios de aceite:**
  - [x] Com a feature `painel`, um `dist` mais antigo que o código do painel e sem `pnpm`, o build termina com erro e a mensagem diz para rodar `pnpm --dir apps/painel build` (RF-08, CT-10, CA-06).
  - [x] Com o `dist` em dia, o build termina sem erro e sem precisar do `pnpm` (RF-08, CT-11).
  - [x] Com o `dist` velho e o `pnpm` disponível, o build reconstrói o `dist` sozinho e termina sem erro, sem parar o ciclo de editar o painel e rodar os testes (RF-08, inferência H).
  - [x] Sem a feature `painel` nada muda, e sem o `dist` o build segue com o aviso de hoje (CB-05).
  - [x] A verificação não altera o repositório e o resto do `vostok` continua compilando e passando (RF-08).
- **Rótulos:** SPEC-034, julgamento
- **Estimativa:** julgamento
- **Bloqueada por:** -
