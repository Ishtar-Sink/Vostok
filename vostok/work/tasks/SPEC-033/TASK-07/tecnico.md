<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** `pedidos.rs` ganhou a função pública `start_dispatch(harness)`. O `vostok painel` (`commands/painel.rs`) a chama depois de `Painel::new` e antes de `server::serve`, que bloqueia a thread principal. A função lê `request_dispatch`, criado pela TASK-01. Com `codex` ou `claude`, ela abre uma única thread com o laço `run_loop`, o mesmo do `vostok pedidos vigiar`: procura o próximo pedido em `pendente/`, chama o agente e espera ele terminar. Se não houver pedido, dorme 2 s. O processamento de um pedido passou de `process_with_codex` para `process_with(agent, ...)`, que monta o comando por `agent_command`, e o `vigiar` passou a usar o mesmo laço.

**Decisões e critérios.**
1. **Conferir `pendente/` a cada 2 s numa thread do painel.** Assim o disparo pega todo pedido gravado, pelo painel ou por qualquer outro caminho, e a listagem lê só essa pasta (RNF-01).
   - Alternativa descartada: avisar o disparador da ação `POST /api/pedidos` por um canal. Seria imediato, mas exigiria mudar `painel/actions.rs`, que a TASK-08 altera na mesma onda, e perderia os pedidos gravados fora do painel.
   - Alternativa descartada: o painel abrir um processo `vostok pedidos vigiar` filho. Isso exigiria cuidar do ciclo de vida do filho, e o `vigiar` só aceita o `codex`.
   - Critério: o menor acoplamento com o código das outras tarefas, com no máximo 2 s de espera.
2. **Uma thread, com espera do processo (`status()`).** Isso garante a RN-04 sem trava nem arquivo de lock. Um `vigiar` ou um `/process-requests` rodando junto não processam o mesmo pedido, porque o `vostok pedidos iniciar` move o arquivo por `rename` (TASK-02) e só um deles consegue.
3. **Comandos dos agentes** (`agent_command`):
   - `codex`: `codex exec --full-auto <instrução>`, como antes;
   - `claude`: `claude -p <instrução> --permission-mode acceptEdits --allowedTools Bash`, porque a `process-requests` precisa editar arquivos e rodar o `vostok` sem perguntar.
   - A instrução cita só o ID: `Use a skill process-requests para processar o pedido <ID> e só ele.`
   - `LEFT_PENDING` virou `left_pending(agent)`. Um pedido que o agente deixa pendente continua recusado, como no `vigiar`, para não ser disparado de novo para sempre.
   - Alternativa descartada: `--dangerously-skip-permissions`, que também ignoraria as regras de negação do projeto. Critério: o mínimo que deixa a skill terminar sozinha, com a trava de aval valendo (RN-02).
4. **Agente fora do PATH (CB-04):** `start_dispatch` imprime um aviso no stderr e não toca nos pedidos, que ficam pendentes para um loop. Se o agente sumir com o painel aberto, a falha ao iniciar o processo aparece como `Disparo sob demanda parado: <erro>`, também sem recusar o pedido.
5. **Valor inválido em `request_dispatch`** impede o painel de subir. A mensagem é a do leitor da TASK-01, que aceita só `""`, `codex` ou `claude` e cita os dois agentes. Alternativa descartada: ignorar em silêncio, o que esconderia um erro de digitação, como `Codex`.
6. **O `vigiar` continua só com `--agente codex`**, com as mesmas mensagens. Aceitar o `claude` nele mudaria o contrato e o teste `vigiar_rejects_an_unknown_agent`, fora da RF-13.

**Riscos.**
- O agente roda com permissão de escrita sem o usuário acompanhar. A mitigação da spec vale: desligado por padrão, um por vez e com a trava de aval.
- Fechar o painel durante uma execução não encerra o agente filho, que termina o pedido sozinho.
- Os testes nunca chamam um agente real: usam `codex` e `claude` falsos num `bin/` temporário no PATH.

**Feito.**
- `apps/cli/src/commands/pedidos.rs` (232 inserções e 21 remoções): `process_with(agent, ...)` e `agent_command`, `left_pending(agent)` no lugar de `LEFT_PENDING`, `run_loop` usado também pelo `vigiar`, `const DISPATCH_INTERVAL` de 2 s e `pub fn start_dispatch`.
- `apps/cli/src/commands/painel.rs` (1 inserção): chamada de `start_dispatch` depois de `Painel::new` e antes de `server::serve`.
- `apps/cli/tests/disparo.rs` (novo, com `#![cfg(feature = "painel")]`): 5 testes com agentes falsos no PATH. Codex chamado uma vez por pedido, em ordem e sem sobreposição; claude com `-p`; disparo desligado não chama o agente; agente fora do PATH avisa e não recusa o pedido; valor inválido recusa o painel.
- Os 4 testes `vigiar_*` continuam passando.
- Verificação, regressão (3 comandos iguais ao baseline, códigos 0, 0, 0) e evidências (2, em `TASK-07/evidencias/`) passaram na primeira tentativa.
- **Desvio.** O passo 2 pedia `const AGENTS` e o passo 3 pedia uma recusa própria do valor inválido, com mensagem própria. Como o leitor `request_dispatch()` da TASK-01 já recusa valor fora de `""`, `codex` e `claude`, a checagem dupla em `start_dispatch` seria código morto e foi removida. A mensagem de erro do valor inválido é a do leitor.

**Pontos de atenção.**
- O `claude` roda com `--permission-mode acceptEdits` e `--allowedTools Bash`: edita arquivos e executa comandos Bash sem perguntar. Essa é a permissão mínima para a skill `process-requests` terminar sozinha. A trava de aval (RN-02) continua valendo.
- O disparo vem desligado por padrão e atende um pedido por vez.
- Fechar o painel não encerra o agente que já está rodando.
- Se o agente sair sem processar o pedido, o pedido fica recusado e a saída deve ser conferida em `vostok pedidos vigiar` ou no `vostok painel`.
- O `vigiar` continua aceitando só `--agente codex`.
- Um valor inválido em `request_dispatch` impede o `vostok painel` de subir.

**Arquivos fora dos componentes do plano.**
- `apps/cli/src/commands/painel.rs`, onde o `vostok painel` inicia o disparo. O `server.rs` não muda.
- `apps/cli/tests/disparo.rs`, novo, com `#![cfg(feature = "painel")]`.

**Arquivos em comum na onda 3.** `apps/cli/src/commands/pedidos.rs` também é alterado pela TASK-05 (`prune`). O Coordenador executa as duas em sequência.

**Referências da spec.** RF-13, RN-02, RN-04, RNF-01, CB-04, CT-14; decisão 1; premissa do `claude -p`; risco do disparo sob demanda.
