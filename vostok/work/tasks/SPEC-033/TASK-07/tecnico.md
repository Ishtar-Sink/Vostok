<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** `pedidos.rs` ganha a função pública `start_dispatch(harness)`. O `vostok painel` (`commands/painel.rs`) a chama antes de `server::serve`, que bloqueia a thread principal. A função lê `request_dispatch`, criado pela TASK-01. Com `codex` ou `claude`, ela abre uma única thread com o mesmo laço do `vostok pedidos vigiar`: procura o próximo pedido em `pendente/`, chama o agente e espera ele terminar. Se não houver pedido, dorme 2 s. O processamento de um pedido (`process_with_codex`) é generalizado para os dois agentes, e o `vigiar` passa a usar o mesmo laço.

**Decisões e critérios.**
1. **Conferir `pendente/` a cada 2 s numa thread do painel.** Assim o disparo pega todo pedido gravado, pelo painel ou por qualquer outro caminho, e a listagem lê só essa pasta (RNF-01).
   - Alternativa descartada: avisar o disparador da ação `POST /api/pedidos` por um canal. Seria imediato, mas exigiria mudar `painel/actions.rs`, que a TASK-08 altera na mesma onda, e perderia os pedidos gravados fora do painel.
   - Alternativa descartada: o painel abrir um processo `vostok pedidos vigiar` filho. Isso exigiria cuidar do ciclo de vida do filho, e o `vigiar` só aceita o `codex`.
   - Critério: o menor acoplamento com o código das outras tarefas, com no máximo 2 s de espera.
2. **Uma thread, com espera do processo (`status()`).** Isso garante a RN-04 sem trava nem arquivo de lock. Um `vigiar` ou um `/process-requests` rodando junto não processam o mesmo pedido, porque o `vostok pedidos iniciar` move o arquivo por `rename` (TASK-02) e só um deles consegue.
3. **Comandos dos agentes:**
   - `codex`: `codex exec --full-auto`, como hoje;
   - `claude`: `claude -p ... --permission-mode acceptEdits --allowedTools Bash`, porque a `process-requests` precisa editar arquivos e rodar o `vostok` sem perguntar.

   Alternativa descartada: `--dangerously-skip-permissions`, que também ignoraria as regras de negação do projeto. Critério: o mínimo que deixa a skill terminar sozinha, com a trava de aval valendo (RN-02).
4. **Agente fora do PATH (CB-04):** um aviso no terminal do painel, sem disparador e sem tocar nos pedidos, que ficam pendentes para um loop. Se o agente sumir com o painel aberto, a falha ao iniciar o processo para o disparador com aviso, também sem recusar o pedido. Já um agente que termina e deixa o pedido pendente continua recusado, como no `vigiar`, para não ser disparado de novo para sempre.
5. **Valor inválido em `request_dispatch`** impede o painel de subir, com os valores aceitos na mensagem. Alternativa descartada: ignorar em silêncio, o que esconderia um erro de digitação, como `Codex`.
6. **O `vigiar` continua só com `--agente codex`.** Aceitar o `claude` nele mudaria o contrato e o teste `vigiar_rejects_an_unknown_agent`, fora da RF-13.

**Riscos.**
- O agente roda com permissão de escrita sem o usuário acompanhar. A mitigação da spec vale: desligado por padrão, um por vez e com a trava de aval.
- Fechar o painel durante uma execução não encerra o agente filho, que termina o pedido sozinho.
- O teste nunca chama um agente real: usa `codex` e `claude` falsos num `bin/` temporário no PATH.

**Arquivos fora dos componentes do plano.**
- `apps/cli/src/commands/painel.rs`, onde o `vostok painel` inicia o disparo. O `server.rs` não muda.
- `apps/cli/tests/disparo.rs`, novo, com `#![cfg(feature = "painel")]`.

**Arquivos em comum na onda 3.** `apps/cli/src/commands/pedidos.rs` é alterado também pela TASK-05 (`prune`). O Coordenador executa as duas em sequência.

**Referências da spec.** RF-13, RN-02, RN-04, RNF-01, CB-04, CT-14; decisão 1; premissa do `claude -p`; risco do disparo sob demanda.
