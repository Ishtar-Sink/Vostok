<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** `apps/cli/src/commands/pedidos.rs` ganha `Requests` e `Questions`, que conhecem a pasta configurada (TASK-01), a subpasta de cada estado e a migração. Todo código que hoje monta `work/pedidos/<ID>.md` ou lê `work/perguntas/` passa por eles: os comandos `vostok pedidos`, o painel (`read.rs` e `actions.rs`) e a trava do commit (`commit.rs` com `aval.rs`).

**Decisões (perfil julgamento: alternativas e critérios).**
- *Onde fica a verdade do estado.* Escolhido: o front-matter (`estado`, `status`) continua sendo a verdade e a pasta é a apresentação (RN-06, decisão 11). A transição grava o front-matter e o histórico e só depois faz o `rename`; uma interrupção entre os dois deixa o arquivo na pasta antiga com o estado novo, e a próxima transição dele o acha pelo `find` nas quatro pastas. Alternativa descartada: renomear primeiro, porque a pasta diria um estado que o arquivo ainda não tem. Critério: nenhum leitor do front-matter vê um estado que não foi gravado.
- *Quando migrar.* Escolhido: no `open`, em todo comando, só para arquivos soltos na raiz e para subpastas cujo nome não está na configuração (formato antigo, FA-04, e nomes trocados, CB-07). As quatro pastas configuradas são só listadas, então `listar pendente` lê só `pendente/` (RNF-01). Alternativas descartadas: um comando de migração explícito (o usuário teria de lembrar de rodar, e a spec pede a migração na primeira execução) e conferir o estado de todos os arquivos em todo comando (quebraria o RNF-01). Critério: custo proporcional ao que está fora do lugar, e nenhum trabalho com as pastas já organizadas.
- *Estado desconhecido.* O arquivo fica onde está e nada é apagado: a migração nunca perde um pedido.
- *Formato do histórico.* Seção `## Histórico` entre `## Pedido` e `## Resultado`, numa tabela `| Data | De | Para |`. É o contrato que a TASK-06 lê para a `GET /api/atividades` (RF-12). Alternativa descartada: lista solta, mais difícil de ler com a mesma função das tabelas de ajustes.
- *Próximo ID.* Calculado pelos nomes de arquivo das quatro pastas e da raiz, sem abrir arquivos, para dois pedidos não terem o mesmo número depois que o primeiro sai de `pendente/`.
- *Painel.* Constrói um `Harness { root }` a partir de `painel.root` em vez de guardar a configuração no `Painel`, para pegar uma troca de configuração sem reiniciar o painel. Com configuração inválida, a leitura mostra listas vazias e a gravação responde 500 com a mensagem do `folders()`.

**Arquivos fora dos componentes da tarefa.** `aval.rs` (componente da TASK-03, mesma onda 2): só a assinatura de `request_state` e `request_text_hash`, que passam a receber o caminho. `actions.rs` (componente da TASK-08, onda 3, que depende desta): a gravação do pedido em `pendente/`, sem ela o painel gravaria no formato antigo. `commit.rs` (de nenhuma tarefa): a conferência do pedido na trava de commit. Os testes `tests/pedidos.rs` e `tests/painel.rs`.

**Arquivos em comum com a onda 2.** `apps/cli/src/aval.rs`, `apps/cli/src/painel/actions.rs` e `apps/cli/src/painel/read.rs` também mudam na TASK-03 (avais por spec); `apps/cli/tests/painel.rs` também. O Coordenador executa a TASK-02 e a TASK-03 em sequência.

**Riscos.** Uma pasta configurada fora de `work/` não é vista pelo aviso ao vivo do painel (`live.rs` vigia só `work/`). As skills que citam `work/pedidos/` são atualizadas na TASK-11.

**Referências da spec.** RF-08, RF-12 (formato do histórico), RF-16, RN-06, RNF-01, FA-04, CB-07, CT-09, CT-17, CT-19 (parte dos pedidos), decisões 5, 8 e 11.
