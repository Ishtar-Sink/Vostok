### Baseline

- `ls docs/estudos | sort`

### Verificação

```bash
grep -n '^## ' docs/estudos/painel-local/testes.md
```

Resultado esperado: uma seção para aprovação, edição, edição simultânea, loop e Codex.

### Evidências

- `testes.diff` (diff): `docs/estudos/painel-local/testes.md`
  - Prova: a aprovação e a edição pelo painel aceitas pelos comandos de validação do `vostok` (PQ-06, CA-06), o resultado da edição simultânea (PQ-07, CA-10) e o formato do pedido com o teste do chat em `/loop` (PQ-08, CA-09).
  - Trecho: as saídas de `vostok task approve validate` e `vostok task adjust validate`, o 409 da edição simultânea e o tempo até o pedido ficar `feito`.
  - Falha: comando de validação recusando o arquivo gravado pelo painel, perda da edição ou pedido sem processar.
  - Espera: `409`
- `pedido-loop.txt` (manual): a saída do chat do Claude Code em `/loop` processando o pedido gravado pelo painel, com o estado final `feito`.
  - Prova: um chat em loop percebe e processa um pedido feito pelo painel (PQ-08, CA-09).
  - Trecho: a leitura do pedido, a skill executada e o estado `feito`.
  - Falha: o chat não encontra o pedido ou não muda o estado.
