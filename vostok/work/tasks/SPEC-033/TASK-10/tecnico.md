<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.**
- **Prints:** a página da tarefa descobre os prints pelos itens `(print)` da `verificacao.md` e sabe se cada um está conferido pela linha do `registro.md` com o SHA-256 da imagem, a mesma regra do `vostok task evidence`. Aceitar e rejeitar chamam `POST /api/conferencias` da TASK-08; rejeitar bloqueia a tarefa (FA-02) e por isso passa por um `ConfirmDialog` (`alertdialog`).
- **Commits:** a CLI ganha `pub fn pending_commits` em `commit.rs` e a rota `GET /api/commits` em `read.rs`; as pendências mostram o bloco só quando há linhas. Liberar usa o `ActionDialog`; desfazer usa o `ConfirmDialog` antes de gravar o pedido (RF-07, decisão 12).
- **Perguntas:** a resposta passa a mandar `pergunta=<id>` e só o texto, como a TASK-08 exige.
- **Limpeza:** botão no cabeçalho das Configurações, lista de `POST /api/limpeza` e apagar só com `confirmar=sim` (RF-11, RNF-03).

**Decisões.**
- Conferência calculada no navegador pelo SHA-256, no lugar de só procurar o nome no registro: descartado, porque um print regerado depois da conferência apareceria como conferido no painel e pendente na CLI.
- Rota `GET /api/commits` nesta tarefa: a TASK-08 não prevê leitura dos commits apontados, e sem ela a tela não tem o que mostrar. Descartado ler a saída do `vostok commit check` no painel por processo, porque a CLI já tem a função.
- Limpeza no cabeçalho das Configurações, não num grupo do menu: descartado criar um item de menu sem arquivo de configuração por trás.

**Pontos a alinhar com a TASK-08.** As rotas e os parâmetros usados vêm do detalhamento dela: `POST /api/conferencias?spec&tarefa&print&resultado=aceito|rejeitado` (200 `{"conferido","resultado"}`, 422 com a mensagem), `POST /api/limpeza` (`{"arquivos":[...]}`) e `?confirmar=sim` (`{"apagados":[...]}`), `responder-pergunta` com `pergunta=`. A TASK-08 não tem rota de leitura dos commits apontados; esta tarefa cria `GET /api/commits` (`{"linhas":[...]}`). Se o Coordenador preferir, ela pode passar para a TASK-08.

**Riscos.** O print `pendencias.png` só mostra o bloco de commits se houver commits sem liberação na hora; os testes cobrem o bloco. O `conferencia-prints.png` depende de `configuracoes.png` e `pendencias.png` serem gerados antes, na ordem das evidências.

**Arquivos fora dos componentes do plano.** `apps/cli/src/commands/commit.rs` e `apps/cli/src/painel/read.rs` (rota de commits), `apps/painel/src/shared/attention.ts`, `apps/painel/src/shared/action-bar.tsx`, os CSS de tarefa e pendências e os testes.

**Arquivos em comum na onda 4.** `attention.ts` e `action-bar.tsx` com a TASK-09: execute a TASK-09 antes.

**Referências da spec.** RF-06, RF-07, RF-11, RN-01, RN-02, RNF-03, RNF-04, FA-02, CB-05, CB-06, CT-07, CT-08, CT-12, CT-18; decisões 9 e 12.
