### Baseline

- `ls docs/estudos | sort`

### Verificação

```bash
grep -n '^## ' docs/estudos/painel-local/levantamento.md
grep -c '^| ' docs/estudos/painel-local/levantamento.md
grep -c '^| [a-z-]* |' vostok/context/skill-catalog.md
```

Resultado esperado: as seções `Escopo e método`, `Inventário das ações`, `Aprovação pelo painel` e `Remoção do servidor`; o número de linhas de tabela do levantamento maior que o número de skills do catálogo.

### Evidências

- `levantamento.diff` (diff): `docs/estudos/painel-local/levantamento.md`
  - Prova: o inventário das ações por forma no painel (PQ-05), a aprovação pelo painel com a recomendação (PQ-06) e as mudanças para remover o servidor (PQ-11).
  - Trecho: as três seções e a linha `Recomendação:`.
  - Falha: arquivo ausente ou seção sem tabela.
  - Espera: `Recomendação:`
