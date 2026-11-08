# Testes do painel local

- Data: 2026-11-02
- Commit da worktree do spike: `cb57802` (branch `spike/painel-local`, em `/home/victor/projetos/ai-hub-painel`), com as mudanças ainda não commitadas
- Variante: `tiny_http` + Preact com compat (`--features painel-tiny`), a mesma do [protótipo](prototipo.md)
- Projeto de teste: cópia descartável de `vostok/`, `docs/estudos/`, `CLAUDE.md`, `AGENTS.md` e `.claude/skills` em `/home/victor/projetos/ai-hub-painel-teste`, com repositório git próprio. A TASK-05 foi devolvida a `pending` com `vostok task adjust record-manual`.
- Painel da cópia: `vostok painel --porta 4710`. As ações foram feitas pela interface, com cliques automáticos pelo Chrome headless (protocolo de depuração), como faria o usuário.

Este documento registra a TASK-04 da SPEC-027. O código do spike não entra no repositório principal.

## O que foi implementado no spike

A aprovação segue a recomendação de [levantamento.md](levantamento.md): a alternativa (a), com o aval da (c).

- **Aprovação no mesmo processo.**
  - `task::approve_from_panel` e `spec::approve_from_panel` usam as mesmas conferências e gravações de `vostok task approve approve` e `vostok spec approve approve`.
  - A origem é um tipo (`Origin::Chat` ou `Origin::Panel`), nunca um argumento de linha de comando. A CLI grava `approved_by: chat` e o painel grava `approved_by: painel`.
- **Aval.**
  - Cada aprovação pelo painel acrescenta uma linha a `vostok/work/avais.log`, que só recebe acréscimos: data, ID, hash aprovado e origem.
  - A conferência desse aval pela CLI (a trava) fica para a spec própria.
- **Token de sessão.**
  - O servidor gera 16 bytes aleatórios ao abrir e os coloca na página, numa `<meta name="painel-token">`.
  - Toda ação (POST e PUT) exige o mesmo valor no cabeçalho `X-Painel-Token`.
  - Uma chamada sem token responde 403.
- **Edição com controle otimista.**
  - `GET /api/fonte?caminho=` devolve o conteúdo e o SHA-256.
  - `PUT /api/arquivo?caminho=&hash=` grava um `.md` de `vostok/work/` só se o hash enviado for o do disco. Senão, responde 409 com o conteúdo e o hash atuais, sem tocar no arquivo.
  - `registro.md` e `README.md` das tarefas são recusados (403), porque só o `vostok` os grava.
  - Depois de gravar, o painel chama o `adjust record-manual` correspondente (tarefa, plano ou spec refinada) e devolve os problemas que o `adjust validate` acusaria.
- **Pedidos ao agente.**
  - `POST /api/pedidos?acao=&spec=&tarefa=`, com o texto do usuário no corpo, grava `vostok/work/pedidos/PED-NNN.md`.
  - Aprovar spec e tarefa não é pedido: o painel aprova direto.
- **Interface.**
  - Os botões de ação da spec, da tarefa e das pendências chamam essas rotas.
  - Cada parte da tarefa (menos o registro) e o arquivo da spec têm um botão "Editar".
  - O diálogo de edição mostra os dois textos lado a lado num conflito.
- **Testes unitários.** São 10 no `painel.rs` (`cargo test --features painel-tiny --lib painel`), todos passando:
  - gravação com hash certo e com hash velho;
  - caminhos fora de `vostok/work/` e arquivos só do `vostok`;
  - arquivo inexistente;
  - classificação do `record-manual`;
  - formato e numeração do pedido, ação desconhecida e token na página.

## Aprovação pelo painel (PQ-06)

Comandos, na cópia:
- clique em "Aprovar tarefa" na página da TASK-05 e depois em "Aprovar tarefa" no diálogo;
- `vostok task status SPEC-027`, `vostok task adjust validate SPEC-027 TASK-05` (spike e 0.2.10 instalado) e `git diff` do `estado.yml`.

Resultado:

```text
sem token: 403
Aprovado pelo painel: SPEC-027/TASK-05. O aval ficou registrado em vostok/work/avais.log.
| TASK-05 | Consolidar o estudo do painel local | arquiteto | padrao | sonnet | 4 | approved |
Tarefa TASK-05 válida: .../vostok/work/tasks/SPEC-027/TASK-05      (spike)
Tarefa TASK-05 válida: .../vostok/work/tasks/SPEC-027/TASK-05      (vostok 0.2.10)
```

```diff
-status: pending
+status: approved
-approved_at: null
-approved_hash: null
+approved_at: 2026-11-02T12:11:42Z
+approved_hash: 8c761682bd9007c6d8e756e5a517c51a79ed15e8b94602137c649b6099e619f5
-state_hash: 5048bbe7...
+state_hash: ...
+approved_by: painel
```

`vostok/work/avais.log`:

```text
2026-11-02T12:11:42Z	SPEC-027/TASK-05	8c761682bd9007c6d8e756e5a517c51a79ed15e8b94602137c649b6099e619f5	painel
```

Antes da aprovação, `vostok task approve validate SPEC-027 TASK-05` respondeu `Tarefas de SPEC-027 que podem ser aprovadas: - TASK-05`. Depois, o mesmo comando recusa por `status 'approved'`, como deve: a tarefa não está mais pendente.

Conclusão:
- A aprovação pelo painel passa nos comandos de validação das duas versões do `vostok`.
- O único campo novo é `approved_by`, e a 0.2.10 o ignora.
- Como a gravação passa pelas funções da biblioteca, o `state_hash` é atualizado e não aparece edição manual sem registro.

Impacto na suíte da CLI:
- Sem a feature, passam 517 testes, e só 2 testes golden falham.
- As 15 diferenças nesses golden são todas de aprovação (`approve-spec` e `approve-task`) e vêm da linha `approved_by: chat`, que a CLI passa a gravar.
- Adotar a origem exige atualizar essas fixtures.

Com `--features painel-tiny`, os testes de integração não compilam:
- a dependência `ascii` do `tiny_http` torna ambígua a comparação de `u8` em `tests/common/golden.rs:507`;
- na `main`, o mesmo arquivo compila.
- A correção é uma anotação de tipo nessa linha, a fazer com a implementação.

Achado: quando um ajuste derruba a aprovação, o `drop_approval` também precisa limpar o `approved_by`. Sem isso, a tarefa volta a `pending` com `approved_by: painel`. Corrigido no spike.

## Edição pelo painel (PQ-01, PQ-07)

Comandos, na cópia:
- clique em "Editar" em "Critérios de conclusão" da TASK-05;
- acréscimo de um critério com o "Como comprovar" e "Gravar";
- depois, `vostok task adjust record-manual SPEC-027 TASK-05` e `vostok task adjust validate SPEC-027 TASK-05`.

Resultado:

```text
Gravado. Ajuste manual registrado.
| 2 | 2026-11-02 | manual | Edição manual: criterios.md |
status: pending   (a aprovação caiu, como no ajuste pela CLI)
vostok task adjust record-manual: Nenhuma edição manual.
Tarefa TASK-05 válida: ...      (spike e vostok 0.2.10)
Tarefas de SPEC-027 que podem ser aprovadas: - TASK-05
```

Caso de texto fora do formato: num ensaio, o critério foi gravado por engano em `verificacao.md`.
- O `vostok` registrou o ajuste, e o `adjust validate` recusou a tarefa (`'Evidências' com item fora do formato`), nas duas versões.
- O painel respondia só "Gravado", e os problemas ficavam no log do servidor.
- Corrigido: a gravação devolve `problemas`, e o diálogo os mostra na hora.

Conclusão:
- A edição pelo painel é registrada como ajuste manual, e o `vostok` continua sendo quem valida.
- Uma edição ruim não passa: a tarefa volta a `pending` e não pode ser aprovada nem executada até ser corrigida.

## Edição simultânea (PQ-07)

**O agente edita enquanto o painel edita.** O diálogo de edição dos critérios estava aberto, com um texto novo digitado. Por fora, outro processo acrescentou uma linha ao arquivo (`printf ... >> criterios.md`), e então veio o clique em "Gravar".

```text
O arquivo mudou no disco enquanto você editava. Nada foi gravado. Compare e escolha: o seu texto continua aqui.
Seu texto | Versão no disco
[Fechar] [Usar a versão do disco] [Gravar a minha por cima]
```

Pela API, com hash velho:

```text
PUT /api/arquivo?caminho=vostok/work/tasks/SPEC-027/TASK-05/criterios.md&hash=<hash antigo>
{"hash":"...","conteudo":"...\n- [ ] Texto do agente (PQ-07).\n"}
HTTP 409
```

O arquivo no disco ficou com a linha do agente, e o texto do painel continuou no diálogo. Gravar por cima só acontece com um segundo clique explícito, que usa o hash novo.

**O agente edita depois do painel.** A página da tarefa estava aberta quando um critério novo foi acrescentado por fora. A tela mostrou o critério novo antes de 250 ms, sem recarregar, pelo aviso `mudou` do WebSocket `/api/ao-vivo`. A spec previa `GET /api/eventos` (Server-Sent Events), que foi trocado na TASK-03, veja [prototipo.md](prototipo.md).

Latência do aviso em 5 rodadas: 329, 396, 318, 332 e 317 ms, com média de 338 ms.

Conclusão:
- Com o controle otimista por hash, nenhum lado perde dados, e não é preciso trava de arquivo.
- A trava ficaria presa se o painel fechasse no meio, e o agente não a respeitaria de qualquer forma.
- Controle recomendado: hash na leitura, 409 com o conteúdo atual na gravação e escolha explícita do usuário.

## Pedido ao agente e loop (PQ-08)

Formato do pedido gravado pelo painel, um arquivo por pedido em `vostok/work/pedidos/`:

```markdown
---
id: PED-002
acao: ajustar-tarefa
spec: SPEC-027
tarefa: TASK-05
estado: pendente            # pendente | em_andamento | feito | recusado
origem: painel
criado_em: 2026-11-02T12:23:08Z
atualizado_em: 2026-11-02T12:23:08Z
---

## Pedido

Acrescentar um critério de conclusão: o estudo registra o tempo que o chat em /loop levou para processar um pedido do painel (PQ-08), com a forma de comprovar.

## Resultado

```

Ações aceitas:
- `aprovar-plano`, `planejar-spec`;
- `ajustar-spec`, `ajustar-plano`, `ajustar-tarefa`;
- `executar-tarefa`, `finalizar-tarefa`, `desbloquear-tarefa`;
- `finalizar-spec`, `responder-pergunta`.

Qualquer outra é recusada com 400. Isso inclui `aprovar-tarefa`, porque aprovação não é pedido. A resposta a uma pergunta do agente vai no texto do pedido, começando por `Pergunta P-NNN`.

O chat foi aberto pelo usuário no Claude Code, na pasta da cópia, com:

```text
/loop 1m Confira vostok/work/pedidos/. Para cada arquivo PED-*.md com "estado: pendente", em ordem: mude o estado para em_andamento e atualize atualizado_em; execute a ação pela skill do harness (ajustar-tarefa → adjust-task, ...); o texto de "## Pedido" é o pedido do usuário. Escreva em "## Resultado" o que foi feito e mude o estado para feito, ou para recusado com o motivo. Sem pedido pendente, não faça nada e não responda.
```

Os pedidos foram gravados pelo botão "Pedir ajuste" da TASK-05.

| Pedido | Gravado | Pego pelo loop | Final | Tempo total | Estado |
|---|---|---|---|---|---|
| PED-001 | 12:21:10 | 12:22:13 | 12:22:40 | 1 min 30 s | `recusado` |
| PED-002 | 12:23:08 | 12:24:11 | 12:24:26 | 1 min 18 s | `feito` |

**PED-001.** O loop recusou com motivo, sem decidir pelo usuário:
- dois critérios deixados pelo teste de edição simultânea estavam sem "Como comprovar";
- o `adjust validate` não passava;
- o loop explicou em `## Resultado` o que corrigir.

**PED-002.** Feito depois de remover esses critérios pelo editor do painel:
- o critério novo entrou com a forma de comprovar;
- o registro ganhou o ajuste 5 (origem `agente`);
- o `adjust validate` passa;
- a tarefa seguiu `pending`, como manda a `adjust-task`;
- o loop deixou uma linha em branco sobrando antes do critério novo e não a corrigiu de propósito, porque qualquer edição à mão em `criterios.md` vira ajuste manual.

O loop do Claude Code foi criado como um job agendado (`*/1 * * * *`, até 7 dias, ligado à sessão). A saída do chat está na evidência `pedido-loop.txt`.

Conclusão:
- Um chat em `/loop 1m` processa pedidos do painel sem nenhuma mudança nas skills.
- A espera é dominada pelo intervalo do loop: cerca de 1 min até pegar o pedido e cerca de 15 a 30 s para processar.
- O estado no arquivo e o `## Resultado` dão ao painel o retorno, inclusive a recusa.
- Para chegar perto do tempo real, o loop poderia rodar a cada 10 a 20 s quando houver pedido pendente, ou ser acordado pelo próprio arquivo, num mecanismo do harness.

Riscos registrados:
- Um pedido é texto do usuário que o agente executa. Ele vale como pedido do usuário porque nasce do clique, mas não carrega aval: com a trava de aval, ações como finalizar tarefa e aprovar plano devem exigir o aval do painel além do pedido.
- O loop precisa de permissão para os comandos `vostok`; um pedido de permissão aumenta o tempo.

## Codex (sem teste)

O Codex não tem um equivalente ao `/loop` dentro de uma sessão interativa. O que muda:

- **Vigiar a pasta:** fica fora do agente, num processo que dispara uma execução não interativa (`codex exec "<instrução do loop>"`) quando aparece um `PED-*.md` pendente.
  - O processo pode ser um laço com `inotifywait`/`fswatch` ou um temporizador do sistema.
  - O próprio `vostok` poderia oferecer esse laço (`vostok painel --agente codex`), com o mesmo prompt.
- **Contexto:** cada `codex exec` começa do zero. Perde-se o contexto entre pedidos, e ganha-se isolamento.
- **Permissões:** o `codex exec` precisa rodar com aprovação automática dos comandos do `vostok` no sandbox de escrita na pasta. Sem isso, o pedido para e espera.
- **Sem equivalente:**
  - acompanhar a execução no chat enquanto ela acontece;
  - responder uma pergunta no meio do pedido, que no Codex vira um novo pedido de `responder-pergunta`.

O formato do pedido e os estados não mudam: o painel não sabe qual agente processa a pasta.
