### Baseline

- `ls docs/estudos | sort`

### Verificação

```bash
git -C /home/victor/projetos/ai-hub-painel branch --show-current
# evidência: medicoes.txt
bash /home/victor/projetos/ai-hub-painel/apps/painel/medir.sh
grep -n '^## ' docs/estudos/painel-local/medicoes.md
```

Resultado esperado: `spike/painel-local`; 5 linhas de variante com os tamanhos; a seção `## Limite proposto` no documento.

### Evidências

- `medicoes.txt` (automática, da verificação)
  - Prova: o tamanho do binário instalado e compactado de cada variante, comparado à base sem painel (PQ-02, PQ-03).
  - Trecho: as linhas `base |`, `tiny+react |`, `tiny+preact |`, `axum+react |` e `axum+preact |` com os bytes.
  - Falha: build com `error` ou menos de 5 linhas de variante.
  - Espera: `axum+preact |`
  - Tempo-limite: 1800
- `medicoes.diff` (diff): `docs/estudos/painel-local/medicoes.md`
  - Prova: a tabela comentada, os comandos para reproduzir e o limite proposto (PQ-02, PQ-03, PQ-04).
  - Trecho: a seção `## Limite proposto` com o número e a variante recomendada.
  - Falha: limite sem número ou sem justificativa.
  - Espera: `Limite proposto`
