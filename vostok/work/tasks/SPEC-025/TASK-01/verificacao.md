### Baseline

- `ls docs/estudos | sort` (muda): o estudo novo entra na lista

### Verificação

```bash
grep -n '^## ' docs/estudos/documentos-finais.md
```

Resultado esperado: as seções do passo 2 ao 4 aparecem.

### Evidências

- `estudo-visual.diff` (diff): `docs/estudos/documentos-finais.md`
  - Prova: o estudo visual com a identidade aplicada, os problemas atuais e as três direções (RF-01, CT-01).
  - Trecho: a seção `## Três direções visuais` com as três direções, uma com abas.
  - Falha: diff sem essa seção ou com menos de três direções.
  - Espera: `Três direções visuais`
