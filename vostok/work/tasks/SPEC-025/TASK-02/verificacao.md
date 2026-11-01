### Baseline

- `ls docs/estudos | sort`

### Verificação

```bash
grep -n '^## \|^### ' docs/estudos/documentos-finais.md
```

Resultado esperado: a seção `## Estudo de conteúdo` com as subseções do passo 3 e 4.

### Evidências

- `estudo-conteudo.diff` (diff): `docs/estudos/documentos-finais.md`
  - Prova: as seções de cada documento, a regra de formato e o KT do técnico (RF-05, RF-08, CT-04).
  - Trecho: a seção `## Estudo de conteúdo` com as referências e as seções dos três documentos.
  - Falha: diff sem a regra de formato ou sem o KT.
  - Espera: `Estudo de conteúdo`
