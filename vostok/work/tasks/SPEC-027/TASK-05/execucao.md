### Ler

- `vostok/work/specs/refined/SPEC-027-estudo-do-painel-local-do-vostok-sem-servidor.md`
- `docs/estudos/painel-local/levantamento.md`
- `docs/estudos/painel-local/medicoes.md`
- `docs/estudos/painel-local/prototipo.md`
- `docs/estudos/painel-local/testes.md`
- `docs/estudos/tarefa-como-pasta.md` (formato dos estudos do projeto)
- `docs/estudos/documentos-finais/README.md` (formato do índice dos protótipos)
- `vostok/work/tasks/SPEC-027/TASK-03/evidencias/` (prints do protótipo)

### Mudar

- `docs/estudos/painel-local.md` (criar): o estudo, com a resposta a cada pergunta, a viabilidade, a arquitetura recomendada e as specs propostas (PQ-01 a PQ-12).
- `docs/estudos/painel-local/README.md` (criar): índice dos documentos de apoio e das imagens do protótipo.
- `docs/estudos/painel-local/imagens/` (criar): cópia dos prints da TASK-03, para o estudo na `main` (CA-13).

### Passos

1. `docs/estudos/painel-local/imagens/` (criar): copie os `.png` de `vostok/work/tasks/SPEC-027/TASK-03/evidencias/`. Confira: `ls docs/estudos/painel-local/imagens`.
2. `docs/estudos/painel-local.md` (criar): título `# Estudo do painel local`, data, versão, `Subsídio da SPEC-027`, e as seções: `## Escopo e método`; `## Conclusão` (viável ou não, com os motivos, em 1 a 3 parágrafos, CA-02); `## Respostas`, com uma subseção `### PQ-NN` por pergunta, de PQ-01 a PQ-12, cada uma com a conclusão e a fonte ou a medição (link para o documento de apoio); `## Medições` (a tabela de `medicoes.md` e o limite proposto, CA-03, CA-04); `## Protótipo` (as imagens com legenda); `## Arquitetura recomendada` (comando, servidor, empacotamento, API, eventos, aprovação, pedidos e loop, numa figura ou tabela); `## Specs propostas` (lista ordenada, com título, tipo e o que cada uma entrega, CA-12; se não for viável, explique o que tornaria viável); `## Remoção do servidor` (resumo de `levantamento.md`, CA-11). Prosa por padrão, tabelas para comparar. Confira: `grep -c '^### PQ-' docs/estudos/painel-local.md` imprime `12`.
3. `docs/estudos/painel-local/README.md` (criar): uma linha por documento de apoio e por imagem, com o que cada um traz, e a observação de que o código do protótipo está só no branch `spike/painel-local`. Confira: os links abrem os arquivos.
4. Confira que nada do spike entrou no repositório principal: `git status --short` só mostra `docs/estudos/painel-local.md`, `docs/estudos/painel-local/` e os arquivos da SPEC-027 em `vostok/work/` (CA-13).
