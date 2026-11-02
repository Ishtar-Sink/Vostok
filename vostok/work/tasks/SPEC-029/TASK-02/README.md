# TASK-02: Criar o comando e o servidor do painel

Arquivo gerado pelo vostok a cada mudança de estado; não edite.

- **Spec:** SPEC-029
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

- **Título:** Criar o comando e o servidor do painel
- **Tipo:** tarefa
- **Descrição:** `vostok painel` acha o harness, sobe o `tiny_http` em `127.0.0.1` com o token de sessão, serve o bundle e despacha para os esqueletos de leitura, ao vivo e ações; o `build.rs` gera o bundle quando há Node.
- **Critérios de aceite:**
  - [x] `vostok painel` acha o harness, escuta só em `127.0.0.1` numa porta livre e para com mensagem fora de um projeto (RF-01, CT-02).
  - [x] Toda ação sem o token de sessão é recusada com 403, e a página sai com o token (RF-08, CT-05).
  - [x] O `vostok` compila sem Node e sem a feature, e o `build.rs` só gera o bundle quando pode (RF-12, CT-14).
- **Rótulos:** SPEC-029, julgamento
- **Estimativa:** julgamento
- **Bloqueada por:** -
