### Baseline

- `ls docs/estudos | sort`

### Verificação

```bash
grep -n -A8 '^## Escolha do modelo' docs/estudos/documentos-finais.md
```

Resultado esperado: o modelo escolhido, o motivo e a data.

### Evidências

- `escolha.diff` (diff): `docs/estudos/documentos-finais.md`
  - Prova: o modelo escolhido e a aprovação do estudo de conteúdo registrados (RF-04, CT-03).
  - Trecho: a seção `## Escolha do modelo` preenchida.
  - Falha: seção ainda com o texto da TASK-01.
  - Espera: `Escolha do modelo`
