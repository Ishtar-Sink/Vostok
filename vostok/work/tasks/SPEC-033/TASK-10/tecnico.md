<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.**
- **Prints:** a página da tarefa descobre os prints pelos itens `(print)` da `verificacao.md` e sabe se cada um está conferido pela linha do `registro.md` com o SHA-256 da imagem, a mesma regra do `vostok task evidence`. Aceitar e rejeitar chamam `POST /api/conferencias` da TASK-08; rejeitar bloqueia a tarefa (FA-02) e por isso passou por um `ConfirmDialog` (`alertdialog`).
- **Commits:** a CLI ganhou `pub fn pending_commits` em `commit.rs`, que devolve as linhas do `commit check` sem recusar nem gravar (vazio sem git ou sem a referência), e a rota `GET /api/commits` em `read.rs` (`{"linhas":[...]}`; erro vira vazio). As pendências mostram o bloco só quando há linhas. Liberar usou o `ActionDialog`; desfazer usou o `ConfirmDialog` antes de gravar o pedido (RF-07, decisão 12). Os pedidos de liberar e desfazer levam no corpo as linhas dos commits apontados, seguidas da observação quando houver, para o aval valer para eles.
- **Perguntas:** a resposta passou a mandar só o texto com `&pergunta=<id>`, como a TASK-08 exige. Antes ia "Pergunta X" no corpo e a TASK-08 respondia 400.
- **Limpeza:** botão no cabeçalho das Configurações, lista pela `POST /api/limpeza` e apagar só com `confirmar=sim` (RF-11, RNF-03).

**Decisões.**
- Conferência calculada no navegador pelo SHA-256, no lugar de só procurar o nome no registro: descartado, porque um print regerado depois da conferência apareceria como conferido no painel e pendente na CLI.
- Rota `GET /api/commits` nesta tarefa: a TASK-08 não prevê leitura dos commits apontados, e sem ela a tela não tinha o que mostrar. Descartado ler a saída do `vostok commit check` no painel por processo, porque a CLI já tem a função.
- Limpeza no cabeçalho das Configurações, não num grupo do menu: descartado criar um item de menu sem arquivo de configuração por trás.

**Pontos a alinhar com a TASK-08.** As rotas e os parâmetros usados seguem o detalhamento dela: `POST /api/conferencias?spec&tarefa&print&resultado=aceito|rejeitado` (200 `{"conferido","resultado"}`, 422 com a mensagem), `POST /api/limpeza` (`{"arquivos":[...]}`) e `?confirmar=sim` (`{"apagados":[...]}`), `responder-pergunta` com `pergunta=`. A TASK-08 não tem rota de leitura dos commits apontados; a `GET /api/commits` foi criada nesta tarefa. Ficou como ponto de alinhamento com o Coordenador a possibilidade de essa rota passar para a TASK-08.

**Riscos.** O print `pendencias.png` só mostra o bloco de commits se houver commits sem liberação no momento da geração; os testes cobrem o bloco. O `conferencia-prints.png` depende de `configuracoes.png` e `pendencias.png` gerados antes, na ordem das evidências; como o `print.sh` não rola até `#evidencias`, a janela desse print foi ajustada (ver Feito).

**Arquivos alterados.**
- `apps/painel/src/features/task/task-page.tsx` (alterado): +104 / -6
- `apps/painel/src/features/task/task-page.module.css` (alterado): +13
- `apps/painel/src/features/task/task-page.test.tsx` (alterado): +71 / -1
- `apps/painel/src/features/inbox/inbox-page.tsx` (alterado): +50 / -1
- `apps/painel/src/features/inbox/inbox-page.module.css` (alterado): +4
- `apps/painel/src/features/inbox/inbox-page.test.tsx` (alterado): +65
- `apps/painel/src/features/settings/settings-page.tsx` (alterado): +64 / -1
- `apps/painel/src/features/settings/settings-page.test.tsx` (alterado): +49 / -1
- `apps/painel/src/shared/attention.ts` (alterado): +19 / -4
- `apps/painel/src/shared/action-bar.tsx` (alterado): +16 / -6
- `apps/cli/src/commands/commit.rs` (alterado): +14
- `apps/cli/src/painel/read.rs` (alterado): +16

**Arquivos fora dos componentes do plano.** `apps/cli/src/commands/commit.rs` e `apps/cli/src/painel/read.rs` (rota de commits), `apps/painel/src/shared/attention.ts`, `apps/painel/src/shared/action-bar.tsx`, os CSS de tarefa e pendências e os testes.

**Arquivos em comum na onda 4.** `attention.ts` e `action-bar.tsx` são compartilhados com a TASK-09, que vem antes no plano.

**Referências da spec.** RF-06, RF-07, RF-11, RN-01, RN-02, RNF-03, RNF-04, FA-02, CB-05, CB-06, CT-07, CT-08, CT-12, CT-18; decisões 9 e 12.

**Feito.**
- Os quatro blocos do plano foram implementados: conferência dos prints na tarefa, commits apontados nas pendências (liberar e desfazer), resposta às perguntas com `pergunta=` e limpeza de pedidos arquivados nas configurações.
- Testes: 6 de conferência na tarefa, 4 de commits e 1 de resposta nas pendências, 4 de limpeza nas configurações; 120 testes do painel passando e `tsc` sem erros.
- Verificação, regressão (3 comandos iguais ao baseline) e evidências passaram.
- Desvio: o teste do print já conferido calcula o hash com `crypto.subtle` em vez de `node:crypto`, porque o painel não tem os tipos do Node e o `tsc` recusava.
- Ajuste 1, aprovado pelo usuário: `conferencia-prints.png` passou a usar a janela 1280,6500, porque o `print.sh` tira o print do topo da página e não rola até `#evidencias` (a seção começa perto de 3700 px). Com isso o print mostra `configuracoes.png` e `pendencias.png` com "a conferir", "Aceitar" e "Rejeitar". Os prints foram conferidos e aceitos pelo usuário.

**Pontos de atenção.**
- Hash gravado: o `vostok task evidence --reviewed` gera os prints de novo antes de gravar a conferência, então o hash gravado é o da imagem regerada (visualmente igual), não o da que o usuário viu. Se uma nova geração mudar os bytes, o painel volta a mostrar o print como "a conferir".
- Bloco de commits: aparece só com commits sem liberação no momento; o print `pendencias.png` depende desse estado.
