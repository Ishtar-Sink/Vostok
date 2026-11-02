### Baseline

- `ls docs/estudos | sort`

### Verificação

```bash
cargo build --release --manifest-path /home/victor/projetos/ai-hub-painel/apps/cli/Cargo.toml --features painel-tiny
grep -n '^## ' docs/estudos/painel-local/prototipo.md
```

Resultado esperado: build sem erro (troque a feature pela variante recomendada na TASK-02); as seções do documento do protótipo. As telas são conferidas pelos prints.

### Evidências

- `visao-geral-ai-hub.png` (print): `B=/home/victor/projetos/ai-hub-painel/apps/cli/target/release/vostok; L="$(mktemp)"; (cd /home/victor/projetos/ai-hub && "$B" painel --sem-navegador > "$L" 2>&1 & echo $! > "$L.pid"); for i in $(seq 50); do grep -q "http://127.0.0.1" "$L" && break; sleep 0.2; done; U="$(grep -o "http://127.0.0.1:[0-9]*" "$L" | head -1)"; CHROME="$(ls -d ~/.cache/ms-playwright/chromium-*/chrome-linux64/chrome | sort | tail -1)"; "$CHROME" --headless=new --no-sandbox --disable-gpu --hide-scrollbars --virtual-time-budget=3000 --window-size=1280,900 --screenshot="$VOSTOK_EVIDENCE/visao-geral-ai-hub.png" "$U/"; kill "$(cat "$L.pid")"`
  - Prova: a visão geral do ai-hub lida dos arquivos do harness, com os componentes do Atria e a marca da pasta (PQ-01, PQ-10).
  - Trecho: SPEC-027 com o status `planned` e as tarefas da spec por status.
  - Falha: página em branco, erro ou dados que não batem com `vostok task status SPEC-027`.
- `spec-ai-hub.png` (print): `B=/home/victor/projetos/ai-hub-painel/apps/cli/target/release/vostok; L="$(mktemp)"; (cd /home/victor/projetos/ai-hub && "$B" painel --sem-navegador > "$L" 2>&1 & echo $! > "$L.pid"); for i in $(seq 50); do grep -q "http://127.0.0.1" "$L" && break; sleep 0.2; done; U="$(grep -o "http://127.0.0.1:[0-9]*" "$L" | head -1)"; CHROME="$(ls -d ~/.cache/ms-playwright/chromium-*/chrome-linux64/chrome | sort | tail -1)"; "$CHROME" --headless=new --no-sandbox --disable-gpu --hide-scrollbars --virtual-time-budget=3000 --window-size=1280,1500 --screenshot="$VOSTOK_EVIDENCE/spec-ai-hub.png" "$U/specs/SPEC-027"; kill "$(cat "$L.pid")"`
  - Prova: a spec renderizada a partir do `.md`, com as tabelas (PQ-01).
  - Trecho: o título da SPEC-027 e a tabela `Perguntas da investigação`.
  - Falha: markdown cru ou tabela quebrada.
- `tarefa-ai-hub.png` (print): `B=/home/victor/projetos/ai-hub-painel/apps/cli/target/release/vostok; L="$(mktemp)"; (cd /home/victor/projetos/ai-hub && "$B" painel --sem-navegador > "$L" 2>&1 & echo $! > "$L.pid"); for i in $(seq 50); do grep -q "http://127.0.0.1" "$L" && break; sleep 0.2; done; U="$(grep -o "http://127.0.0.1:[0-9]*" "$L" | head -1)"; CHROME="$(ls -d ~/.cache/ms-playwright/chromium-*/chrome-linux64/chrome | sort | tail -1)"; "$CHROME" --headless=new --no-sandbox --disable-gpu --hide-scrollbars --virtual-time-budget=3000 --window-size=1280,1500 --screenshot="$VOSTOK_EVIDENCE/tarefa-ai-hub.png" "$U/tarefas/SPEC-027/TASK-02"; kill "$(cat "$L.pid")"`
  - Prova: uma tarefa com a execução, os critérios e as evidências (PQ-01).
  - Trecho: a lista de evidências da TASK-02 com `medicoes.txt`.
  - Falha: tarefa sem evidências ou com arquivos faltando.
- `plano-exemplo.png` (print): `T="$(mktemp -d)"; cp -r /home/victor/projetos/ai-hub/vostok "$T/"; mkdir -p "$T/vostok/work/plans"; cp /home/victor/projetos/ai-hub/vostok/work/tasks/SPEC-027/TASK-03/plano-exemplo.md "$T/vostok/work/plans/SPEC-027.md"; B=/home/victor/projetos/ai-hub-painel/apps/cli/target/release/vostok; L="$(mktemp)"; (cd "$T" && "$B" painel --sem-navegador > "$L" 2>&1 & echo $! > "$L.pid"); for i in $(seq 50); do grep -q "http://127.0.0.1" "$L" && break; sleep 0.2; done; U="$(grep -o "http://127.0.0.1:[0-9]*" "$L" | head -1)"; CHROME="$(ls -d ~/.cache/ms-playwright/chromium-*/chrome-linux64/chrome | sort | tail -1)"; "$CHROME" --headless=new --no-sandbox --disable-gpu --hide-scrollbars --virtual-time-budget=3000 --window-size=1280,1500 --screenshot="$VOSTOK_EVIDENCE/plano-exemplo.png" "$U/planos/SPEC-027"; kill "$(cat "$L.pid")"`
  - Prova: a tela de plano com a tabela de tarefas, ondas e perfis (PQ-01).
  - Trecho: as linhas TASK-01 a TASK-05 com as ondas 1 a 4.
  - Falha: plano não encontrado.
- `visao-geral-atria.png` (print): `B=/home/victor/projetos/ai-hub-painel/apps/cli/target/release/vostok; L="$(mktemp)"; (cd /home/victor/projetos/ishtar-sink/atria && "$B" painel --sem-navegador > "$L" 2>&1 & echo $! > "$L.pid"); for i in $(seq 50); do grep -q "http://127.0.0.1" "$L" && break; sleep 0.2; done; U="$(grep -o "http://127.0.0.1:[0-9]*" "$L" | head -1)"; CHROME="$(ls -d ~/.cache/ms-playwright/chromium-*/chrome-linux64/chrome | sort | tail -1)"; "$CHROME" --headless=new --no-sandbox --disable-gpu --hide-scrollbars --virtual-time-budget=3000 --window-size=1280,900 --screenshot="$VOSTOK_EVIDENCE/visao-geral-atria.png" "$U/"; kill "$(cat "$L.pid")"`
  - Prova: o mesmo binário aberto em outra pasta mostra os dados e a marca dela (PQ-09, PQ-10).
  - Trecho: as specs do Atria e as cores da marca do Atria.
  - Falha: dados ou marca do ai-hub.
- `prototipo.diff` (diff): `docs/estudos/painel-local/prototipo.md`
  - Prova: as telas, o mapeamento da marca, a atualização ao vivo e os dois painéis registrados (PQ-01, PQ-09, PQ-10, PQ-12).
  - Trecho: a comparação de consulta periódica contra `notify` com os números e o retorno do usuário.
  - Falha: seção sem números ou sem o retorno do usuário.
  - Espera: `notify`
