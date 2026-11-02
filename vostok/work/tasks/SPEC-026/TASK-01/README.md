# TASK-01: Comando brand e derivação das cores

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-026
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

- **Título:** Comando brand e derivação das cores
- **Tipo:** historia
- **Descrição:** O `vostok documents brand` grava a identidade, deriva os tokens dos dois temas com contraste AA, volta ao padrão com `--reset` e mostra o antes e depois.
- **Critérios de aceite:**
  - [x] O comando grava a cor principal, a secundária, o nome e o ícone no `documents.yml` (RF-04, RF-05, RF-06, RF-07).
  - [x] As cores de estado não mudam (RN-01).
  - [x] Os pares de texto passam 4,5:1 nos dois temas e o ajuste é avisado (RF-09, CT-08).
  - [x] `--reset` volta à identidade padrão (RF-10, CT-09).
  - [x] A saída mostra o antes e depois de cada token (RF-11, CT-10).
  - [x] Entradas inválidas são recusadas sem gravar nada (CT-14, CB-01, CB-06).
  - [x] A suíte de `init_update` e o clippy passam.
- **Rótulos:** SPEC-026, padrao
- **Estimativa:** padrao
- **Bloqueada por:** -
